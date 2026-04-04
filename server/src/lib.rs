pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod middleware;
pub mod services;

use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::{Extension, Router};

use api::AppState;
use domain::ports::DbPort;

pub fn create_router<D: DbPort + Clone + 'static>(db: D, jwt_secret: String) -> Router {
    let state = AppState { db };

    let auth_routes = Router::new()
        .route("/me/profile", get(api::tags::get_my_profile::<D>))
        .route("/tags", get(api::tags::list_tags::<D>))
        .route("/me/tags", get(api::tags::get_my_tags::<D>))
        .route("/me/tags", post(api::tags::save_my_tags::<D>))
        .layer(from_fn(middleware::auth::require_auth))
        .layer(Extension(jwt_secret));

    Router::new()
        .route("/health", get(api::health::health_check))
        .nest("/api", auth_routes)
        .layer(Extension(state))
}
