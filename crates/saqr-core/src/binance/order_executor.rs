use std::sync::Arc;

use parking_lot::Mutex;
use uuid::Uuid;

use super::exchange_rules::ExchangeRulesCache;
use super::signed_rest::BinanceSignedRest;
use crate::config::env_settings::SaqrSettings;
use crate::conquest::selected_battalion::SelectedBattalion;
use crate::resilience::orphan_guardian::OrphanOrderGuardian;
use crate::resilience::rate_limit_shield::RateLimitShield;
use crate::resilience::rocksdb_wal::RocksDbWalJournal;
use crate::routing::compliant_order_router::CompliantOrderRouter;
use crate::routing::self_trade_prevention_mode::SelfTradePreventionMode;

pub struct OrderExecutor {
    settings: SaqrSettings,
    signed: BinanceSignedRest,
    rules: Arc<ExchangeRulesCache>,
    wal: Arc<Mutex<RocksDbWalJournal>>,
    orphan: Arc<OrphanOrderGuardian>,
    rate_shield: Arc<RateLimitShield>,
    epoch: u64,
}

impl OrderExecutor {
    pub fn new(
        settings: SaqrSettings,
        signed: BinanceSignedRest,
        rules: Arc<ExchangeRulesCache>,
        wal: Arc<Mutex<RocksDbWalJournal>>,
        orphan: Arc<OrphanOrderGuardian>,
        rate_shield: Arc<RateLimitShield>,
        epoch: u64,
    ) -> Self {
        Self {
            settings,
            signed,
            rules,
            wal,
            orphan,
            rate_shield,
            epoch,
        }
    }

    pub async fn execute_limit_ioc_buy(
        &self,
        battalion: &SelectedBattalion,
        kelly_fraction: f64,
        now_ms: u64,
    ) -> Result<String, String> {
        if !self.settings.enable_trading {
            return Err("SAQR_ENABLE_TRADING=false".into());
        }
        if !matches!(self.settings.mode.as_str(), "testnet" | "live") {
            return Err("order execution only in testnet/live".into());
        }
        if !self.rate_shield.allow(1, now_ms) {
            return Err("rate limit shield".into());
        }

        let rules = self.rules.get();
        let notional = self.settings.initial_balance_usd * kelly_fraction;
        let qty = (notional / battalion.entry_price).max(rules.min_qty);
        let qty = round_step(qty, rules.step_size);
        let price = round_step(battalion.entry_price, rules.tick_size);

        let qty_scaled = (qty * 100_000_000.0) as i64;
        let price_scaled = (price * 100_000_000.0) as i64;

        let client_order_id = format!("saqr_{}", Uuid::new_v4().simple());
        let payload = CompliantOrderRouter::build_safe_order_payload(
            &self.settings.symbol,
            "BUY",
            price_scaled,
            qty_scaled,
            SelfTradePreventionMode::ExpireMaker,
        );

        {
            let mut wal = self.wal.lock();
            wal.append_pre_send(crate::resilience::rocksdb_wal::WalOrderRecord {
                client_order_id: client_order_id.clone(),
                payload_json: payload.clone(),
                epoch: self.epoch,
                pre_send: true,
            });
        }

        self.orphan
            .register_local_order(client_order_id.clone(), self.epoch);

        let query = self.signed.build_signed_order_query(&[
            ("symbol", self.settings.symbol.clone()),
            ("side", "BUY".into()),
            ("type", "LIMIT".into()),
            ("timeInForce", "IOC".into()),
            ("quantity", format!("{qty:.8}")),
            ("price", format!("{price:.8}")),
            ("newClientOrderId", client_order_id.clone()),
            (
                "selfTradePreventionMode",
                SelfTradePreventionMode::ExpireMaker.as_binance_str().to_string(),
            ),
        ])?;

        let resp = self.signed.place_order_form(&query).await?;
        tracing::info!(
            "order placed id={} status={}",
            resp.order_id,
            resp.status
        );
        Ok(client_order_id)
    }
}

fn round_step(value: f64, step: f64) -> f64 {
    if step <= 0.0 {
        return value;
    }
    (value / step).floor() * step
}
