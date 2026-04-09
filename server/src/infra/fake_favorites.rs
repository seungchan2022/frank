use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::FavoritesPort;

type SummaryStore = Arc<Mutex<HashMap<(Uuid, String), (String, String)>>>;

/// 테스트용 인메모리 FavoritesAdapter.
/// call_count로 update_favorite_summary 호출 여부 검증 가능.
#[derive(Debug, Clone)]
pub struct FakeFavoritesAdapter {
    store: SummaryStore,
    should_fail: bool,
    call_count: Arc<Mutex<usize>>,
}

impl FakeFavoritesAdapter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            should_fail: false,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn failing() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            should_fail: true,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn update_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl Default for FakeFavoritesAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FavoritesPort for FakeFavoritesAdapter {
    fn update_favorite_summary<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        summary: &'a str,
        insight: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        let url = url.to_string();
        let summary = summary.to_string();
        let insight = insight.to_string();

        Box::pin(async move {
            *self.call_count.lock().unwrap() += 1;

            if self.should_fail {
                return Err(AppError::Internal("Fake favorites failure".to_string()));
            }

            self.store
                .lock()
                .unwrap()
                .insert((user_id, url), (summary, insight));

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn update_succeeds_and_tracks_call() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .update_favorite_summary(user_id, "https://example.com", "요약", "인사이트")
            .await;

        assert!(result.is_ok());
        assert_eq!(adapter.update_call_count(), 1);
    }

    #[tokio::test]
    async fn failing_returns_error() {
        let adapter = FakeFavoritesAdapter::failing();
        let user_id = Uuid::new_v4();

        let result = adapter
            .update_favorite_summary(user_id, "https://example.com", "요약", "인사이트")
            .await;

        assert!(result.is_err());
        assert_eq!(adapter.update_call_count(), 1);
    }

    #[tokio::test]
    async fn no_favorite_row_still_ok() {
        // url이 favorites에 없어도 에러 없이 Ok(()) 반환
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .update_favorite_summary(user_id, "https://not-in-favorites.com", "요약", "인사이트")
            .await;

        assert!(result.is_ok());
    }
}
