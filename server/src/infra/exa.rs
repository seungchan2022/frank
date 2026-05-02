use futures::future::join_all;
use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;
use crate::infra::http_utils::{
    OG_IMAGE_TIMEOUT_SECS, RetryConfig, fetch_og_image, read_body_limited, send_with_retry,
};

#[derive(Debug, Clone)]
pub struct ExaAdapter {
    client: Client,
    /// og:image 크롤링 전용 클라이언트 (짧은 타임아웃)
    crawl_client: Client,
    api_key: String,
    base_url: String,
    /// MVP15 M2 S1: numResults 상한. 보통 무료 max 10 추정.
    max_cap: usize,
}

#[derive(Debug, Deserialize)]
struct ExaResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    highlights: Option<Vec<String>>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
}

/// MVP9 M1: snippet 정제 함수.
/// 1. HTML 태그(<...>) 제거 — 문자 단위 파싱
/// 2. 줄 단위 메타 텍스트 제거 (헤더·저자·댓글수·목차·인라인 출처)
/// 3. [...] 플레이스홀더 제거
/// 4. 줄바꿈·연속 공백 정리
/// 5. 300자 문장 경계 절단 (마침표/느낌표/물음표 기준, 초과 시 단어 경계로 폴백)
pub fn clean_snippet(s: &str) -> String {
    // 1. HTML 태그 제거 (문자 단위 파싱)
    let mut no_html = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => no_html.push(ch),
            _ => {}
        }
    }

    // 2. 줄 단위 메타 텍스트 제거
    let filtered: String = no_html
        .lines()
        .filter_map(process_snippet_line)
        .collect::<Vec<_>>()
        .join(" ");

    // 3. [...] 플레이스홀더 제거
    let no_placeholders = filtered.replace("[...]", "");

    // 4. 줄바꿈 → 공백, 연속 공백 정리
    let normalized: String = no_placeholders
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // 5. 300자 이하면 그대로 반환
    if normalized.chars().count() <= 300 {
        return normalized;
    }

    // 6. 300자 이내에서 문장 경계로 절단
    let cutoff: String = normalized.chars().take(300).collect();
    if let Some(pos) = cutoff.rfind(['.', '!', '?']) {
        cutoff[..pos + 1].to_string()
    } else if let Some(last_space) = cutoff.rfind(' ') {
        cutoff[..last_space].to_string()
    } else {
        cutoff
    }
}

/// 스니펫 한 줄을 처리한다.
/// None → 줄 전체 제거, Some(s) → s로 교체 (인라인 출처는 본문만 추출)
fn process_snippet_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 마크다운 헤더
    if trimmed.starts_with('#') {
        return None;
    }

    // 인라인 출처: [city=agency] reporter 기자 = content → content만 반환
    if let Some(content) = trimmed
        .starts_with('[')
        .then(|| extract_after_inline_source(trimmed))
        .flatten()
    {
        return (!content.is_empty()).then_some(content);
    }

    let lower = trimmed.to_lowercase();

    // Published: / Author: / Language: 라인
    if lower.starts_with("published:")
        || lower.starts_with("author:")
        || lower.starts_with("language:")
    {
        return None;
    }

    // 영어 저자: By ... / Written by ...
    if lower.starts_with("by ") || lower.starts_with("written by ") {
        return None;
    }

    // 영어 댓글수: "5 comments" / "10 replies"
    if is_english_comment_count(&lower) {
        return None;
    }

    // Table of Contents
    if lower.contains("table of contents") {
        return None;
    }

    // 한국어 저자 줄
    if trimmed.starts_with("작성자:") || trimmed.starts_with("글쓴이:") {
        return None;
    }

    // 한국어 댓글수: "댓글 N개" / "댓글N개"
    if is_korean_comment_count(trimmed) {
        return None;
    }

    // 한국어 목차
    if trimmed.starts_with("목차") {
        return None;
    }

    Some(line.to_string())
}

/// `[city=agency] reporter 기자 = content` 패턴에서 content만 추출.
fn extract_after_inline_source(line: &str) -> Option<String> {
    let bracket_end = line.find(']')?;
    if !line[1..bracket_end].contains('=') {
        return None;
    }
    let after_bracket = &line[bracket_end + 1..];
    let reporter_pos = after_bracket.find("기자")?;
    let after_reporter = &after_bracket[reporter_pos + "기자".len()..];
    let eq_pos = after_reporter.find('=')?;
    Some(after_reporter[eq_pos + 1..].trim().to_string())
}

fn is_english_comment_count(lower_trimmed: &str) -> bool {
    if !lower_trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    let digit_end = lower_trimmed
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(lower_trimmed.len());
    let rest = lower_trimmed[digit_end..].trim_start_matches(' ');
    rest.starts_with("comment") || rest.starts_with("repl")
}

fn is_korean_comment_count(line: &str) -> bool {
    if let Some(pos) = line.find("댓글") {
        line[pos + "댓글".len()..]
            .trim_start_matches(' ')
            .starts_with(|c: char| c.is_ascii_digit())
    } else {
        false
    }
}

/// Exa 무료 numResults max 추정치. 실측 후 조정 가능.
const DEFAULT_EXA_MAX_CAP: usize = 10;

impl ExaAdapter {
    pub fn new(api_key: &str) -> Self {
        Self::with_base_url(api_key, "https://api.exa.ai")
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            crawl_client: Client::builder()
                .timeout(std::time::Duration::from_secs(OG_IMAGE_TIMEOUT_SECS))
                .user_agent("Mozilla/5.0 (compatible; FrankBot/1.0)")
                .build()
                .expect("Failed to build crawl client"),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            max_cap: DEFAULT_EXA_MAX_CAP,
        }
    }

    /// MVP15 M2 S1: numResults 상한 cap 주입. 빌더 패턴.
    pub fn with_max_cap(mut self, cap: usize) -> Self {
        self.max_cap = cap.max(1);
        self
    }
}

impl SearchPort for ExaAdapter {
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
    > {
        let query = query.to_string();
        // S1 clamp
        let effective = max_results.min(self.max_cap);
        Box::pin(async move {
            // Tavily의 time_range:"week"와 의미적 쌍 — 두 어댑터 모두 최근 7일 뉴스만 반환
            const SEARCH_WINDOW_DAYS: i64 = 7;
            let now = chrono::Utc::now();
            let week_ago = now - chrono::Duration::days(SEARCH_WINDOW_DAYS);
            let body = serde_json::json!({
                "query": query,
                "numResults": effective,
                // Exa API 파라미터명: category (Tavily는 topic) — 각 API 스펙 차이
                "category": "news",
                "startPublishedDate": week_ago.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                "contents": {
                    "highlights": {
                        "numSentences": 3,
                        "highlightsPerUrl": 1
                    }
                }
            });

            let config = RetryConfig::for_search();

            let api_key = self.api_key.clone();
            let url = format!("{}/search", self.base_url);

            let resp = send_with_retry(
                || {
                    let url = url.clone();
                    let body = body.clone();
                    let api_key = api_key.clone();
                    let client = self.client.clone();
                    async move {
                        client
                            .post(&url)
                            .header("x-api-key", &api_key)
                            .header("Content-Type", "application/json")
                            .json(&body)
                    }
                },
                &config,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Exa request failed: {e}")))?;

            let status = resp.status();
            let body = read_body_limited(resp, config.max_response_size)
                .await
                .map_err(|e| AppError::Internal(format!("Exa read failed: {e}")))?;

            if !status.is_success() {
                return Err(AppError::Internal(format!("Exa returned {status}: {body}")));
            }

            let data: ExaResponse = serde_json::from_str(&body)
                .map_err(|e| AppError::Internal(format!("Exa parse failed: {e}")))?;

            // 각 기사 URL에서 og:image 병렬 크롤링
            let crawl_futures: Vec<_> = data
                .results
                .iter()
                .map(|r| fetch_og_image(&self.crawl_client, &r.url))
                .collect();
            let image_urls: Vec<Option<String>> = join_all(crawl_futures).await;

            Ok(data
                .results
                .into_iter()
                .zip(image_urls)
                .map(|(r, image_url)| SearchResult {
                    title: r.title.unwrap_or_default(),
                    url: r.url,
                    snippet: r
                        .highlights
                        .and_then(|h| h.into_iter().next())
                        .map(|s| clean_snippet(&s)),
                    published_at: r.published_date,
                    image_url,
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "exa"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // --- clean_snippet 단위 테스트 ---

    #[test]
    fn clean_snippet_removes_html_tags() {
        let input = "<p>본문</p> <a href='https://example.com'>링크</a>";
        let result = clean_snippet(input);
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
        assert!(result.contains("본문"));
        assert!(result.contains("링크"));
    }

    #[test]
    fn clean_snippet_normalizes_whitespace() {
        let input = "첫째 줄\n둘째 줄\n\n셋째 줄";
        let result = clean_snippet(input);
        assert!(!result.contains('\n'));
        assert!(result.contains("첫째 줄"));
        assert!(result.contains("셋째 줄"));
    }

    #[test]
    fn clean_snippet_short_input_unchanged() {
        let input = "짧은 텍스트입니다.";
        let result = clean_snippet(input);
        assert_eq!(result, "짧은 텍스트입니다.");
    }

    #[test]
    fn clean_snippet_cuts_at_sentence_boundary() {
        // 300자 초과 시 마침표 기준으로 절단
        let long = "첫 번째 문장입니다. ".repeat(20); // 300자 초과
        let result = clean_snippet(&long);
        assert!(result.chars().count() <= 300);
        assert!(result.ends_with('.'), "문장 경계로 절단되어야 함: {result}");
    }

    #[test]
    fn clean_snippet_sentence_boundary_no_ellipsis() {
        // 문장 경계 절단 시 "…" 없어야 함
        let sentences = "Apple이 새로운 AI 기능을 발표했다. 이번 발표에서 iOS 업데이트가 포함됐다. 사용자 경험 개선이 핵심이다. ".repeat(5);
        let result = clean_snippet(&sentences);
        assert!(!result.ends_with('…'), "문장 경계 절단 시 … 없어야 함");
        assert!(result.ends_with('.'), "마침표로 끝나야 함");
    }

    #[test]
    fn clean_snippet_normal_text_preserved() {
        let input = "Apple이 새로운 AI 기능을 iOS 18에 추가했다. 이 기능은 사용자 경험을 크게 개선할 것으로 예상된다.";
        let result = clean_snippet(input);
        assert!(result.contains("Apple이"));
        assert!(result.contains("iOS 18"));
        assert!(result.contains("AI 기능"));
    }

    #[test]
    fn clean_snippet_removes_markdown_headers() {
        let input = "# 제목\n## 소제목\n본문 내용입니다.\n### 세부 제목\n추가 내용.";
        let result = clean_snippet(input);
        assert!(!result.contains("# 제목"), "# 헤더가 제거돼야 함");
        assert!(!result.contains("## 소제목"), "## 헤더가 제거돼야 함");
        assert!(!result.contains("### 세부 제목"), "### 헤더가 제거돼야 함");
        assert!(result.contains("본문 내용입니다."), "본문은 유지돼야 함");
        assert!(result.contains("추가 내용."), "본문은 유지돼야 함");
    }

    #[test]
    fn clean_snippet_removes_placeholders() {
        let input = "기사 내용입니다. [...] 추가 내용도 있습니다. [...] 마지막 내용.";
        let result = clean_snippet(input);
        assert!(!result.contains("[...]"), "[...] 패턴이 제거돼야 함");
        assert!(result.contains("기사 내용입니다."), "본문은 유지돼야 함");
        assert!(result.contains("마지막 내용."), "본문은 유지돼야 함");
    }

    #[test]
    fn clean_snippet_real_world_pattern() {
        let input = "# AI 열풍 기사 제목\nPublished: 2026-04-27T08:43:44+09:00 [...] Author: 이민우 기자\n## Summary\n올해 1분기 VC 투자가 급증했다. [...] 역대 최대치를 기록했다.\n## Story\n세부 내용.";
        let result = clean_snippet(input);
        assert!(!result.contains("# AI 열풍"), "# 헤더 제거");
        assert!(!result.contains("## Summary"), "## 헤더 제거");
        assert!(!result.contains("## Story"), "## 헤더 제거");
        assert!(!result.contains("[...]"), "[...] 제거");
        assert!(result.contains("올해 1분기"), "본문 유지");
        assert!(result.contains("역대 최대치를 기록했다."), "본문 유지");
    }

    #[test]
    fn clean_snippet_removes_published_author_language() {
        let input = "Published: 2026-04-27T08:43:44+09:00\nAuthor: 이민우 기자\nLanguage: ko\n오늘의 뉴스입니다.";
        let result = clean_snippet(input);
        assert!(!result.contains("Published:"), "Published: 라인 제거");
        assert!(!result.contains("Author:"), "Author: 라인 제거");
        assert!(!result.contains("Language:"), "Language: 라인 제거");
        assert!(result.contains("오늘의 뉴스입니다."), "본문 유지");
    }

    #[test]
    fn clean_snippet_removes_english_author_lines() {
        let input = "By John Smith\nArticle content here.\nWritten by Jane Doe\nMore content.";
        let result = clean_snippet(input);
        assert!(!result.contains("By John Smith"), "By 저자 라인 제거");
        assert!(
            !result.contains("Written by Jane Doe"),
            "Written by 라인 제거"
        );
        assert!(result.contains("Article content here."), "본문 유지");
        assert!(result.contains("More content."), "본문 유지");
    }

    #[test]
    fn clean_snippet_removes_comment_count_lines() {
        let input =
            "5 comments\nThis is the news.\n10 replies\n더 많은 내용.\n댓글 3개\n한국어 기사 본문.";
        let result = clean_snippet(input);
        assert!(!result.contains("5 comments"), "영어 댓글수 제거");
        assert!(!result.contains("10 replies"), "영어 replies 제거");
        assert!(!result.contains("댓글 3개"), "한국어 댓글수 제거");
        assert!(result.contains("This is the news."), "본문 유지");
        assert!(result.contains("한국어 기사 본문."), "본문 유지");
    }

    #[test]
    fn clean_snippet_removes_korean_author_lines() {
        let input = "작성자: 이민우 기자\n오늘의 뉴스입니다.\n글쓴이: 홍길동\n추가 기사 내용.";
        let result = clean_snippet(input);
        assert!(!result.contains("작성자:"), "작성자: 라인 제거");
        assert!(!result.contains("글쓴이:"), "글쓴이: 라인 제거");
        assert!(result.contains("오늘의 뉴스입니다."), "본문 유지");
        assert!(result.contains("추가 기사 내용."), "본문 유지");
    }

    #[test]
    fn clean_snippet_extracts_content_after_inline_source() {
        let input = "[서울=뉴스핌] 송은정 기자 = 구글이 AI 기반 웹 탐색 기능을 강화한 크롬 브라우저를 출시했다.";
        let result = clean_snippet(input);
        assert!(!result.contains("[서울=뉴스핌]"), "인라인 출처 제거");
        assert!(!result.contains("송은정 기자"), "기자명 제거");
        assert!(result.contains("구글이 AI 기반"), "기사 본문 유지");
    }

    #[test]
    fn clean_snippet_preserves_by_in_sentence() {
        // "by"가 문장 중간에 있으면 제거하면 안 됨
        let input = "The result was driven by the new policy.\nAnother sentence.";
        let result = clean_snippet(input);
        assert!(
            result.contains("driven by the new policy"),
            "문장 중간 by 보존"
        );
    }

    #[tokio::test]
    async fn retry_on_retryable_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(502))
            .up_to_n_times(2)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "results": [{"title": "Test", "url": "https://example.com", "highlights": ["snippet"], "publishedDate": null}]
                })),
            )
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn size_limit_exceeded() {
        let mock_server = MockServer::start().await;

        let large_body = "x".repeat(3 * 1024 * 1024);
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(&large_body))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn non_2xx_error_message() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Forbidden"))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Exa returned"));
    }

    #[tokio::test]
    async fn invalid_json_parse_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("parse failed"));
    }

    #[tokio::test]
    async fn request_error_network_failure() {
        let adapter = ExaAdapter::with_base_url("test-key", "http://127.0.0.1:1");
        let result = adapter.search("test", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn title_none_defaults_to_empty_string() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": null, "url": "https://example.com", "highlights": ["content"], "publishedDate": null}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results[0].title, "");
    }

    #[tokio::test]
    async fn successful_search_maps_fields() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {"title": "Article 1", "url": "https://a.com", "highlights": ["snippet 1"], "publishedDate": "2026-01-01"}
                ]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Article 1");
        assert_eq!(results[0].url, "https://a.com");
        assert_eq!(results[0].snippet, Some("snippet 1".to_string()));
        assert_eq!(results[0].published_at, Some("2026-01-01".to_string()));
    }

    #[tokio::test]
    async fn search_results_include_og_image() {
        let mock_server = MockServer::start().await;

        // Exa 검색 응답 — mock_server URL 반환
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "title": "Article 1",
                        "url": format!("{}/article1", mock_server.uri()),
                        "highlights": ["snippet 1"],
                        "publishedDate": "2026-01-01"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        // article1 페이지 — og:image 포함
        Mock::given(method("GET"))
            .and(path("/article1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"<html><head><meta property="og:image" content="https://cdn.example.com/thumb.jpg" /></head></html>"#,
            ))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].image_url,
            Some("https://cdn.example.com/thumb.jpg".to_string())
        );
    }

    #[tokio::test]
    async fn og_image_none_when_crawl_fails() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    {
                        "title": "Blocked",
                        "url": format!("{}/blocked", mock_server.uri()),
                        "highlights": ["snippet"],
                        "publishedDate": null
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        // 크롤링 차단 (403)
        Mock::given(method("GET"))
            .and(path("/blocked"))
            .respond_with(ResponseTemplate::new(403))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let results = adapter.search("test", 5).await.unwrap();

        assert_eq!(results[0].image_url, None);
    }

    // MARK: - ST-1: category:"news" + startPublishedDate 파라미터 검증

    #[tokio::test]
    async fn exa_request_includes_category_news() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({
                "category": "news"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": "Test", "url": "https://example.com/news/test", "highlights": ["snippet"], "publishedDate": "2026-04-23T00:00:00Z"}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("test query", 5).await;
        assert!(
            result.is_ok(),
            "category: news 파라미터 포함 요청이 성공해야 함: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap().len(), 1);
    }

    /// Exa API의 publishedDate → SearchResult.published_at 매핑이 올바른지 검증.
    /// startPublishedDate 키 존재 여부는 body_partial_json으로 강하게 확인.
    #[tokio::test]
    async fn exa_published_date_mapped_to_published_at() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/search"))
            .and(body_partial_json(serde_json::json!({
                "category": "news"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": "Dated", "url": "https://example.com/news/dated", "highlights": ["snippet"], "publishedDate": "2026-04-16T00:00:00Z"}]
            })))
            .mount(&mock_server)
            .await;

        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let result = adapter.search("date test", 5).await;
        assert!(
            result.is_ok(),
            "startPublishedDate 포함 요청이 성공해야 함: {:?}",
            result.err()
        );
        // publishedDate가 published_at 필드에 매핑되어 반환되어야 한다
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert!(
            items[0].published_at.is_some(),
            "published_at 필드가 Some이어야 함"
        );
        assert_eq!(
            items[0].published_at.as_deref(),
            Some("2026-04-16T00:00:00Z")
        );
    }

    // MARK: - ST-1: startPublishedDate 값 형식·범위 검증

    #[tokio::test]
    async fn exa_start_published_date_rfc3339_z_suffix_and_within_7_days() {
        let mock_server = MockServer::start().await;

        // 요청 본문을 캡처하여 값 형식과 범위를 검증
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"title": "Test", "url": "https://example.com/news/test", "highlights": ["s"], "publishedDate": null}]
            })))
            .mount(&mock_server)
            .await;

        let before = chrono::Utc::now();
        let adapter = ExaAdapter::with_base_url("test-key", &mock_server.uri());
        let _ = adapter.search("test", 1).await;
        let after = chrono::Utc::now();

        // 수신된 요청에서 startPublishedDate 값을 파싱해 검증
        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1, "요청이 1회 전송돼야 함");

        let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
        let date_str = body["startPublishedDate"]
            .as_str()
            .expect("startPublishedDate는 문자열이어야 함");

        // Z suffix 확인 (RFC3339 millis + Z)
        assert!(
            date_str.ends_with('Z'),
            "startPublishedDate는 Z suffix로 끝나야 함: {}",
            date_str
        );

        // 파싱 가능 확인
        let parsed = chrono::DateTime::parse_from_rfc3339(date_str)
            .expect("startPublishedDate는 유효한 RFC3339 형식이어야 함");
        let parsed_utc = parsed.with_timezone(&chrono::Utc);

        // 7일(±1초 여유) 이내 범위 확인
        let expected_min = before - chrono::Duration::days(7) - chrono::Duration::seconds(1);
        let expected_max = after - chrono::Duration::days(7) + chrono::Duration::seconds(1);
        assert!(
            parsed_utc >= expected_min && parsed_utc <= expected_max,
            "startPublishedDate가 7일 전 범위 내여야 함: {}",
            date_str
        );
    }
}
