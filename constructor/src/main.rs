mod block;
mod nbt;
mod nlp;
mod schematic;
mod scripting;

use std::fs::File;

use block::Material;
use rlua::{Lua, StdLib, AnyUserData};
use schematic::Schematic;
use scripting::LuaInit;

use crate::block::BlockData;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    unsafe { Material::init_string_map(); }
    let key = std::env::var("OPENAI_API_KEY").unwrap();
    let prompt = "A cylinder with a radius of 5 and a height of 15 where every layer is stone and every other layer is tnt";

    let script = nlp::generate(&key, prompt).await.unwrap();

    let lua = Lua::new_with(StdLib::MATH);

    lua.context(|ctx| {
        Schematic::initialize_lua(ctx).unwrap();
        BlockData::initialize_lua(ctx).unwrap();

        let data: AnyUserData = ctx.load(&script)
            .eval().unwrap();
        
        let schem = data.borrow::<Schematic>().unwrap();
        schem.serialize(&mut File::create("test.schem").unwrap());
    });
}
