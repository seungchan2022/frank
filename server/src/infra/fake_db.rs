use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Favorite, Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct FakeDbAdapter {
    profiles: Arc<Mutex<HashMap<Uuid, Profile>>>,
    tags: Arc<Mutex<Vec<Tag>>>,
    user_tags: Arc<Mutex<Vec<UserTag>>>,
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
            favorites: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn seed_profile(&self, profile: Profile) {
        self.profiles.lock().unwrap().insert(profile.id, profile);
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn get_user_tags_returns_only_this_users_tags() {
        let db = FakeDbAdapter::new();
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let tags = db.get_tags();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;

        db.seed_user_tag(user_a, tag_a);
        db.seed_user_tag(user_b, tag_b);

        let a_tags = db.get_user_tags(user_a).await.unwrap();
        assert_eq!(a_tags.len(), 1);
        assert_eq!(a_tags[0].tag_id, tag_a);
    }

    #[tokio::test]
    async fn set_user_tags_replaces_existing() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let tags = db.get_tags();
        let tag_a = tags[0].id;
        let tag_b = tags[1].id;

        db.set_user_tags(user_id, vec![tag_a]).await.unwrap();
        let before = db.get_user_tags(user_id).await.unwrap();
        assert_eq!(before.len(), 1);

        // tag_b로 교체
        db.set_user_tags(user_id, vec![tag_b]).await.unwrap();
        let after = db.get_user_tags(user_id).await.unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].tag_id, tag_b);
    }

    #[tokio::test]
    async fn get_profile_not_found() {
        let db = FakeDbAdapter::new();
        let result = db.get_profile(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_profile_not_found() {
        let db = FakeDbAdapter::new();
        let result = db.update_profile(Uuid::new_v4(), Some(true), None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_profile_display_name() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: false,
        });

        let updated = db
            .update_profile(user_id, None, Some("Alice".to_string()))
            .await
            .unwrap();
        assert_eq!(updated.display_name, Some("Alice".to_string()));
        assert!(!updated.onboarding_completed);
    }
}
