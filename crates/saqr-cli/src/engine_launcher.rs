use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tokio::sync::Mutex;
use saqr_core::binance::exchange_rules::ExchangeRulesCache;
use saqr_core::binance::order_executor::OrderExecutor;
use saqr_core::binance::signed_rest::BinanceSignedRest;
use saqr_core::binance::ws_client::BinanceWsClient;
use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::conquest::sovereign_conquest_engine::SovereignConquestEngine;
use saqr_core::engine::hot_loop::HotLoopContext;
use saqr_core::engine::market_state_updater::MarketStateUpdater;
use saqr_core::engine::runtime::EngineRuntime;
use saqr_core::orchestrator::decision_cascade::DecisionCascade;
use saqr_core::resilience::orphan_guardian::OrphanOrderGuardian;
use saqr_core::resilience::rate_limit_shield::RateLimitShield;
use saqr_core::resilience::rocksdb_wal::shared_wal;
use saqr_core::time::chrono_guard::ChronoGuard;
use tokio::sync::mpsc;

use crate::chrono_sync_worker;
use crate::orphan_sweep;

pub struct LaunchedEngine {
    pub runtime: Arc<EngineRuntime>,
    pub time_offset: Arc<AtomicI64>,
}

pub async fn launch(settings: SaqrSettings, gc_tx: mpsc::Sender<u64>) -> LaunchedEngine {
    let runtime = EngineRuntime::new(settings.clone());
    let time_offset = Arc::new(AtomicI64::new(0));
    let chrono = ChronoGuard::new(Arc::clone(&time_offset));
    let wal = shared_wal();
    let orphan = Arc::new(OrphanOrderGuardian::new());
    let rate_shield = Arc::new(RateLimitShield::new());

    let updater = Arc::new(MarketStateUpdater::new(Arc::clone(&runtime)));
    BinanceWsClient::spawn(settings.clone(), Arc::clone(&updater));

    chrono_sync_worker::spawn(settings.clone(), Arc::clone(&time_offset));

    let rules = ExchangeRulesCache::refresh_from_rest(&settings).await;

    let order_executor = if settings.enable_trading
        && matches!(settings.mode.as_str(), "testnet" | "live")
        && !settings.api_key.is_empty()
    {
        let signed = BinanceSignedRest::new(settings.clone(), Arc::clone(&time_offset));
        Some(Arc::new(OrderExecutor::new(
            settings.clone(),
            signed,
            rules,
            Arc::clone(&wal),
            Arc::clone(&orphan),
            rate_shield,
            1,
        )))
    } else {
        None
    };

    if order_executor.is_some() {
        orphan_sweep::spawn(
            settings.clone(),
            Arc::clone(&orphan),
            Arc::clone(&time_offset),
        );
    }

    let conquest = Arc::new(Mutex::new(SovereignConquestEngine::new(
        settings.initial_balance_usd,
        gc_tx,
    )));

    HotLoopContext {
        runtime: Arc::clone(&runtime),
        cascade: DecisionCascade::new(),
        chrono,
        conquest,
        order_executor,
        wal,
    }
    .spawn();

    LaunchedEngine {
        runtime,
        time_offset,
    }
}
