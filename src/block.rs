use std::collections::HashMap;

use strum::IntoEnumIterator;
use strum_macros::{IntoStaticStr, EnumIter};

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
    golden_rail,
    detector_rail,
    sticky_piston,
    web,
    tallgrass,
    deadbush,
    piston,
    piston_head,
    wool,
    piston_extension,
    yellow_flower,
    red_flower,
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
    reeds,
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
    stained_glass,
    trapdoor,
    monster_egg,
    stonebrick,
    brown_mushroom_block,
    red_mushroom_block,
    iron_bars,
    glass_pane,
    melon_block,
    pumpkin_stem,
    melon_stem,
    vine,
    fence_gate,
    brick_stairs,
    stone_brick_stairs,
    mycelium,
    waterlily,
    nether_brick,
    nether_brick_fence,
    nether_brick_stairs,
    nether_wart,
    enchanting_table,
    brewing_stand,
    cauldron,
    end_portal,
    end_portal_frame,
    end_stone,
    dragon_egg,
    redstone_lamp,
    lit_redstone_lamp,
    double_wooden_slab,
    wooden_slab,
    cocoa,
    sandstone_stairs,
    emerald_ore,
    ender_chest,
    tripwire_hook,
    tripwire,
    emerald_block,
    spruce_stairs,
    birch_stairs,
    jungle_stairs,
    command_block,
    beacon,
    cobblestone_wall,
    flower_pot,
    carrots,
    potatoes,
    wooden_button,
    skull,
    anvil,
    trapped_chest,
    light_weighted_pressure_plate,
    heavy_weighted_pressure_plate,
    unpowered_comparator,
    powered_comparator,
    daylight_detector,
    redstone_block,
    quartz_ore,
    hopper,
    quartz_block,
    quartz_stairs,
    activator_rail,
    dropper,
    stained_hardened_clay,
    stained_glass_pane,
    leaves2,
    log2,
    acacia_stairs,
    dark_oak_stairs,
    slime,
    barrier,
    iron_trapdoor,
    prismarine,
    sea_lantern,
    hay_block,
    carpet,
    hardened_clay,
    coal_block,
    packed_ice,
    double_plant,
    standing_banner,
    wall_banner,
    daylight_detector_inverted,
    red_sandstone,
    red_sandstone_stairs,
    double_stone_slab2,
    stone_slab2,
    spruce_fence_gate,
    birch_fence_gate,
    jungle_fence_gate,
    dark_oak_fence_gate,
    acacia_fence_gate,
    spruce_fence,
    birch_fence,
    jungle_fence,
    dark_oak_fence,
    acacia_fence,
    spruce_door,
    birch_door,
    jungle_door,
    acacia_door,
    dark_oak_door,
    end_rod,
    chorus_plant,
    chorus_flower,
    purpur_block,
    purpur_pillar,
    purpur_stairs,
    purpur_double_slab,
    purpur_slab,
    end_bricks,
    beetroots,
    grass_path,
    end_gateway,
    repeating_command_block,
    chain_command_block,
    frosted_ice,
    magma,
    nether_wart_block,
    red_nether_brick,
    bone_block,
    structure_void,
    observer,
    white_shulker_box,
    orange_shulker_box,
    magenta_shulker_box,
    light_blue_shulker_box,
    yellow_shulker_box,
    lime_shulker_box,
    pink_shulker_box,
    gray_shulker_box,
    light_gray_shulker_box,
    cyan_shulker_box,
    purple_shulker_box,
    blue_shulker_box,
    brown_shulker_box,
    green_shulker_box,
    red_shulker_box,
    black_shulker_box,
    structure_block,
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

#[derive(Clone, Copy)]
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

    pub fn material(self) -> Material {
        self.material
    }

    pub fn data(self) -> u8 {
        self.data
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
