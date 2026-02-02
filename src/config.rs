use crate::error::{AppError, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub default_link_domain: Option<String>,
    #[serde(default)]
    pub default_text_domain: Option<String>,
    #[serde(default)]
    pub default_file_domain: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Load from config file first (lowest priority)
        if let Some(file_config) = Self::load_from_file()? {
            config = file_config;
        }

        // Override with environment variables (higher priority)
        if let Ok(api_key) = std::env::var("SEE_API_KEY") {
            config.api_key = Some(api_key);
        }
        if let Ok(base_url) = std::env::var("SEE_BASE_URL") {
            config.base_url = Some(base_url);
        }
        if let Ok(timeout) = std::env::var("SEE_TIMEOUT") {
            if let Ok(t) = timeout.parse() {
                config.timeout = Some(t);
            }
        }

        Ok(config)
    }

    fn load_from_file() -> Result<Option<Self>> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(Some(config))
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn config_dir() -> Result<PathBuf> {
        BaseDirs::new()
            .map(|dirs| dirs.config_dir().join("see"))
            .ok_or_else(|| AppError::Config("Could not determine config directory".to_string()))
    }

    pub fn data_dir() -> Result<PathBuf> {
        BaseDirs::new()
            .map(|dirs| dirs.data_dir().join("see"))
            .ok_or_else(|| AppError::Config("Could not determine data directory".to_string()))
    }

    fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    pub fn base_url(&self) -> &str {
        self.base_url.as_deref().unwrap_or("https://s.ee/api/v1")
    }

    pub fn timeout(&self) -> u64 {
        self.timeout.unwrap_or(30)
    }

    pub fn default_link_domain(&self) -> Option<&str> {
        self.default_link_domain.as_deref()
    }

    pub fn default_text_domain(&self) -> Option<&str> {
        self.default_text_domain.as_deref()
    }

    pub fn default_file_domain(&self) -> Option<&str> {
        self.default_file_domain.as_deref()
    }
}
