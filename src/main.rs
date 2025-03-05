use config::app_config::AppConfiguration;
use rag::files::get_markdown_files;

mod config;
mod rag;

#[tokio::main]
async fn main() {
    let config = AppConfiguration::load().unwrap();
    let _ = get_markdown_files(&config);
}
