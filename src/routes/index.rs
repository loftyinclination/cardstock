use crate::routes::ResponseResult;
use askama::Template;
use rocket::get;
use rocket::response::content::RawHtml;

#[get("/")]
pub fn index() -> ResponseResult<RawHtml<String>> {
    // FIXME: use json file for this? read dynamically?
    let expansion_seasons = vec![
        Season {
            index: 24,
            start_date: "Jul 26, 2021".into(),
            end_date: "Aug 1, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 23,
            start_date: "Jul 19, 2021".into(),
            end_date: "Jul 25, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 22,
            start_date: "Jun 28, 2021".into(),
            end_date: "Jun 4, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 21,
            start_date: "Jun 21, 2021".into(),
            end_date: "Jun 27, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 20,
            start_date: "Jun 14, 2021".into(),
            end_date: "Jun 20, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 19,
            start_date: "May 17, 2021".into(),
            end_date: "May 23, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 18,
            start_date: "May 10, 2021".into(),
            end_date: "May 16, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 17,
            start_date: "Apr 19, 2021".into(),
            end_date: "Apr 25, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 16,
            start_date: "Apr 12, 2021".into(),
            end_date: "Aor 18, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 15,
            start_date: "Apr 5, 2021".into(),
            end_date: "Apr 11, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 14,
            start_date: "Mar 15, 2021".into(),
            end_date: "Mar 21, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 13,
            start_date: "Mar 8, 2021".into(),
            end_date: "Mar 14, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 12,
            start_date: "Mar 1, 2021".into(),
            end_date: "Mar 7, 2021".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
    ];

    let expansion = Era {
        name: "Expansion Era".into(),
        seasons: expansion_seasons,
    };

    let discipline_seasons = vec![
        Season {
            index: 11,
            start_date: "Oct 19, 2020".into(),
            end_date: "Oct 25, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 10,
            start_date: "Oct 12, 2020".into(),
            end_date: "Oct 18, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 9,
            start_date: "Oct 5, 2020".into(),
            end_date: "Oct 11, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 8,
            start_date: "Sep 21, 2020".into(),
            end_date: "Sep 27, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 7,
            start_date: "Sep 14, 2020".into(),
            end_date: "Sep 20, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
        Season {
            index: 6,
            start_date: "Sep 7, 2020".into(),
            end_date: "Sep 13, 2020".into(),
            election_offset: "2021-02-03T12:00:00Z".into(),
        },
    ];

    let discipline = Era {
        name: "Discipline Era".into(),
        seasons: discipline_seasons,
    };

    let eras = vec![expansion, discipline];

    let html_content = IndexPage { eras }
        .render()
        .map_err(anyhow::Error::from)
        .unwrap();
    Ok(RawHtml(html_content))
}

struct Season {
    index: i8,
    start_date: String,
    end_date: String,
    election_offset: String,
}

struct Era {
    name: String,
    seasons: Vec<Season>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexPage {
    eras: Vec<Era>,
}
