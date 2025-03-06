use qdrant_client::Qdrant;

pub struct VectorDB {
    pub client: Qdrant,
}

impl VectorDB {
    pub fn new() -> Self {
        let client = Qdrant::from_url("http://localhost:6334")
            //.api_key(std::env::var("QDRANT_API_KEY"))
            .skip_compatibility_check()
            .build()
            .expect("Unable to connect to qdrant db");
        Self { client }
    }
}
