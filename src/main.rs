mod block;
mod nbt;
mod schematic;
mod scripting;

use std::fs::File;

use block::Material;
use rlua::{Lua, StdLib, AnyUserData};
use schematic::Schematic;
use scripting::LuaInit;

fn main() {
    unsafe { Material::init_string_map(); }

    let lua = Lua::new_with(StdLib::MATH);

    lua.context(|ctx| {
        Schematic::initialize_lua(ctx).unwrap();

        let data: AnyUserData = ctx.load(&std::fs::read_to_string("script.lua").unwrap())
            .eval().unwrap();
        
        let schem = data.borrow::<Schematic>().unwrap();
        schem.serialize(&mut File::create("test.schem").unwrap());
    });
}
