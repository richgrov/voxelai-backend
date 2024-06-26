use std::time::Instant;

use crate::nlp;
use crate::storage::ObjectStorage;
use rocket::{http::Status, post, routes, State};

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
        .launch()
        .await
        .unwrap();
}

#[post("/generate?<id>&<prompt>")]
async fn generate(server: &State<Server>, id: &str, prompt: &str) -> Result<String, Status> {
    let start = Instant::now();
    let schem = match nlp::build(&server.openai_api_key, prompt).await {
        Ok(s) => {
            tracing::info!("built after {:?}", start.elapsed());
            s
        }
        Err(e) => {
            tracing::error!("failed to generate build: {}", e);
            return Err(Status::InternalServerError);
        }
    };

    let mut data = Vec::with_capacity(256);
    match schem.serialize(&mut data) {
        Ok(_) => tracing::info!("serialized after {:?}", start.elapsed()),
        Err(e) => {
            tracing::error!("failed to serialize build: {}", e);
            return Err(Status::InternalServerError);
        }
    }

    match server.object_storage.put(id, &data).await {
        Ok(loc) => Ok(loc),
        Err(e) => {
            tracing::error!("failed to store build: {}", e);
            Err(Status::InternalServerError)
        }
    }
}
