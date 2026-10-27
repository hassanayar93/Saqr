use std::env;

use super::settings_validate::sanitize_env_value;

#[derive(Debug, Clone)]
pub struct BinanceEndpoints {
    pub rest_base_url: String,
    pub ws_base_url: String,
    pub spot_web_url: String,
    pub recv_window_ms: u64,
    pub testnet: bool,
}

#[derive(Debug, Clone)]
pub struct TelegramSettings {
    pub bot_token: String,
    pub chat_id: String,
    pub secondary_bot_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CloudServerSettings {
    pub host: String,
    pub port: u16,
    pub ssh_user: String,
    pub password: String,
    pub public_dashboard_url: String,
}

#[derive(Debug, Clone)]
pub struct SaqrSettings {
    pub mode: String,
    pub initial_balance_usd: f64,
    pub symbol: String,
    pub api_key: String,
    pub api_secret: String,
    /// عند true: مسار التنفيذ (testnet/live) — افتراضي false في demo
    pub enable_trading: bool,
    /// مطلوب SAQR_ENABLE_LIVE_ACK=true قبل SAQR_MODE=live
    pub enable_live_ack: bool,
    pub binance: BinanceEndpoints,
    pub telegram: TelegramSettings,
    pub cloud: CloudServerSettings,
    pub web_bind_host: String,
    pub web_bind_port: u16,
}

impl SaqrSettings {
    pub fn from_env() -> Self {
        let testnet = env_bool("BINANCE_TESTNET", true);
        let rest = env::var("BINANCE_REST_URL").unwrap_or_else(|_| {
            if testnet {
                "https://testnet.binance.vision".into()
            } else {
                "https://api.binance.com".into()
            }
        });
        let ws = env::var("BINANCE_WS_URL").unwrap_or_else(|_| {
            if testnet {
                "wss://testnet.binance.vision".into()
            } else {
                "wss://stream.binance.com:9443".into()
            }
        });

        let mode = sanitize_env_value(&env::var("SAQR_MODE").unwrap_or_else(|_| "demo".into()))
            .to_ascii_lowercase();

        Self {
            mode,
            initial_balance_usd: env_f64("SAQR_INITIAL_BALANCE_USD", 500.0),
            symbol: sanitize_env_value(
                &env::var("SAQR_SYMBOL").unwrap_or_else(|_| "BTCUSDT".into()),
            )
            .to_ascii_uppercase(),
            api_key: env_secret("BINANCE_API_KEY"),
            api_secret: env_secret("BINANCE_API_SECRET"),
            enable_trading: env_bool("SAQR_ENABLE_TRADING", false),
            enable_live_ack: env_bool("SAQR_ENABLE_LIVE_ACK", false),
            binance: BinanceEndpoints {
                rest_base_url: sanitize_env_value(&rest),
                ws_base_url: sanitize_env_value(&ws),
                spot_web_url: sanitize_env_value(
                    &env::var("BINANCE_SPOT_WEB_URL").unwrap_or_default(),
                ),
                recv_window_ms: env::var("BINANCE_RECV_WINDOW")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5000),
                testnet,
            },
            telegram: TelegramSettings {
                bot_token: env_secret("TELEGRAM_BOT_TOKEN"),
                chat_id: sanitize_env_value(
                    &env::var("TELEGRAM_CHAT_ID").unwrap_or_default(),
                ),
                secondary_bot_token: env::var("TELEGRAM_SECONDARY_BOT_TOKEN")
                    .ok()
                    .map(|v| env_secret_raw(&v)),
            },
            cloud: CloudServerSettings {
                host: env::var("CLOUD_SERVER_HOST").unwrap_or_default(),
                port: env_u16("CLOUD_SERVER_PORT", 22),
                ssh_user: env::var("CLOUD_SERVER_SSH_USER").unwrap_or_default(),
                password: env::var("CLOUD_SERVER_PASSWORD").unwrap_or_default(),
                public_dashboard_url: env::var("SAQR_PUBLIC_DASHBOARD_URL").unwrap_or_default(),
            },
            web_bind_host: env::var("SAQR_WEB_BIND_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            web_bind_port: env_u16("SAQR_WEB_BIND_PORT", 8787),
        }
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u16(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_secret(key: &str) -> String {
    env::var(key)
        .map(|v| env_secret_raw(&v))
        .unwrap_or_default()
}

fn env_secret_raw(raw: &str) -> String {
    sanitize_env_value(raw)
}
