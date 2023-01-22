use std::io::Write;

use crate::block::{Block, Material};
use crate::lua_err;
use crate::nbt::NbtWriter;
use crate::scripting::LuaInit;

pub struct Schematic {
    x_size: u8,
    y_size: u8,
    z_size: u8,
    blocks: Vec<Block>,
}

impl Schematic {
    pub fn new(x_size: u8, y_size: u8, z_size: u8) -> Self {
        let capacity = x_size as usize * y_size as usize * z_size as usize;
        let air = Block::new(Material::air);

        Schematic {
            x_size,
            y_size,
            z_size,
            blocks: vec![air; capacity],
        }
    }

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: Block) -> Result<(), rlua::Error> {
        if x >= self.x_size() {
            lua_err!("x={} >= x_size={}", x, self.x_size())
        }

        if y >= self.y_size() {
            lua_err!("y={} >= y_size={}", x, self.x_size())
        }

        if z >= self.z_size() {
            lua_err!("z={} >= z_size={}", x, self.x_size())
        }

        let index = (y as usize * self.z_size as usize + z as usize) * self.x_size as usize + x as usize;
        self.blocks[index] = block;
        Ok(())
    }

    pub fn x_size(&self) -> u8 {
        self.x_size
    }

    pub fn y_size(&self) -> u8 {
        self.y_size
    }

    pub fn z_size(&self) -> u8 {
        self.z_size
    }

    pub fn serialize<W: Write>(&self, w: &mut W) {
        let mut nbt = NbtWriter::new(w);
        nbt.begin_compound("Schematic");

        nbt.write_short("Width", self.x_size() as i16);
        nbt.write_short("Height", self.y_size() as i16);
        nbt.write_short("Length", self.z_size() as i16);

        nbt.write_string("Materials", "Alpha");

        let mut block_ids = Vec::<u8>::with_capacity(self.blocks.len());
        let mut block_data = Vec::<u8>::with_capacity(self.blocks.len());

        for block in &self.blocks {
            block_ids.push(block.material() as u8);
            block_data.push(block.data());
        }

        nbt.write_byte_array("Blocks", &block_ids);
        nbt.write_byte_array("Data", &block_data);

        nbt.end_compound();
        nbt.finish();
    }
}

impl LuaInit for Schematic {
    fn initialize_lua(ctx: rlua::Context) -> Result<(), rlua::Error> {
        let ctor = ctx.create_function(|_, (x_size, y_size, z_size): (u8, u8, u8)| {
            Ok(Schematic::new(x_size, y_size, z_size))
        })?;

        ctx.globals().set("Schematic", ctor)
    }
}

impl rlua::UserData for Schematic {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut("Set", move |_, schematic, (x, y, z, material): (_, _, _, String)| {
            let block = match Material::try_from(material.as_str()) {
                Ok(m) => Block::new(m),
                Err(_) => lua_err!("material {} not found", material)
            };

            schematic.set_block(x, y, z, block)
        });

        methods.add_method("xSize", |_, schematic, ()| {
            Ok(schematic.x_size())
        });

        methods.add_method("ySize", |_, schematic, ()| {
            Ok(schematic.y_size())
        });

        methods.add_method("zSize", |_, schematic, ()| {
            Ok(schematic.z_size())
        });
    }
}
