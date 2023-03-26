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
        let index = self.get_index(x, y, z)?;
        self.blocks[index] = block;
        Ok(())
    }

    #[cfg(test)]
    fn get_block(&self, x: u8, y: u8, z: u8) -> Result<Block, rlua::Error> {
        let index = self.get_index(x, y, z)?;
        Ok(self.blocks[index])
    }

    pub fn fill(
        &mut self, x1: u8, y1: u8, z1: u8, x2: u8, y2: u8, z2: u8, block: Block
    ) -> Result<(), rlua::Error> {
        for x in x1..=x2 {
            for y in y1..=y2 {
                for z in z1..=z2 {
                    self.set_block(x, y, z, block)?;
                }
            }
        }

        Ok(())
    }

    fn get_index(&self, x: u8, y: u8, z: u8) -> Result<usize, rlua::Error> {
        if x >= self.x_size() {
            lua_err!("invalid x {}", x)
        }

        if y >= self.y_size() {
            lua_err!("invalid y {}", y)
        }

        if z >= self.z_size() {
            lua_err!("invalid z {}", z)
        }

        Ok((y as usize * self.z_size as usize + z as usize) * self.x_size as usize + x as usize)
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
        methods.add_method_mut("Set", |
            _,
            schematic,
            (x, y, z, material, data): (_, _, _, String, Option<u8>)
        | {
            let block = match Material::try_from(material.as_str()) {
                Ok(m) => match data {
                    Some(d) => Block::new_with_data(m, d),
                    None => Block::new(m),
                },
                Err(_) => lua_err!("material {} not found", material)
            };

            schematic.set_block(x, y, z, block)
        });

        methods.add_method_mut("Fill", |
            _,
            schematic,
            (x1, y1, z1, x2, y2, z2, material, data): (_, _, _, _, _, _, String, Option<u8>)
        | {
                let block = match Material::try_from(material.as_str()) {
                    Ok(m) => match data {
                        Some(d) => Block::new_with_data(m, d),
                        None => Block::new(m),
                    },
                    Err(_) => lua_err!("material {} not found", material)
                };
    
                schematic.fill(x1, y1, z1, x2, y2, z2, block)
            }
        );

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

#[cfg(test)]
mod tests {
    use crate::block::{Block, Material};

    use super::Schematic;

    #[test]
    fn test_coordinates() {
        let mut schem = Schematic::new(10, 10, 10);
        schem.set_block(0, 0, 0, Block::new(Material::cactus)).unwrap();
        schem.set_block(9, 9, 9, Block::new(Material::bedrock)).unwrap();

        assert_eq!(schem.get_block(0, 0, 0).unwrap(), Block::new(Material::cactus)); 
        assert_eq!(schem.get_block(9, 9, 9).unwrap(), Block::new(Material::bedrock)); 
    }

    #[test]
    fn test_fill() {
        let mut schem = Schematic::new(10, 10, 10);
        schem.fill(1, 2, 3, 7, 8, 9, Block::new(Material::bedrock)).unwrap();

        for x in 0..schem.x_size() {
            for y in 0..schem.y_size() {
                for z in 0..schem.z_size() {
                    let expected = if (1..=7).contains(&x) &&
                                      (2..=8).contains(&y) &&
                                      (3..=9).contains(&z) {
                        Block::new(Material::bedrock)
                    } else {
                        Block::new(Material::air)
                    };

                    assert_eq!(schem.get_block(x, y, z).unwrap(), expected);
                }
            }
        }
    }
}
