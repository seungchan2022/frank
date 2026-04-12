use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Favorite, QuizConcept};
use crate::domain::ports::FavoritesPort;

/// should_fail 시 반환할 에러 메시지 상수.
const FAKE_FAIL_MSG: &str = "Fake favorites failure";

/// 인메모리 favorites 저장소.
/// 키: (user_id, url) → Favorite.
/// 삽입 순서 추적을 위해 별도 Vec 보관.
type FavoriteStore = Arc<Mutex<(HashMap<(Uuid, String), Favorite>, Vec<(Uuid, String)>)>>;

/// 테스트용 인메모리 FavoritesAdapter.
/// call_count로 update_favorite_summary 호출 여부 검증 가능.
#[derive(Debug, Clone)]
pub struct FakeFavoritesAdapter {
    store: FavoriteStore,
    should_fail: bool,
    call_count: Arc<Mutex<usize>>,
}

impl FakeFavoritesAdapter {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new((HashMap::new(), Vec::new()))),
            should_fail: false,
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn failing() -> Self {
        Self {
            store: Arc::new(Mutex::new((HashMap::new(), Vec::new()))),
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
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            // url이 favorites에 없으면 no-op (M2 의미 유지)
            let mut guard = self.store.lock().unwrap();
            if let Some(fav) = guard.0.get_mut(&(user_id, url)) {
                fav.summary = Some(summary);
                fav.insight = Some(insight);
            }

            Ok(())
        })
    }

    fn add_favorite<'a>(
        &'a self,
        user_id: Uuid,
        item: &'a Favorite,
    ) -> Pin<Box<dyn Future<Output = Result<Favorite, AppError>> + Send + 'a>> {
        let item = item.clone();
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            let key = (user_id, item.url.clone());
            let mut guard = self.store.lock().unwrap();

            if guard.0.contains_key(&key) {
                return Err(AppError::Conflict(
                    "이미 즐겨찾기에 추가된 기사입니다.".to_string(),
                ));
            }

            let now = Utc::now();
            let favorite = Favorite {
                id: Uuid::new_v4(),
                user_id,
                title: item.title.clone(),
                url: item.url.clone(),
                snippet: item.snippet.clone(),
                source: item.source.clone(),
                published_at: item.published_at,
                tag_id: item.tag_id,
                summary: item.summary.clone(),
                insight: item.insight.clone(),
                liked_at: Some(now),
                created_at: Some(now),
                image_url: item.image_url.clone(),
                concepts: None,
            };

            guard.1.push(key.clone());
            guard.0.insert(key, favorite.clone());
            Ok(favorite)
        })
    }

    fn delete_favorite<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        let url = url.to_string();
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            let key = (user_id, url);
            let mut guard = self.store.lock().unwrap();
            guard.0.remove(&key);
            guard.1.retain(|k| k != &key);
            Ok(())
        })
    }

    fn list_favorites(
        &self,
        user_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Favorite>, AppError>> + Send + '_>> {
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            let guard = self.store.lock().unwrap();
            // 삽입 역순으로 반환 (created_at DESC 모사)
            let mut items: Vec<Favorite> = guard
                .1
                .iter()
                .filter(|(uid, _)| *uid == user_id)
                .filter_map(|k| guard.0.get(k).cloned())
                .collect();
            items.reverse();
            Ok(items)
        })
    }

    fn get_favorite_by_url<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Favorite>, AppError>> + Send + 'a>> {
        let url = url.to_string();
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            let guard = self.store.lock().unwrap();
            Ok(guard.0.get(&(user_id, url)).cloned())
        })
    }

    fn update_favorite_concepts<'a>(
        &'a self,
        user_id: Uuid,
        url: &'a str,
        concepts: Vec<QuizConcept>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        let url = url.to_string();
        Box::pin(async move {
            if self.should_fail {
                return Err(AppError::Internal(FAKE_FAIL_MSG.to_string()));
            }

            let concepts_value = serde_json::to_value(&concepts)
                .map_err(|e| AppError::Internal(format!("concepts serialize failed: {e}")))?;

            let mut guard = self.store.lock().unwrap();
            if let Some(fav) = guard.0.get_mut(&(user_id, url)) {
                fav.concepts = Some(concepts_value);
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Favorite;

    fn make_favorite(url: &str) -> Favorite {
        Favorite {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            title: "테스트 기사".to_string(),
            url: url.to_string(),
            snippet: None,
            source: "테스트".to_string(),
            published_at: None,
            tag_id: None,
            summary: None,
            insight: None,
            liked_at: None,
            created_at: None,
            image_url: None,
            concepts: None,
        }
    }

    #[tokio::test]
    async fn update_succeeds_and_tracks_call() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let url = "https://example.com";

        // update_call_count 검증을 위해 먼저 add_favorite으로 행을 삽입
        let mut item = make_favorite(url);
        item.user_id = user_id;
        adapter.add_favorite(user_id, &item).await.unwrap();

        let result = adapter
            .update_favorite_summary(user_id, url, "요약", "인사이트")
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
        // url이 favorites에 없어도 에러 없이 Ok(()) 반환 (no-op)
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .update_favorite_summary(user_id, "https://not-in-favorites.com", "요약", "인사이트")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_favorite_returns_favorite_with_id() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let item = make_favorite("https://example.com");

        let result = adapter.add_favorite(user_id, &item).await.unwrap();

        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.user_id, user_id);
        assert!(result.created_at.is_some());
    }

    #[tokio::test]
    async fn add_favorite_duplicate_returns_conflict() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let item = make_favorite("https://example.com");

        adapter.add_favorite(user_id, &item).await.unwrap();
        let result = adapter.add_favorite(user_id, &item).await;

        assert!(matches!(result, Err(AppError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_favorite_removes_item() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let item = make_favorite("https://example.com");

        adapter.add_favorite(user_id, &item).await.unwrap();
        adapter
            .delete_favorite(user_id, "https://example.com")
            .await
            .unwrap();

        let list = adapter.list_favorites(user_id).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_ok() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .delete_favorite(user_id, "https://not-exist.com")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn list_favorites_returns_desc_order() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        adapter
            .add_favorite(user_id, &make_favorite("https://first.com"))
            .await
            .unwrap();
        adapter
            .add_favorite(user_id, &make_favorite("https://second.com"))
            .await
            .unwrap();

        let list = adapter.list_favorites(user_id).await.unwrap();
        assert_eq!(list.len(), 2);
        // 역순: second → first
        assert_eq!(list[0].url, "https://second.com");
        assert_eq!(list[1].url, "https://first.com");
    }

    #[tokio::test]
    async fn list_favorites_only_returns_own_items() {
        let adapter = FakeFavoritesAdapter::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        adapter
            .add_favorite(user1, &make_favorite("https://user1.com"))
            .await
            .unwrap();
        adapter
            .add_favorite(user2, &make_favorite("https://user2.com"))
            .await
            .unwrap();

        let list = adapter.list_favorites(user1).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].url, "https://user1.com");
    }

    #[tokio::test]
    async fn get_favorite_by_url_returns_favorite() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let url = "https://example.com/article";
        let mut item = make_favorite(url);
        item.user_id = user_id;
        adapter.add_favorite(user_id, &item).await.unwrap();

        let result = adapter.get_favorite_by_url(user_id, url).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().url, url);
    }

    #[tokio::test]
    async fn get_favorite_by_url_returns_none_when_not_found() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .get_favorite_by_url(user_id, "https://not-exist.com")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn update_favorite_concepts_sets_concepts() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();
        let url = "https://example.com/article";
        let mut item = make_favorite(url);
        item.user_id = user_id;
        adapter.add_favorite(user_id, &item).await.unwrap();

        let concepts = vec![
            crate::domain::models::QuizConcept {
                term: "Swift".to_string(),
                explanation: "애플 프로그래밍 언어".to_string(),
            },
        ];
        adapter
            .update_favorite_concepts(user_id, url, concepts)
            .await
            .unwrap();

        let fav = adapter.get_favorite_by_url(user_id, url).await.unwrap().unwrap();
        assert!(fav.concepts.is_some());
    }

    #[tokio::test]
    async fn update_favorite_concepts_no_row_is_noop() {
        let adapter = FakeFavoritesAdapter::new();
        let user_id = Uuid::new_v4();

        let result = adapter
            .update_favorite_concepts(user_id, "https://not-exist.com", vec![])
            .await;
        assert!(result.is_ok());
    }
}
