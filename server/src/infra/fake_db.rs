use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::{Profile, Tag, UserTag};
use crate::domain::ports::DbPort;

#[derive(Debug, Clone)]
pub struct FakeDbAdapter {
    profiles: Arc<Mutex<HashMap<Uuid, Profile>>>,
    tags: Arc<Mutex<Vec<Tag>>>,
    user_tags: Arc<Mutex<Vec<UserTag>>>,
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
        }
    }

    pub fn seed_profile(&self, profile: Profile) {
        self.profiles.lock().unwrap().insert(profile.id, profile);
    }

    pub fn get_tags(&self) -> Vec<Tag> {
        self.tags.lock().unwrap().clone()
    }
}

impl DbPort for FakeDbAdapter {
    async fn get_profile(&self, user_id: Uuid, _auth_token: &str) -> Result<Profile, AppError> {
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
        _auth_token: &str,
    ) -> Result<(), AppError> {
        let mut profiles = self.profiles.lock().unwrap();
        let profile = profiles
            .get_mut(&user_id)
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;
        profile.onboarding_completed = completed;
        Ok(())
    }

    async fn list_tags(&self, _auth_token: &str) -> Result<Vec<Tag>, AppError> {
        Ok(self.tags.lock().unwrap().clone())
    }

    async fn get_user_tags(
        &self,
        user_id: Uuid,
        _auth_token: &str,
    ) -> Result<Vec<UserTag>, AppError> {
        let tags = self.user_tags.lock().unwrap();
        Ok(tags
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn set_user_tags(
        &self,
        user_id: Uuid,
        tag_ids: Vec<Uuid>,
        _auth_token: &str,
    ) -> Result<(), AppError> {
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

    #[tokio::test]
    async fn fake_db_crud_flow() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let token = "fake-token";

        // seed profile
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Test User".to_string()),
            onboarding_completed: false,
        });

        // get profile
        let profile = db.get_profile(user_id, token).await.unwrap();
        assert!(!profile.onboarding_completed);

        // list tags
        let tags = db.list_tags(token).await.unwrap();
        assert_eq!(tags.len(), 3);

        // set user tags
        let tag_ids: Vec<Uuid> = tags.iter().take(2).map(|t| t.id).collect();
        db.set_user_tags(user_id, tag_ids.clone(), token)
            .await
            .unwrap();

        // get user tags
        let user_tags = db.get_user_tags(user_id, token).await.unwrap();
        assert_eq!(user_tags.len(), 2);

        // update onboarding
        db.update_profile_onboarding(user_id, true, token)
            .await
            .unwrap();
        let profile = db.get_profile(user_id, token).await.unwrap();
        assert!(profile.onboarding_completed);
    }
}
