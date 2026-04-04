use std::sync::Arc;

use uuid::Uuid;

use crate::domain::error::AppError;
use crate::domain::models::Article;
use crate::domain::ports::{CrawlPort, DbPort};
use crate::infra::search_chain::SearchFallbackChain;

/// 사용자의 태그를 기반으로 뉴스를 검색하고 DB에 저장한다.
pub async fn collect_for_user<D: DbPort>(
    db: &D,
    search_chain: &Arc<SearchFallbackChain>,
    crawl: &dyn CrawlPort,
    user_id: Uuid,
) -> Result<usize, AppError> {
    // 1. 사용자 태그 조회
    let user_tags = db.get_user_tags(user_id).await?;
    if user_tags.is_empty() {
        return Err(AppError::BadRequest(
            "No tags selected. Complete onboarding first.".to_string(),
        ));
    }

    // 태그 이름을 얻기 위해 전체 태그 목록 조회
    let all_tags = db.list_tags().await?;

    let mut all_articles = Vec::new();

    for user_tag in &user_tags {
        let tag_name = all_tags
            .iter()
            .find(|t| t.id == user_tag.tag_id)
            .map(|t| t.name.as_str())
            .unwrap_or("unknown");

        let query = format!("{tag_name} latest news");

        // 2. 검색 (폴백 체인)
        let search_result = search_chain.search(&query, 5).await;
        let (results, source) = match search_result {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(tag = tag_name, error = %e, "search failed for tag, skipping");
                continue;
            }
        };

        // 3. SearchResult → Article 변환
        for sr in results {
            all_articles.push(Article {
                id: Uuid::new_v4(),
                user_id,
                tag_id: Some(user_tag.tag_id),
                title: sr.title,
                url: sr.url,
                snippet: sr.snippet,
                source: source.clone(),
                search_query: Some(query.clone()),
                summary: None,
                insight: None,
                summarized_at: None,
                published_at: sr.published_at.and_then(|s| s.parse().ok()),
                created_at: None,
                title_ko: None,
                content: None,
                llm_model: None,
                prompt_tokens: None,
                completion_tokens: None,
            });
        }
    }

    // 4. DB에 저장
    let count = all_articles.len();
    if !all_articles.is_empty() {
        // 크롤링할 기사 정보를 미리 복사
        let crawl_targets: Vec<(Uuid, String)> =
            all_articles.iter().map(|a| (a.id, a.url.clone())).collect();

        db.save_articles(all_articles).await?;

        // 5. 각 기사 URL에서 본문 크롤링
        for (article_id, url) in &crawl_targets {
            match crawl.scrape(url).await {
                Ok(content) => {
                    if let Err(e) = db.update_article_content(*article_id, &content).await {
                        tracing::warn!(
                            article_id = %article_id,
                            error = %e,
                            "failed to save crawled content"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        url = %url,
                        error = %e,
                        "crawl failed, skipping content extraction"
                    );
                }
            }
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Profile, SearchResult};
    use crate::infra::fake_crawl::FakeCrawlAdapter;
    use crate::infra::fake_db::FakeDbAdapter;
    use crate::infra::fake_search::FakeSearchAdapter;

    fn make_chain(results: Vec<SearchResult>, should_fail: bool) -> Arc<SearchFallbackChain> {
        let adapter = FakeSearchAdapter::new("fake", results, should_fail);
        Arc::new(SearchFallbackChain::new(vec![Box::new(adapter)]))
    }

    fn setup_db_with_tags(db: &FakeDbAdapter) -> (Uuid, Vec<Uuid>) {
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: Some("Tester".to_string()),
            onboarding_completed: true,
        });

        let tags = db.get_tags();
        let tag_ids: Vec<Uuid> = tags.iter().take(2).map(|t| t.id).collect();
        (user_id, tag_ids)
    }

    #[tokio::test]
    async fn collect_returns_article_count() {
        let db = FakeDbAdapter::new();
        let crawl = FakeCrawlAdapter::new();
        let (user_id, tag_ids) = setup_db_with_tags(&db);

        // 태그 설정
        db.set_user_tags(user_id, tag_ids).await.unwrap();

        let chain = make_chain(
            vec![
                SearchResult {
                    title: "Article 1".to_string(),
                    url: "https://example.com/1".to_string(),
                    snippet: Some("snippet 1".to_string()),
                    published_at: None,
                },
                SearchResult {
                    title: "Article 2".to_string(),
                    url: "https://example.com/2".to_string(),
                    snippet: None,
                    published_at: None,
                },
            ],
            false,
        );

        let count = collect_for_user(&db, &chain, &crawl, user_id)
            .await
            .unwrap();

        // 2 tags x 2 results = 4 articles collected
        assert_eq!(count, 4);

        // DB에 저장 시 URL+user_id 중복 제거 → 2개 (동일 URL)
        let articles = db.get_user_articles(user_id, 100).await.unwrap();
        assert_eq!(articles.len(), 2);

        // 크롤링된 콘텐츠가 저장되었는지 확인
        for article in &articles {
            assert!(article.content.is_some());
        }
    }

    #[tokio::test]
    async fn collect_with_no_tags_returns_error() {
        let db = FakeDbAdapter::new();
        let crawl = FakeCrawlAdapter::new();
        let user_id = Uuid::new_v4();
        db.seed_profile(Profile {
            id: user_id,
            display_name: None,
            onboarding_completed: false,
        });

        let chain = make_chain(vec![], false);

        let result = collect_for_user(&db, &chain, &crawl, user_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn collect_skips_failed_searches() {
        let db = FakeDbAdapter::new();
        let crawl = FakeCrawlAdapter::new();
        let (user_id, tag_ids) = setup_db_with_tags(&db);
        db.set_user_tags(user_id, tag_ids).await.unwrap();

        // 검색이 실패해도 에러가 아닌 0 반환
        let chain = make_chain(vec![], true);

        let count = collect_for_user(&db, &chain, &crawl, user_id)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn collect_continues_on_crawl_failure() {
        let db = FakeDbAdapter::new();
        let crawl = FakeCrawlAdapter::failing();
        let (user_id, tag_ids) = setup_db_with_tags(&db);
        db.set_user_tags(user_id, tag_ids).await.unwrap();

        let chain = make_chain(
            vec![SearchResult {
                title: "Article 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: Some("snippet 1".to_string()),
                published_at: None,
            }],
            false,
        );

        let count = collect_for_user(&db, &chain, &crawl, user_id)
            .await
            .unwrap();

        // 기사는 저장되지만 content는 None (크롤 실패)
        assert_eq!(count, 2); // 2 tags x 1 result
        let articles = db.get_user_articles(user_id, 100).await.unwrap();
        for article in &articles {
            assert!(article.content.is_none());
        }
    }
}
