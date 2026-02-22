mod config;
mod input;
mod obs;
mod server;
mod state;
mod swapper;
mod windows;

use std::net::SocketAddr;
use std::sync::Arc;

use config::ConfigManager;
use state::AppState;
use swapper::Swapper;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = std::env::current_dir()?.join("config.toml");
    let config_manager = Arc::new(ConfigManager::new(config_path)?);
    let app_state = Arc::new(AppState::new());
    let swapper = Arc::new(Swapper::new(
        config_manager.config(),
        app_state.clone(),
    ));

    let swapper_clone = swapper.clone();
    tokio::spawn(async move {
        swapper_clone.run().await;
    });

    let app_state_clone = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let state = app_state_clone.get_state().await;
            app_state_clone.broadcaster.broadcast(state);
        }
    });

    let app = server::create_app(
        config_manager.clone(),
        app_state.clone(),
        swapper.clone(),
    );

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    info!("===============================================");
    info!("DonaldSwap is running!");
    info!("Dashboard:       http://{}", addr);
    info!("OBS Browser Src: http://{}/obs.html", addr);
    info!("===============================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
