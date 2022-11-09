mod errors;

use errors::*;

#[tokio::main]
async fn main() -> Result {
    let app = axum::Router::new().route("/seasons/:id", axum::routing::get(seasons));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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
struct Show {
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
