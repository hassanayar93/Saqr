use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use saqr_core::config::env_settings::SaqrSettings;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<SaqrSettings>,
}

pub async fn start_web(settings: SaqrSettings, web_root: &str) -> Result<(), std::io::Error> {
    let state = AppState {
        settings: Arc::new(settings.clone()),
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/status", get(status))
        .nest_service("/", ServeDir::new(web_root))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", settings.web_bind_host, settings.web_bind_port)
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:8787".parse().unwrap());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("SAQR Control Terminal on http://{addr}");
    axum::serve(listener, app).await
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true, "engine": "SAQR v6.0"}))
}

async fn status(State(st): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "mode": st.settings.mode,
        "symbol": st.settings.symbol,
        "binance_testnet": st.settings.binance.testnet,
        "cloud_host_configured": !st.settings.cloud.host.is_empty(),
        "telegram_configured": !st.settings.telegram.bot_token.is_empty(),
        "dashboard_url": st.settings.cloud.public_dashboard_url,
    }))
}
