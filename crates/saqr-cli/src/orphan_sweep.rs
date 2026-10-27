use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use saqr_core::binance::signed_rest::BinanceSignedRest;
use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::resilience::orphan_guardian::OrphanOrderGuardian;
use tokio::time::{interval, Duration};

pub fn spawn(
    settings: SaqrSettings,
    orphan: Arc<OrphanOrderGuardian>,
    offset: Arc<AtomicI64>,
) {
    tokio::spawn(async move {
        let secs = std::env::var("SAQR_ORPHAN_SWEEP_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(120u64);
        let mut tick = interval(Duration::from_secs(secs));
        let signed = BinanceSignedRest::new(settings.clone(), offset);
        loop {
            tick.tick().await;
            if settings.api_key.is_empty() {
                continue;
            }
            match signed.open_orders(&settings.symbol).await {
                Ok(open) => {
                    let ids: Vec<String> = open.iter().map(|o| o.client_order_id.clone()).collect();
                    let ghosts = orphan.reconcile_with_exchange(&ids);
                    for ghost in ghosts {
                        tracing::warn!("orphan local order without exchange match: {ghost}");
                        let _ = signed.cancel_order(&settings.symbol, &ghost).await;
                    }
                }
                Err(e) => tracing::warn!("orphan sweep openOrders: {e}"),
            }
        }
    });
}
