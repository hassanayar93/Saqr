use super::env_settings::SaqrSettings;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SettingsValidationError {
    #[error("SAQR_MODE must be demo, testnet, or live (got '{0}')")]
    InvalidMode(String),
    #[error("SAQR_SYMBOL invalid: must be 6–20 uppercase alphanumeric characters")]
    InvalidSymbol,
    #[error("SAQR_INITIAL_BALANCE_USD must be > 0")]
    InvalidBalance,
    #[error("BINANCE_API_KEY is required for mode '{0}'")]
    MissingBinanceApiKey(String),
    #[error("BINANCE_API_SECRET is required for mode '{0}'")]
    MissingBinanceApiSecret(String),
    #[error("BINANCE_API_KEY looks corrupted (hidden whitespace or quotes)")]
    CorruptBinanceApiKey,
    #[error("BINANCE_API_SECRET looks corrupted (hidden whitespace or quotes)")]
    CorruptBinanceApiSecret,
    #[error("BINANCE_REST_URL must start with https://")]
    InvalidRestUrl,
    #[error("BINANCE_WS_URL must start with wss://")]
    InvalidWsUrl,
    #[error("BINANCE_RECV_WINDOW must be between 1000 and 60000 ms")]
    InvalidRecvWindow,
    #[error("mode testnet requires BINANCE_TESTNET=true")]
    TestnetFlagMismatch,
    #[error("mode live requires BINANCE_TESTNET=false")]
    LiveTestnetFlagMismatch,
    #[error("live trading requires SAQR_ENABLE_LIVE_ACK=true (explicit risk acceptance)")]
    LiveAckMissing,
    #[error("TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID must both be set or both empty")]
    TelegramPartialConfig,
    #[error("TELEGRAM_BOT_TOKEN looks invalid or corrupted")]
    CorruptTelegramToken,
    #[error("TELEGRAM_CHAT_ID must be a numeric chat id")]
    InvalidTelegramChatId,
    #[error("SAQR_ENABLE_TRADING=true requires mode testnet or live")]
    TradingFlagModeMismatch,
    #[error("SAQR_WEB_BIND_PORT must be 1–65535")]
    InvalidWebPort,
}

impl SaqrSettings {
    /// يرفض التشغيل عند بيانات ناقصة، متناقضة، أو توكنات تالفة (مسافات/اقتباسات مخفية).
    pub fn validate(&self) -> Result<(), Vec<SettingsValidationError>> {
        let mut errors = Vec::new();

        let mode = self.mode.to_ascii_lowercase();
        if !matches!(mode.as_str(), "demo" | "testnet" | "live") {
            errors.push(SettingsValidationError::InvalidMode(self.mode.clone()));
        }

        if !is_valid_symbol(&self.symbol) {
            errors.push(SettingsValidationError::InvalidSymbol);
        }

        if self.initial_balance_usd <= 0.0 || !self.initial_balance_usd.is_finite() {
            errors.push(SettingsValidationError::InvalidBalance);
        }

        if self.web_bind_port == 0 {
            errors.push(SettingsValidationError::InvalidWebPort);
        }

        if !self.binance.rest_base_url.starts_with("https://") {
            errors.push(SettingsValidationError::InvalidRestUrl);
        }
        if !self.binance.ws_base_url.starts_with("wss://") {
            errors.push(SettingsValidationError::InvalidWsUrl);
        }
        if !(1_000..=60_000).contains(&self.binance.recv_window_ms) {
            errors.push(SettingsValidationError::InvalidRecvWindow);
        }

        if self.enable_trading && !matches!(mode.as_str(), "testnet" | "live") {
            errors.push(SettingsValidationError::TradingFlagModeMismatch);
        }

        if matches!(mode.as_str(), "testnet" | "live") {
            if self.api_key.is_empty() {
                errors.push(SettingsValidationError::MissingBinanceApiKey(mode.clone()));
            }
            if self.api_secret.is_empty() {
                errors.push(SettingsValidationError::MissingBinanceApiSecret(mode.clone()));
            }
        }

        if had_untrimmed_secret("BINANCE_API_KEY") {
            errors.push(SettingsValidationError::CorruptBinanceApiKey);
        }
        if had_untrimmed_secret("BINANCE_API_SECRET") {
            errors.push(SettingsValidationError::CorruptBinanceApiSecret);
        }

        if mode == "testnet" && !self.binance.testnet {
            errors.push(SettingsValidationError::TestnetFlagMismatch);
        }
        if mode == "live" {
            if self.binance.testnet {
                errors.push(SettingsValidationError::LiveTestnetFlagMismatch);
            }
            if !self.enable_live_ack {
                errors.push(SettingsValidationError::LiveAckMissing);
            }
        }

        let tg_token = self.telegram.bot_token.is_empty();
        let tg_chat = self.telegram.chat_id.is_empty();
        if tg_token ^ tg_chat {
            errors.push(SettingsValidationError::TelegramPartialConfig);
        }
        if !tg_token {
            if had_untrimmed_secret("TELEGRAM_BOT_TOKEN") {
                errors.push(SettingsValidationError::CorruptTelegramToken);
            } else if !is_plausible_telegram_token(&self.telegram.bot_token) {
                errors.push(SettingsValidationError::CorruptTelegramToken);
            }
            if !is_plausible_chat_id(&self.telegram.chat_id) {
                errors.push(SettingsValidationError::InvalidTelegramChatId);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn is_valid_symbol(symbol: &str) -> bool {
    let s = symbol.trim();
    (6..=20).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_uppercase() && b.is_ascii_alphanumeric())
}

fn is_plausible_telegram_token(token: &str) -> bool {
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    parts[0].chars().all(|c| c.is_ascii_digit())
        && parts[1].len() >= 20
        && parts[1]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn is_plausible_chat_id(chat_id: &str) -> bool {
    let t = chat_id.trim();
    if t.is_empty() {
        return false;
    }
    let body = t.strip_prefix('-').unwrap_or(t);
    !body.is_empty() && body.chars().all(|c| c.is_ascii_digit())
}

fn had_untrimmed_secret(key: &str) -> bool {
    match std::env::var(key) {
        Ok(raw) => {
            let trimmed = sanitize_env_value(&raw);
            raw != trimmed && !trimmed.is_empty()
        }
        Err(_) => false,
    }
}

pub fn sanitize_env_value(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env_settings::{
        BinanceEndpoints, CloudServerSettings, SaqrSettings, TelegramSettings,
    };

    fn base_settings(mode: &str) -> SaqrSettings {
        SaqrSettings {
            mode: mode.into(),
            initial_balance_usd: 500.0,
            symbol: "BTCUSDT".into(),
            api_key: "k".repeat(64),
            api_secret: "s".repeat(64),
            enable_trading: false,
            enable_live_ack: false,
            binance: BinanceEndpoints {
                rest_base_url: "https://testnet.binance.vision".into(),
                ws_base_url: "wss://testnet.binance.vision".into(),
                spot_web_url: String::new(),
                recv_window_ms: 5000,
                testnet: true,
            },
            telegram: TelegramSettings {
                bot_token: String::new(),
                chat_id: String::new(),
                secondary_bot_token: None,
            },
            cloud: CloudServerSettings {
                host: String::new(),
                port: 22,
                ssh_user: String::new(),
                password: String::new(),
                public_dashboard_url: String::new(),
            },
            web_bind_host: "127.0.0.1".into(),
            web_bind_port: 8787,
        }
    }

    #[test]
    fn demo_mode_allows_empty_keys() {
        let mut s = base_settings("demo");
        s.api_key.clear();
        s.api_secret.clear();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn testnet_requires_keys() {
        let mut s = base_settings("testnet");
        s.api_key.clear();
        let err = s.validate().unwrap_err();
        assert!(err
            .iter()
            .any(|e| matches!(e, SettingsValidationError::MissingBinanceApiKey(_))));
    }

    #[test]
    fn live_requires_ack() {
        let mut s = base_settings("live");
        s.binance.testnet = false;
        s.enable_live_ack = false;
        let err = s.validate().unwrap_err();
        assert!(err
            .iter()
            .any(|e| matches!(e, SettingsValidationError::LiveAckMissing)));
    }

    #[test]
    fn telegram_partial_fails() {
        let mut s = base_settings("demo");
        s.telegram.bot_token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdef".into();
        let err = s.validate().unwrap_err();
        assert!(err
            .iter()
            .any(|e| matches!(e, SettingsValidationError::TelegramPartialConfig)));
    }
}
