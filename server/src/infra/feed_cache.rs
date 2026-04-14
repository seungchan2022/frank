use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::domain::models::FeedItem;
use crate::domain::ports::FeedCachePort;

// ── NoopFeedCache ──────────────────────────────────────────────────────────────

/// 테스트 전용 캐시 — 항상 MISS, set/invalidate no-op.
/// 기존 테스트의 결정적 동작을 보장한다.
#[derive(Debug, Clone, Default)]
pub struct NoopFeedCache;

impl FeedCachePort for NoopFeedCache {
    fn get(&self, _key: &str) -> Option<Vec<FeedItem>> {
        None
    }

    fn set(&self, _key: &str, _items: Vec<FeedItem>, _ttl: Duration) {}

    fn invalidate_user(&self, _user_id: Uuid) {}
}

// ── InMemoryFeedCache ──────────────────────────────────────────────────────────

struct CacheEntry {
    items: Vec<FeedItem>,
    expires_at: Instant,
    inserted_at: Instant,
}

impl CacheEntry {
    fn new(items: Vec<FeedItem>, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            items,
            expires_at: now + ttl,
            inserted_at: now,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// 프로덕션용 TTL 인메모리 피드 캐시.
///
/// - 최대 `max_entries`개 엔트리 유지 (초과 시 만료 → 가장 오래된 순으로 제거)
/// - 캐시 키: `"{user_id}:{sorted_tag_ids}"` 형식
/// - `invalidate_user(user_id)`: 해당 `user_id` prefix 키 전체 제거
pub struct InMemoryFeedCache {
    store: Mutex<HashMap<String, CacheEntry>>,
    max_entries: usize,
}

impl InMemoryFeedCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
            max_entries,
        }
    }

    /// 만료 엔트리 우선 제거. 모두 유효하면 가장 오래된 엔트리 1개 제거.
    fn evict_if_needed(store: &mut HashMap<String, CacheEntry>, max_entries: usize) {
        if store.len() < max_entries {
            return;
        }
        // 만료 엔트리 먼저 제거
        store.retain(|_, v| !v.is_expired());
        if store.len() < max_entries {
            return;
        }
        // 여전히 초과 — 가장 오래된 엔트리 1개 제거
        let oldest_key = store
            .iter()
            .min_by_key(|(_, v)| v.inserted_at)
            .map(|(k, _)| k.clone());
        if let Some(key) = oldest_key {
            store.remove(&key);
        }
    }
}

impl FeedCachePort for InMemoryFeedCache {
    fn get(&self, key: &str) -> Option<Vec<FeedItem>> {
        let store = self.store.lock().expect("feed cache lock poisoned");
        store.get(key).and_then(|entry| {
            if entry.is_expired() {
                tracing::debug!(cache_key = %key, "feed cache MISS (expired)");
                None
            } else {
                tracing::debug!(cache_key = %key, "feed cache HIT");
                Some(entry.items.clone())
            }
        })
    }

    fn set(&self, key: &str, items: Vec<FeedItem>, ttl: Duration) {
        let mut store = self.store.lock().expect("feed cache lock poisoned");
        Self::evict_if_needed(&mut store, self.max_entries);
        tracing::debug!(cache_key = %key, ttl_secs = ttl.as_secs(), "feed cache SET");
        store.insert(key.to_string(), CacheEntry::new(items, ttl));
    }

    fn invalidate_user(&self, user_id: Uuid) {
        let prefix = user_id.to_string();
        let mut store = self.store.lock().expect("feed cache lock poisoned");
        store.retain(|k, _| !k.starts_with(&prefix));
        tracing::debug!(user_id = %user_id, "feed cache INVALIDATE user");
    }
}

// ── 단위 테스트 ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::FeedItem;
    use chrono::Utc;

    fn sample_items(n: usize) -> Vec<FeedItem> {
        (0..n)
            .map(|i| FeedItem {
                title: format!("Article {i}"),
                url: format!("https://example.com/article/{i}"),
                snippet: None,
                source: "test".to_string(),
                published_at: Some(Utc::now()),
                tag_id: None,
                image_url: None,
            })
            .collect()
    }

    #[test]
    fn noop_always_returns_none() {
        let cache = NoopFeedCache;
        assert!(cache.get("any-key").is_none());
        cache.set("any-key", sample_items(1), Duration::from_secs(300));
        assert!(cache.get("any-key").is_none());
    }

    #[test]
    fn set_then_get_returns_items() {
        let cache = InMemoryFeedCache::new(10);
        let items = sample_items(3);
        cache.set("u1:ai", items.clone(), Duration::from_secs(300));
        let result = cache.get("u1:ai").expect("캐시 HIT여야 함");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].title, "Article 0");
    }

    #[test]
    fn miss_on_unknown_key() {
        let cache = InMemoryFeedCache::new(10);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn expired_entry_returns_none() {
        let cache = InMemoryFeedCache::new(10);
        // TTL = 0ms → 삽입 즉시 만료
        cache.set("u1:ai", sample_items(2), Duration::from_millis(0));
        // 짧은 sleep 없이도 Instant::now() >= expires_at 보장 (ttl=0이므로)
        std::thread::sleep(Duration::from_millis(1));
        assert!(cache.get("u1:ai").is_none(), "TTL 만료 후 MISS여야 함");
    }

    #[test]
    fn invalidate_user_removes_all_user_entries() {
        let cache = InMemoryFeedCache::new(10);
        let user_id = Uuid::new_v4();
        let other_id = Uuid::new_v4();
        let key_all = format!("{user_id}:all");
        let key_ai = format!("{user_id}:ai");
        let key_other = format!("{other_id}:all");

        cache.set(&key_all, sample_items(1), Duration::from_secs(300));
        cache.set(&key_ai, sample_items(2), Duration::from_secs(300));
        cache.set(&key_other, sample_items(1), Duration::from_secs(300));

        cache.invalidate_user(user_id);

        assert!(cache.get(&key_all).is_none(), "user 캐시 전체 무효화");
        assert!(cache.get(&key_ai).is_none(), "user 캐시 전체 무효화");
        assert!(
            cache.get(&key_other).is_some(),
            "다른 user 캐시는 유지되어야 함"
        );
    }

    #[test]
    fn user_isolation_different_users_separate_entries() {
        let cache = InMemoryFeedCache::new(10);
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let items_a = sample_items(1);
        let items_b = sample_items(2);

        cache.set(&format!("{user_a}:all"), items_a, Duration::from_secs(300));
        cache.set(&format!("{user_b}:all"), items_b, Duration::from_secs(300));

        let result_a = cache.get(&format!("{user_a}:all")).unwrap();
        let result_b = cache.get(&format!("{user_b}:all")).unwrap();
        assert_eq!(result_a.len(), 1);
        assert_eq!(result_b.len(), 2, "다른 사용자 캐시는 독립적이어야 함");
    }

    #[test]
    fn max_entries_expired_evicted_first() {
        let cache = InMemoryFeedCache::new(3);
        // 만료 엔트리 1개 삽입 (ttl=0)
        cache.set("u1:expired", sample_items(1), Duration::from_millis(0));
        std::thread::sleep(Duration::from_millis(1));
        // 유효 엔트리 3개 삽입 → 총 4개 → 만료 엔트리 제거 후 3개 유지
        cache.set("u2:all", sample_items(1), Duration::from_secs(300));
        cache.set("u3:all", sample_items(1), Duration::from_secs(300));
        cache.set("u4:all", sample_items(1), Duration::from_secs(300));

        // 유효 엔트리 3개 모두 접근 가능해야 함
        assert!(cache.get("u2:all").is_some());
        assert!(cache.get("u3:all").is_some());
        assert!(cache.get("u4:all").is_some());
    }

    #[test]
    fn max_entries_all_valid_oldest_evicted() {
        let cache = InMemoryFeedCache::new(2);
        cache.set("u1:all", sample_items(1), Duration::from_secs(300));
        std::thread::sleep(Duration::from_millis(1));
        cache.set("u2:all", sample_items(1), Duration::from_secs(300));
        // 3번째 삽입 → max_entries(2) 초과 → 가장 오래된 u1:all 제거
        cache.set("u3:all", sample_items(1), Duration::from_secs(300));

        assert!(cache.get("u2:all").is_some(), "두 번째 엔트리는 유지");
        assert!(cache.get("u3:all").is_some(), "세 번째 엔트리는 유지");
    }
}
