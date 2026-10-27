use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Deserialize;
use serde::Serialize;
use tokio::time::{sleep, Duration};

use super::anti_flood::TelegramAntiFlood;
use super::dispatcher::TelegramDispatcher;
use crate::config::env_settings::TelegramSettings;

#[derive(Serialize)]
struct SendMessageBody<'a> {
    chat_id: &'a str,
    text: &'a str,
    parse_mode: &'a str,
}

#[derive(Debug, Deserialize)]
struct TelegramApiResponse {
    ok: bool,
    #[serde(default)]
    description: String,
    result: Option<TelegramUser>,
}

#[derive(Debug, Deserialize)]
struct TelegramUser {
    username: Option<String>,
}

pub struct TelegramClient {
    dispatcher: TelegramDispatcher,
    http: reqwest::Client,
    anti_flood: Arc<TelegramAntiFlood>,
}

impl TelegramClient {
    pub fn new(settings: TelegramSettings) -> Self {
        Self::with_anti_flood(settings, Arc::new(TelegramAntiFlood::new()))
    }

    pub fn with_anti_flood(settings: TelegramSettings, anti_flood: Arc<TelegramAntiFlood>) -> Self {
        Self {
            dispatcher: TelegramDispatcher::new(settings),
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(25))
                .build()
                .unwrap_or_default(),
            anti_flood,
        }
    }

    pub fn anti_flood(&self) -> Arc<TelegramAntiFlood> {
        Arc::clone(&self.anti_flood)
    }

    pub fn is_configured(&self) -> bool {
        let url = self.dispatcher.api_send_url();
        !self.dispatcher.default_chat_id().is_empty()
            && url.starts_with("https://api.telegram.org/bot")
            && url.len() > 45
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    async fn wait_for_flood_gate(&self) {
        let delay = self.anti_flood.gate_delay_ms(Self::now_ms());
        if delay > 0 {
            sleep(Duration::from_millis(delay)).await;
        }
    }

    pub async fn verify_bot_token(&self) -> Result<String, String> {
        if self.dispatcher.bot_token().is_empty() {
            return Err("TELEGRAM_BOT_TOKEN missing".into());
        }
        self.wait_for_flood_gate().await;
        let url = format!(
            "https://api.telegram.org/bot{}/getMe",
            self.dispatcher.bot_token()
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status == 429 {
            self.anti_flood
                .note_http_rate_limited(Self::now_ms(), status);
            return Err(format!("Telegram rate limited (429): {body}"));
        }
        if status != 200 {
            return Err(format!("Telegram getMe HTTP {status}: {body}"));
        }
        let parsed: TelegramApiResponse = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if !parsed.ok {
            return Err(format!("Telegram getMe rejected: {}", parsed.description));
        }
        self.anti_flood.reset_on_success();
        Ok(parsed
            .result
            .and_then(|u| u.username)
            .unwrap_or_else(|| "unknown".into()))
    }

    pub async fn send_text(&self, text: &str) -> Result<(), String> {
        if self.dispatcher.default_chat_id().is_empty() {
            return Err("TELEGRAM_CHAT_ID missing".into());
        }
        let url = self.dispatcher.api_send_url();
        if url == "https://api.telegram.org/bot/sendMessage" {
            return Err("TELEGRAM_BOT_TOKEN missing".into());
        }

        self.wait_for_flood_gate().await;
        match self.post_message(&url, text).await {
            Ok(()) => {
                self.anti_flood.reset_on_success();
                Ok(())
            }
            Err(e) if e.contains("429") || e.to_lowercase().contains("too many requests") => {
                self.anti_flood
                    .note_http_rate_limited(Self::now_ms(), 429);
                if let Some(fallback) = self.dispatcher.fallback_api_send_url() {
                    tracing::warn!("Telegram primary flooded — trying secondary bot");
                    self.wait_for_flood_gate().await;
                    self.post_message(&fallback, text).await?;
                    self.anti_flood.reset_on_success();
                    Ok(())
                } else {
                    Err(e)
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn post_message(&self, url: &str, text: &str) -> Result<(), String> {
        let body = SendMessageBody {
            chat_id: self.dispatcher.default_chat_id(),
            text,
            parse_mode: "HTML",
        };

        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body = resp.text().await.unwrap_or_default();
            if code == 429 {
                self.anti_flood
                    .note_http_rate_limited(Self::now_ms(), code);
            }
            return Err(format!("Telegram HTTP {code}: {body}"));
        }
        Ok(())
    }
}
