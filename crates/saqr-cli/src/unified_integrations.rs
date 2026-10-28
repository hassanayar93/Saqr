use std::sync::Arc;

use saqr_core::binance::http_client::BinanceHttpClient;
use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::engine::runtime::EngineRuntime;
use saqr_core::telegram::anti_flood::TelegramAntiFlood;
use saqr_core::telegram::send_message::TelegramClient;
use tokio::time::{interval, Duration};

/// جسر موحّد: Binance Testnet (بيانات حقيقية) + Telegram + حالة المحرك للوحة.
pub fn spawn_unified_integrations(
    settings: SaqrSettings,
    telegram_flood: Arc<TelegramAntiFlood>,
    engine: Arc<EngineRuntime>,
) {
    tokio::spawn(async move {
        let period = std::env::var("SAQR_INTEGRATION_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300u64);

        if let Err(e) = run_cycle(&settings, &telegram_flood, &engine).await {
            tracing::error!("unified integration cycle: {e}");
        }

        let mut tick = interval(Duration::from_secs(period));
        tick.tick().await;
        loop {
            tick.tick().await;
            if let Err(e) = run_cycle(&settings, &telegram_flood, &engine).await {
                tracing::error!("unified integration cycle: {e}");
            }
        }
    });
}

async fn run_cycle(
    settings: &SaqrSettings,
    telegram_flood: &Arc<TelegramAntiFlood>,
    engine: &Arc<EngineRuntime>,
) -> Result<(), String> {
    engine.refresh_status_snapshot();
    let st = engine.status.read().clone();

    let binance = BinanceHttpClient::new(settings.clone());
    let binance_block = match binance.ping().await {
        Ok(()) => {
            let price = binance.ticker_price().await.unwrap_or_else(|e| e);
            let server_ms = binance
                .server_time()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|e| e);
            format!(
                "📊 <b>Binance Testnet (حقيقي)</b>\nSymbol: <code>{}</code>\nPrice: <code>{}</code>\nServer: <code>{}</code>",
                settings.symbol, price, server_ms
            )
        }
        Err(e) => format!("⚠️ Binance: <code>{e}</code>"),
    };

    let engine_block = format!(
        "🖥 <b>Engine</b>\nWS: <code>{}</code> | Price: <code>{:.4}</code>\nVerdict: <code>{}</code> | Signals: <code>{}</code>",
        st.ws_connected,
        st.last_price,
        st.last_verdict,
        engine.signals.read().len()
    );

    let tg = TelegramClient::with_anti_flood(settings.telegram.clone(), Arc::clone(telegram_flood));
    if !tg.is_configured() {
        tracing::warn!("Telegram not configured — dashboard/API still live");
        return Ok(());
    }

    let dashboard = format!("http://127.0.0.1:{}", settings.web_bind_port);
    let text = format!(
        "🦅 <b>صقر SAQR</b> — تكامل Demo\nMode: <code>{}</code> | Trading: <code>{}</code>\n{}\n{}\n🔗 لوحة: <code>{}</code>",
        settings.mode,
        settings.enable_trading,
        binance_block,
        engine_block,
        dashboard
    );
    tg.send_text(&text).await?;
    tracing::info!("unified integration: Binance + Engine + Telegram");
    Ok(())
}
