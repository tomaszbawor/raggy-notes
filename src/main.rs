use config::app_config::AppConfiguration;
use llama::consts;
use log::info;
use ollama_rs::{
    generation::{chat::request::ChatMessageRequest, completion::request::GenerationRequest},
    Ollama,
};
use rag::{files::get_markdown_files, vectors::VectorDB};

mod config;
mod llama;
mod rag;

#[tokio::main]
async fn main() {
    initialize_logging();
    info!("Application starting ...")

    // Loading config
    //let config = AppConfiguration::load().unwrap();

    // get files:
    //let files_to_embed = get_markdown_files(&config);

    // VectorDb connection
    //let vector_db = VectorDB::new();
    //let a = vector_db.client.list_collections().await;
    /*
        let llama_client = Ollama::new("http://localhost", 11434);

        let models = llama_client.list_local_models().await;
        println!("Local modesls: ");
        dbg!(models);
        let req = GenerationRequest::new(
            consts::AI_MODEL.to_string(),
            "Czy wiesz czemu wilk tak wyje w księżycową noc?".to_string(),
        );

        let response = llama_client.generate(req).await;
        //dbg!(response);
    */
}

fn initialize_logging() {
    env_logger::init();
}
