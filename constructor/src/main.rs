mod block;
mod nbt;
mod nlp;
mod schematic;
mod scripting;

use block::Material;
use rlua::{Lua, StdLib};
use rocket::{State, post, routes};
use schematic::Schematic;
use scripting::LuaInit;

use crate::block::BlockData;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    unsafe { Material::init_string_map(); }

    let port = match std::env::var("PORT") {
        Ok(p) => {
            println!("Using port {}", p);
            p.parse().expect("Invalid PORT environment variable")
        },
        Err(_) => {
            println!("Using default port 8080");
            8080
        },
    };

    let openai_key = std::env::var("OPENAI_API_KEY").expect("Environment variable OPENAI_API_KEY is not set");

    let config = rocket::Config {
        port,
        ..rocket::Config::release_default()
    };

    rocket::custom(config)
        .manage(openai_key)
        .mount("/", routes![generate])
        .launch().await.unwrap();
}

#[derive(rocket::Responder)]
enum ErrorResponse {
    #[response(status = 500)]
    Internal(String),
    #[response(status = 500)]
    Scripting(String),
}

#[post("/generate?<prompt>")]
async fn generate(openai_key: &State<String>, prompt: &str) -> Result<Vec<u8>, ErrorResponse> {
    let script = match nlp::generate(&openai_key, prompt).await {
        Ok(s) => s,
        Err(e) => return Err(ErrorResponse::Internal(e.to_string())),
    };

    match build_schematic(&script) {
        Ok(schem) => {
            let mut data = Vec::with_capacity(256);
            schem.serialize(&mut data);
            Ok(data)
        },
        Err(e) => Err(ErrorResponse::Scripting(e.to_string())),
    }
}

fn build_schematic(lua_src: &str) -> Result<Schematic, rlua::Error> {
    let lua = Lua::new_with(StdLib::MATH);

    lua.context(|ctx| {
        Schematic::initialize_lua(ctx)?;
        BlockData::initialize_lua(ctx)?;

        Ok(ctx.load(&lua_src).eval()?)
    })
}
