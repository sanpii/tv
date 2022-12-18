mod errors;

use errors::*;

#[tokio::main]
async fn main() -> Result {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    env_logger::init();

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/seasons/:id", axum::routing::get(seasons));

    let bind = format!("{}:{}", env("LISTEN_IP"), env("LISTEN_PORT")).parse()?;

    axum::Server::bind(&bind)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("Missing {name} env variable"))
}

#[derive(Debug, Default, serde::Deserialize)]
struct Search {
    q: Option<String>,
}

async fn index(axum::extract::Query(params): axum::extract::Query<Search>) -> Result<maud::Markup> {
    let results: Vec<SearchResult> = if let Some(ref q) = params.q {
        reqwest::get(&format!("https://api.tvmaze.com/search/shows?q={q}"))
            .await?
            .json()
            .await?
    } else {
        Vec::new()
    };

    let contents = maud::html! {
        head {
        }
        body {
            form {
                input type="text" name="q" value=(params.q.unwrap_or_default());
                " "
                input type="submit" value="Search";
            }
            @if !results.is_empty() {
                ul {
                    @for result in &results {
                        li {
                            a href={ "/seasons/" (result.show.id) } {
                                (result.show.name)
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(contents)
}

async fn seasons(
    axum::extract::Path(id): axum::extract::Path<u32>,
) -> Result<axum::response::Json<Feed>> {
    let show: Show = reqwest::get(&format!("https://api.tvmaze.com/shows/{id}"))
        .await?
        .json()
        .await?;

    let seasons: Vec<Season> = reqwest::get(&format!("https://api.tvmaze.com/shows/{id}/seasons"))
        .await?
        .json()
        .await?;

    let feed = Feed::from(show, seasons);

    Ok(axum::response::Json(feed))
}

#[derive(Debug, serde::Deserialize)]
struct SearchResult {
    show: Show,
}

#[derive(Debug, serde::Deserialize)]
struct Show {
    id: u32,
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct Season {
    id: u32,
    number: u32,
    url: String,
}

#[derive(Debug, serde::Serialize)]
struct Feed {
    version: &'static str,
    title: String,
    items: Vec<Item>,
}

impl Feed {
    fn from(show: Show, seasons: Vec<Season>) -> Self {
        Self {
            items: seasons
                .iter()
                .map(|x| Item {
                    id: x.id.to_string(),
                    title: format!("{} - Saison {}", show.name, x.number),
                    url: x.url.clone(),
                })
                .collect(),
            title: show.name,
            version: "https://jsonfeed.org/version/1.1",
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct Item {
    id: String,
    title: String,
    url: String,
}
