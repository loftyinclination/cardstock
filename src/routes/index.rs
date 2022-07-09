use crate::routes::ResponseResult;
use askama::Template;
use rocket::get;
use rocket::response::content::RawHtml;

#[get("/")]
pub fn index() -> ResponseResult<RawHtml<String>> {
    let seasons = vec![
        6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ];
    let html_content = IndexPage { seasons }
        .render()
        .map_err(anyhow::Error::from)
        .unwrap();
    Ok(RawHtml(html_content))
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    seasons: Vec<i8>,
}
