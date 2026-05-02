pub mod favorites;
pub mod feed;
pub mod health;
pub mod likes;
pub mod profile;
pub mod quiz;
pub mod quiz_wrong_answers;
pub mod related;
pub mod summarize;
pub mod tags;

use std::sync::Arc;

use crate::domain::ports::{
    CounterPort, CrawlPort, DbPort, FavoritesPort, FeedCachePort, LlmPort, NotificationPort,
    QuizWrongAnswerPort, SearchChainPort,
};

#[derive(Clone)]
pub struct AppState<D: DbPort> {
    pub db: D,
    pub search_chain: Arc<dyn SearchChainPort>,
    pub llm: Arc<dyn LlmPort>,
    pub crawl: Arc<dyn CrawlPort>,
    pub notifier: Arc<dyn NotificationPort>,
    pub favorites: Arc<dyn FavoritesPort>,
    pub quiz_wrong_answers: Arc<dyn QuizWrongAnswerPort>,
    pub feed_cache: Arc<dyn FeedCachePort>,
    /// MVP15 M2: 엔진별 월간 호출 카운터 (한도 보호 인프라).
    /// 사용처: 80% 도달 시 캐시 TTL 강화 (S3), 셋 다 한도 시 회복 시각 안내 (S5).
    /// 100% 차단 + 알림은 `CountedSearchAdapter`(데코레이터)에서 처리.
    pub counter: Arc<dyn CounterPort>,
}

impl<D: DbPort + std::fmt::Debug> std::fmt::Debug for AppState<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db", &self.db)
            .field("search_chain", &"<dyn SearchChainPort>")
            .field("llm", &"<dyn LlmPort>")
            .field("crawl", &"<dyn CrawlPort>")
            .field("notifier", &"<dyn NotificationPort>")
            .field("favorites", &"<dyn FavoritesPort>")
            .field("quiz_wrong_answers", &"<dyn QuizWrongAnswerPort>")
            .field("feed_cache", &"<dyn FeedCachePort>")
            .field("counter", &"<dyn CounterPort>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_favorites::FakeFavoritesAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use crate::infra::fake_quiz_wrong_answers::FakeQuizWrongAnswerAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;
    use crate::infra::feed_cache::NoopFeedCache;
    use crate::infra::in_memory_counter::InMemoryCounter;
    use crate::infra::search_chain::SearchFallbackChain;

    #[test]
    fn app_state_debug_format() {
        let db = FakeDbAdapter::new();
        let chain = SearchFallbackChain::new(vec![Box::new(FakeSearchAdapter::new(
            "test",
            vec![],
            false,
        ))]);
        let state = AppState {
            db,
            search_chain: Arc::new(chain) as Arc<dyn SearchChainPort>,
            llm: Arc::new(FakeLlmAdapter::new()),
            crawl: Arc::new(FakeCrawlAdapter::new()),
            notifier: Arc::new(FakeNotificationAdapter::new()),
            favorites: Arc::new(FakeFavoritesAdapter::new()),
            quiz_wrong_answers: Arc::new(FakeQuizWrongAnswerAdapter::new()),
            feed_cache: Arc::new(NoopFeedCache),
            counter: Arc::new(InMemoryCounter::new()),
        };

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
        assert!(debug_str.contains("<dyn SearchChainPort>"));
        assert!(debug_str.contains("<dyn LlmPort>"));
        assert!(debug_str.contains("<dyn CrawlPort>"));
        assert!(debug_str.contains("<dyn NotificationPort>"));
        assert!(debug_str.contains("<dyn FeedCachePort>"));
        assert!(debug_str.contains("<dyn CounterPort>"));
    }
}
