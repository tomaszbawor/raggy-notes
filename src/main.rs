use config::app_config::AppConfiguration;
use llama::consts::AI_MODEL;
use log::error;
use log::info;
use ollama_rs::generation::embeddings::request::EmbeddingsInput;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;

use prelude::*;
use rag::{files::get_markdown_files, vectors::VectorDB};

mod config;
mod llama;
mod prelude;
mod rag;

#[tokio::main]
async fn main() -> Result<()> {
    initialize_logging();
    info!("Application starting ...");

    // Loading config
    let config = AppConfiguration::load().unwrap();
    info!("Configuration loaded.");

    // get files:
    let files_to_embed = get_markdown_files(&config).unwrap();
    info!("Found {} files to process", files_to_embed.len());

    // VectorDb connection
    let vector_db = VectorDB::new();
    match vector_db.client.list_collections().await {
        Ok(_) => info!("Qdrant connection established "),
        Err(_) => error!("Problem with qdrant connection"),
    }
    vector_db.initialize_collections().await;

    let llama_client = Ollama::new("http://localhost", 11434);

    let models = llama_client.list_local_models().await;
    let models_name_list: Vec<String> = models
        .unwrap()
        .iter()
        .map(|loc_model| loc_model.name.to_owned())
        .collect();

    info!("LLM Models availible: [{}]", models_name_list.join(", "));
    let emb = llama_client
        .generate_embeddings(GenerateEmbeddingsRequest::new(
            AI_MODEL.to_string(),
            EmbeddingsInput::Single("My name is Tomasz".to_string()),
        ))
        .await;

    vector_db.save_vector().await;

    match emb {
        Ok(res) => {
            info!("I have embeddings: {}", res.embeddings.len());
        }
        Err(_err) => {
            error!("Unable to make embedding");
        }
    };
    // Example Ollama query

    /*
    let req = GenerationRequest::new(
        consts::AI_MODEL.to_string(),
        "Whats the color of silence?".to_string(),
    );

    let ollama_response = llama_client.generate(req).await; //dbg!(response);
    let text_response = ollama_response.unwrap().response;
    info!("Ollama response: {}", text_response);
    */
    Ok(())
}

fn initialize_logging() {
    //TODO: Initialize logging to tmp file on release and to console on development
    env_logger::init();
}
