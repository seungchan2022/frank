pub mod api;
pub mod domain;
pub mod infra;
pub mod services;

use axum::Router;
use axum::routing::get;

pub fn create_router() -> Router {
    Router::new().route("/health", get(api::health::health_check))
}
