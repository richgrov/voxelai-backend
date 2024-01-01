use rocket::async_trait;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>>;
}
