use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

// Match these to your actual bundle identifier
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "webpaige";
const APPLICATION: &str = "h2gp";

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Deserialize)]
struct RawConfig {
    port: toml::Value,
    pub purge_time: u32,
    pub purge_interval: u32,
    pub default_addr: u8,
    pub receivers: [u8; 2],
    pub send_wait: u8,
}

#[derive(Serialize, Clone)]
pub struct Config {
    pub port: String,
    pub purge_time: u32,
    pub purge_interval: u32,
    pub default_addr: u8,
    pub receivers: [u8; 2],
    pub send_wait: u8,
}

/// macOS:   ~/Library/Application Support/com.yourcompany.yourapp/config.toml
/// Linux:   ~/.config/yourapp/config.toml
/// Windows: %APPDATA%\yourcompany\yourapp\config\config.toml
pub fn config_path() -> PathBuf {
    let dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .expect("could not determine a config directory for this platform");
    let dir = dirs.config_dir();
    fs::create_dir_all(dir).expect("failed to create config directory");
    dir.join("config.toml")
}

/// First run: write the bundled default out, then load it.
pub fn load() -> Config {
    let path = config_path();
    if !path.exists() {
        fs::write(&path, DEFAULT_CONFIG).expect("failed to write default config");
    }
    load_from(path)
}

pub fn load_from(path: impl AsRef<Path>) -> Config {
    let text = fs::read_to_string(path).expect("failed to read config file");
    parse(&text).unwrap_or_else(|e| panic!("{e}"))
}

/// Non-panicking version — use this from inside a running GUI.
pub fn try_load_from(path: impl AsRef<Path>) -> Result<Config, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    parse(&text)
}

fn parse(text: &str) -> Result<Config, String> {
    let raw: RawConfig = toml::from_str(text).map_err(|e| format!("invalid config: {e}"))?;
    let port = match raw.port {
        toml::Value::Integer(1) => "/dev/tty.usbmodem101".to_string(),
        toml::Value::Integer(2) => "/dev/tty.usbmodem1101".to_string(),
        toml::Value::String(s) => s,
        other => return Err(format!("port must be 1, 2, or a string path, got: {other}")),
    };
    Ok(Config {
        port,
        default_addr: raw.default_addr,
        purge_time: raw.purge_time,
        purge_interval: raw.purge_interval,
        receivers: raw.receivers,
        send_wait: raw.send_wait,
    })
}

/// Writes the resolved config back to disk. Note: this always writes `port` as a
/// plain string — the 1/2 shorthand is only meant for hand-editing the TOML file.
pub fn save(cfg: &Config) -> Result<(), String> {
    save_to(cfg, config_path())
}

pub fn save_to(cfg: &Config, path: impl AsRef<Path>) -> Result<(), String> {
    let text = toml::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}
