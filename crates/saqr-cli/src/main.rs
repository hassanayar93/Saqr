mod chrono_sync_worker;
mod engine_launcher;
mod gc_worker;
mod integrations_bridge;
mod orphan_sweep;
mod startup_checklist;
mod web_server;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::orchestrator::phase4_master_orchestrator::Phase4MasterOrchestrator;
use saqr_core::telegram::anti_flood::TelegramAntiFlood;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let settings = SaqrSettings::from_env();
    if let Err(errors) = settings.validate() {
        for e in &errors {
            tracing::error!("settings validation: {e}");
        }
        tracing::error!("Refusing to start — fix .env (see .env.example) and retry.");
        return ExitCode::from(1);
    }

    let telegram_flood = Arc::new(TelegramAntiFlood::new());
    let checklist =
        startup_checklist::run_startup_checklist(&settings, Arc::clone(&telegram_flood)).await;
    checklist.log_summary();
    if checklist.hard_failure {
        tracing::error!("Startup checklist failed for mode '{}'.", settings.mode);
        return ExitCode::from(2);
    }

    if let Err(e) = run_engine(settings, telegram_flood).await {
        tracing::error!("engine stopped: {e}");
        return ExitCode::from(3);
    }
    ExitCode::SUCCESS
}

async fn run_engine(
    settings: SaqrSettings,
    telegram_flood: Arc<TelegramAntiFlood>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (gc_tx, gc_rx) = mpsc::channel(1024);
    tokio::spawn(gc_worker::run_gc_worker(gc_rx));

    let orchestrator = Phase4MasterOrchestrator::new(settings.initial_balance_usd);
    orchestrator.heartbeat();

    let launched = engine_launcher::launch(settings.clone(), gc_tx.clone()).await;

    integrations_bridge::spawn_integrations_bridge(settings.clone(), telegram_flood);

    let web_root = resolve_web_root();
    let engine = Arc::clone(&launched.runtime);
    let settings_web = settings.clone();
    tokio::spawn(async move {
        if let Err(e) = web_server::start_web(settings_web, engine, &web_root).await {
            tracing::error!("web server error: {e}");
        }
    });

    let _ = gc_tx.send(0).await;
    orchestrator.heartbeat();

    tracing::info!(
        "صقر SAQR Engine v6.0 — hot loop + WS active (web {}:{})",
        settings.web_bind_host,
        settings.web_bind_port
    );

    tokio::signal::ctrl_c().await?;
    Ok(())
}

fn resolve_web_root() -> String {
    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/control_terminal"),
        PathBuf::from("/home/ubuntu/saqr/web/control_terminal"),
    ];
    for p in candidates {
        if p.join("index.html").is_file() {
            return p.to_string_lossy().into_owned();
        }
    }
    "web/control_terminal".into()
}
