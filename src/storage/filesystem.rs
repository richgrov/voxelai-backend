use rocket::async_trait;

use super::ObjectStorage;

pub struct FileSystemStorage;

#[async_trait]
impl ObjectStorage for FileSystemStorage {
    async fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("{}.glb", id);
        std::fs::write(&path, data)?;
        Ok(path)
    }
}
