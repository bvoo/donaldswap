use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapHistoryItem {
    pub game_name: String,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SwapState {
    pub current_game: Option<String>,
    pub current_exe: Option<String>,
    pub last_swap_at: Option<DateTime<Utc>>,
    pub next_swap_at: Option<DateTime<Utc>>,
    pub is_paused: bool,
    pub swap_count: u64,
    #[serde(default)]
    pub time_since_swap_seconds: Option<i64>,
    #[serde(default)]
    pub time_until_swap_seconds: Option<i64>,
    #[serde(default)]
    pub history: Vec<SwapHistoryItem>,
}


#[derive(Clone)]
pub struct StateBroadcaster {
    sender: broadcast::Sender<SwapState>,
}

impl StateBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(16);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SwapState> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, state: SwapState) {
        let _ = self.sender.send(state);
    }
}

impl Default for StateBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AppState {
    pub swap_state: Arc<RwLock<SwapState>>,
    pub broadcaster: StateBroadcaster,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            swap_state: Arc::new(RwLock::new(SwapState::default())),
            broadcaster: StateBroadcaster::new(),
        }
    }

    pub async fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut SwapState),
    {
        let mut state = self.swap_state.write().await;
        f(&mut state);
        
        let now = Utc::now();
        state.time_since_swap_seconds = state.last_swap_at.map(|t| (now - t).num_seconds());
        state.time_until_swap_seconds = state.next_swap_at.map(|t| (t - now).num_seconds());
        
        let state_clone = state.clone();
        drop(state);
        self.broadcaster.broadcast(state_clone);
    }

    pub async fn get_state(&self) -> SwapState {
        let mut state = self.swap_state.read().await.clone();
        let now = Utc::now();
        state.time_since_swap_seconds = state.last_swap_at.map(|t| (now - t).num_seconds());
        state.time_until_swap_seconds = state.next_swap_at.map(|t| (t - now).num_seconds());
        state
    }
}
