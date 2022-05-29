use askama::Template;
use rocket::fairing::AdHoc;
use rocket::http::ContentType;
use rocket::response::content::RawHtml;
use rocket::response::Debug;
use rocket::{get, launch, routes};
use serde::Deserialize;
use std::fs;

type ResponseResult<T> = std::result::Result<T, Debug<anyhow::Error>>;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, css, cardstock, manifest,])
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
fn season(season: u8) -> ResponseResult<Option<RawHtml<String>>> {
    Ok(match load_season(season)? {
        Some(idol_boards) => Some(RawHtml(idol_boards.render().map_err(anyhow::Error::from)?)),
        None => None,
    })
}

fn load_season(season: u8) -> Result<Option<HomePage>, Debug<anyhow::Error>> {
    Ok(None)
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
#[serde(rename_all = "camelCase")]
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
