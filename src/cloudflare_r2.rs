use rocket::async_trait;

use crate::server::ObjectStorage;

pub struct CloudflareR2Storage {
    bucket: s3::Bucket,
    public_url: String,
}

impl CloudflareR2Storage {
    pub fn new(
        bucket_name: &str,
        account_id: String,
        credentials: s3::creds::Credentials,
        public_url: String,
    ) -> Result<CloudflareR2Storage, s3::error::S3Error> {
        let bucket = s3::Bucket::new(
            bucket_name,
            s3::Region::R2 { account_id },
            credentials,
        )?.with_path_style();

        Ok(CloudflareR2Storage {
            bucket,
            public_url,
        })
    }
}

#[async_trait]
impl ObjectStorage for CloudflareR2Storage {
    async fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let file = format!("{}.schem", id);
        self.bucket.put_object(file.clone(), data).await?;
        Ok(format!("{}/{}", self.public_url, file))
    }
}
