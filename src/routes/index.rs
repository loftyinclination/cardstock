use crate::routes::routes::asset;
use crate::routes::ResponseResult;
use askama::Template;
use rocket::get;
use rocket::response::content::RawHtml;
use serde::Deserialize;

#[get("/")]
pub fn index() -> ResponseResult<RawHtml<String>> {
    let mut eras = serde_json::from_str::<Vec<Era>>(asset!("/data/elections.json"))
	.map_err(anyhow::Error::from)?;

    eras.iter_mut().for_each(|era| era.seasons.sort_by(|a, b| b.index.cmp(&a.index)));
    eras.sort_by(|a, b| b.seasons[0].index.cmp(&a.seasons[0].index));

    let html_content = IndexPage { eras }.render().map_err(anyhow::Error::from)?;

    Ok(RawHtml(html_content))
}

#[derive(Deserialize)]
struct Season {
    index: i8,
    start_date: String,
    end_date: String,
    election_offset: Option<String>,
}

#[derive(Deserialize)]
struct Era {
    name: String,
    seasons: Vec<Season>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    eras: Vec<Era>,
}
