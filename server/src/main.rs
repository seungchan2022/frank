use std::sync::Arc;

use server::config::AppConfig;
use server::infra::exa::ExaAdapter;
use server::infra::firecrawl::FirecrawlAdapter;
use server::infra::imessage::{ImessageAdapter, LogOnlyNotificationAdapter};
use server::infra::openrouter::OpenRouterAdapter;
use server::infra::postgres_db::PostgresDbAdapter;
use server::infra::search_chain::SearchFallbackChain;
use server::infra::tavily::TavilyAdapter;
use server::middleware::auth::SupabaseConfig;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = AppConfig::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("connected to PostgreSQL (pool size: {})", pool.size());

    let db = PostgresDbAdapter::new(pool);

    let search_chain = SearchFallbackChain::new(vec![
        Box::new(TavilyAdapter::new(&config.tavily_api_key)),
        Box::new(ExaAdapter::new(&config.exa_api_key)),
        Box::new(FirecrawlAdapter::new(&config.firecrawl_api_key)),
    ]);

    let llm = Arc::new(OpenRouterAdapter::new(
        &config.openrouter_api_key,
        &config.llm_model,
    ));

    let supabase_config = SupabaseConfig {
        url: config.supabase_url.clone(),
        anon_key: config.supabase_anon_key.clone(),
    };

    let crawl: Arc<dyn server::domain::ports::CrawlPort> =
        Arc::new(FirecrawlAdapter::new(&config.firecrawl_api_key));

    // iMessage 알림: IMESSAGE_RECIPIENT 환경변수가 설정된 경우만 활성화
    // Docker 컨테이너 내부에서는 osascript 미지원이므로 LogOnly 사용
    let notifier: Arc<dyn server::domain::ports::NotificationPort> =
        match std::env::var("IMESSAGE_RECIPIENT") {
            Ok(recipient) if !recipient.is_empty() => {
                tracing::info!(recipient = %recipient, "iMessage 알림 활성화");
                Arc::new(ImessageAdapter::new(&recipient))
            }
            _ => {
                tracing::info!("iMessage 알림 비활성화 (IMESSAGE_RECIPIENT 미설정)");
                Arc::new(LogOnlyNotificationAdapter)
            }
        };

    let app = server::create_router(db, supabase_config, search_chain, llm, crawl, notifier);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("server listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}
