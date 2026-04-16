use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keystroke_capture: KeystrokeConfig,
    pub pull_trigger: PullTriggerConfig,
    pub rarity_thresholds: RarityThresholds,
    pub llm: LlmConfig,
    pub ascii: AsciiConfig,
    pub klipy_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystrokeConfig {
    pub enabled: bool,
    pub deaf_mode: bool,
    pub deaf_mode_shortcut: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullTriggerConfig {
    pub mode: String,
    pub scheduled_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarityThresholds {
    pub uncommon: u64,
    pub rare: u64,
    pub epic: u64,
    pub legendary: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub mode: String,
    pub cli_model: String,
    pub cli_effort: String,
    pub api_key: Option<String>,
    pub api_temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsciiConfig {
    pub columns: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keystroke_capture: KeystrokeConfig {
                enabled: true,
                deaf_mode: false,
                deaf_mode_shortcut: "Cmd+Shift+D".to_string(),
            },
            pull_trigger: PullTriggerConfig {
                mode: "manual".to_string(),
                scheduled_time: "18:00".to_string(),
            },
            rarity_thresholds: RarityThresholds {
                uncommon: 10_000,
                rare: 30_000,
                epic: 60_000,
                legendary: 100_000,
            },
            llm: LlmConfig {
                mode: "cli".to_string(),
                cli_model: "haiku".to_string(),
                cli_effort: "low".to_string(),
                api_key: None,
                api_temperature: 0.99,
            },
            ascii: AsciiConfig {
                columns: 100,
            },
            klipy_api_key: None,
        }
    }
}

pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("no home directory")
        .join(".dagashi")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
