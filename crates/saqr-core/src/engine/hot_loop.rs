use std::sync::atomic::Ordering;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::algorithms::hira::HiraEngine;
use crate::algorithms::mmid::MmidDetector;
use crate::algorithms::mvrs::MvrsPredictor;
use crate::binance::order_executor::OrderExecutor;
use crate::conquest::selected_battalion::SelectedBattalion;
use crate::conquest::sovereign_conquest_engine::SovereignConquestEngine;
use crate::core::unified_market_state::SCALE_8;
use crate::engine::runtime::{now_ms, EngineRuntime, EngineSignalDto};
use crate::liquidity::order_side::OrderSide;
use crate::orchestrator::decision_cascade::{CascadeVerdict, DecisionCascade};
use crate::orchestrator::sovereign_matrix::{SovereignMatrix, SovereignMatrixInputs};
use crate::resilience::rocksdb_wal::SharedWalJournal;
use crate::time::chrono_guard::ChronoGuard;

pub struct HotLoopContext {
    pub runtime: Arc<EngineRuntime>,
    pub cascade: DecisionCascade,
    pub chrono: ChronoGuard,
    pub conquest: Arc<Mutex<SovereignConquestEngine>>,
    pub order_executor: Option<Arc<OrderExecutor>>,
    pub wal: SharedWalJournal,
}

impl HotLoopContext {
    pub fn spawn(self) {
        tokio::spawn(async move {
            loop {
                self.tick().await;
                sleep(Duration::from_millis(2)).await;
            }
        });
    }

    async fn tick(&self) {
        self.runtime
            .hot_loop_ticks
            .fetch_add(1, Ordering::Relaxed);

        let mut processed = 0u32;
        while processed < 32 {
            let Some(event) = self.runtime.event_bus.try_pop() else {
                break;
            };
            processed += 1;
            let _ = event;
        }

        let state = *self.runtime.market_state.read();
        if state.adjusted_price_scaled == 0 {
            self.runtime.refresh_status_snapshot();
            return;
        }

        let price = state.adjusted_price_scaled as f64 / SCALE_8 as f64;
        let bid_depth = *self.runtime.last_bid_depth.read();
        let ask_depth = *self.runtime.last_ask_depth.read();

        let local_bucket_ok = self
            .conquest
            .lock()
            .await
            .weight_bucket
            .can_send_request(5);

        let l0 = self.cascade.level0_macro(0, local_bucket_ok);
        if l0 != CascadeVerdict::Pass {
            self.store_verdict(l0);
            self.runtime.refresh_status_snapshot();
            return;
        }

        let consensus = state.consensus_score_scaled as f64 / 1_000_000.0;
        let l1 = DecisionCascade::level1_battalion(consensus, 0.05);
        if l1 != CascadeVerdict::Pass {
            self.store_verdict(l1);
            self.runtime.refresh_status_snapshot();
            return;
        }

        let l2 = DecisionCascade::level2_environment(
            &self.chrono,
            OrderSide::Buy,
            0.01,
            bid_depth,
            ask_depth,
            &state,
        );
        if l2 != CascadeVerdict::Pass {
            self.store_verdict(l2);
            self.runtime.refresh_status_snapshot();
            return;
        }

        let hidden = HiraEngine::estimate_hidden_size_scaled(state.cvd_h1_scaled / 100, 60_000_000);
        let mmid = MmidDetector::trap_score_scaled(20_000_000, 10_000_000);
        let mvrs = MvrsPredictor::entropy_burst_scaled(mmid, state.cvd_h1_scaled);
        let sovereign_score = (consensus * 0.5)
            + (hidden as f64 / SCALE_8 as f64 * 10.0)
            + (mvrs as f64 / 1_000_000.0);

        let l3 = DecisionCascade::level3_execution(consensus, sovereign_score);
        self.store_verdict(l3);
        if l3 != CascadeVerdict::Pass {
            self.runtime.refresh_status_snapshot();
            return;
        }

        let matrix_ok = SovereignMatrix::commander_may_fire(SovereignMatrixInputs {
            is_execution_safe: state.is_safe_to_execute() && !self.runtime.emergency.is_locked(),
            macro_risk_score: 0.05,
            momentum_consensus: consensus,
            sovereign_score,
        });
        if !matrix_ok {
            self.runtime.refresh_status_snapshot();
            return;
        }

        let signal_id = self
            .runtime
            .signal_id_seq
            .fetch_add(1, Ordering::Relaxed);
        self.runtime.push_signal(EngineSignalDto {
            id: signal_id,
            pair: format!("{}/USDT", trim_usdt(&self.runtime.settings.symbol)),
            direction: "BUY".into(),
            strength_score: consensus as u32,
            timeframe: "live".into(),
            executed: false,
            sovereign_score,
            hira_boost: hidden as f64 / SCALE_8 as f64,
        });

        {
            let mut st = self.runtime.status.write();
            st.sovereign_score = sovereign_score;
        }

        if let Some(exec) = &self.order_executor {
            let now = now_ms();
            let last = self
                .runtime
                .last_order_attempt_ms
                .load(Ordering::Acquire);
            if now.saturating_sub(last) < 30_000 {
                self.runtime.refresh_status_snapshot();
                return;
            }
            if self.runtime.positions.read().len() >= 1 {
                self.runtime.refresh_status_snapshot();
                return;
            }
            self.runtime
                .last_order_attempt_ms
                .store(now, Ordering::Release);
            let battalion = SelectedBattalion {
                symbol: self.runtime.settings.symbol.clone(),
                volume_acceleration: 1.1,
                consensus_score: consensus,
                entry_price: price,
                atr_volatility: 0.012,
                win_rate_historical: 0.62,
                win_loss_ratio: 1.8,
            };
            let order_plan = {
                let mut conquest = self.conquest.lock().await;
                conquest.select_prime_battalion(vec![battalion])
            };
            if let Some((prime, kelly)) = order_plan {
                let now = now_ms();
                match exec
                    .execute_limit_ioc_buy(&prime, kelly, now)
                    .await
                {
                    Ok(client_id) => {
                        let pos_id = self
                            .runtime
                            .position_id_seq
                            .fetch_add(1, Ordering::Relaxed);
                        self.runtime.positions.write().push(
                            crate::engine::runtime::EnginePositionDto {
                                id: pos_id,
                                pair: format!("{}/USDT", trim_usdt(&self.runtime.settings.symbol)),
                                side: "BUY".into(),
                                lot: format!("{kelly:.4}"),
                                entry: price,
                                current: price,
                                pnl: 0.0,
                                client_order_id: client_id,
                            },
                        );
                        if let Some(sig) = self.runtime.signals.write().first_mut() {
                            sig.executed = true;
                        }
                    }
                    Err(e) => tracing::warn!("order execution skipped: {e}"),
                }
            }
        }

        self.runtime.wal_pending.store(
            self.wal.lock().pending_count() as u64,
            Ordering::Release,
        );
        self.runtime.refresh_status_snapshot();
    }

    fn store_verdict(&self, v: CascadeVerdict) {
        *self.runtime.last_verdict.write() = v;
    }
}

fn trim_usdt(symbol: &str) -> &str {
    symbol.strip_suffix("USDT").unwrap_or(symbol)
}
