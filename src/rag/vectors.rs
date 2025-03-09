use log::info;
use qdrant_client::{
    qdrant::{CreateCollectionBuilder, PointStruct, UpsertPointsBuilder},
    Payload, Qdrant,
};
use uuid::Uuid;

pub struct VectorDB {
    pub client: Qdrant,
}

pub const NOTES_QDRANT_COLLECTION_NAME: &str = "notes";

impl VectorDB {
    pub fn new() -> Self {
        let client = Qdrant::from_url("http://localhost:6334")
            //.api_key(std::env::var("QDRANT_API_KEY"))
            .skip_compatibility_check()
            .build()
            .expect("Unable to connect to qdrant db");
        Self { client }
    }

    pub fn test_connection() {
        todo!("Implement");
    }

    pub async fn initialize_collections(&self) {
        //TODO: Do not run if collection exists
        let cc = self
            .client
            .create_collection(CreateCollectionBuilder::new(NOTES_QDRANT_COLLECTION_NAME).build())
            .await;

        match cc {
            Ok(_) => info!("Successfuly initiualized collection for vectors"),
            Err(err) => info!("Failed to create colleciton: {}", err),
        }
    }

    pub async fn save_vector(&self) {
        //TODO: Create some struct and convert it to pointstruct
        let r = self
            .client
            .upsert_points(
                UpsertPointsBuilder::new(
                    NOTES_QDRANT_COLLECTION_NAME,
                    vec![PointStruct::new(
                        Uuid::new_v4().to_string(),
                        vec![12.1, 122223.1],
                        Payload::new(),
                    )],
                )
                .build(),
            )
            .await;

        match r {
            Ok(_) => info!("Created Vector: {:?}", r.unwrap()),
            Err(err) => info!("Failed to create vector: {}", err),
        }
    }
}
