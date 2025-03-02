#[derive(Debug)]
pub struct AppConfiguration {
    field: String,
}

pub fn generate_xgd_config() {
    let xdg_directories = xdg::BaseDirectories::with_prefix("raggy_notes");

    let config_file_path = xdg_directories
        .unwrap()
        .place_config_file("config.json")
        .expect("Cannot store config file");
}
