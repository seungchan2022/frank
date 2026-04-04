pub mod articles;
pub mod health;
pub mod tags;

use std::sync::Arc;

use crate::domain::ports::DbPort;
use crate::infra::search_chain::SearchFallbackChain;

#[derive(Debug, Clone)]
pub struct AppState<D: DbPort> {
    pub db: D,
    pub search_chain: Arc<SearchFallbackChain>,
}
