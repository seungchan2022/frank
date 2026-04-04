pub mod articles;
pub mod health;
pub mod tags;

use std::sync::Arc;

use crate::domain::ports::{CrawlPort, DbPort, LlmPort, NotificationPort};
use crate::infra::search_chain::SearchFallbackChain;

#[derive(Clone)]
pub struct AppState<D: DbPort> {
    pub db: D,
    pub search_chain: Arc<SearchFallbackChain>,
    pub llm: Arc<dyn LlmPort>,
    pub crawl: Arc<dyn CrawlPort>,
    pub notifier: Arc<dyn NotificationPort>,
}

impl<D: DbPort + std::fmt::Debug> std::fmt::Debug for AppState<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db", &self.db)
            .field("search_chain", &self.search_chain)
            .field("llm", &"<dyn LlmPort>")
            .field("crawl", &"<dyn CrawlPort>")
            .field("notifier", &"<dyn NotificationPort>")
            .finish()
    }
}
