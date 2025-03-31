// src/llama/mod.rs
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::embeddings::request::{EmbeddingsInput, GenerateEmbeddingsRequest};
use ollama_rs::Ollama;

use crate::prelude::*;
use std::fs;
use std::path::PathBuf;

pub mod consts {
    pub const AI_MODEL: &str = "deepseek-r1:7b";
    pub const EMBEDDING_MODEL: &str = "nomic-embed-text"; // or another appropriate embedding model
    pub const EMBEDDING_SIZE: usize = 768; // Update this to match your model's embedding size
}

pub struct LlamaService {
    client: Ollama,
}

impl LlamaService {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            client: Ollama::new(host, port),
        }
    }

    pub async fn test_connection(&self) -> Result<Vec<String>> {
        let models = self.client.list_local_models().await?;
        let models_name_list: Vec<String> = models
            .iter()
            .map(|loc_model| loc_model.name.to_owned())
            .collect();

        Ok(models_name_list)
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Vec<Vec<f32>>> {
        let request = GenerateEmbeddingsRequest::new(
            consts::EMBEDDING_MODEL.to_string(),
            EmbeddingsInput::Single(text.to_string()),
        );

        let response = self.client.generate_embeddings(request).await?;

        Ok(response.embeddings)
    }

    pub async fn generate_completion(&self, prompt: &str) -> Result<String> {
        let request = GenerationRequest::new(consts::AI_MODEL.to_string(), prompt.to_string());

        let response = self.client.generate(request).await?;
        Ok(response.response)
    }

    pub async fn extract_text_from_markdown(
        &self,
        file_path: &PathBuf,
    ) -> Result<(String, String)> {
        let content = fs::read_to_string(file_path)?;
        let title = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Ok((title, content))
    }
}
