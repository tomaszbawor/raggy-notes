use std::future::IntoFuture;

use config::app_config::AppConfiguration;
use rag::vectors::VectorDB;

mod config;
mod rag;

#[tokio::main]
async fn main() {
    let config = AppConfiguration::new("/home/me/notes");
    let _ = config.save();
    let vector_db = VectorDB::new();
    let call = vector_db.client.list_collections().into_future().await;
    println!("{:?}", call)
}
