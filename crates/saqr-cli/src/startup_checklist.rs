use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use saqr_core::binance::http_client::BinanceHttpClient;
use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::telegram::anti_flood::TelegramAntiFlood;
use saqr_core::telegram::send_message::TelegramClient;

#[derive(Debug, Clone)]
pub struct CheckLine {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug)]
pub struct StartupChecklistReport {
    pub lines: Vec<CheckLine>,
    pub warnings: Vec<String>,
    pub hard_failure: bool,
}

impl StartupChecklistReport {
    pub fn log_summary(&self) {
        tracing::info!("════════════════ SAQR Startup Checklist ════════════════");
        for line in &self.lines {
            let mark = if line.ok { "✓" } else { "✗" };
            tracing::info!("  [{mark}] {} — {}", line.name, line.detail);
        }
        for w in &self.warnings {
            tracing::warn!("  ⚠ {w}");
        }
        tracing::info!("═══════════════════════════════════════════════════════");
    }
}

pub async fn run_startup_checklist(
    settings: &SaqrSettings,
    telegram_flood: Arc<TelegramAntiFlood>,
) -> StartupChecklistReport {
    let mut lines = Vec::new();
    let mut warnings = Vec::new();
    let mode = settings.mode.to_ascii_lowercase();
    let strict_network = matches!(mode.as_str(), "testnet" | "live");

    lines.push(CheckLine {
        name: "Configuration",
        ok: true,
        detail: format!(
            "mode={} symbol={} trading={} web={}:{}",
            settings.mode,
            settings.symbol,
            settings.enable_trading,
            settings.web_bind_host,
            settings.web_bind_port
        ),
    });

    let binance = BinanceHttpClient::new(settings.clone());
    let binance_ping = binance.ping().await;
    let (binance_ok, binance_detail) = match binance_ping {
        Ok(()) => {
            let skew = match binance.server_time().await {
                Ok(server_ms) => {
                    let local_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    let skew = server_ms as i64 - local_ms as i64;
                    format!("reachable, clock skew ≈ {skew} ms")
                }
                Err(e) => format!("reachable, serverTime failed: {e}"),
            };
            (true, skew)
        }
        Err(e) => (false, e),
    };
    lines.push(CheckLine {
        name: "Binance REST ping",
        ok: binance_ok,
        detail: binance_detail,
    });
    if !binance_ok && !strict_network {
        warnings.push(
            "Binance ping failed in demo mode — bridge may be limited until VPS/network allows."
                .into(),
        );
    }

    let tg = TelegramClient::with_anti_flood(settings.telegram.clone(), telegram_flood);
    if tg.is_configured() {
        match tg.verify_bot_token().await {
            Ok(username) => {
                lines.push(CheckLine {
                    name: "Telegram getMe",
                    ok: true,
                    detail: format!("@{username}"),
                });
            }
            Err(e) => {
                lines.push(CheckLine {
                    name: "Telegram getMe",
                    ok: false,
                    detail: e,
                });
            }
        }
    } else {
        lines.push(CheckLine {
            name: "Telegram",
            ok: true,
            detail: "not configured (skipped)".into(),
        });
        if strict_network {
            warnings.push(
                "Telegram not configured — alerts disabled during testnet/live.".into(),
            );
        }
    }

    let hard_failure = if strict_network {
        !binance_ok
            || (tg.is_configured()
                && lines
                    .iter()
                    .any(|l| l.name == "Telegram getMe" && !l.ok))
    } else {
        false
    };

    StartupChecklistReport {
        lines,
        warnings,
        hard_failure,
    }
}
