use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub exe_name: String,
    pub display_name: String,
    #[serde(default = "default_true")]
    pub send_esc_on_leave: bool,
    #[serde(default = "default_true")]
    pub send_esc_on_enter: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub obs_scene: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            exe_name: String::new(),
            display_name: String::new(),
            send_esc_on_leave: true,
            send_esc_on_enter: true,
            enabled: true,
            obs_scene: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub games: Vec<GameConfig>,
    #[serde(default = "default_min_swap")]
    pub min_swap_minutes: u32,
    #[serde(default = "default_max_swap")]
    pub max_swap_minutes: u32,
    #[serde(default = "default_true")]
    pub auto_swap_enabled: bool,
    #[serde(default)]
    pub hide_next_swap: bool,
    #[serde(default = "default_obs_host")]
    pub obs_ws_host: String,
    #[serde(default = "default_obs_port")]
    pub obs_ws_port: u16,
    #[serde(default)]
    pub obs_ws_password: Option<String>,
}

fn default_min_swap() -> u32 {
    5
}

fn default_max_swap() -> u32 {
    15
}

fn default_obs_host() -> String {
    "localhost".to_string()
}

fn default_obs_port() -> u16 {
    4455
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            games: Vec::new(),
            min_swap_minutes: 5,
            max_swap_minutes: 15,
            auto_swap_enabled: true,
            hide_next_swap: false,
            obs_ws_host: "localhost".to_string(),
            obs_ws_port: 4455,
            obs_ws_password: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            let config = Self::default();
            config.save(path)?;
            return Ok(config);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        toml::from_str(&content)
            .with_context(|| "Failed to parse config file")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| "Failed to create config directory")?;
        }

        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    path: std::path::PathBuf,
}

impl ConfigManager {
    pub fn new(path: std::path::PathBuf) -> Result<Self> {
        let config = AppConfig::load(&path)?;
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            path,
        })
    }

    pub fn config(&self) -> Arc<RwLock<AppConfig>> {
        self.config.clone()
    }

    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    pub async fn update<F>(&self, f: F) -> Result<AppConfig>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
        config.save(&self.path)?;
        Ok(config.clone())
    }
}
