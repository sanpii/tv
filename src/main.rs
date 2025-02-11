mod cache;
mod errors;
mod feed;

use cache::Cache;
use errors::*;
use feed::Feed;

#[tokio::main]
async fn main() -> Result {
    envir::init();

    let limit_rate = envir::try_parse("LIMIT_RATE")?.unwrap_or(1);
    let cache = Cache::new()?;

    let app = axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/seasons/{id}", axum::routing::get(seasons))
        .layer(tower::limit::GlobalConcurrencyLimitLayer::new(limit_rate))
        .with_state(cache.into());

    let bind = format!(
        "{}:{}",
        envir::get("LISTEN_IP")?,
        envir::get("LISTEN_PORT")?
    );
    let listener = tokio::net::TcpListener::bind(bind).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Default, serde::Deserialize)]
struct Search {
    q: Option<String>,
}

async fn index(
    axum::extract::Query(params): axum::extract::Query<Search>,
) -> Result<axum::response::Html<String>> {
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

    Ok(axum::response::Html(contents.into_string()))
}

async fn seasons(
    axum::extract::Path(id): axum::extract::Path<u32>,
    axum::extract::State(cache): axum::extract::State<std::sync::Arc<Cache>>,
) -> Result<axum::response::Response<axum::body::Body>> {
    use axum::response::IntoResponse as _;

    let show = Show::get(&cache, id).await?;

    let request = reqwest::get(&format!("https://api.tvmaze.com/shows/{id}/seasons")).await?;
    let headers = request.headers().clone();
    let seasons: Vec<Season> = request.json().await?;

    let feed = Feed::from(show, &seasons);

    let mut response = axum::response::Json(feed).into_response();
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/feed+json"),
    );

    if let Some(last_modified) = headers.get(axum::http::header::LAST_MODIFIED) {
        response
            .headers_mut()
            .insert(axum::http::header::LAST_MODIFIED, last_modified.clone());
    }

    Ok(response)
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

impl Show {
    async fn get(cache: &Cache, id: u32) -> Result<Self> {
        let contents = cache.get(id).await?;

        serde_json::from_str(&contents).map_err(Into::into)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Season {
    id: u32,
    number: u32,
    url: String,
    premiere_date: Option<String>,
}
