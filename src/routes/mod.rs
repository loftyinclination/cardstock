pub mod index;
pub mod player;
pub mod season;

use crate::entities::player::{PlayerData, PlayerDisplayable};
use crate::entities::team::TeamDisplayable;
use crate::idol::Idols;
use crate::TeamData;
use crate::{Key, SeasonDayKey, BEGINNING_OF_TIME, DAYS_TREE, DB, END_OF_TIME};
use chrono::{DateTime, FixedOffset};
use rocket::response::Debug;
use rocket::{get, http::ContentType};
use sled::Tree;
use uuid::Uuid;
use zerocopy::{AsBytes, FromBytes};

pub type ResponseResult<T> = std::result::Result<T, Debug<anyhow::Error>>;

pub struct Timestamp {
    timestamp: DateTime<FixedOffset>,
    day: u8,
    time_since_game_start: f32, // not sure about units for this but its blaseball so float is probably correct
}

pub fn convert_db_contents_into_format_for_page(
    database_contents: sled::Iter,
    player_tree: Tree,
    team_tree: Tree,
    limit: Option<u16>,
) -> Vec<(Timestamp, Vec<PlayerDisplayable>)> {
    let result = database_contents
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

            log::info!("timestamp {}", timestamp);

            let timestamp = Timestamp {
                timestamp,
                day: 255,
                time_since_game_start: 33.0,
            };
            (timestamp, idol_data)
        });

    match limit {
        Some(number_to_limit_to) => result.take(number_to_limit_to.into()).collect(),
        None => result.collect(),
    }
}

fn get_displayable_data_for_player(
    id: Uuid,
    timestamp: DateTime<FixedOffset>,
    player_tree: &Tree,
    team_tree: &Tree,
) -> Result<PlayerDisplayable, anyhow::Error> {
    let result = player_tree
        .get_lt(Key::new(id, timestamp).as_bytes())?
        .unwrap();

    let result_id = Uuid::from_slice(&Key::read_from(result.0.as_bytes()).unwrap().id)?;
    assert!(result_id == id, "no data existed for player {}", id);

    let player_data: PlayerData = serde_json::from_slice(result.1.as_bytes())?;

    let team = match player_data.team {
        Some(team_id) => {
            //log::info!("getting data for team {}", team_id);
            let team_fetch_result = team_tree.get(team_id.as_bytes())?;
            if team_fetch_result.is_none() {
                log::info!("uhhh");
            }
            let team: TeamData = serde_json::from_slice(&team_fetch_result.unwrap().as_bytes())?;
            TeamDisplayable::new(team.full_name, team.colour, team.emoji)
        }
        None => TeamDisplayable {
            name: "nullteam".into(),
            colour: "#999999".into(),
            emoji: "❓".into(),
        },
    };

    Ok(PlayerDisplayable {
        id,
        name: player_data.name,
        team,
        deceased: player_data.deceased,
        ego: 0,
    })
}

fn get_bounds_for_season(
    season: i16,
) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>), anyhow::Error> {
    let days_tree = DB.open_tree(DAYS_TREE)?;

    let key = SeasonDayKey {
        season: season.into(),
        day: 0,
    };
    let timestamp_of_first_day = days_tree
        .get(key.as_bytes())?
        .map(|v| DateTime::parse_from_rfc3339(std::str::from_utf8(&v).unwrap()).unwrap())
        .unwrap_or(DateTime::parse_from_rfc3339(BEGINNING_OF_TIME).unwrap());
    let key = SeasonDayKey {
        season: season.into(),
        day: 255,
    };
    let timestamp_of_last_day = days_tree
        .get(key.as_bytes())?
        .map(|v| DateTime::parse_from_rfc3339(std::str::from_utf8(&v).unwrap()).unwrap())
        .unwrap_or(DateTime::parse_from_rfc3339(END_OF_TIME).unwrap());

    Ok((timestamp_of_first_day, timestamp_of_last_day))
}

mod routes {
    macro_rules! asset {
        ($path:expr) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))
        };
    }
    pub(crate) use asset;
}

#[get("/styles.css")]
pub fn css() -> (ContentType, &'static str) {
    (ContentType::CSS, routes::asset!("/main.css"))
}

#[get("/cardstock.svg")]
pub fn cardstock() -> (ContentType, &'static str) {
    (ContentType::SVG, routes::asset!("cardstock.svg"))
}

#[get("/images/nav-first.svg")]
pub fn nav_first_image() -> (ContentType, &'static str) {
    (ContentType::SVG, routes::asset!("cardstock.svg"))
}

#[get("/images/nav-back.svg")]
pub fn nav_back_image() -> (ContentType, &'static str) {
    (ContentType::SVG, routes::asset!("cardstock.svg"))
}

#[get("/images/nav-next.svg")]
pub fn nav_next_image() -> (ContentType, &'static str) {
    (ContentType::SVG, routes::asset!("cardstock.svg"))
}

#[get("/images/nav-last.svg")]
pub fn nav_last_image() -> (ContentType, &'static str) {
    (ContentType::SVG, routes::asset!("cardstock.svg"))
}

#[get("/manifest.webmanifest")]
pub fn manifest() -> (ContentType, &'static str) {
    (ContentType::JSON, routes::asset!("manifest.webmanifest"))
}
