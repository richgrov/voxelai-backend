mod color;
mod nlp;
mod schematic;
mod server;
mod storage;

use storage::CloudflareR2Storage;
use storage::FileSystemStorage;
use storage::ObjectStorage;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    #[cfg(debug_assertions)]
    dotenvy::from_filename(".env.local").unwrap();

    #[cfg(debug_assertions)]
    let default_cfg = rocket::Config::debug_default();

    #[cfg(not(debug_assertions))]
    let default_cfg = rocket::Config::release_default();

    let config = rocket::Config {
        address: parse_env("BIND", "127.0.0.1".parse().unwrap()),
        port: parse_env("PORT", 8080),
        ..default_cfg
    };

    let openai_key = expect_env("OPENAI_API_KEY");

    let storage: Box<dyn ObjectStorage> = if std::env::var("FILE_SYSTEM_STORAGE").is_ok() {
        tracing::info!("Generations will be stored on the file system");
        Box::new(FileSystemStorage)
    } else {
        tracing::info!("Using Cloudflare R2 for object storage");
        let bucket_name = expect_env("R2_BUCKET_NAME");
        let account_id = expect_env("R2_ACCOUNT_ID");
        let public_url = expect_env("R2_PUBLIC_URL");

        Box::new(
            CloudflareR2Storage::new(
                &bucket_name,
                account_id,
                s3::creds::Credentials::default().unwrap(), // loads from ENV
                public_url,
            )
            .unwrap(),
        )
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
