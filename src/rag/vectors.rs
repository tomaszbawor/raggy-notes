use qdrant_client::Qdrant;

pub struct VectorDB {
    pub client: Qdrant,
}

impl VectorDB {
    pub fn new() -> Self {
        let client = Qdrant::from_url("https://localhost:6333")
            .api_key(std::env::var("QDRANT_API_KEY"))
            .skip_compatibility_check()
            .build()
            .unwrap();
        Self { client }
    }
}
