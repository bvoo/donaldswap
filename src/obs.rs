use obws::Client;
use tracing::{error, info};

pub async fn switch_scene(
    host: &str,
    port: u16,
    password: Option<&str>,
    scene_name: &str,
) -> anyhow::Result<()> {
    info!("Connecting to OBS at {}:{}...", host, port);
    
    // Attempt to connect to OBS WebSocket
    let client = match Client::connect(host, port, password).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to OBS WebSocket: {:?}", e);
            anyhow::bail!("Failed to connect to OBS WebSocket: {:?}", e);
        }
    };

    info!("Connected to OBS. Switching to scene: {}", scene_name);
    
    // Set the current program scene
    if let Err(e) = client.scenes().set_current_program_scene(scene_name).await {
        error!("Failed to switch OBS scene to {}: {:?}", scene_name, e);
        anyhow::bail!("Failed to switch OBS scene to {}: {:?}", scene_name, e);
    }
    
    info!("OBS scene successfully changed to: {}", scene_name);
    Ok(())
}
