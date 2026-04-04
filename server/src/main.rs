use std::sync::Arc;

use server::config::AppConfig;
use server::infra::exa::ExaAdapter;
use server::infra::firecrawl::FirecrawlAdapter;
use server::infra::openrouter::OpenRouterAdapter;
use server::infra::search_chain::SearchFallbackChain;
use server::infra::supabase_db::SupabaseDbAdapter;
use server::infra::tavily::TavilyAdapter;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = AppConfig::from_env();
    let db = SupabaseDbAdapter::new(&config);

    let search_chain = SearchFallbackChain::new(vec![
        Box::new(TavilyAdapter::new(&config.tavily_api_key)),
        Box::new(ExaAdapter::new(&config.exa_api_key)),
        Box::new(FirecrawlAdapter::new(&config.firecrawl_api_key)),
    ]);

    let llm = Arc::new(OpenRouterAdapter::new(
        &config.openrouter_api_key,
        &config.llm_model,
    ));

    let app = server::create_router(db, config.supabase_jwt_secret.clone(), search_chain, llm);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("server listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}
