use std::net::SocketAddr;
use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use saqr_core::binance::signed_rest::BinanceSignedRest;
use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::engine::runtime::EngineRuntime;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<SaqrSettings>,
    pub engine: Arc<EngineRuntime>,
    pub time_offset: Arc<AtomicI64>,
}

pub async fn start_web(
    settings: SaqrSettings,
    engine: Arc<EngineRuntime>,
    time_offset: Arc<AtomicI64>,
    web_root: &str,
) -> Result<(), std::io::Error> {
    let state = AppState {
        settings: Arc::new(settings.clone()),
        engine,
        time_offset,
    };
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/status", get(status))
        .route("/api/engine", get(engine_status))
        .route("/api/signals", get(signals))
        .route("/api/positions", get(positions))
        .route("/api/binance_mirror", get(binance_mirror))
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
    st.engine.refresh_status_snapshot();
    let eng = st.engine.status.read().clone();
    Json(serde_json::json!({
        "mode": st.settings.mode,
        "symbol": st.settings.symbol,
        "binance_testnet": st.settings.binance.testnet,
        "binance_live_price": eng.last_price,
        "ws_connected": eng.ws_connected,
        "integration": "binance_testnet_real + telegram + dashboard",
        "binance_rest_url": st.settings.binance.rest_base_url,
        "binance_ws_url": st.settings.binance.ws_base_url,
        "binance_spot_web_url": st.settings.binance.spot_web_url,
        "binance_configured": !st.settings.api_key.is_empty(),
        "cloud_host": st.settings.cloud.host,
        "cloud_host_configured": !st.settings.cloud.host.is_empty(),
        "telegram_configured": !st.settings.telegram.bot_token.is_empty(),
        "dashboard_url": st.settings.cloud.public_dashboard_url,
        "web_port": st.settings.web_bind_port,
        "enable_trading": st.settings.enable_trading,
        "initial_balance_usd": st.settings.initial_balance_usd,
        "signal_count": st.engine.signals.read().len(),
        "last_verdict": eng.last_verdict,
    }))
}

async fn engine_status(State(st): State<AppState>) -> impl IntoResponse {
    st.engine.refresh_status_snapshot();
    let snapshot = st.engine.status.read().clone();
    Json(snapshot)
}

async fn signals(State(st): State<AppState>) -> impl IntoResponse {
    Json(st.engine.signals.read().clone())
}

async fn positions(State(st): State<AppState>) -> impl IntoResponse {
    let mut list = st.engine.positions.read().clone();
    let price = st.engine.status.read().last_price;
    for p in &mut list {
        p.current = price;
        p.pnl = (price - p.entry) * p.lot.parse::<f64>().unwrap_or(0.0);
    }
    Json(list)
}

/// مرآة Binance Testnet: أرصدة + أوامr مفتوحة (لا بيانات وهمية).
async fn binance_mirror(State(st): State<AppState>) -> impl IntoResponse {
    let symbol = st.settings.symbol.clone();
    let eng = st.engine.status.read().clone();
    let mut out = serde_json::json!({
        "mirror": true,
        "source": "binance_testnet + saqr_engine",
        "symbol": symbol,
        "ws_connected": eng.ws_connected,
        "engine_price": eng.last_price,
        "engine_verdict": eng.last_verdict,
        "engine_signal_count": st.engine.signals.read().len(),
        "engine_position_count": st.engine.positions.read().len(),
        "enable_trading": st.settings.enable_trading,
        "mode": st.settings.mode,
        "api_configured": !st.settings.api_key.is_empty(),
        "usdt_free": null,
        "base_asset_free": null,
        "open_orders_on_binance": 0,
        "binance_error": null,
    });

    if st.settings.api_key.is_empty() {
        out["binance_error"] = serde_json::json!("BINANCE_API_KEY not set — public price only");
        return Json(out);
    }

    let signed = BinanceSignedRest::new(st.settings.as_ref().clone(), Arc::clone(&st.time_offset));
    match signed.get_account().await {
        Ok(acct) => {
            for b in acct.balances {
                let free: f64 = b.free.parse().unwrap_or(0.0);
                if free <= 0.0 {
                    continue;
                }
                if b.asset == "USDT" {
                    out["usdt_free"] = serde_json::json!(free);
                }
                let base = symbol.trim_end_matches("USDT");
                if b.asset == base {
                    out["base_asset_free"] = serde_json::json!(free);
                }
            }
        }
        Err(e) => {
            out["binance_error"] = serde_json::json!(e);
        }
    }

    if let Ok(orders) = signed.open_orders(&symbol).await {
        out["open_orders_on_binance"] = serde_json::json!(orders.len());
    }

    Json(out)
}
