use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Article, Favorite, Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct FakeDbAdapter {
    profiles: Arc<Mutex<HashMap<Uuid, Profile>>>,
    tags: Arc<Mutex<Vec<Tag>>>,
    user_tags: Arc<Mutex<Vec<UserTag>>>,
    articles: Arc<Mutex<Vec<Article>>>,
    // MVP5 M3에서 favorites 엔드포인트 구현 시 사용
    #[allow(dead_code)]
    favorites: Arc<Mutex<Vec<Favorite>>>,
}

impl Default for FakeDbAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeDbAdapter {
    pub fn new() -> Self {
        let tags = vec![
            Tag {
                id: Uuid::new_v4(),
                name: "AI/ML".to_string(),
                category: Some("기술".to_string()),
            },
            Tag {
                id: Uuid::new_v4(),
                name: "웹 개발".to_string(),
                category: Some("기술".to_string()),
            },
            Tag {
                id: Uuid::new_v4(),
                name: "스타트업".to_string(),
                category: Some("비즈니스".to_string()),
            },
        ];

        Self {
            profiles: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(tags)),
            user_tags: Arc::new(Mutex::new(Vec::new())),
            articles: Arc::new(Mutex::new(Vec::new())),
            favorites: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn seed_profile(&self, profile: Profile) {
        self.profiles.lock().unwrap().insert(profile.id, profile);
    }

    pub fn seed_article(&self, article: Article) {
        self.articles.lock().unwrap().push(article);
    }

    pub fn get_tags(&self) -> Vec<Tag> {
        self.tags.lock().unwrap().clone()
    }

    pub fn seed_user_tag(&self, user_id: Uuid, tag_id: Uuid) {
        self.user_tags
            .lock()
            .unwrap()
            .push(UserTag { user_id, tag_id });
    }

    /// 유저의 활성 태그 ID 집합을 반환한다.
    ///
    /// `user_tags` 락을 취득해 `HashSet<Uuid>`로 materialize한 뒤
    /// 함수 종료 시 락이 해제된다. 호출부에서 이후 `articles` 락을
    /// 취득해도 중첩 락이 발생하지 않으므로 deadlock이 방지된다.
    fn active_tag_ids_for(&self, user_id: Uuid) -> HashSet<Uuid> {
        let user_tags = self.user_tags.lock().unwrap();
        user_tags
            .iter()
            .filter(|t| t.user_id == user_id)
            .map(|t| t.tag_id)
            .collect()
    }
}

impl DbPort for FakeDbAdapter {
    async fn get_profile(&self, user_id: Uuid) -> Result<Profile, AppError> {
        self.profiles
            .lock()
            .unwrap()
            .get(&user_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    async fn update_profile_onboarding(
        &self,
        user_id: Uuid,
        completed: bool,
    ) -> Result<(), AppError> {
        let mut profiles = self.profiles.lock().unwrap();
        let profile = profiles
            .get_mut(&user_id)
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;
        profile.onboarding_completed = completed;
        Ok(())
    }

    async fn update_profile(
        &self,
        user_id: Uuid,
        onboarding_completed: Option<bool>,
        display_name: Option<String>,
    ) -> Result<Profile, AppError> {
        let mut profiles = self.profiles.lock().unwrap();
        let profile = profiles
            .get_mut(&user_id)
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;
        if let Some(oc) = onboarding_completed {
            profile.onboarding_completed = oc;
        }
        if let Some(dn) = display_name {
            profile.display_name = Some(dn);
        }
        Ok(profile.clone())
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, AppError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    async fn get_user_tags(&self, user_id: Uuid) -> Result<Vec<UserTag>, AppError> {
        let tags = self.user_tags.lock().unwrap();
        Ok(tags
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn set_user_tags(&self, user_id: Uuid, tag_ids: Vec<Uuid>) -> Result<(), AppError> {
        let mut user_tags = self.user_tags.lock().unwrap();
        user_tags.retain(|t| t.user_id != user_id);
        for tag_id in tag_ids {
            user_tags.push(UserTag { user_id, tag_id });
        }
        Ok(())
    }

    async fn save_articles(&self, new_articles: Vec<Article>) -> Result<usize, AppError> {
        let mut articles = self.articles.lock().unwrap();
        let count = new_articles.len();
        for article in new_articles {
            if !articles
                .iter()
                .any(|a| a.url == article.url && a.user_id == article.user_id)
            {
                articles.push(article);
            }
        }
        Ok(count)
    }

    async fn get_user_articles(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        tag_id: Option<Uuid>,
    ) -> Result<Vec<Article>, AppError> {
        let active_tag_ids = self.active_tag_ids_for(user_id);
        let articles = self.articles.lock().unwrap();
        // Postgres와 동일하게 created_at DESC 정렬 후 offset/limit 적용
        let mut filtered: Vec<_> = articles
            .iter()
            .filter(|a| a.user_id == user_id)
            .filter(|a| match tag_id {
                // 특정 태그: 활성 태그인 경우만
                Some(tid) => a.tag_id == Some(tid) && active_tag_ids.contains(&tid),
                // 전체 피드: 현재 활성 태그 기사만
                None => a.tag_id.is_some_and(|tid| active_tag_ids.contains(&tid)),
            })
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        let user_articles: Vec<_> = filtered
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect();
        Ok(user_articles)
    }

    async fn get_user_article_by_id(
        &self,
        user_id: Uuid,
        article_id: Uuid,
    ) -> Result<Option<Article>, AppError> {
        let active_tag_ids = self.active_tag_ids_for(user_id);
        let articles = self.articles.lock().unwrap();
        Ok(articles
            .iter()
            .find(|a| {
                a.id == article_id
                    && a.user_id == user_id
                    && a.tag_id.is_some_and(|tid| active_tag_ids.contains(&tid))
            })
            .cloned())
    }

    async fn get_all_user_ids(&self) -> Result<Vec<Uuid>, AppError> {
        // user_tags와 profiles를 순서대로 락 취득 (deadlock 방지: 항상 동일 순서)
        let from_tags: HashSet<Uuid> = {
            let user_tags = self.user_tags.lock().unwrap();
            user_tags.iter().map(|t| t.user_id).collect()
        };
        let from_profiles: HashSet<Uuid> = {
            let profiles = self.profiles.lock().unwrap();
            profiles.keys().copied().collect()
        };
        Ok(from_tags.union(&from_profiles).copied().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_article(user_id: Uuid, tag_id: Uuid, url: &str) -> Article {
        Article {
            id: Uuid::new_v4(),
            user_id,
            tag_id: Some(tag_id),
            title: url.to_string(),
            url: url.to_string(),
            snippet: None,
            source: "test".to_string(),
            published_at: None,
            created_at: None,
        }
    }

    // 태그 삭제 후 전체 피드에서 stale 기사 제외 검증
    #[tokio::test]
    async fn get_user_articles_excludes_removed_tag_in_all_feed() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.list_tags().await.unwrap();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;

        db.set_user_tags(user_id, vec![tag_a, tag_b]).await.unwrap();
        db.seed_article(make_article(user_id, tag_a, "https://a.com"));
        db.seed_article(make_article(user_id, tag_b, "https://b.com"));

        // 삭제 전: 2개
        let before = db.get_user_articles(user_id, 10, 0, None).await.unwrap();
        assert_eq!(before.len(), 2);

        // tag_b 삭제 후: tag_a 기사만
        db.set_user_tags(user_id, vec![tag_a]).await.unwrap();
        let after = db.get_user_articles(user_id, 10, 0, None).await.unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].tag_id, Some(tag_a));
    }

    // 삭제된 태그 ID로 특정 태그 조회 시 빈 배열
    #[tokio::test]
    async fn get_user_articles_returns_empty_for_removed_tag() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.list_tags().await.unwrap();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;

        db.set_user_tags(user_id, vec![tag_a, tag_b]).await.unwrap();
        db.seed_article(make_article(user_id, tag_b, "https://b.com"));

        db.set_user_tags(user_id, vec![tag_a]).await.unwrap();
        let result = db
            .get_user_articles(user_id, 10, 0, Some(tag_b))
            .await
            .unwrap();
        assert!(result.is_empty());
    }

    // 단건 조회: 삭제된 태그 기사 → None
    #[tokio::test]
    async fn get_user_article_by_id_returns_none_for_removed_tag() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.list_tags().await.unwrap();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;

        db.set_user_tags(user_id, vec![tag_a, tag_b]).await.unwrap();
        let article = make_article(user_id, tag_b, "https://b.com");
        let article_id = article.id;
        db.seed_article(article);

        // 활성 상태: 조회 가능
        let found = db
            .get_user_article_by_id(user_id, article_id)
            .await
            .unwrap();
        assert!(found.is_some());

        // tag_b 삭제 후: None
        db.set_user_tags(user_id, vec![tag_a]).await.unwrap();
        let not_found = db
            .get_user_article_by_id(user_id, article_id)
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    // created_at DESC 정렬 — tag_id=None (전체 피드)
    #[tokio::test]
    async fn get_user_articles_sorted_by_created_at_desc_all_feed() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.list_tags().await.unwrap();
        let tag_id = tags[0].id;
        db.set_user_tags(user_id, vec![tag_id]).await.unwrap();

        let now = chrono::Utc::now();
        let old = make_article(user_id, tag_id, "https://old.com");
        let new_ = Article {
            created_at: Some(now),
            ..make_article(user_id, tag_id, "https://new.com")
        };
        let older = Article {
            created_at: Some(now - chrono::Duration::hours(2)),
            ..make_article(user_id, tag_id, "https://older.com")
        };
        // 삽입 순서: old(None), new_(latest), older
        db.seed_article(old);
        db.seed_article(new_.clone());
        db.seed_article(older.clone());

        let result = db.get_user_articles(user_id, 10, 0, None).await.unwrap();
        assert_eq!(result.len(), 3);
        // created_at=Some(now) 가 최상단, created_at=None 은 최하단
        assert_eq!(result[0].url, "https://new.com");
        assert_eq!(result[1].url, "https://older.com");
    }

    // created_at DESC 정렬 — tag_id=Some (특정 태그 피드)
    #[tokio::test]
    async fn get_user_articles_sorted_by_created_at_desc_tag_filter() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.list_tags().await.unwrap();
        let tag_id = tags[0].id;
        db.set_user_tags(user_id, vec![tag_id]).await.unwrap();

        let now = chrono::Utc::now();
        let first = Article {
            created_at: Some(now - chrono::Duration::hours(1)),
            ..make_article(user_id, tag_id, "https://first.com")
        };
        let second = Article {
            created_at: Some(now),
            ..make_article(user_id, tag_id, "https://second.com")
        };
        db.seed_article(first);
        db.seed_article(second);

        let result = db
            .get_user_articles(user_id, 10, 0, Some(tag_id))
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].url, "https://second.com"); // 최신이 첫 번째
        assert_eq!(result[1].url, "https://first.com");
    }

    #[test]
    fn fake_db_default() {
        let db = FakeDbAdapter::default();
        let tags = db.get_tags();
        assert_eq!(tags.len(), 3);
    }

    #[tokio::test]
    async fn fake_db_crud_flow() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();

        // seed profile
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Test User".to_string()),
            onboarding_completed: false,
        });

        // get profile
        let profile = db.get_profile(user_id).await.unwrap();
        assert!(!profile.onboarding_completed);

        // list tags
        let tags = db.list_tags().await.unwrap();
        assert_eq!(tags.len(), 3);

        // set user tags
        let tag_ids: Vec<Uuid> = tags.iter().take(2).map(|t| t.id).collect();
        db.set_user_tags(user_id, tag_ids.clone()).await.unwrap();

        // get user tags
        let user_tags = db.get_user_tags(user_id).await.unwrap();
        assert_eq!(user_tags.len(), 2);

        // update onboarding
        db.update_profile_onboarding(user_id, true).await.unwrap();
        let profile = db.get_profile(user_id).await.unwrap();
        assert!(profile.onboarding_completed);
    }

    #[tokio::test]
    async fn get_all_user_ids_returns_all_users() {
        let db = FakeDbAdapter::new();

        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();

        db.seed_profile(Profile {
            id: user_a,
            display_name: None,
            onboarding_completed: true,
        });
        db.seed_profile(Profile {
            id: user_b,
            display_name: None,
            onboarding_completed: true,
        });

        let ids = db.get_all_user_ids().await.unwrap();
        assert!(ids.contains(&user_a));
        assert!(ids.contains(&user_b));
    }

    #[tokio::test]
    async fn get_all_user_ids_empty_when_no_users() {
        let db = FakeDbAdapter::new();
        let ids = db.get_all_user_ids().await.unwrap();
        assert!(ids.is_empty());
    }
}
