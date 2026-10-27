use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;

use crate::config::env_settings::SaqrSettings;
use crate::core::market_event_bus::MarketEventBus;
use crate::core::unified_market_state::{AtomicEmergencyLock, UnifiedMarketState};
use crate::momentum::stream_freeze_watchdog::StreamFreezeWatchdog;
use crate::orchestrator::decision_cascade::CascadeVerdict;

#[derive(Debug, Clone, Serialize)]
pub struct EngineSignalDto {
    pub id: u64,
    pub pair: String,
    pub direction: String,
    pub strength_score: u32,
    pub timeframe: String,
    pub executed: bool,
    pub sovereign_score: f64,
    pub hira_boost: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnginePositionDto {
    pub id: u64,
    pub pair: String,
    pub side: String,
    pub lot: String,
    pub entry: f64,
    pub current: f64,
    pub pnl: f64,
    pub client_order_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineStatusDto {
    pub hot_loop_ticks: u64,
    pub ring_pending: u64,
    pub last_verdict: String,
    pub stream_frozen: bool,
    pub emergency_lock: bool,
    pub ws_connected: bool,
    pub last_price: f64,
    pub consensus_score: f64,
    pub sovereign_score: f64,
    pub wal_pending: usize,
    pub trading_enabled: bool,
}

pub struct EngineRuntime {
    pub settings: SaqrSettings,
    pub market_state: RwLock<UnifiedMarketState>,
    pub event_bus: Arc<MarketEventBus>,
    pub freeze_watchdog: Arc<StreamFreezeWatchdog>,
    pub emergency: Arc<AtomicEmergencyLock>,
    pub signals: RwLock<Vec<EngineSignalDto>>,
    pub positions: RwLock<Vec<EnginePositionDto>>,
    pub status: RwLock<EngineStatusDto>,
    pub signal_id_seq: AtomicU64,
    pub position_id_seq: AtomicU64,
    pub ws_connected: AtomicU64,
    pub hot_loop_ticks: AtomicU64,
    pub last_verdict: RwLock<CascadeVerdict>,
    pub wal_pending: AtomicU64,
    pub last_order_attempt_ms: AtomicU64,
    pub last_bid_depth: RwLock<f64>,
    pub last_ask_depth: RwLock<f64>,
}

impl EngineRuntime {
    pub fn new(settings: SaqrSettings) -> Arc<Self> {
        Arc::new(Self {
            settings: settings.clone(),
            market_state: RwLock::new(UnifiedMarketState {
                symbol_hash: hash_symbol(&settings.symbol),
                timestamp_mono_ns: 0,
                adjusted_price_scaled: 0,
                cvd_h1_scaled: 0,
                consensus_score_scaled: 0,
                lock_status: 0,
            }),
            event_bus: Arc::new(MarketEventBus::new()),
            freeze_watchdog: Arc::new(StreamFreezeWatchdog::new(1500)),
            emergency: Arc::new(AtomicEmergencyLock::new()),
            signals: RwLock::new(Vec::new()),
            positions: RwLock::new(Vec::new()),
            status: RwLock::new(EngineStatusDto {
                hot_loop_ticks: 0,
                ring_pending: 0,
                last_verdict: "Pass".into(),
                stream_frozen: false,
                emergency_lock: false,
                ws_connected: false,
                last_price: 0.0,
                consensus_score: 0.0,
                sovereign_score: 0.0,
                wal_pending: 0,
                trading_enabled: settings.enable_trading,
            }),
            signal_id_seq: AtomicU64::new(100),
            position_id_seq: AtomicU64::new(200),
            ws_connected: AtomicU64::new(0),
            hot_loop_ticks: AtomicU64::new(0),
            last_verdict: RwLock::new(CascadeVerdict::Pass),
            wal_pending: AtomicU64::new(0),
            last_order_attempt_ms: AtomicU64::new(0),
            last_bid_depth: RwLock::new(1000.0),
            last_ask_depth: RwLock::new(1000.0),
        })
    }

    pub fn set_ws_connected(&self, connected: bool) {
        self.ws_connected
            .store(if connected { 1 } else { 0 }, Ordering::Release);
    }

    pub fn is_ws_connected(&self) -> bool {
        self.ws_connected.load(Ordering::Acquire) == 1
    }

    pub fn push_signal(&self, signal: EngineSignalDto) {
        let mut guard = self.signals.write();
        guard.insert(0, signal);
        guard.truncate(50);
    }

    pub fn refresh_status_snapshot(&self) {
        let state = self.market_state.read();
        let price = state.adjusted_price_scaled as f64 / 100_000_000.0;
        let consensus = state.consensus_score_scaled as f64 / 1_000_000.0;
        let frozen = {
            let now = now_ms();
            self.freeze_watchdog.is_stream_frozen(now)
        };
        if frozen {
            self.emergency.record_silent_ms(1501);
        }
        let mut st = self.status.write();
        st.hot_loop_ticks = self.hot_loop_ticks.load(Ordering::Acquire);
        st.ring_pending = self.event_bus.pending_estimate();
        st.stream_frozen = frozen;
        st.emergency_lock = self.emergency.is_locked();
        st.ws_connected = self.is_ws_connected();
        st.last_price = price;
        st.consensus_score = consensus;
        st.last_verdict = format!("{:?}", *self.last_verdict.read());
        st.wal_pending = self.wal_pending.load(Ordering::Acquire) as usize;
        st.trading_enabled = self.settings.enable_trading;
    }
}

pub fn hash_symbol(symbol: &str) -> u64 {
    symbol.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
