use crate::entities::player::PlayerDisplayable;
use crate::routes::{ResponseResult, Timestamp};
use crate::{DB, IDOLS_TREE, PLAYER_TREE, TEAM_TREE};

use askama::Template;
use rocket::get;
use rocket::response::content::RawHtml;

use super::convert_db_contents_into_format_for_page;
use super::get_bounds_for_season;

#[get("/season/<season>?<limit>")]
pub fn season(season: i16, limit: Option<u16>) -> ResponseResult<Option<RawHtml<String>>> {
    Ok(
        match load_season(season - 1, limit).map_err(anyhow::Error::from)? {
            Some(idol_boards) => Some(RawHtml(idol_boards.render().map_err(anyhow::Error::from)?)),
            None => None,
        },
    )
}

fn load_season(season: i16, limit: Option<u16>) -> Result<Option<SeasonPage>, anyhow::Error> {
    let (timestamp_of_first_day, timestamp_of_last_day) = get_bounds_for_season(season)?;

    let idols_tree = DB.open_tree(IDOLS_TREE)?;
    let player_tree = DB.open_tree(PLAYER_TREE)?;
    let team_tree = DB.open_tree(TEAM_TREE)?;

    let page_content = SeasonPage {
        boards: convert_db_contents_into_format_for_page(
            idols_tree.range(
                timestamp_of_first_day.to_rfc3339().as_bytes()
                    ..timestamp_of_last_day.to_rfc3339().as_bytes(),
            ),
            player_tree,
            team_tree,
            limit,
        ),
    };
    Ok(Some(page_content))
}

#[derive(Template)]
#[template(path = "season.html")]
struct SeasonPage {
    boards: Vec<(Timestamp, Vec<PlayerDisplayable>)>,
}
