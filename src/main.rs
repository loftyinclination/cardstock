use askama::Template;
use chrono::{DateTime, FixedOffset, Utc};
use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::response::content::RawHtml;
use rocket::response::Debug;
use rocket::{get, launch, routes};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::fs;
use uuid::Uuid;
use zerocopy::{AsBytes, BigEndian, FromBytes, I64};

lazy_static::lazy_static! {
    static ref DB: Db = sled::Config::default()
        .path(std::env::var_os("CARDSTOCK_SLED_V1").expect("CARDSTOCK_SLED_V1 not set in environment"))
        .use_compression(true)
        .open()
        .unwrap();

    static ref CLIENT: Client = Client::builder()
        .user_agent("cardstock/0.0 (lofty@sibr.dev)")
        .build()
        .unwrap();
}

const CHRONICLER_BASE: &str = "https://api.sibr.dev/chronicler";

const DAYS_TREE: &str = "games_v1";
const PLAYER_TREE: &str = "players_v1";
const IDOLS_TREE: &str = "idols_v1";
const TEAM_TREE: &str = "teams_v1";

const ZEROTH_SEASON_WITH_IDOL_BOARD: i16 = 4;

const BEGINNING_OF_TIME: &str = "2020-01-01T00:00:00Z";
const END_OF_TIME: &str = "2099-01-01T00:00:00Z";

async fn start_task() -> Result<(), anyhow::Error> {

    cache_season_days()?;
    cache_teams()?;

    let idols_tree = DB.open_tree(IDOLS_TREE)?;

    let contents = fs::read_to_string("data/idols.json").unwrap();
    log::info!("read idol board data from file");

    let chron_idols_data: Chron2Response<Idols> = serde_json::from_str(&contents).unwrap();

    let mut player_set = std::collections::HashSet::new();
    let player_tree = DB.open_tree(PLAYER_TREE)?;

    for idol_board_version in chron_idols_data.items.into_iter() {
        let idol_data = match idol_board_version.data {
            Idols::IdolArray(array) => IdolsClass {
                data: Data {
                    strictly_confidential: 20,
                },
                idols: array.into_iter().map(|y| y.player_id).collect(),
            },
            Idols::IdolsClass(idols_class) => idols_class,
        };

        log::info!(
            "processed idol board data for timestamp {}",
            idol_board_version.valid_from
        );

        for player in &idol_data.idols {
            if player_set.contains(player) {
                continue;
            }

            player_set.insert(player.clone());

            if does_any_data_exist_in_tree_for_player(player, &player_tree) {
                log::info!(
                    "data exists for player {} in tree from previous run of cardstock",
                    player
                );
                continue;
            }

            let url = format!(
                "{}/v2/versions?type=Player&id={}",
                CHRONICLER_BASE,
                player.to_string()
            );
            log::info!("performing request to {}", url);

            let player_versions: Chron2Response<PlayerData> =
                CLIENT.get(url).send().await?.json().await?;

            log::info!("got data for player {}", player);

            for version in player_versions.items.into_iter() {
                player_tree
                    .insert(
                        Key::new(*player, version.valid_from).as_bytes(),
                        serde_json::to_vec(&version.data)?,
                    )
                    .expect("failed to insert player into db");
            }
        }

        idols_tree
            .insert(
                idol_board_version.valid_from.to_rfc3339().as_bytes(),
                serde_json::to_vec(&idol_data)?,
            )
            .expect("failed to insert idol into db");
    }

    Ok(())
}

fn cache_season_days() -> Result<(), anyhow::Error> {
    let days_tree = DB.open_tree(DAYS_TREE)?;

    let contents = fs::read_to_string("data/games.json")?;
    let games: GameData = serde_json::from_str(&contents)?;

    Ok(for game in games.data.into_iter() {
        if game.data.sim.is_none() && game.data.season > ZEROTH_SEASON_WITH_IDOL_BOARD {
            if let Some(start_time) = game.start_time {
                let data = game.data;
                let mut key = data.season.to_be_bytes().to_vec();
                key.extend(data.day.to_be_bytes());
                days_tree.insert(&key, start_time.to_rfc3339().as_bytes())?;
            }
        }
    })
}

fn cache_teams() -> Result<(), anyhow::Error> {
    let teams_tree = DB.open_tree(TEAM_TREE)?;

    let contents = fs::read_to_string("data/teams.json")?;
    let teams: Vec<ChronV2Versions<TeamData>> = serde_json::from_str(&contents)?;

    Ok(for team_data in teams.into_iter() {
        let team = team_data.data;
        log::info!("adding data for team {}, {}", team.id, team.full_name);
        teams_tree.insert(&team.id.as_bytes(), serde_json::to_vec(&team)?)?;
    })
}

fn does_any_data_exist_in_tree_for_player(player: &Uuid, tree: &Tree) -> bool {
    let result = tree
        .get_lt(Key::new(*player, DateTime::parse_from_rfc3339(END_OF_TIME).unwrap()).as_bytes())
        .unwrap();

    match result {
        Some((key_bytes, _)) => {
            let player_in_key =
                Uuid::from_slice(&Key::read_from(key_bytes.as_bytes()).unwrap().id).unwrap();
            *player == player_in_key
        }
        None => false,
    }
}

#[derive(AsBytes, FromBytes)]
#[repr(C)]
struct Key {
    id: [u8; 16],
    valid_from: I64<BigEndian>,
}

impl Key {
    fn new<T: chrono::TimeZone>(id: Uuid, valid_from: DateTime<T>) -> Key {
        Key {
            id: *id.as_bytes(),
            valid_from: valid_from.timestamp_nanos().into(),
        }
    }
}

type ResponseResult<T> = std::result::Result<T, Debug<anyhow::Error>>;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, season, css, cardstock, manifest,])
        .attach(AdHoc::on_liftoff("Background tasks", |_rocket| {
            Box::pin(async {
                match start_task().await {
                    Err(err) => {
                        log::error!("{:#}", err);
                    }
                    _ => (),
                }
            })
        }))
}

#[get("/")]
fn index() -> ResponseResult<RawHtml<String>> {
    let idols_tree = DB.open_tree(IDOLS_TREE).map_err(anyhow::Error::from)?;
    let player_tree = DB.open_tree(PLAYER_TREE).map_err(anyhow::Error::from)?;
    let team_tree = DB.open_tree(TEAM_TREE).map_err(anyhow::Error::from)?;

    let html_content = HomePage {
        boards: convert_db_contents_into_format_for_page(idols_tree.iter(), player_tree, team_tree),
    }
    .render()
    .map_err(anyhow::Error::from)
    .unwrap();
    Ok(RawHtml(html_content))
}

#[get("/season/<season>")]
fn season(season: i16) -> ResponseResult<Option<RawHtml<String>>> {
    Ok(
        match load_season(season - 1).map_err(anyhow::Error::from)? {
            Some(idol_boards) => Some(RawHtml(idol_boards.render().map_err(anyhow::Error::from)?)),
            None => None,
        },
    )
}

fn load_season(season: i16) -> Result<Option<HomePage>, anyhow::Error> {
    let (timestamp_of_first_day, timestamp_of_last_day) = get_bounds_for_season(season)?;

    let idols_tree = DB.open_tree(IDOLS_TREE)?;
    let player_tree = DB.open_tree(PLAYER_TREE)?;
    let team_tree = DB.open_tree(TEAM_TREE)?;

    let page_content = HomePage {
        boards: convert_db_contents_into_format_for_page(
            idols_tree.range(
                timestamp_of_first_day.to_rfc3339().as_bytes()
                    ..timestamp_of_last_day.to_rfc3339().as_bytes(),
            ),
            player_tree,
            team_tree,
        ),
    };
    Ok(Some(page_content))
}

fn get_bounds_for_season(
    season: i16,
) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>), anyhow::Error> {
    let days_tree = DB.open_tree(DAYS_TREE)?;

    let mut key = season.to_be_bytes().to_vec();
    key.push(0);
    let timestamp_of_first_day = days_tree
        .get(&*key)?
        .map(|v| DateTime::parse_from_rfc3339(std::str::from_utf8(&v).unwrap()).unwrap())
        .unwrap_or(DateTime::parse_from_rfc3339(BEGINNING_OF_TIME).unwrap());
    key[2] = 0xFF;
    let timestamp_of_last_day = days_tree
        .get_lt(&*key)?
        .map(|(_, v)| DateTime::parse_from_rfc3339(std::str::from_utf8(&v).unwrap()).unwrap())
        .unwrap_or(DateTime::parse_from_rfc3339(END_OF_TIME).unwrap());

    Ok((timestamp_of_first_day, timestamp_of_last_day))
}

fn convert_db_contents_into_format_for_page(
    database_contents: sled::Iter,
    player_tree: Tree,
    team_tree: Tree,
) -> Vec<(DateTime<FixedOffset>, Vec<PlayerDisplayable>)> {
    database_contents
        .map(|x| {
            let result = x.unwrap();
            let timestamp =
                DateTime::parse_from_rfc3339(std::str::from_utf8(result.0.as_bytes()).unwrap())
                    .unwrap();

            let idols: Idols = serde_json::from_slice(result.1.as_bytes()).unwrap();
            let idol_data = (match idols {
                Idols::IdolArray(array) => {
                    let player_ids: Vec<Uuid> = array.into_iter().map(|y| y.player_id).collect();
                    player_ids
                }
                Idols::IdolsClass(idols_class) => idols_class.idols,
            })
            .into_iter()
            .map(|player_id| {
                get_displayable_data_for_player(player_id, timestamp, &player_tree, &team_tree)
                    .unwrap()
            })
            .collect();
            (timestamp, idol_data)
        })
        .collect()
}

fn get_displayable_data_for_player(
    player_id: Uuid,
    timestamp: DateTime<FixedOffset>,
    player_tree: &Tree,
    team_tree: &Tree,
) -> Result<PlayerDisplayable, anyhow::Error> {
    let result = player_tree
        .get_lt(Key::new(player_id, timestamp).as_bytes())?
        .unwrap();

    let result_id = Uuid::from_slice(&Key::read_from(result.0.as_bytes()).unwrap().id)?;
    assert!(result_id == player_id, "no data existed for player");

    let player_data: PlayerData = serde_json::from_slice(result.1.as_bytes())?;

    let team_data = match player_data.team {
        Some(team_id) => {
            log::info!("getting data for team {}", team_id);
            let team_fetch_result = team_tree.get(team_id.as_bytes())?;
            if team_fetch_result.is_none() {
                log::info!("uhhh");
            }
            let team: TeamData =
                serde_json::from_slice(&team_fetch_result.unwrap().as_bytes())?;
            team
        }
        None => TeamData {
            id: Uuid::nil(),
            colour: "#999999".to_string(),
            full_name: "nullteam".to_string(),
            emoji: "❓".to_string(),
        },
    };

    Ok(PlayerDisplayable {
        id: player_id,
        name: player_data.name,
        team_name: team_data.full_name,
        team_colour: team_data.colour,
        team_emoji: team_data.emoji,
        deceased: player_data.deceased,
        ego: 0,
    })
}

macro_rules! asset {
    ($path:expr) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
    };
}

#[get("/styles.css")]
pub fn css() -> (ContentType, &'static str) {
    (ContentType::CSS, asset!("/main.css"))
}

#[get("/cardstock.svg")]
pub fn cardstock() -> (ContentType, &'static str) {
    (ContentType::SVG, asset!("cardstock.svg"))
}

#[get("/manifest.webmanifest")]
pub fn manifest() -> (ContentType, &'static str) {
    (ContentType::JSON, asset!("manifest.webmanifest"))
}

#[derive(Deserialize)]
struct GameData {
    data: Vec<Chron1Versions>,
}

#[derive(Deserialize)]
struct Chron1Versions {
    #[serde(rename = "startTime")]
    start_time: Option<DateTime<Utc>>,
    data: Game,
}

#[derive(Deserialize)]
struct Game {
    day: u8,
    season: i16,
    sim: Option<String>,
}

#[derive(Deserialize)]
struct Chron2Response<T> {
    items: Vec<ChronV2Versions<T>>,
}

#[derive(Deserialize)]
struct ChronV2Versions<T> {
    #[serde(rename = "validFrom")]
    valid_from: DateTime<Utc>,
    data: T,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Idol {
    #[serde(rename = "id")]
    pub id: Option<Uuid>,

    #[serde(rename = "playerId")]
    pub player_id: Uuid,

    #[serde(rename = "total")]
    pub total: Option<i64>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerData {
    #[serde(alias = "_id")]
    pub id: Uuid,

    pub name: String,

    #[serde(rename = "leagueTeamId")]
    pub team: Option<Uuid>,

    pub deceased: bool,

    #[serde(rename = "permAttr")]
    pub permanent_attributes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct TeamData {
    pub id: Uuid,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "mainColor")]
    pub colour: String,
    pub emoji: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdolsClass {
    #[serde(rename = "data")]
    pub data: Data,

    #[serde(rename = "idols")]
    pub idols: Vec<Uuid>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    #[serde(rename = "strictlyConfidential")]
    pub strictly_confidential: i64,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Idols {
    IdolArray(Vec<Idol>),

    IdolsClass(IdolsClass),
}

struct PlayerDisplayable {
    id: Uuid,
    name: String,
    team_name: String,
    team_colour: String,
    team_emoji: String,
    deceased: bool,
    ego: i8,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomePage {
    boards: Vec<(DateTime<FixedOffset>, Vec<PlayerDisplayable>)>,
}
