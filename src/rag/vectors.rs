use log::{error, info};
use qdrant_client::qdrant::{
    CreateCollection, CreateCollectionBuilder, Distance, PointStruct, ScoredPoint, SearchPoints,
    SearchResponse, UpsertPointsBuilder, VectorParams, VectorsConfig, VectorsConfigBuilder,
    WithPayloadSelector, WithVectorsSelector,
};
use qdrant_client::{Payload, Qdrant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::llama::consts::EMBEDDING_SIZE;
use crate::prelude::*;
use std::path::PathBuf;

pub const NOTES_QDRANT_COLLECTION_NAME: &str = "private_notes";

#[derive(Debug, Serialize, Deserialize)]
pub struct NotePayload {
    pub title: String,
    pub content: String,
    pub file_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct NoteVector {
    pub id: String,
    pub embedding: Vec<Vec<f32>>,
    pub payload: NotePayload,
}

impl NoteVector {
    pub fn new(
        title: String,
        content: String,
        file_path: PathBuf,
        embedding: Vec<Vec<f32>>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            embedding,
            payload: NotePayload {
                title,
                content,
                file_path: file_path.to_string_lossy().to_string(),
                created_at: now,
                updated_at: now,
            },
        }
    }

    fn to_point_structs(&self) -> Vec<PointStruct> {
        let mut payload = Payload::new();
        payload.insert("title", self.payload.title.clone());
        payload.insert("content", self.payload.content.clone());
        payload.insert("file_path", self.payload.file_path.clone());
        payload.insert("created_at", self.payload.created_at.to_rfc3339());
        payload.insert("updated_at", self.payload.updated_at.to_rfc3339());

        let mut list: Vec<PointStruct> = vec![];

        for float_vector in self.embedding.clone() {
            list.push(PointStruct::new(
                self.id.clone(),
                float_vector,
                payload.clone(),
            ));
        }
        list
    }
}

pub struct VectorDB {
    pub client: Qdrant,
}

impl VectorDB {
    pub fn new() -> Result<Self> {
        let client = Qdrant::from_url("http://localhost:6334").build()?;

        Ok(Self { client })
    }

    pub async fn test_connection(&self) -> Result<()> {
        let collections_response = self.client.list_collections().await?;
        info!(
            "Connected to Qdrant. Found {} collections",
            collections_response.collections.len()
        );
        Ok(())
    }

    pub async fn initialize_collections(&self) -> Result<()> {
        // Check if collection already exists
        let collections_response = self.client.list_collections().await?;

        if collections_response
            .collections
            .iter()
            .any(|c| c.name == NOTES_QDRANT_COLLECTION_NAME)
        {
            info!(
                "Collection '{}' already exists",
                NOTES_QDRANT_COLLECTION_NAME
            );
            return Ok(());
        }

        // Create collection with the appropriate vector size for our embeddings

        let create_collection =
            CreateCollectionBuilder::new(NOTES_QDRANT_COLLECTION_NAME.to_string())
                .vectors_config(VectorsConfig {
                    config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                        VectorParams {
                            size: EMBEDDING_SIZE as u64,
                            distance: Distance::Cosine.into(),
                            on_disk: Some(false),
                            hnsw_config: None,
                            quantization_config: None,
                            datatype: None,
                            multivector_config: None,
                        },
                    )),
                })
                .build();

        self.client.create_collection(create_collection).await?;
        info!(
            "Successfully initialized collection '{}'",
            NOTES_QDRANT_COLLECTION_NAME
        );
        Ok(())
    }

    pub async fn save_note_vector(&self, note_vector: NoteVector) -> Result<()> {
        let points = note_vector.to_point_structs();

        let up = UpsertPointsBuilder::new(NOTES_QDRANT_COLLECTION_NAME.to_string(), points);

        self.client.upsert_points(up).await?;

        info!("Successfully saved vector with id: {}", note_vector.id);
        Ok(())
    }

    pub async fn search_similar_notes(
        &self,
        query_vector: Vec<f32>,
        limit: u64,
    ) -> Result<SearchResponse> {
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: NOTES_QDRANT_COLLECTION_NAME.to_string(),
                vector: query_vector,
                limit,
                with_payload: Some(WithPayloadSelector {
                    selector_options: Some(
                        qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                    ),
                }),
                with_vectors: Some(WithVectorsSelector {
                    selector_options: Some(
                        qdrant_client::qdrant::with_vectors_selector::SelectorOptions::Enable(true),
                    ),
                }),
                ..Default::default()
            })
            .await?;

        Ok(search_result)
    }
}
