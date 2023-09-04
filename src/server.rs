use rlua::{StdLib, Lua};
use rocket::{routes, State, post, async_trait};

use crate::{nlp, schematic::Schematic, scripting::LuaInit, block::BlockData};

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

#[derive(rocket::Responder)]
enum ErrorResponse {
    #[response(status = 500)]
    Internal(String),
    #[response(status = 504)]
    NotCompletable(String),
}

#[post("/generate?<id>&<prompt>")]
async fn generate(server: &State<Server>, id: &str, prompt: &str) -> Result<String, ErrorResponse> {
    let script = nlp::generate(&server.openai_api_key, prompt).await
        .map_err(|e| ErrorResponse::Internal(e.to_string()))?;

    let schem = build_schematic(&script)
        .map_err(|e| ErrorResponse::NotCompletable(e.to_string()))?;

    let mut data = Vec::with_capacity(256);
    schem.serialize(&mut data);

    server.object_storage.put(id, &data).await
        .map_err(|e| ErrorResponse::Internal(e.to_string()))
}

fn build_schematic(lua_src: &str) -> Result<Schematic, rlua::Error> {
    let lua = Lua::new_with(StdLib::MATH);

    lua.context(|ctx| {
        Schematic::initialize_lua(ctx)?;
        BlockData::initialize_lua(ctx)?;

        Ok(ctx.load(&lua_src).eval()?)
    })
}
