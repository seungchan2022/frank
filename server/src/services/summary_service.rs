use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::ports::{DbPort, LlmPort};

/// 사용자의 미요약 기사를 LLM으로 요약한다.
pub async fn summarize_articles<D: DbPort>(
    db: &D,
    llm: &dyn LlmPort,
    user_id: Uuid,
) -> Result<usize, AppError> {
    // 1. 사용자 기사 전체 조회 (충분히 큰 limit)
    let articles = db.get_user_articles(user_id, 1000).await?;

    let mut count = 0;

    for article in &articles {
        // summary가 이미 있는 기사는 건너뛴다
        if article.summary.is_some() {
            continue;
        }

        // snippet이 없으면 요약 불가
        let snippet = match &article.snippet {
            Some(s) if !s.is_empty() => s.as_str(),
            _ => continue,
        };

        // 2. LLM 호출
        let result = llm.summarize(&article.title, snippet).await;

        match result {
            Ok(llm_summary) => {
                // 3. DB 저장
                db.update_article_summary(article.id, &llm_summary.summary, &llm_summary.insight)
                    .await?;
                count += 1;
            }
            Err(e) => {
                tracing::warn!(
                    article_id = %article.id,
                    error = %e,
                    "LLM summarization failed, skipping"
                );
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Article, Profile};
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;

    fn setup_db_with_articles(db: &FakeDbAdapter, user_id: Uuid) {
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Tester".to_string()),
            onboarding_completed: true,
        });
    }

    async fn insert_articles(db: &FakeDbAdapter, user_id: Uuid, articles: Vec<Article>) {
        db.save_articles(articles).await.unwrap();
        let _ = db.get_user_articles(user_id, 100).await;
    }

    fn make_article(user_id: Uuid, title: &str, snippet: Option<&str>) -> Article {
        Article {
            id: Uuid::new_v4(),
            user_id,
            tag_id: None,
            title: title.to_string(),
            url: format!("https://example.com/{}", title.replace(' ', "-")),
            snippet: snippet.map(|s| s.to_string()),
            source: "test".to_string(),
            search_query: None,
            summary: None,
            insight: None,
            summarized_at: None,
            published_at: None,
            created_at: None,
        }
    }

    #[tokio::test]
    async fn summarize_articles_with_snippets() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![
            make_article(user_id, "AI News", Some("AI is transforming...")),
            make_article(user_id, "Web Dev", Some("Web development trends...")),
        ];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, user_id).await.unwrap();

        assert_eq!(count, 2);

        // 요약이 저장되었는지 확인
        let saved = db.get_user_articles(user_id, 100).await.unwrap();
        for article in &saved {
            assert!(article.summary.is_some());
            assert!(article.insight.is_some());
            assert!(article.summarized_at.is_some());
        }
    }

    #[tokio::test]
    async fn summarize_skips_articles_without_snippet() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![
            make_article(user_id, "Has Snippet", Some("Some content")),
            make_article(user_id, "No Snippet", None),
        ];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, user_id).await.unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn summarize_skips_already_summarized() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut article = make_article(user_id, "Already Done", Some("content"));
        article.summary = Some("existing summary".to_string());
        article.insight = Some("existing insight".to_string());
        article.summarized_at = Some(chrono::Utc::now());

        insert_articles(&db, user_id, vec![article]).await;

        let count = summarize_articles(&db, &llm, user_id).await.unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn summarize_continues_on_llm_failure() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::failing();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![make_article(user_id, "Will Fail", Some("content"))];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, user_id).await.unwrap();

        // LLM 실패 시 건너뛰므로 0
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn summarize_no_articles_returns_zero() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let count = summarize_articles(&db, &llm, user_id).await.unwrap();

        assert_eq!(count, 0);
    }
}
