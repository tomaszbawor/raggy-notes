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
    pub async fn generate_rag_completion(
        &self,
        user_query: &str,
        vector_db: &crate::rag::vectors::VectorDB,
    ) -> Result<String> {
        // Step 1: Generate embedding for the user query
        let embedding = self.get_embedding(user_query).await?;

        // Step 2: Search for relevant notes using the embedding
        let search_results = vector_db.search_similar_notes(embedding, 5).await?;

        // Step 3: Prepare context from relevant notes
        let mut context = String::new();

        if search_results.result.is_empty() {
            context = "No relevant notes found.".to_string();
        } else {
            context.push_str("Here are some relevant notes from your knowledge base:\n\n");

            for (i, point) in search_results.result.iter().enumerate() {
                // Extract title and content from payload
                let title = point.payload.get("title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Untitled".to_string());

                let content = point.payload.get("content")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "No content".to_string());

                // Add a snippet of the note content (to avoid exceeding context window)
                let content_snippet = if content.len() > 500 {
                    format!("{}...", &content[..500])
                } else {
                    content.to_string()
                };

                // Add to context
                context.push_str(&format!("Note {}: {} (relevance: {:.2})\n{}\n\n",
                                          i + 1,
                                          title,
                                          point.score,
                                          content_snippet
                ));
            }
        }

        // Step 4: Build the augmented prompt
        let augmented_prompt = format!(
            "You are a helpful AI assistant with access to the user's notes. \
        Answer the following question using the provided notes when relevant. \
        If the notes don't contain relevant information, just answer based on your knowledge.\n\n\
        {}\n\n\
        User question: {}\n\
        Helpful answer:",
            context,
            user_query
        );

        // Step 5: Generate completion with the augmented prompt
        let response = self.generate_completion(&augmented_prompt).await?;

        Ok(response)
    }
}
