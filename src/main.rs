use config::app_config::AppConfiguration;

mod config;

fn main() {
    let config = AppConfiguration::new("/home/me/notes");
    let _ = config.save();
}
