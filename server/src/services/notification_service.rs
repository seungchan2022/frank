use crate::domain::models::Article;
use crate::domain::ports::NotificationPort;

/// 요약 완료된 기사들을 알림 메시지로 포맷한다.
pub fn format_notification(articles: &[Article]) -> String {
    let count = articles.len();
    let mut msg = format!("Frank News Summary ({count} articles)\n\n");

    for (i, article) in articles.iter().enumerate() {
        let title_ko = article.title_ko.as_deref().unwrap_or(&article.title);
        let summary = article
            .summary
            .as_deref()
            .unwrap_or("(summary not available)");
        let insight = article
            .insight
            .as_deref()
            .unwrap_or("(insight not available)");
        let url = &article.url;

        msg.push_str(&format!(
            "{}. {}\n{}\nInsight: {}\nLink: {}\n",
            i + 1,
            title_ko,
            summary,
            insight,
            url
        ));

        if i < count - 1 {
            msg.push('\n');
        }
    }

    msg
}

/// 요약 완료된 기사가 있으면 알림을 전송한다.
/// 전송 실패 시 로그만 남기고 에러를 전파하지 않는다.
pub fn notify_if_any(notifier: &dyn NotificationPort, articles: &[Article]) {
    if articles.is_empty() {
        return;
    }

    let message = format_notification(articles);
    if let Err(e) = notifier.send(&message) {
        tracing::warn!(error = %e, "알림 전송 실패, 건너뜀");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::Article;
    use crate::infra::fake_notification::FakeNotificationAdapter;
    use uuid::Uuid;

    fn make_summarized_article(
        title: &str,
        title_ko: &str,
        summary: &str,
        insight: &str,
    ) -> Article {
        Article {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tag_id: None,
            title: title.to_string(),
            url: format!("https://example.com/{}", title.replace(' ', "-")),
            snippet: None,
            source: "test".to_string(),
            search_query: None,
            summary: Some(summary.to_string()),
            insight: Some(insight.to_string()),
            summarized_at: Some(chrono::Utc::now()),
            published_at: None,
            created_at: None,
            title_ko: Some(title_ko.to_string()),
            content: None,
            llm_model: Some("test-model".to_string()),
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
        }
    }

    #[test]
    fn format_notification_single_article() {
        let articles = vec![make_summarized_article(
            "AI News",
            "AI 뉴스",
            "AI가 발전하고 있습니다.",
            "주목할 트렌드입니다.",
        )];

        let msg = format_notification(&articles);
        assert!(msg.contains("Frank News Summary (1 articles)"));
        assert!(msg.contains("1. AI 뉴스"));
        assert!(msg.contains("AI가 발전하고 있습니다."));
        assert!(msg.contains("Insight: 주목할 트렌드입니다."));
        assert!(msg.contains("Link: https://example.com/AI-News"));
    }

    #[test]
    fn format_notification_multiple_articles() {
        let articles = vec![
            make_summarized_article("A", "가", "요약A", "인사이트A"),
            make_summarized_article("B", "나", "요약B", "인사이트B"),
        ];

        let msg = format_notification(&articles);
        assert!(msg.contains("(2 articles)"));
        assert!(msg.contains("1. 가"));
        assert!(msg.contains("2. 나"));
    }

    #[test]
    fn format_notification_uses_title_fallback_when_no_title_ko() {
        let mut article = make_summarized_article("Fallback", "한글", "요약", "인사이트");
        article.title_ko = None;

        let msg = format_notification(&[article]);
        assert!(msg.contains("1. Fallback"));
    }

    #[test]
    fn notify_if_any_sends_when_articles_exist() {
        let notifier = FakeNotificationAdapter::new();
        let articles = vec![make_summarized_article(
            "Test",
            "테스트",
            "요약",
            "인사이트",
        )];

        notify_if_any(&notifier, &articles);

        let messages = notifier.sent_messages();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("테스트"));
    }

    #[test]
    fn notify_if_any_skips_when_empty() {
        let notifier = FakeNotificationAdapter::new();
        notify_if_any(&notifier, &[]);

        assert!(notifier.sent_messages().is_empty());
    }

    #[test]
    fn notify_if_any_does_not_panic_on_failure() {
        let notifier = FakeNotificationAdapter::failing();
        let articles = vec![make_summarized_article(
            "Test",
            "테스트",
            "요약",
            "인사이트",
        )];

        // 실패해도 패닉하지 않아야 한다
        notify_if_any(&notifier, &articles);
    }
}
