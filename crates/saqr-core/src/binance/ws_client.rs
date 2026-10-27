use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::binance::ws_stream::BinanceWsStream;
use crate::config::env_settings::SaqrSettings;
use crate::core::market_event::{MARKET_EVENT_DEPTH, MARKET_EVENT_TRADE};
use crate::engine::market_state_updater::{parse_combined_stream, MarketStateUpdater};

pub struct BinanceWsClient;

impl BinanceWsClient {
    pub fn spawn(settings: SaqrSettings, updater: Arc<MarketStateUpdater>) {
        tokio::spawn(async move {
            let reconnect_base = std::env::var("SAQR_WS_RECONNECT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3u64);
            let mut backoff = reconnect_base;
            loop {
                match run_session(&settings, &updater).await {
                    Ok(()) => backoff = reconnect_base,
                    Err(e) => {
                        tracing::warn!("ws session ended: {e}");
                        updater.on_ws_connected(false);
                    }
                }
                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(60);
            }
        });
    }
}

async fn run_session(settings: &SaqrSettings, updater: &MarketStateUpdater) -> Result<(), String> {
    let sym = settings.symbol.to_ascii_lowercase();
    let _streams = BinanceWsStream::new(settings.binance.clone());
    let combined = format!(
        "{}/stream?streams={}@trade/{}@depth10@100ms",
        settings.binance.ws_base_url.trim_end_matches('/'),
        sym,
        sym
    );
    let url = if settings.binance.testnet {
        combined
    } else {
        combined
    };

    let (mut ws, _) = connect_async(&url)
        .await
        .map_err(|e| format!("connect {url}: {e}"))?;
    tracing::info!("Binance WS connected: {url}");
    updater.on_ws_connected(true);

    while let Some(msg) = ws.next().await {
        let msg = msg.map_err(|e| e.to_string())?;
        match msg {
            Message::Text(text) => handle_text(&text, updater),
            Message::Ping(p) => {
                ws.send(Message::Pong(p))
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    updater.on_ws_connected(false);
    Ok(())
}

fn handle_text(text: &str, updater: &MarketStateUpdater) {
    let Some((kind, data)) = parse_combined_stream(text) else {
        return;
    };
    match kind {
        MARKET_EVENT_TRADE => {
            let price = data.get("p").and_then(|v| v.as_str()).unwrap_or("0");
            let qty = data.get("q").and_then(|v| v.as_str()).unwrap_or("0");
            let buyer_maker = data.get("m").and_then(|v| v.as_bool()).unwrap_or(false);
            let t = data.get("T").and_then(|v| v.as_u64()).unwrap_or(0);
            updater.apply_trade_json(price, qty, buyer_maker, t);
        }
        MARKET_EVENT_DEPTH => {
            let bids = parse_levels(data.get("bids").or_else(|| data.get("b")));
            let asks = parse_levels(data.get("asks").or_else(|| data.get("a")));
            updater.apply_depth_json(&bids, &asks);
        }
        _ => {}
    }
}

fn parse_levels(v: Option<&serde_json::Value>) -> Vec<(String, String)> {
    let Some(arr) = v.and_then(|x| x.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|row| {
            let pair = row.as_array()?;
            let p = pair.first()?.as_str()?.to_string();
            let q = pair.get(1)?.as_str()?.to_string();
            Some((p, q))
        })
        .collect()
}
