#[derive(Clone)]
pub(crate) struct Cache {
    dir: String,
}

impl Cache {
    pub fn new() -> crate::Result<Self> {
        let dir = envir::get("CACHE_DIR")?;
        if !std::fs::exists(&dir)? {
            std::fs::create_dir(&dir)?;
        }

        Ok(Self {
            dir,
        })
    }

    pub async fn get(&self, id: u32) -> crate::Result<String> {
        match self.try_from_cache(id).await {
            Some(contents) => Ok(contents),
            None => self.fetch(id).await,
        }
    }

    async fn try_from_cache(&self, id: u32) -> Option<String> {
        let cache = self.file(id);

        if std::fs::exists(&cache).ok()? {
            std::fs::read_to_string(&cache).ok()
        } else {
            None
        }
    }


    async fn fetch(&self, id: u32) -> crate::Result<String> {
        let contents = reqwest::get(&format!("https://api.tvmaze.com/shows/{id}"))
            .await?
            .text()
            .await?;

        if let Err(err) = std::fs::write(self.file(id), &contents) {
            log::error!("Failed to cache response: {err}");
        }

        Ok(contents)
    }

    fn file(&self, id: u32) -> String {
        format!("{}/{id}", self.dir)
    }
}
