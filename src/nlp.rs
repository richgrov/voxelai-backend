use serde::{Serialize, Deserialize};
use serde_json::json;

const SYSTEM_MESSAGE: &str = r#"You are a program that generates Minecraft builds based on a \
prompt. You generate Lua code which is executed in a sandbox to construct a Minecraft schematic. \
The Lua API is as follows:

-- Creates a schematic
-- Max size along any axis is 255
function Schematic(xSize: number, ySize: number, zSize: number): Schematic

-- Bounds are [0, the size of the axis)
function Schematic:Set(x: number, y: number, z: number, block: string)

-- Bounds are [0, the size of the axis)
function Schematic:Fill(x1: number, y1: number, z1: number, x2: number, y2: number, z2: number, block: string)

Available blocks:
air
stone
grass
dirt
cobblestone
planks
sapling
bedrock
flowing_water
water
flowing_lava
lava
sand
gravel
gold_ore
iron_ore
coal_ore
log
leaves
sponge
glass
lapis_ore
lapis_block
dispenser
sandstone
noteblock
bed
powered_rail
detector_rail
sticky_piston
web
tallgrass
deadbush
piston
piston_head
wool
piston_extension
dandelion
rose
brown_mushroom
red_mushroom
gold_block
iron_block
double_stone_slab
stone_slab
brick_block
tnt
bookshelf
mossy_cobblestone
obsidian
torch
fire
mob_spawner
oak_stairs
chest
redstone_wire
diamond_ore
diamond_block
crafting_table
wheat
farmland
furnace
lit_furnace
standing_sign
wooden_door
ladder
rail
stone_stairs
wall_sign
lever
stone_pressure_plate
iron_door
wooden_pressure_plate
redstone_ore
lit_redstone_ore
unlit_redstone_torch
redstone_torch
stone_button
snow_layer
ice
snow
cactus
clay
sugarcane
jukebox
fence
pumpkin
netherrack
soul_sand
glowstone
portal
lit_pumpkin
cake
unpowered_repeater
powered_repeater
trapdoor

You do not need to generate explanations as your response will not be shown to the user, only the \
result of the code you produce will be apparent. The code *must* end with a return statement that \
designates which schematic to be generated."#;

#[derive(Debug, Deserialize)]
struct Response {
    choices: Vec<ResponseMessage>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    index: u32,
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
}

pub async fn generate(key: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.post("https://api.openai.com/v1/chat/completions")
        .json(&json!({
            "model": "gpt-4",
            "messages": [
                { "role": "system", "content": SYSTEM_MESSAGE },
                { "role": "user", "content": prompt },
            ],
            "max_tokens": 512,
            "temperature": 0.0,
        }))
        .header("Authorization", format!("Bearer {}", key))
        .send().await?
        .json::<Response>().await?;

    println!("{:?}", response);

    let response = &response.choices[0].message.content;
    let code = match response.strip_prefix("```lua") {
        Some(str) => str[..str.len() - 3].to_owned(),
        None => response.to_owned(),
    };

    Ok(code)
}
