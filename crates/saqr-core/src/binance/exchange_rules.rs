use std::sync::Arc;

use parking_lot::RwLock;
use reqwest::Client;
use serde::Deserialize;

use crate::config::env_settings::SaqrSettings;

#[derive(Debug, Clone, Default)]
pub struct SymbolRules {
    pub min_qty: f64,
    pub step_size: f64,
    pub tick_size: f64,
}

#[derive(Debug, Deserialize)]
struct ExchangeInfo {
    symbols: Vec<SymbolInfo>,
}

#[derive(Debug, Deserialize)]
struct SymbolInfo {
    symbol: String,
    filters: Vec<serde_json::Value>,
}

pub struct ExchangeRulesCache {
    inner: RwLock<SymbolRules>,
}

impl ExchangeRulesCache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(SymbolRules {
                min_qty: 0.00001,
                step_size: 0.00001,
                tick_size: 0.01,
            }),
        }
    }

    pub fn get(&self) -> SymbolRules {
        self.inner.read().clone()
    }

    pub async fn refresh_from_rest(settings: &SaqrSettings) -> Arc<Self> {
        let cache = Arc::new(Self::new());
        let url = format!(
            "{}/api/v3/exchangeInfo?symbol={}",
            settings.binance.rest_base_url.trim_end_matches('/'),
            settings.symbol
        );
        let http = Client::new();
        let Ok(resp) = http.get(&url).send().await else {
            return cache;
        };
        let Ok(info) = resp.json::<ExchangeInfo>().await else {
            return cache;
        };
        if let Some(sym) = info.symbols.into_iter().find(|s| s.symbol == settings.symbol) {
            let mut rules = SymbolRules::default();
            for f in sym.filters {
                let filter_type = f.get("filterType").and_then(|v| v.as_str()).unwrap_or("");
                match filter_type {
                    "LOT_SIZE" => {
                        rules.min_qty = parse_filter_f64(&f, "minQty", rules.min_qty);
                        rules.step_size = parse_filter_f64(&f, "stepSize", rules.step_size);
                    }
                    "PRICE_FILTER" => {
                        rules.tick_size = parse_filter_f64(&f, "tickSize", rules.tick_size);
                    }
                    _ => {}
                }
            }
            *cache.inner.write() = rules;
        }
        cache
    }
}

fn parse_filter_f64(v: &serde_json::Value, key: &str, default: f64) -> f64 {
    v.get(key)
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
