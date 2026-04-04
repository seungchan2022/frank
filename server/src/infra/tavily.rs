use reqwest::Client;
use serde::Deserialize;

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;

#[derive(Debug, Clone)]
pub struct TavilyAdapter {
    client: Client,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: Option<String>,
    published_date: Option<String>,
}

impl TavilyAdapter {
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

impl SearchPort for TavilyAdapter {
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
                "max_results": max_results,
                "search_depth": "advanced",
                "include_answer": false,
            });

            let resp = self
                .client
                .post("https://api.tavily.com/search")
                .header("Content-Type", "application/json")
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("Tavily request failed: {e}")))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(AppError::Internal(format!(
                    "Tavily returned {status}: {body}"
                )));
            }

            let data: TavilyResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("Tavily parse failed: {e}")))?;

            Ok(data
                .results
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.content,
                    published_at: r.published_date,
                })
                .collect())
        })
    }

    fn source_name(&self) -> &str {
        "tavily"
    }
}
