use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct EndpointDef {
    pub path: String,
    pub data: serde_json::Value,
    pub methods: Option<Vec<String>>,
}

pub fn load_endpoints(path: PathBuf) -> Option<Vec<EndpointDef>> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub endpoints_file: Option<PathBuf>,
    pub theme: Option<ThemeConfig>,
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

pub fn load(path: Option<PathBuf>) -> Config {
    path.or_else(|| env::var("ADPTV_CONFIG").map(PathBuf::from).ok())
        .and_then(load_from_file)
        .unwrap_or_default()
}

pub fn load_from_file(path: PathBuf) -> Option<Config> {
    let content = fs::read_to_string(&path).ok()?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => toml::from_str(&content).ok(),
        Some("json") => serde_json::from_str(&content).ok(),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_load_toml() {
        let mut file = tempfile::NamedTempFile::with_suffix(".toml").unwrap();
        write!(
            file,
            r#"
            [server]
            address = "0.0.0.0"
            port = 8080
        "#
        )
        .unwrap();

        let config = load_from_file(file.path().to_path_buf()).unwrap();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.address, "0.0.0.0");
    }

    #[test]
    fn test_load_json() {
        let mut file = tempfile::NamedTempFile::with_suffix(".json").unwrap();
        write!(file, r#"{{"server": {{"port": 9000}}}}"#).unwrap();

        let config = load_from_file(file.path().to_path_buf()).unwrap();
        assert_eq!(config.server.port, 9000);
    }

    #[test]
    fn test_missing_file_returns_none() {
        let result = load_from_file(PathBuf::from("/nonexistent/config.toml"));
        assert!(result.is_none());
    }

    #[test]
    fn test_unknown_extension_returns_none() {
        let mut file = tempfile::NamedTempFile::with_suffix(".yaml").unwrap();
        write!(file, "server:\n  port: 8080").unwrap();

        let result = load_from_file(file.path().to_path_buf());
        assert!(result.is_none());
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.address, "127.0.0.1");
        assert!(config.theme.is_none());
        assert!(config.endpoints_file.is_none());
    }
}
