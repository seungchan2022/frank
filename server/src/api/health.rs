use axum::Json;

use crate::domain::health::HealthStatus;

pub async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus::ok())
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum::routing::get;
    use axum_test::{TestResponse, TestServer};

    use super::*;

    fn app() -> Router {
        Router::new().route("/health", get(health_check))
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let server = TestServer::new(app());
        let res: TestResponse = server.get("/health").await;
        res.assert_status_ok();
        res.assert_json_contains(&serde_json::json!({"status": "ok"}));
    }
}
