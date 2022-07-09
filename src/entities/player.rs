use crate::entities::team::TeamDisplayable;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

pub struct PlayerDisplayable {
    pub id: Uuid,
    pub name: String,
    pub team: TeamDisplayable,
    pub deceased: bool,
    pub ego: i8,
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