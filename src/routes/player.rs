// use crate::entities::player::PlayerDisplayable;
// use crate::routes::ResponseResult;

// use anyhow::Result;
// use askama::Template;
// use chrono::{DateTime, FixedOffset};
// use rocket::get;
// use rocket::response::content::RawHtml;
// use uuid::Uuid;

// #[get("/player/{player_id}")]
// pub fn player_page(player_id: Uuid) -> ResponseResult<Option<RawHtml<String>>> {
//     Ok(
//         match load_player_page(player_id).map_err(anyhow::Error::from)? {
//             Some(player_page) => Some(RawHtml(player_page.render().map_err(anyhow::Error::from)?)),
//             None => None,
//     })
// }

// fn load_player_page(player_id: Uuid) -> Result<Option<PlayerPage>> {
//     // let player_data = routes::get_all_data_for_player(player_id)?;

//      Ok(None)
// }

// #[derive(Template)]
// #[template("player.html")]
// struct PlayerPage {
//     name: String,
//     id: Uuid,
//     versions: Vec<(DateTime<FixedOffset>, Vec<PlayerDisplayable>)>,
// }
