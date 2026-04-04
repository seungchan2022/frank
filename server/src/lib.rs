pub mod api;
pub mod config;
pub mod domain;
pub mod infra;
pub mod middleware;
pub mod services;

use std::sync::Arc;

use axum::middleware::from_fn;
use axum::routing::{get, post};
use axum::{Extension, Router};

use api::AppState;
use domain::ports::{CrawlPort, DbPort, LlmPort, NotificationPort};
use infra::search_chain::SearchFallbackChain;
use middleware::auth::SupabaseConfig;

pub fn create_router<D: DbPort + Clone + 'static>(
    db: D,
    supabase_config: SupabaseConfig,
    search_chain: SearchFallbackChain,
    llm: Arc<dyn LlmPort>,
    crawl: Arc<dyn CrawlPort>,
    notifier: Arc<dyn NotificationPort>,
) -> Router {
    let state = AppState {
        db,
        search_chain: Arc::new(search_chain),
        llm,
        crawl,
        notifier,
    };

    let auth_routes = Router::new()
        .route("/me/profile", get(api::tags::get_my_profile::<D>))
        .route("/tags", get(api::tags::list_tags::<D>))
        .route("/me/tags", get(api::tags::get_my_tags::<D>))
        .route("/me/tags", post(api::tags::save_my_tags::<D>))
        .route("/me/collect", post(api::articles::collect_articles::<D>))
        .route("/me/articles", get(api::articles::list_articles::<D>))
        .route(
            "/me/summarize",
            post(api::articles::summarize_articles::<D>),
        )
        .layer(from_fn(middleware::auth::require_auth))
        .layer(Extension(supabase_config));

    Router::new()
        .route("/health", get(api::health::health_check))
        .nest("/api", auth_routes)
        .layer(Extension(state))
}
