use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
