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
    CrawlPort, DbPort, FavoritesPort, LlmPort, NotificationPort, QuizWrongAnswerPort,
    SearchChainPort,
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
        };

        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
        assert!(debug_str.contains("<dyn SearchChainPort>"));
        assert!(debug_str.contains("<dyn LlmPort>"));
        assert!(debug_str.contains("<dyn CrawlPort>"));
        assert!(debug_str.contains("<dyn NotificationPort>"));
    }
}
