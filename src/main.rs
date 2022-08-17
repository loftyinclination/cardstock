#![feature(iter_intersperse)]

mod entities;
mod routes;

use crate::entities::idol;
use crate::entities::player::PlayerData;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::{launch, routes};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::fs;
use uuid::Uuid;
use zerocopy::{AsBytes, BigEndian, FromBytes, I16, I64};

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

    let chron_idols_data: Chron2Response<idol::Idols> = serde_json::from_str(&contents).unwrap();

    let mut player_set = std::collections::HashSet::new();
    let player_tree = DB.open_tree(PLAYER_TREE)?;

    for idol_board_version in chron_idols_data.items.into_iter() {
        let idol_data = match idol_board_version.data {
            idol::Idols::IdolArray(array) => idol::IdolsClass {
                data: idol::Data {
                    strictly_confidential: 20,
                },
                idols: array.into_iter().map(|y| y.player_id).collect(),
            },
            idol::Idols::IdolsClass(idols_class) => idols_class,
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

            cache_player(player, &player_tree).await?;
        }

        idols_tree
            .insert(
                idol_board_version.valid_from.to_rfc3339().as_bytes(),
                serde_json::to_vec(&idol_data)?,
            )
            .expect("failed to insert idol into db");
    }

    fix_necromancy(&player_tree)?;

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
                let key = SeasonDayKey {
                    season: data.season.into(),
                    day: data.day,
                };
                days_tree.insert(key.as_bytes(), start_time.to_rfc3339().as_bytes())?;
            }
        }
    })
}

#[derive(AsBytes, FromBytes)]
#[repr(C)]
pub struct SeasonDayKey {
    season: I16<BigEndian>,
    day: u8,
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

async fn cache_player(player: &Uuid, player_tree: &Tree) -> Result<(), anyhow::Error> {
    let url = format!(
        "{}/v2/versions?type=Player&id={}",
        CHRONICLER_BASE,
        player.to_string()
    );
    log::info!("performing request to {}", url);

    let player_versions: Chron2Response<PlayerData> = CLIENT.get(url).send().await?.json().await?;

    log::info!("got data for player {}", player);

    for version in player_versions.items.into_iter() {
        player_tree
            .insert(
                Key::new(*player, version.valid_from).as_bytes(),
                serde_json::to_vec(&version.data)?,
            )
            .expect("failed to insert player into db");
    }

    let mut next_page = player_versions.next_page;

    while next_page.is_some() {
        let url = format!(
            "{}/v2/versions?type=Player&id={}&page={}",
            CHRONICLER_BASE,
            player.to_string(),
            next_page.unwrap()
        );

        let player_versions: Chron2Response<PlayerData> =
            CLIENT.get(url).send().await?.json().await?;

        next_page = player_versions.next_page;

        log::info!("got data for player {}", player);

        for version in player_versions.items.into_iter() {
            log::info!("valid_from {}", version.valid_from);
            player_tree
                .insert(
                    Key::new(*player, version.valid_from).as_bytes(),
                    serde_json::to_vec(&version.data)?,
                )
                .expect("failed to insert player into db");
        }
    }

    Ok(())
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

fn fix_necromancy(player_tree: &Tree) -> Result<(), anyhow::Error> {
    let id = Uuid::parse_str("04e14d7b-5021-4250-a3cd-932ba8e0a889")?;

    let key = Key::new(id, DateTime::parse_from_rfc3339(BEGINNING_OF_TIME)?);

    let value = PlayerData {
        id,
        deceased: true,
        name: "Jaylen Hotdogfingers".into(),
        permanent_attributes: None,
        team: Some(Uuid::parse_str("105bc3ff-1320-4e37-8ef0-8d595cb95dd0")?),
    };

    player_tree.insert(key.as_bytes(), serde_json::to_vec(&value)?)?;
    Ok(())
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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/",
            routes![
                routes::index::index,
                routes::season::season,
                routes::css,
                routes::cardstock,
                routes::nav_first_image,
                routes::nav_back_image,
                routes::nav_next_image,
                routes::nav_last_image,
                routes::manifest,
            ],
        )
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
    #[serde(rename = "nextPage")]
    next_page: Option<String>,
    items: Vec<ChronV2Versions<T>>,
}

#[derive(Deserialize)]
struct ChronV2Versions<T> {
    #[serde(rename = "validFrom")]
    valid_from: DateTime<Utc>,
    data: T,
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
