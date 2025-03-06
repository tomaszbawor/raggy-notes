use config::app_config::AppConfiguration;
use rag::{files::get_markdown_files, vectors::VectorDB};

mod config;
mod rag;

#[tokio::main]
async fn main() {
    // Loading config
    let config = AppConfiguration::load().unwrap();

    // get files:
    let files_to_embed = get_markdown_files(&config);

    // VectorDb connection
    let vector_db = VectorDB::new();
    let a = vector_db.client.list_collections().await;
    println!("{:?}", a.unwrap())
}
