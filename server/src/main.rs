use server::config::AppConfig;
use server::infra::supabase_db::SupabaseDbAdapter;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = AppConfig::from_env();
    let db = SupabaseDbAdapter::new(&config);
    let app = server::create_router(db, config.supabase_jwt_secret.clone());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("failed to bind to {addr}"));

    tracing::info!("server listening on http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}
