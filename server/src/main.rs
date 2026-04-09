use std::sync::Arc;

use server::config::AppConfig;
use server::domain::ports::SearchChainPort;
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

    check_apple_client_secret_expiry();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("connected to PostgreSQL (pool size: {})", pool.size());

    let db = PostgresDbAdapter::new(pool);

    let search_chain: Arc<dyn SearchChainPort> = Arc::new(SearchFallbackChain::new(vec![
        Box::new(TavilyAdapter::new(&config.tavily_api_key)),
        Box::new(ExaAdapter::new(&config.exa_api_key)),
        Box::new(FirecrawlAdapter::new(&config.firecrawl_api_key)),
    ]));

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

    // MVP5 M1: 백그라운드 스케줄러 제거 — 피드는 GET /me/feed 온디맨드 호출
    let app = server::create_router(
        db,
        supabase_config,
        search_chain,
        llm,
        crawl,
        notifier,
    );

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("server listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}

fn check_apple_client_secret_expiry() {
    let Ok(expires_str) = std::env::var("APPLE_CLIENT_SECRET_EXPIRES_AT") else {
        return;
    };

    match chrono::NaiveDate::parse_from_str(&expires_str, "%Y-%m-%d") {
        Ok(expires_date) => {
            let today = chrono::Utc::now().date_naive();
            let days = (expires_date - today).num_days();

            if days < 0 {
                tracing::error!(
                    expires_at = %expires_str,
                    days_remaining = days,
                    "Apple Client Secret EXPIRED — Apple login is broken. Renew immediately."
                );
            } else if days <= 7 {
                tracing::error!(
                    expires_at = %expires_str,
                    days_remaining = days,
                    "Apple Client Secret renewal CRITICAL — renew immediately."
                );
            } else if days <= 30 {
                tracing::warn!(
                    expires_at = %expires_str,
                    days_remaining = days,
                    "Apple Client Secret renewal window — plan renewal now."
                );
            } else if days <= 60 {
                tracing::info!(
                    expires_at = %expires_str,
                    days_remaining = days,
                    "Apple Client Secret D-60 notice — prepare for renewal."
                );
            }
        }
        Err(_) => {
            tracing::warn!(
                value = %expires_str,
                "APPLE_CLIENT_SECRET_EXPIRES_AT has invalid format. Expected YYYY-MM-DD"
            );
        }
    }
}
