use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub endpoints_file: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgb([u8; 3]),
    Hex(String),
}

#[derive(Deserialize)]
pub struct ThemeConfig {
    pub border: ColorValue,
    pub title: ColorValue,
    pub text: ColorValue,
    pub bg: ColorValue,
    pub emph: ColorValue,
    pub warn: ColorValue,
    pub error: ColorValue,
    pub log: ColorValue,
}
