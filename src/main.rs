use askama::Template;
use rocket::{get, launch, routes};
use rocket::response::Debug;
use rocket::http::ContentType;
use rocket::response::content::RawHtml;
use std::fs;
use serde::{Deserialize};

type ResponseResult<T> = std::result::Result<T, Debug<anyhow::Error>>;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
           "/",
            routes![
                index,
                css,
                cardstock,
                manifest,
            ],
        )
}

#[get("/")]
fn index() -> ResponseResult<RawHtml<String>> {
    let contents = fs::read_to_string("idols.json").unwrap();

    let idols_data: IdolsData = serde_json::from_str(&contents).unwrap();
    let html_content = idols_data.items[0].data.render().map_err(anyhow::Error::from).unwrap();
    Ok(RawHtml(html_content))
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
#[serde(rename_all="camelCase")]
struct IdolsData {
    items: Vec<ChronV2Versions>,
}

#[derive(Deserialize)]
struct ChronV2Versions {
    #[serde(rename="validFrom")]
    valid_from: String,
    #[serde(rename="validTo")]
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

#[derive(Clone, PartialEq, Deserialize, Template)]
#[serde(untagged)]
#[template(path = "home.html")]
pub enum Idols {
    IdolArray(Vec<Idol>),

    IdolsClass(IdolsClass),
}