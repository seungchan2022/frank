use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Tag;
use crate::domain::ports::DbPort;

pub async fn list_tags<D: DbPort>(db: &D) -> Result<Vec<Tag>, AppError> {
    db.list_tags().await
}

pub async fn get_user_tag_ids<D: DbPort>(db: &D, user_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    let user_tags = db.get_user_tags(user_id).await?;
    Ok(user_tags.into_iter().map(|ut| ut.tag_id).collect())
}

pub async fn save_user_tags<D: DbPort>(
    db: &D,
    user_id: Uuid,
    tag_ids: Vec<Uuid>,
) -> Result<(), AppError> {
    if tag_ids.is_empty() {
        return Err(AppError::BadRequest(
            "At least one tag must be selected".to_string(),
        ));
    }

    db.set_user_tags(user_id, tag_ids).await?;
    db.update_profile_onboarding(user_id, true).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Profile;
    use crate::infra::fake_db::FakeDbAdapter;

    #[tokio::test]
    async fn list_tags_returns_seeded_tags() {
        let db = FakeDbAdapter::new();
        let tags = list_tags(&db).await.unwrap();
        assert_eq!(tags.len(), 3);
    }

    #[tokio::test]
    async fn save_and_get_user_tags() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: false,
        });

        let tags = db.get_tags();
        let tag_ids: Vec<Uuid> = tags.iter().take(2).map(|t| t.id).collect();

        save_user_tags(&db, user_id, tag_ids.clone()).await.unwrap();

        let result = get_user_tag_ids(&db, user_id).await.unwrap();
        assert_eq!(result.len(), 2);

        // onboarding should be completed
        let profile = db.get_profile(user_id).await.unwrap();
        assert!(profile.onboarding_completed);
    }

    #[tokio::test]
    async fn save_empty_tags_fails() {
        let db = FakeDbAdapter::new();
        let user_id = Uuid::new_v4();
        let result = save_user_tags(&db, user_id, vec![]).await;
        assert!(result.is_err());
    }
}
