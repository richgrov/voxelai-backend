mod cloudflare_r2;
mod color;
mod nlp;
mod schematic;
mod scripting;
mod server;

use cloudflare_r2::CloudflareR2Storage;
use rocket::async_trait;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    #[cfg(debug_assertions)]
    dotenvy::from_filename(".env.local").unwrap();

    let config = rocket::Config {
        address: parse_env("BIND", "127.0.0.1".parse().unwrap()),
        port: parse_env("PORT", 8080),
        ..rocket::Config::release_default()
    };

    let openai_key = expect_env("OPENAI_API_KEY");

    let storage: Box<dyn server::ObjectStorage> = if std::env::var("FILE_SYSTEM_STORAGE").is_ok() {
        Box::new(FileSystemStorage)
    } else {
        let bucket_name = expect_env("R2_BUCKET_NAME");
        let account_id = expect_env("R2_ACCOUNT_ID");
        let public_url = expect_env("R2_PUBLIC_URL");

        Box::new(CloudflareR2Storage::new(
            &bucket_name,
            account_id,
            s3::creds::Credentials::default().unwrap(), // loads from ENV
            public_url,
        ).unwrap())
    };

    server::run(config, openai_key, storage).await;
}

fn parse_env<T: std::str::FromStr>(var: &str, default: T) -> T {
    match std::env::var(var) {
        Ok(e) => match e.parse() {
            Ok(t) => t,
            Err(_) => panic!("variable {} is invalid", var),
        },
        Err(_) => default,
    }
}

fn expect_env(var: &str) -> String {
    std::env::var(var).expect(&format!("environment variable {} not set", var))
}

struct FileSystemStorage;

#[async_trait]
impl server::ObjectStorage for FileSystemStorage {
    async fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("{}.schem", id);
        std::fs::write(&path, data)?;
        Ok(path)
    }
}
