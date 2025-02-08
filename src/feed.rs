#[derive(Debug, serde::Serialize)]
pub(crate) struct Item {
    id: String,
    title: String,
    url: String,
    date_published: String,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct Feed {
    icon: &'static str,
    version: &'static str,
    title: String,
    items: Vec<Item>,
}

impl Feed {
    pub fn from(show: crate::Show, seasons: &[crate::Season]) -> Self {
        Self {
            icon: "https://static.tvmaze.com/images/favico/favicon.ico",
            items: seasons
                .iter()
                .filter(|x| x.premiere_date.is_some())
                .map(|x| Item {
                    id: x.id.to_string(),
                    title: format!("{} - Saison {}", show.name, x.number),
                    url: x.url.clone(),
                    date_published: format!("{}T00:00:00-00:00", x.premiere_date.as_ref().unwrap()),
                })
                .collect(),
            title: show.name,
            version: "https://jsonfeed.org/version/1.1",
        }
    }
}
