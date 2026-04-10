use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::error::AppError;
use crate::domain::models::SearchResult;
use crate::domain::ports::SearchPort;

#[derive(Debug, Clone)]
pub struct FakeSearchAdapter {
    name: String,
    results: Vec<SearchResult>,
    should_fail: bool,
    /// 쿼리별 응답 맵. 키가 일치하면 맵 값 우선 사용.
    query_map: HashMap<String, Result<Vec<SearchResult>, String>>,
    /// 실제로 search()가 호출된 쿼리를 기록 (순서 보장).
    call_log: Arc<Mutex<Vec<String>>>,
}

impl FakeSearchAdapter {
    fn new_inner(
        name: &str,
        results: Vec<SearchResult>,
        should_fail: bool,
        query_map: HashMap<String, Result<Vec<SearchResult>, String>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            results,
            should_fail,
            query_map,
            call_log: Arc::new(Mutex::new(vec![])),
        }
    }

    /// 기존 API 유지.
    pub fn new(name: &str, results: Vec<SearchResult>, should_fail: bool) -> Self {
        Self::new_inner(name, results, should_fail, HashMap::new())
    }

    /// 쿼리별 다른 결과/실패를 지정하는 생성자.
    /// - `query_map`: 쿼리 문자열 → Ok(결과) 또는 Err(에러 메시지)
    /// - 맵에 없는 쿼리는 기존 `results` / `should_fail` 동작
    pub fn with_query_map(
        name: &str,
        query_map: HashMap<String, Result<Vec<SearchResult>, String>>,
    ) -> Self {
        Self::new_inner(name, vec![], false, query_map)
    }

    /// 실제로 호출된 쿼리 목록을 반환 (테스트용).
    #[cfg(test)]
    pub fn calls(&self) -> Vec<String> {
        self.call_log.lock().map(|l| l.clone()).unwrap_or_default()
    }
}

impl SearchPort for FakeSearchAdapter {
    fn search(
        &self,
        query: &str,
        _max_results: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Vec<SearchResult>, AppError>> + Send + '_>,
    > {
        let query_owned = query.to_string();
        let name = self.name.clone();
        let results = self.results.clone();
        let should_fail = self.should_fail;
        let call_log = Arc::clone(&self.call_log);

        // 쿼리 맵에서 먼저 탐색
        let map_entry = self.query_map.get(query).cloned();

        Box::pin(async move {
            // 호출 로그 기록
            if let Ok(mut log) = call_log.lock() {
                log.push(query_owned.clone());
            }

            if let Some(entry) = map_entry {
                return match entry {
                    Ok(items) => Ok(items),
                    Err(msg) => Err(AppError::Internal(msg)),
                };
            }

            // 기본 동작
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
