use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;

use super::rest_client::BinanceRestClient;
use crate::config::env_settings::SaqrSettings;
use crate::time::chrono_guard::ChronoGuard;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct AccountResponse {
    pub balances: Vec<BalanceEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BalanceEntry {
    pub asset: String,
    #[serde(rename = "free")]
    pub free: String,
}

#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub status: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenOrderRow {
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub symbol: String,
    pub status: String,
}

pub struct BinanceSignedRest {
    settings: SaqrSettings,
    rest: BinanceRestClient,
    http: Client,
    chrono: ChronoGuard,
}

impl BinanceSignedRest {
    pub fn new(settings: SaqrSettings, time_offset: Arc<AtomicI64>) -> Self {
        let rest = BinanceRestClient::new(settings.binance.clone());
        Self {
            settings,
            rest,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .unwrap_or_default(),
            chrono: ChronoGuard::new(time_offset),
        }
    }

    fn sign_query(&self, query: &str) -> Result<String, String> {
        let mut mac = HmacSha256::new_from_slice(self.settings.api_secret.as_bytes())
            .map_err(|e| e.to_string())?;
        mac.update(query.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    fn timestamped_query(&self, params: &[(&str, String)]) -> Result<String, String> {
        let ts = self
            .chrono
            .get_safe_exchange_timestamp()
            .map_err(|e| e.to_string())?;
        let mut pairs: Vec<(String, String)> = params
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        pairs.push(("timestamp".into(), ts.to_string()));
        pairs.push((
            "recvWindow".into(),
            self.settings.binance.recv_window_ms.to_string(),
        ));
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let query = pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        let sig = self.sign_query(&query)?;
        Ok(format!("{query}&signature={sig}"))
    }

    pub async fn get_account(&self) -> Result<AccountResponse, String> {
        let query = self.timestamped_query(&[])?;
        let url = format!(
            "{}/api/v3/account?{query}",
            self.settings.binance.rest_base_url.trim_end_matches('/')
        );
        self.signed_get(&url).await
    }

    pub async fn open_orders(&self, symbol: &str) -> Result<Vec<OpenOrderRow>, String> {
        let query = self.timestamped_query(&[("symbol", symbol.to_string())])?;
        let url = format!(
            "{}/api/v3/openOrders?{query}",
            self.settings.binance.rest_base_url.trim_end_matches('/')
        );
        self.signed_get(&url).await
    }

    pub async fn place_order_form(&self, form: &str) -> Result<OrderResponse, String> {
        let url = format!(
            "{}/api/v3/order?{form}",
            self.settings.binance.rest_base_url.trim_end_matches('/')
        );
        self.signed_post(&url).await
    }

    pub fn build_signed_order_query(
        &self,
        params: &[(&str, String)],
    ) -> Result<String, String> {
        self.timestamped_query(params)
    }

    pub async fn cancel_order(&self, symbol: &str, client_order_id: &str) -> Result<(), String> {
        let query = self.timestamped_query(&[
            ("symbol", symbol.to_string()),
            ("origClientOrderId", client_order_id.to_string()),
        ])?;
        let url = format!(
            "{}/api/v3/order?{query}",
            self.settings.binance.rest_base_url.trim_end_matches('/')
        );
        let _: serde_json::Value = self.signed_delete(&url).await?;
        Ok(())
    }

    async fn signed_get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        let resp = self
            .http
            .get(url)
            .header("X-MBX-APIKEY", &self.settings.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_json(resp).await
    }

    async fn signed_post<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        let resp = self
            .http
            .post(url)
            .header("X-MBX-APIKEY", &self.settings.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_json(resp).await
    }

    async fn signed_delete<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        let resp = self
            .http
            .delete(url)
            .header("X-MBX-APIKEY", &self.settings.api_key)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_json(resp).await
    }
}

async fn parse_json<T: for<'de> Deserialize<'de>>(
    resp: reqwest::Response,
) -> Result<T, String> {
    let status = resp.status();
    let body = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Binance HTTP {}: {body}", status.as_u16()));
    }
    serde_json::from_str(&body).map_err(|e| format!("json: {e} body={body}"))
}
