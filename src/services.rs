use crate::cache::AppCache;
use crate::config::Config;
use std::sync::Arc;

pub mod announcements;
pub mod cafeteria;
pub mod cafeteria_scraper;
pub mod calendar;
pub mod earthquake;
pub mod schedule;
pub mod semester;
pub mod weather;
pub mod magnum;

/// Shared application services injected into handlers via `dptree::deps!`.
///
/// Constructed once in `main` and cloned (`Arc`) into each handler. Replaces
/// the Python module-level singletons (`_announcement_service`, etc.).
#[derive(Clone)]
pub struct Services {
    pub config: Arc<Config>,
    pub cache: AppCache,
    pub db: Arc<crate::db::Db>,
    pub http_client: reqwest::Client,
    pub weather: weather::SharedWeatherService,
    pub earthquake: earthquake::SharedEarthquakeService,
    pub magnum: magnum::SharedMagnumService,
}

impl Services {
    pub fn new(config: Config, cache: AppCache, db: crate::db::Db) -> Self {
        let http_client = crate::scraper::build_client().unwrap_or_else(|_| reqwest::Client::new());
        let db_arc = Arc::new(db);
        let magnum_svc = Arc::new(magnum::MagnumService::new(
            config.magnum_dir.clone(),
            config.magnum_compiler_script.clone(),
            Arc::clone(&db_arc),
        ));

        Self {
            config: Arc::new(config),
            cache,
            db: db_arc,
            http_client,
            weather: Arc::new(weather::WeatherService::new()),
            earthquake: Arc::new(earthquake::EarthquakeService::new()),
            magnum: magnum_svc,
        }
    }
}

