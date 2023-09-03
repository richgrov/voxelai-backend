mod block;
mod nbt;
mod nlp;
mod schematic;
mod scripting;
mod server;

use block::Material;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    unsafe { Material::init_string_map(); }

    let config = rocket::Config {
        address: parse_env("BIND", "127.0.0.1".parse().unwrap()),
        port: parse_env("PORT", 8080),
        ..rocket::Config::release_default()
    };

    let openai_key = std::env::var("OPENAI_API_KEY")
        .expect("Environment variable OPENAI_API_KEY is not set");

    let storage = FileSystemStorage;

    server::run(config, openai_key, Box::new(storage)).await;
}

fn parse_env<T: std::str::FromStr>(var: &str, default: T) -> T {
    match std::env::var(var) {
        Ok(e) => match e.parse() {
            Ok(t) => t,
            Err(_) => panic!("invalid {} environment variable", var),
        },
        Err(_) => default,
    }
}

struct FileSystemStorage;

impl server::ObjectStorage for FileSystemStorage {
    fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("{}.schem", id);
        std::fs::write(&path, data)?;
        Ok(path)
    }
}
