use config::app_config::AppConfiguration;
use rag::{files::get_markdown_files, vectors::VectorDB};

mod config;
mod rag;

#[tokio::main]
async fn main() {
    let config = AppConfiguration::load().unwrap();
    let _ = get_markdown_files(&config);
    let vector_db = VectorDB::new();
    let a = vector_db.client.list_collections().await;
    println!("{:?}", a.unwrap())
}
