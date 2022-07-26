use crate::routes::ResponseResult;
use askama::Template;
use rocket::get;
use rocket::response::content::RawHtml;

#[get("/")]
pub fn index() -> ResponseResult<RawHtml<String>> {
    let expansion_seasons = vec![
        12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    ];

    let expansion = Era {
        name: "Expansion",
        seasons: expansion_seasons,
    };

    let discipline_seasons = vec![
        6, 7, 8, 9, 10, 11,
    ];

    let discipline = Era {
        name: "Discipline",
        seasons: discipline_seasons,
    };

    let eras = vec![
        expansion, discipline
    ]
    let html_content = IndexPage { eras }
        .render()
        .map_err(anyhow::Error::from)
        .unwrap();
    Ok(RawHtml(html_content))
}

struct Era {
    name: String,
    seasons: Vec<i8>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    eras: Vec<Era>,
}
