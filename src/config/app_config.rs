use serde::{Deserialize, Serialize};
use std::{error::Error, fs, fs::File, io::Write, path::PathBuf};
use xdg::BaseDirectories;

const DEFAULT_PREFIX: &str = "raggy_notes";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AppConfiguration {
    pub scan_path: String,
}

impl AppConfiguration {
    pub fn new(scan_path: &str) -> Self {
        Self {
            scan_path: scan_path.to_owned(),
        }
    }

    pub fn load() -> Result<Self, Box<dyn Error>> {
        Self::load_from_xdg(DEFAULT_PREFIX)
    }

    pub fn save(&self) -> Result<PathBuf, Box<dyn Error>> {
        self.save_to_xdg(DEFAULT_PREFIX)
    }

    fn save_to_xdg(&self, prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
        let xdg_dirs = BaseDirectories::with_prefix(prefix)?;
        let config_file_path = xdg_dirs.place_config_file("config.json")?;

        let serialized = serde_json::to_string_pretty(&self)?;
        fs::write(&config_file_path, serialized)?;

        Ok(config_file_path)
    }

    fn load_from_xdg(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let xdg_dirs = BaseDirectories::with_prefix(prefix)?;
        let config_file_path = xdg_dirs
            .find_config_file("config.json")
            .ok_or_else(|| "Config file not found")?;

        let content = fs::read_to_string(config_file_path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_configuration_save_and_load() {
        let dir = tempdir().unwrap();
        let prefix = dir.path().to_str().unwrap();

        let original_config = AppConfiguration::new("/tmp/scan");
        let config_path = original_config.save_to_xdg(prefix).unwrap();

        assert!(config_path.exists(), "Config file was not created");

        let loaded_config = AppConfiguration::load_from_xdg(prefix).unwrap();
        assert_eq!(original_config, loaded_config);
    }

    #[test]
    fn test_configuration_file_missing() {
        let dir = tempdir().unwrap();
        let prefix = dir.path().to_str().unwrap();

        let result = AppConfiguration::load_from_xdg(prefix);
        assert!(result.is_err(), "Should error when config file is missing");
    }
}
