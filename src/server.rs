use std::time::Instant;

use rocket::{routes, State, post, async_trait, http::Status};
use crate::nlp;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put(&self, id: &str, data: &[u8]) -> Result<String, Box<dyn std::error::Error>>;
}

struct Server {
    openai_api_key: String,
    object_storage: Box<dyn ObjectStorage>,
}

pub async fn run(
    config: rocket::Config,
    openai_api_key: String,
    object_storage: Box<dyn ObjectStorage>,
) {
    rocket::custom(config)
        .manage(Server {
            openai_api_key,
            object_storage,
        })
        .mount("/", routes![generate])
        .launch().await.unwrap();
}

#[post("/generate?<id>&<prompt>")]
async fn generate(server: &State<Server>, id: &str, prompt: &str) -> Result<String, Status> {
    let start = Instant::now();
    let schem = match nlp::build(&server.openai_api_key, prompt).await {
        Ok(s) => {
            tracing::info!("built after {:?}", start.elapsed());
            s
        },
        Err(e) => {
            tracing::error!("failed to generate build: {}", e);
            return Err(Status::InternalServerError)
        },
    };

    let mut data = Vec::with_capacity(256);
    match schem.serialize(&mut data) {
        Ok(_) => tracing::info!("serialized after {:?}", start.elapsed()),
        Err(e) => {
            tracing::error!("failed to serialize build: {}", e);
            return Err(Status::InternalServerError)
        },
    }

    match server.object_storage.put(id, &data).await {
        Ok(loc) => Ok(loc),
        Err(e) => {
            tracing::error!("failed to store build: {}", e);
            Err(Status::InternalServerError)
        },
    }
}
