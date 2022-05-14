use std::fs;
use serde::{Deserialize};

fn main() -> () {
    let contents = fs::read_to_string("idols.json").unwrap();

    let idols_data: IdolsData = serde_json::from_str(&contents).unwrap();

    println!("{}", idols_data.items[22819].valid_from);
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

#[derive(Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Idols {
    IdolArray(Vec<Idol>),

    IdolsClass(IdolsClass),
}