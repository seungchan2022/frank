use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;

#[derive(Debug, Clone)]
pub struct FakeSearchAdapter {
    name: String,
    results: Vec<SearchResult>,
    should_fail: bool,
}

impl FakeSearchAdapter {
    pub fn new(name: &str, results: Vec<SearchResult>, should_fail: bool) -> Self {
        Self {
            name: name.to_string(),
            results,
            should_fail,
        }
    }
}

impl SearchPort for FakeSearchAdapter {
    fn search(
        &self,
        _query: &str,
        _max_results: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
    > {
        let should_fail = self.should_fail;
        let name = self.name.clone();
        let results = self.results.clone();
        Box::pin(async move {
            if should_fail {
                return Err(AppError::Internal(format!("{name} failed")));
            }
            Ok(results)
        })
    }

    fn source_name(&self) -> &str {
        &self.name
    }
}
