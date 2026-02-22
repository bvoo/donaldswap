use crate::config::{AppConfig, GameConfig};
use crate::input::send_esc;
use crate::state::AppState;
use crate::windows::{find_window_by_exe, focus_window};
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub struct Swapper {
    config: Arc<RwLock<AppConfig>>,
    app_state: Arc<AppState>,
}

impl Swapper {
    pub fn new(config: Arc<RwLock<AppConfig>>, app_state: Arc<AppState>) -> Self {
        Self { config, app_state }
    }

    pub async fn run(&self) {
        loop {
            let state = self.app_state.get_state().await;

            if state.is_paused {
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let config = self.config.read().await;

            if !config.auto_swap_enabled {
                drop(config);
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let delay_seconds = self.calculate_delay(&config);
            drop(config);

            let next_swap = chrono::Utc::now() + chrono::Duration::seconds(delay_seconds as i64);
            self.app_state
                .update_state(|s| s.next_swap_at = Some(next_swap))
                .await;

            sleep(Duration::from_secs(delay_seconds)).await;

            if self.app_state.get_state().await.is_paused {
                continue;
            }

            if let Err(e) = self.do_swap().await {
                error!("Swap failed: {:?}", e);
            }
        }
    }

    fn calculate_delay(&self, config: &AppConfig) -> u64 {
        use rand::Rng;
        let min = config.min_swap_minutes * 60;
        let max = config.max_swap_minutes * 60;
        if min >= max {
            return max as u64;
        }
        rand::thread_rng().gen_range(min..=max) as u64
    }

    async fn do_swap(&self) -> Result<()> {
        let config = self.config.read().await;
        let enabled_games: Vec<&GameConfig> = config.games.iter().filter(|g| g.enabled).collect();

        if enabled_games.is_empty() {
            warn!("No enabled games configured");
            return Ok(());
        }

        let current_exe = self.app_state.get_state().await.current_exe.clone();

        let next_game = self.find_next_game(&enabled_games, current_exe.as_deref())?;

        info!("Swapping to: {}", next_game.display_name);

        let current_config = current_exe
            .and_then(|exe| config.games.iter().find(|g| g.exe_name == exe));

        if let Some(current) = current_config {
            if current.send_esc_on_leave {
                info!("Sending ESC to leave: {}", current.display_name);
                if let Err(e) = send_esc() {
                    warn!("Failed to send ESC: {:?}", e);
                }
            }
        }

        sleep(Duration::from_millis(100)).await;

        if let Some(hwnd) = find_window_by_exe(&next_game.exe_name) {
            info!("Found window handle: {}", hwnd);
            
            if let Err(e) = focus_window(hwnd) {
                warn!("Failed to focus window: {:?}", e);
                anyhow::bail!("Failed to focus window: {:?}", e);
            }

            sleep(Duration::from_millis(100)).await;

            if next_game.send_esc_on_enter {
                info!("Sending ESC to enter: {}", next_game.display_name);
                if let Err(e) = send_esc() {
                    warn!("Failed to send ESC: {:?}", e);
                }
            }

            let now = chrono::Utc::now();
            let state = self.app_state.get_state().await;
            
            let mut new_history = state.history.clone();
            if let (Some(prev_game), Some(prev_time)) = (state.current_game, state.last_swap_at) {
                let duration = (now - prev_time).num_seconds().max(0) as u64;
                new_history.insert(0, crate::state::SwapHistoryItem {
                    game_name: prev_game,
                    duration_seconds: duration,
                });
                
                // Keep only last 10 entries to prevent infinite growth
                if new_history.len() > 10 {
                    new_history.pop();
                }
            }
            
            let swap_count = state.swap_count + 1;

            self.app_state
                .update_state(|s| {
                    s.current_game = Some(next_game.display_name.clone());
                    s.current_exe = Some(next_game.exe_name.clone());
                    s.last_swap_at = Some(now);
                    s.swap_count = swap_count;
                    s.history = new_history;
                })
                .await;

            info!("Swap complete");
        } else {
            warn!("Game window not found: {}", next_game.exe_name);
        }

        Ok(())
    }

    fn find_next_game<'a>(
        &self,
        games: &[&'a GameConfig],
        current_exe: Option<&str>,
    ) -> Result<&'a GameConfig> {
        let available: Vec<&&GameConfig> = games
            .iter()
            .filter(|g| {
                if let Some(current) = current_exe {
                    g.exe_name != current
                } else {
                    true
                }
            })
            .filter(|g| find_window_by_exe(&g.exe_name).is_some())
            .collect();

        if available.is_empty() {
            if games.is_empty() {
                anyhow::bail!("No games configured");
            }
            return Ok(games[0]);
        }

        use rand::Rng;
        let idx = rand::thread_rng().gen_range(0..available.len());
        Ok(available[idx])
    }

    pub async fn force_swap(&self) -> Result<()> {
        self.do_swap().await
    }
}
