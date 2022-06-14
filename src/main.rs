use askama::Template;
// use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::response::content::RawHtml;
use rocket::response::Debug;
use rocket::{get, launch, routes};
use serde::Deserialize;
use sled::Db;
use std::fs;

lazy_static::lazy_static! {
    static ref DB: Db = sled::Config::default()
        .path(std::env::var_os("CARDSTOCK_SLED_V1").expect("CARDSTOCK_SLED_V1 not set in environment"))
        .use_compression(true)
        .open()
        .unwrap();

    // static ref CLIENT: Client = Client::builder()
        // .user_agent("cardstock/0.0 (lofty@sibr.dev)")
        // .build()
        // .unwrap();
}

// const CHRONICLER_BASE: &str = "https://api.sibr.dev/chronicler/";
const DAYS_TREE: &str = "games_v1";
const NUMBER_OF_DAYS_FOR_SEASON_TREE: &str = "number_of_days_for_season_v1";

const ZEROTH_SEASON_WITH_IDOL_BOARD: i16 = 4;

const BEGINNING_OF_TIME: &str = "2020-01-01T00:00:00Z";
const END_OF_TIME: &str = "2099-01-01T00:00:00Z";

async fn start_task() -> Result<(), sled::Error> {
    let days_tree = DB.open_tree(DAYS_TREE)?;
    let number_of_days_for_season_tree = DB.open_tree(NUMBER_OF_DAYS_FOR_SEASON_TREE)?;

    fn keep_greater_merge(
        _key: &[u8],
        old_value: Option<&[u8]>,
        new_value: &[u8],
    ) -> Option<Vec<u8>> {
        if let Some(current_value) = old_value {
            if current_value[0] > new_value[0] {
                Some(current_value.to_vec())
            } else {
                Some(new_value.to_vec())
            }
        } else {
            Some(new_value.to_vec())
        }
    }

    number_of_days_for_season_tree.set_merge_operator(keep_greater_merge);

    // let games: GameData = CLIENT
    // .get(format!("{}/v1/games", CHRONICLER_BASE))
    // .send()
    // .await?
    // .json()
    // .await?;
    let contents = fs::read_to_string("games.json").unwrap();

    let games: GameData = serde_json::from_str(&contents).unwrap();
    for game in games.data.into_iter() {
        if game.data.sim.is_none() && game.data.season > ZEROTH_SEASON_WITH_IDOL_BOARD {
            if let Some(start_time) = game.start_time {
                let data = game.data;
                let mut key = data.season.to_be_bytes().to_vec();
                key.extend(data.day.to_be_bytes());
                days_tree.insert(&key, start_time.as_bytes())?;

                number_of_days_for_season_tree
                    .merge(data.season.to_be_bytes(), (data.day + 1).to_be_bytes())?;
            }
        }
    }

    Ok(())
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
    let contents = fs::read_to_string("idols.json").unwrap();

    let chron_idols_data: IdolsData = serde_json::from_str(&contents).unwrap();
    let html_content = HomePage {
        boards: chron_idols_data
            .items
            .into_iter()
            .map(|x| {
                let idols: Idols = x.data;
                let idol_data: IdolsClass = match idols {
                    Idols::IdolArray(array) => IdolsClass {
                        data: Data {
                            strictly_confidential: 20,
                        },
                        idols: array.into_iter().map(|y| y.player_id).collect(),
                    },
                    Idols::IdolsClass(idols_class) => idols_class,
                };
                (x.valid_from, idol_data)
            })
            .collect(),
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
    let number_of_days_for_season_tree = DB.open_tree(NUMBER_OF_DAYS_FOR_SEASON_TREE)?;
    let number_of_days = number_of_days_for_season_tree.get(season.to_be_bytes())?;

    if number_of_days.is_none() {
        return Ok(None);
    }

    let number_of_days = number_of_days.unwrap();

    let days_tree = DB.open_tree(DAYS_TREE)?;

    let mut key = season.to_be_bytes().to_vec();
    key.push(0);
    let timestamp_of_first_day = days_tree
        .get(&*key)?
        .map(|v| std::str::from_utf8(&v).unwrap().to_string())
        .unwrap_or(BEGINNING_OF_TIME.into());
    key[2] = number_of_days[0];
    let timestamp_of_last_day = days_tree
        .get(&*key)?
        .map(|v| std::str::from_utf8(&v).unwrap().to_string())
        .unwrap_or(END_OF_TIME.into());

    // FIXME: don't read the entire idols data structure here. honestly, what are you doing.
    let contents = fs::read_to_string("idols.json").unwrap();

    let chron_idols_data: IdolsData = serde_json::from_str(&contents).unwrap();
    let page_content = HomePage {
        boards: chron_idols_data
            .items
            .into_iter()
            .filter(|x| {
                timestamp_of_first_day <= x.valid_from && x.valid_from <= timestamp_of_last_day
            })
            .map(|x| {
                let idols: Idols = x.data;
                let idol_data: IdolsClass = match idols {
                    Idols::IdolArray(array) => IdolsClass {
                        data: Data {
                            strictly_confidential: 20,
                        },
                        idols: array.into_iter().map(|y| y.player_id).collect(),
                    },
                    Idols::IdolsClass(idols_class) => idols_class,
                };
                (x.valid_from, idol_data)
            })
            .collect(),
    };
    Ok(Some(page_content))
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
    start_time: Option<String>,
    #[serde(rename = "endTime")]
    end_time: Option<String>,
    data: Game,
}

#[derive(Deserialize)]
struct Game {
    #[serde(alias = "_id")]
    id: String,
    day: u8,
    season: i16,
    sim: Option<String>,
}

#[derive(Deserialize)]
struct IdolsData {
    items: Vec<ChronV2Versions>,
}

#[derive(Deserialize)]
struct ChronV2Versions {
    #[serde(rename = "validFrom")]
    valid_from: String,
    #[serde(rename = "validTo")]
    valid_to: Option<String>,
    data: Idols,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Idol {
    #[serde(rename = "id")]
    pub id: Option<String>,

    #[serde(rename = "playerId")]
    pub player_id: String,

    #[serde(rename = "total")]
    pub total: Option<i64>,
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdolsClass {
    #[serde(rename = "data")]
    pub data: Data,

    #[serde(rename = "idols")]
    pub idols: Vec<String>,
}

#[derive(Clone, PartialEq, Deserialize)]
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

#[derive(Template)]
#[template(path = "home.html")]
struct HomePage {
    boards: Vec<(String, IdolsClass)>,
}
