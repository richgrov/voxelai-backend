use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::{IntoStaticStr, EnumIter};

use crate::scripting::LuaInit;

#[allow(dead_code, non_camel_case_types)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, IntoStaticStr, EnumIter)]
pub enum Material {
    air,
    stone,
    grass,
    dirt,
    cobblestone,
    planks,
    sapling,
    bedrock,
    flowing_water,
    water,
    flowing_lava,
    lava,
    sand,
    gravel,
    gold_ore,
    iron_ore,
    coal_ore,
    log,
    leaves,
    sponge,
    glass,
    lapis_ore,
    lapis_block,
    dispenser,
    sandstone,
    noteblock,
    bed,
    powered_rail,
    detector_rail,
    sticky_piston,
    web,
    tallgrass,
    deadbush,
    piston,
    piston_head,
    wool,
    piston_extension,
    dandelion,
    rose,
    brown_mushroom,
    red_mushroom,
    gold_block,
    iron_block,
    double_stone_slab,
    stone_slab,
    brick_block,
    tnt,
    bookshelf,
    mossy_cobblestone,
    obsidian,
    torch,
    fire,
    mob_spawner,
    oak_stairs,
    chest,
    redstone_wire,
    diamond_ore,
    diamond_block,
    crafting_table,
    wheat,
    farmland,
    furnace,
    lit_furnace,
    standing_sign,
    wooden_door,
    ladder,
    rail,
    stone_stairs,
    wall_sign,
    lever,
    stone_pressure_plate,
    iron_door,
    wooden_pressure_plate,
    redstone_ore,
    lit_redstone_ore,
    unlit_redstone_torch,
    redstone_torch,
    stone_button,
    snow_layer,
    ice,
    snow,
    cactus,
    clay,
    sugarcane,
    jukebox,
    fence,
    pumpkin,
    netherrack,
    soul_sand,
    glowstone,
    portal,
    lit_pumpkin,
    cake,
    unpowered_repeater,
    powered_repeater,
    locked_chest,
    trapdoor,
}

static mut STR_TO_MATERIAL: Option<HashMap<&'static str, Material>> = None;

impl Material {
    pub unsafe fn init_string_map() {
        assert!(STR_TO_MATERIAL.is_none(), "str_to_material already initialized");

        let mut map = HashMap::new();
        for mat in Material::iter() {
            map.insert(mat.into(), mat);
        }

        STR_TO_MATERIAL = Some(map);
    }
}

impl TryFrom<&str> for Material {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        unsafe {
            match &STR_TO_MATERIAL {
                Some(map) => match map.get(value) {
                    Some(mat) => Ok(*mat),
                    None => Err(format!("Material {} not found", value).into()),
                },
                None => panic!("str_to_material not initialized"),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    material: Material,
    data: u8,
}

impl Block {
    pub fn new(material: Material) -> Block {
        Block {
            material,
            data: 0,
        }
    }

    pub fn new_with_data(material: Material, data: u8) -> Block {
        Block {
            material,
            data,
        }
    }

    pub fn material(self) -> Material {
        self.material
    }

    pub fn data(self) -> u8 {
        self.data
    }
}

pub struct BlockData;

impl LuaInit for BlockData {
    fn initialize_lua(ctx: rlua::Context) -> Result<(), rlua::Error> {
        let table = ctx.create_table()?;
        
        let color = ctx.create_table()?;
        color.set("White", 0)?;
        color.set("Orange", 1)?;
        color.set("Magenta", 2)?;
        color.set("LightBlue", 3)?;
        color.set("Yellow", 4)?;
        color.set("Lime", 5)?;
        color.set("Pink", 6)?;
        color.set("Gray", 7)?;
        color.set("LightGray", 8)?;
        color.set("Cyan", 9)?;
        color.set("Purple", 10)?;
        color.set("Blue", 11)?;
        color.set("Brown", 12)?;
        color.set("Green", 13)?;
        color.set("Red", 14)?;
        color.set("Black", 15)?;
        table.set("Color", color)?;

        ctx.globals().set("BlockData", table)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use strum::IntoEnumIterator;

    use crate::block::Block;

    use super::Material;

    #[test]
    fn test_block_size() {
        assert_eq!(size_of::<Block>(), 2);
    }

    #[test]
    fn test_str_to_material() {
        unsafe { Material::init_string_map(); }

        for mat in Material::iter() {
            let mat_str: &'static str = mat.into();
            assert_eq!(mat, Material::try_from(mat_str).unwrap());
        }
    }
}
