use std::{error::Error, fs::File, io::Write, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfiguration {
    scan_path: String,
}

pub fn generate_xgd_config(scan_path: &str) -> Result<(), Box<dyn Error>> {
    let xdg_directories = xdg::BaseDirectories::with_prefix("raggy_notes");

    let config_file_path = xdg_directories
        .unwrap()
        .place_config_file("config.json")
        .expect("Cannot store config file");

    let mut file = File::create(config_file_path)?;

    let config = AppConfiguration {
        scan_path: scan_path.to_owned(),
    };
    let parsed_string = serde_json::to_string(&config)?;

    file.write_all(parsed_string.into_bytes().deref());

    Ok(())
}
