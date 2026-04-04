use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::{CrawlPort, SearchPort};

#[derive(Debug, Clone)]
pub struct FirecrawlAdapter {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct FirecrawlResponse {
    data: Vec<FirecrawlResult>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlResult {
    title: Option<String>,
    url: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlScrapeResponse {
    data: Option<FirecrawlScrapeData>,
}

#[derive(Debug, Deserialize)]
struct FirecrawlScrapeData {
    markdown: Option<String>,
}

impl FirecrawlAdapter {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            api_key: api_key.to_string(),
        }
    }
}

impl SearchPort for FirecrawlAdapter {
    fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
    > {
        let query = query.to_string();
        Box::pin(async move {
            let body = serde_json::json!({
                "query": query,
                "limit": max_results,
            });

            let resp = self
                .client
                .post("https://api.firecrawl.dev/v1/search")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl request failed: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!(
                    "Firecrawl returned {status}: {body}"
                )));
            }

            let data: FirecrawlResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl parse failed: {e}")))?;

            Ok(data
                .data
                .into_iter()
                .filter_map(|r| {
                    Some(SearchResult {
                        title: r.title.unwrap_or_default(),
                        url: r.url?,
                        snippet: r.description,
                        published_at: None,
                    })
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "firecrawl"
    }
}

impl CrawlPort for FirecrawlAdapter {
    fn scrape(
        &self,
        url: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, AppError>> + Send + '_>>
    {
        let url = url.to_string();
        Box::pin(async move {
            let body = serde_json::json!({
                "url": url,
                "formats": ["markdown"],
                "onlyMainContent": true,
            });

            let resp = self
                .client
                .post("https://api.firecrawl.dev/v1/scrape")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl scrape request failed: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!(
                    "Firecrawl scrape returned {status}: {body}"
                )));
            }

            let data: FirecrawlScrapeResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Firecrawl scrape parse failed: {e}")))?;

            data.data.and_then(|d| d.markdown).ok_or_else(|| {
                AppError::Internal("Firecrawl scrape returned no markdown".to_string())
            })
        })
    }
}
