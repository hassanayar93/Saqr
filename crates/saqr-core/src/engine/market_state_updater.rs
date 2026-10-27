use std::sync::Arc;

use crate::binance::scale::decimal_str_to_scaled;
use crate::core::market_event::{MarketEvent, MARKET_EVENT_DEPTH, MARKET_EVENT_TRADE};
use crate::core::market_event_bus::MarketEventBus;
use crate::core::unified_market_state::UnifiedMarketState;
use crate::engine::runtime::{hash_symbol, now_ms, EngineRuntime};
use crate::momentum::apex_emperor_core::ApexEmperorCore;
use crate::momentum::stream_freeze_watchdog::StreamFreezeWatchdog;

pub struct MarketStateUpdater {
    runtime: Arc<EngineRuntime>,
    symbol_hash: u64,
    apex: ApexEmperorCore,
}

impl MarketStateUpdater {
    pub fn new(runtime: Arc<EngineRuntime>) -> Self {
        let symbol_hash = hash_symbol(&runtime.settings.symbol);
        Self {
            runtime,
            symbol_hash,
            apex: ApexEmperorCore::new(),
        }
    }

    pub fn on_ws_connected(&self, connected: bool) {
        self.runtime.set_ws_connected(connected);
    }

    pub fn apply_trade_json(&self, price: &str, qty: &str, buyer_maker: bool, trade_ms: u64) {
        let now = now_ms();
        self.runtime
            .freeze_watchdog
            .on_tick_received(trade_ms.max(now));

        let price_scaled = decimal_str_to_scaled(price).unwrap_or(0);
        let vol_scaled = decimal_str_to_scaled(qty).unwrap_or(0);
        let ts_ns = trade_ms.saturating_mul(1_000_000);

        let _ = self.runtime.event_bus.try_push(MarketEvent::trade(
            self.symbol_hash,
            price_scaled,
            vol_scaled,
            buyer_maker,
            ts_ns,
        ));

        let flow = ApexEmperorCore::calculate_real_flow_scaled(
            vol_scaled,
            if buyer_maker { -50_000_000 } else { 50_000_000 },
            0,
        );

        let mut state = self.runtime.market_state.write();
        state.timestamp_mono_ns = ts_ns;
        state.adjusted_price_scaled = price_scaled;
        state.cvd_h1_scaled = state.cvd_h1_scaled.saturating_add(flow);
        let micro_accel = flow / 10;
        let vol_factor = 100_000_000u32;
        if self.apex.evaluate_consensus(flow, micro_accel, vol_factor) {
            state.consensus_score_scaled = 90_000_000;
        } else {
            state.consensus_score_scaled = 70_000_000;
        }
        if self.runtime.emergency.is_locked() {
            state.lock_status = 1;
        }
    }

    pub fn apply_depth_json(&self, bids: &[(String, String)], asks: &[(String, String)]) {
        let now = now_ms();
        self.runtime.freeze_watchdog.on_tick_received(now);

        let bid_depth: f64 = bids
            .iter()
            .filter_map(|(_, q)| q.parse::<f64>().ok())
            .sum();
        let ask_depth: f64 = asks
            .iter()
            .filter_map(|(_, q)| q.parse::<f64>().ok())
            .sum();
        let bid_scaled = (bid_depth * 100_000_000.0) as i64;
        let ask_scaled = (ask_depth * 100_000_000.0) as i64;

        let _ = self.runtime.event_bus.try_push(MarketEvent::depth(
            self.symbol_hash,
            bid_scaled,
            ask_scaled,
            now.saturating_mul(1_000_000),
        ));

        *self.runtime.last_bid_depth.write() = bid_depth;
        *self.runtime.last_ask_depth.write() = ask_depth;

        let mut state = self.runtime.market_state.write();
        state.timestamp_mono_ns = now.saturating_mul(1_000_000);

        if let Ok(segment) = try_shm_segment() {
            let _ = segment.write_market_state(&state);
        }
    }

    pub fn event_bus(&self) -> Arc<MarketEventBus> {
        Arc::clone(&self.runtime.event_bus)
    }

    pub fn snapshot_state(&self) -> UnifiedMarketState {
        *self.runtime.market_state.read()
    }

    pub fn watchdog(&self) -> Arc<StreamFreezeWatchdog> {
        Arc::clone(&self.runtime.freeze_watchdog)
    }
}

fn try_shm_segment() -> Result<crate::ipc::shared_memory_segment::PosixSharedMemorySegment, ()> {
    let seg = crate::ipc::shared_memory_segment::PosixSharedMemorySegment::new(
        "saqr_market".into(),
        std::mem::size_of::<UnifiedMarketState>() * 2,
    );
    if seg.layout_valid() {
        Ok(seg)
    } else {
        Err(())
    }
}

pub fn parse_combined_stream(text: &str) -> Option<(u8, serde_json::Value)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    if let Some(stream) = v.get("stream").and_then(|s| s.as_str()) {
        let data = v.get("data")?.clone();
        if stream.contains("@trade") {
            return Some((MARKET_EVENT_TRADE, data));
        }
        if stream.contains("@depth") {
            return Some((MARKET_EVENT_DEPTH, data));
        }
    }
    if v.get("e").and_then(|e| e.as_str()) == Some("trade") {
        return Some((MARKET_EVENT_TRADE, v));
    }
    None
}
