use futures::stream::{self, StreamExt};
use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Article;
use crate::domain::ports::{DbPort, LlmPort, NotificationPort};
use crate::services::notification_service;

const MAX_CONCURRENT_SUMMARIES: usize = 5;

/// 사용자의 미요약 기사를 LLM으로 요약한다.
/// 요약 완료 후 새로 요약된 기사가 있으면 알림을 전송한다.
pub async fn summarize_articles<D: DbPort>(
    db: &D,
    llm: &dyn LlmPort,
    notifier: &dyn NotificationPort,
    user_id: Uuid,
) -> Result<usize, AppError> {
    // 1. 사용자 기사 전체 조회 (충분히 큰 limit)
    let articles = db.get_user_articles(user_id, 1000).await?;

    // 요약 대상 필터링 (소유형 문자열로 수집)
    let targets: Vec<(Uuid, String, String)> = articles
        .iter()
        .filter(|a| a.summary.is_none())
        .filter_map(|a| {
            let text = a
                .content
                .as_deref()
                .filter(|s| !s.is_empty())
                .or(a.snippet.as_deref().filter(|s| !s.is_empty()));
            text.map(|t| (a.id, a.title.clone(), t.to_string()))
        })
        .collect();

    // LLM 병렬 호출 (최대 MAX_CONCURRENT_SUMMARIES개 동시)
    let llm_results: Vec<_> = stream::iter(targets)
        .map(|(article_id, title, text)| async move {
            let result = llm.summarize(&title, &text).await;
            (article_id, result)
        })
        .buffer_unordered(MAX_CONCURRENT_SUMMARIES)
        .collect()
        .await;

    // 원본 기사를 id → Article 맵으로 보관 (알림용)
    let article_map: std::collections::HashMap<Uuid, &Article> =
        articles.iter().map(|a| (a.id, a)).collect();

    let mut count = 0;
    let mut summarized_articles: Vec<Article> = Vec::new();

    for (article_id, result) in llm_results {
        match result {
            Ok(llm_resp) => {
                db.update_article_summary(
                    article_id,
                    &llm_resp.summary.summary,
                    &llm_resp.summary.insight,
                    &llm_resp.summary.title_ko,
                    &llm_resp.model,
                    llm_resp.prompt_tokens,
                    llm_resp.completion_tokens,
                )
                .await?;

                // 알림용: 원본 기사에 요약 정보를 채워서 보관 (DB 재조회 불필요)
                if let Some(&original) = article_map.get(&article_id) {
                    let mut article = original.clone();
                    article.summary = Some(llm_resp.summary.summary);
                    article.insight = Some(llm_resp.summary.insight);
                    article.title_ko = Some(llm_resp.summary.title_ko);
                    summarized_articles.push(article);
                }

                count += 1;
            }
            Err(e) => {
                tracing::warn!(
                    article_id = %article_id,
                    error = %e,
                    "LLM summarization failed, skipping"
                );
            }
        }
    }

    // 4. 새로 요약된 기사가 있으면 알림 전송 (DB 재조회 없이 메모리에서 직접)
    if !summarized_articles.is_empty() {
        notification_service::notify_if_any(notifier, &summarized_articles);
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Article, Profile};
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_llm::FakeLlmAdapter;
    use crate::infra::fake_notification::FakeNotificationAdapter;

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
            title_ko: None,
            content: None,
            llm_model: None,
            prompt_tokens: None,
            completion_tokens: None,
        }
    }

    #[tokio::test]
    async fn summarize_articles_with_snippets() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![
            make_article(user_id, "AI News", Some("AI is transforming...")),
            make_article(user_id, "Web Dev", Some("Web development trends...")),
        ];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        assert_eq!(count, 2);

        // 요약이 저장되었는지 확인
        let saved = db.get_user_articles(user_id, 100).await.unwrap();
        for article in &saved {
            assert!(article.summary.is_some());
            assert!(article.insight.is_some());
            assert!(article.summarized_at.is_some());
        }

        // 알림이 전송되었는지 확인
        let messages = notifier.sent_messages();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("2 articles"));
    }

    #[tokio::test]
    async fn summarize_skips_articles_without_snippet() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![
            make_article(user_id, "Has Snippet", Some("Some content")),
            make_article(user_id, "No Snippet", None),
        ];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn summarize_skips_already_summarized() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut article = make_article(user_id, "Already Done", Some("content"));
        article.summary = Some("existing summary".to_string());
        article.insight = Some("existing insight".to_string());
        article.summarized_at = Some(chrono::Utc::now());

        insert_articles(&db, user_id, vec![article]).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        assert_eq!(count, 0);

        // 새 요약이 없으므로 알림 전송 안 됨
        assert!(notifier.sent_messages().is_empty());
    }

    #[tokio::test]
    async fn summarize_continues_on_llm_failure() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::failing();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let articles = vec![make_article(user_id, "Will Fail", Some("content"))];
        insert_articles(&db, user_id, articles).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        // LLM 실패 시 건너뛰므로 0
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn summarize_prefers_content_over_snippet() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut article = make_article(user_id, "Both", Some("snippet text"));
        article.content = Some("full content text".to_string());
        insert_articles(&db, user_id, vec![article]).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        // content 존재 시 content가 LLM에 전달됨 (snippet 무시)
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn summarize_uses_snippet_when_content_empty_string() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut article = make_article(user_id, "EmptyContent", Some("valid snippet"));
        article.content = Some(String::new()); // 빈 문자열
        insert_articles(&db, user_id, vec![article]).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        // content가 빈 문자열이면 snippet으로 폴백
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn summarize_skips_both_empty_strings() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut article = make_article(user_id, "AllEmpty", None);
        article.content = Some(String::new());
        article.snippet = Some(String::new());
        insert_articles(&db, user_id, vec![article]).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        // 둘 다 빈 문자열이면 건너뜀
        assert_eq!(count, 0);
        assert!(notifier.sent_messages().is_empty());
    }

    #[tokio::test]
    async fn summarize_mixed_summarized_and_new() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let mut already_done = make_article(user_id, "Done", Some("content"));
        already_done.summary = Some("existing".to_string());
        already_done.insight = Some("existing".to_string());
        already_done.summarized_at = Some(chrono::Utc::now());

        let new_article = make_article(user_id, "New", Some("new content"));

        insert_articles(&db, user_id, vec![already_done, new_article]).await;

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        // 이미 요약된 1건 건너뛰고, 새 1건만 요약
        assert_eq!(count, 1);
        // 알림은 새로 요약된 1건에 대해서만
        let messages = notifier.sent_messages();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("1 articles"));
    }

    #[tokio::test]
    async fn summarize_no_articles_returns_zero() {
        let db = FakeDbAdapter::new();
        let llm = FakeLlmAdapter::new();
        let notifier = FakeNotificationAdapter::new();
        let user_id = Uuid::new_v4();
        setup_db_with_articles(&db, user_id);

        let count = summarize_articles(&db, &llm, &notifier, user_id)
            .await
            .unwrap();

        assert_eq!(count, 0);
        assert!(notifier.sent_messages().is_empty());
    }
}
