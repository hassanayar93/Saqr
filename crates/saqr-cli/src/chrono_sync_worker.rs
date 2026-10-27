use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use saqr_core::binance::http_client::BinanceHttpClient;
use saqr_core::binance::time_sync::BinanceTimeSync;
use saqr_core::config::env_settings::SaqrSettings;
use tokio::time::{interval, Duration};

pub fn spawn(settings: SaqrSettings, offset: Arc<AtomicI64>) {
    tokio::spawn(async move {
        let sync = BinanceTimeSync::new(Arc::clone(&offset));
        let http = BinanceHttpClient::new(settings);
        if let Ok(ms) = http.server_time().await {
            sync.apply_server_time_ms(ms);
            tracing::info!("ChronoGuard initial sync (serverTime={ms})");
        }
        let mut tick = interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            if !sync.guard().needs_recalibration() {
                continue;
            }
            match http.server_time().await {
                Ok(ms) => {
                    sync.apply_server_time_ms(ms);
                    tracing::debug!("ChronoGuard offset recalibrated (serverTime={ms})");
                }
                Err(e) => tracing::warn!("chrono sync failed: {e}"),
            }
        }
    });
}
