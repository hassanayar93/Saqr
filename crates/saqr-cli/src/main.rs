mod gc_worker;
mod integrations_bridge;
mod web_server;

use std::path::PathBuf;
use std::sync::Arc;

use saqr_core::config::env_settings::SaqrSettings;
use saqr_core::orchestrator::decision_cascade::DecisionCascade;
use saqr_core::orchestrator::phase4_master_orchestrator::Phase4MasterOrchestrator;
use saqr_core::orchestrator::sovereign_matrix::{SovereignMatrix, SovereignMatrixInputs};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let settings = SaqrSettings::from_env();
    let (gc_tx, gc_rx) = mpsc::channel(1024);
    tokio::spawn(gc_worker::run_gc_worker(gc_rx));

    let orchestrator = Phase4MasterOrchestrator::new(settings.initial_balance_usd);
    let cascade = DecisionCascade::new();
    orchestrator.heartbeat();

    let level0 = cascade.level0_macro(0, true);
    tracing::info!("Decision cascade level0: {:?}", level0);

    let matrix_ok = SovereignMatrix::commander_may_fire(SovereignMatrixInputs {
        is_execution_safe: true,
        macro_risk_score: 0.05,
        momentum_consensus: 90.0,
        sovereign_score: 92.0,
    });
    tracing::info!("Sovereign matrix pass={matrix_ok} mode={}", settings.mode);

    integrations_bridge::spawn_integrations_bridge(settings.clone());

    let web_root = resolve_web_root();
    let settings_arc = settings.clone();
    tokio::spawn(async move {
        if let Err(e) = web_server::start_web(settings_arc, &web_root).await {
            tracing::error!("web server error: {e}");
        }
    });

    let _ = gc_tx.send(0).await;
    orchestrator.heartbeat();

    tracing::info!(
        "صقر SAQR Engine v6.0 — orchestrator running (offset={:?})",
        Arc::strong_count(&orchestrator.time_offset())
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
