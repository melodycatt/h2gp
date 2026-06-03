use serde::Deserialize;
use std::{fs, path::Path};

const CONFIG_PATH: &str = "config.toml";

#[derive(Deserialize)]
struct RawConfig {
    port: toml::Value,
    default_addr: u8,
}

pub struct Config {
    pub port: String,
    pub default_addr: u8,
}

// Adding a new field:
//   1. Add it to RawConfig with its serde type
//   2. Add it to Config with its resolved type
//   3. Handle it in `load()` below

pub fn load() -> Config {
    load_from(CONFIG_PATH)
}

pub fn load_from(path: impl AsRef<Path>) -> Config {
    let text = fs::read_to_string(path).expect("failed to read config file");
    let raw: RawConfig = toml::from_str(&text).expect("invalid config");

    let port = match raw.port {
        toml::Value::Integer(1) => "/dev/tty.usbmodem101".to_string(),
        toml::Value::Integer(2) => "/dev/tty.usbmodem1101".to_string(),
        toml::Value::String(s) => s,
        other => panic!("port must be 1, 2, or a string path, got: {other}"),
    };

    Config {
        port,
        default_addr: raw.default_addr,
    }
}
