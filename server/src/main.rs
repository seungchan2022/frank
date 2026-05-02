use std::sync::Arc;

use server::config::AppConfig;
use server::domain::models::SearchResult;
use server::domain::ports::{CounterPort, NotificationPort, SearchChainPort, SearchPort};
use server::infra::counted_search::CountedSearchAdapter;
use server::infra::exa::ExaAdapter;
use server::infra::fake_search::FakeSearchAdapter;
use server::infra::feed_cache::InMemoryFeedCache;
use server::infra::firecrawl::FirecrawlAdapter;
use server::infra::groq::GroqAdapter;
use server::infra::imessage::{ImessageAdapter, LogOnlyNotificationAdapter};
use server::infra::in_memory_counter::InMemoryCounter;
use server::infra::postgres_counters::PostgresCounterAdapter;
use server::infra::postgres_db::PostgresDbAdapter;
use server::infra::postgres_favorites::PostgresFavoritesAdapter;
use server::infra::postgres_quiz_wrong_answers::PostgresQuizWrongAnswerAdapter;
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

    let db = PostgresDbAdapter::new(pool.clone());

    // MVP15 M2: 알림 어댑터 — 검색 체인 wrap에 필요하므로 먼저 구성
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

    // MVP15 M2 S6: FRANK_DEV_MOCK_SEARCH 토글 + S2: CountedSearchAdapter wrap
    // - mock 모드: FakeSearchAdapter + InMemoryCounter (DB 미오염)
    // - real 모드: 실제 어댑터 + PostgresCounterAdapter
    // 두 경로 모두 CountedSearchAdapter로 wrap → 데코레이터 path 검증 일관성 (#5)
    let mock_search = std::env::var("FRANK_DEV_MOCK_SEARCH")
        .map(|v| v == "1")
        .unwrap_or(false);

    let counter: Arc<dyn CounterPort> = if mock_search {
        Arc::new(InMemoryCounter::new())
    } else {
        Arc::new(PostgresCounterAdapter::new(pool.clone()))
    };

    let search_chain: Arc<dyn SearchChainPort> =
        Arc::new(SearchFallbackChain::new(build_search_sources(
            &config,
            mock_search,
            Arc::clone(&counter),
            Arc::clone(&notifier),
        )));

    if mock_search {
        tracing::warn!("⚠️  MOCK SEARCH MODE — FRANK_DEV_MOCK_SEARCH=1, FakeSearchAdapter 사용");
    } else {
        tracing::info!("real search mode — Tavily + Exa + Firecrawl 체인");
    }

    let llm = Arc::new(GroqAdapter::new(&config.groq_api_key));

    let supabase_config = SupabaseConfig {
        url: config.supabase_url.clone(),
        anon_key: config.supabase_anon_key.clone(),
    };

    let crawl: Arc<dyn server::domain::ports::CrawlPort> =
        Arc::new(FirecrawlAdapter::new(&config.firecrawl_api_key));

    let favorites: Arc<dyn server::domain::ports::FavoritesPort> =
        Arc::new(PostgresFavoritesAdapter::new(pool.clone()));

    let quiz_wrong_answers: Arc<dyn server::domain::ports::QuizWrongAnswerPort> =
        Arc::new(PostgresQuizWrongAnswerAdapter::new(pool.clone()));

    let feed_cache: Arc<dyn server::domain::ports::FeedCachePort> =
        Arc::new(InMemoryFeedCache::new(100));

    // MVP5 M1: 백그라운드 스케줄러 제거 — 피드는 GET /me/feed 온디맨드 호출
    let app = server::create_router(
        db,
        supabase_config,
        search_chain,
        llm,
        crawl,
        notifier,
        favorites,
        quiz_wrong_answers,
        feed_cache,
        counter,
    );

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("server listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}

/// MVP15 M2: 검색 소스 체인 구성.
///
/// - mock 모드: 단일 FakeSearchAdapter (DB 미오염, R2 가드)
/// - real 모드: Tavily + Exa + Firecrawl 순서 폴백
///
/// 두 경로 모두 CountedSearchAdapter로 wrap → 데코레이터 path 검증 일관성 (#5).
fn build_search_sources(
    config: &AppConfig,
    mock: bool,
    counter: Arc<dyn CounterPort>,
    notifier: Arc<dyn NotificationPort>,
) -> Vec<Box<dyn SearchPort>> {
    if mock {
        // mock 모드: 단일 FakeSearchAdapter (1건 결과)
        let fake = FakeSearchAdapter::new(
            "tavily", // 카운터 PK 일관성 위해 실제 엔진명 사용
            vec![SearchResult {
                title: "MOCK 결과".to_string(),
                url: "https://example.com/news/mock-article".to_string(),
                snippet: Some("MOCK SEARCH MODE: 실제 API 호출 없음".to_string()),
                published_at: None,
                image_url: None,
            }],
            false,
        );
        vec![Box::new(CountedSearchAdapter::new(
            Box::new(fake),
            Arc::clone(&counter),
            Arc::clone(&notifier),
        ))]
    } else {
        // S1: 어댑터별 max_cap 주입. limit 5→20 (Tavily), 5→10 (Exa 추정), 5 유지 (Firecrawl)
        let tavily = TavilyAdapter::new(&config.tavily_api_key).with_max_cap(20);
        let exa = ExaAdapter::new(&config.exa_api_key).with_max_cap(10);
        let firecrawl = FirecrawlAdapter::new(&config.firecrawl_api_key).with_max_cap(5);
        vec![
            Box::new(CountedSearchAdapter::new(
                Box::new(tavily),
                Arc::clone(&counter),
                Arc::clone(&notifier),
            )),
            Box::new(CountedSearchAdapter::new(
                Box::new(exa),
                Arc::clone(&counter),
                Arc::clone(&notifier),
            )),
            Box::new(CountedSearchAdapter::new(
                Box::new(firecrawl),
                Arc::clone(&counter),
                Arc::clone(&notifier),
            )),
        ]
    }
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
