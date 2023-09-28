use std::fmt;

use rlua::{StdLib, Lua};
use rlua::Error::RuntimeError;
use serde::Deserialize;
use serde_json::json;

use crate::{schematic::Schematic, color::Color};

const SYSTEM_MESSAGE: &str = r#"You are a program that generates voxel art based on a prompt. You \
generate Lua code which is executed in a sandbox to construct the mesh. \
The Lua API is as follows:

-- Creates a schematic
-- Max size along any axis is 255
function Schematic(xSize: number, ySize: number, zSize: number): Schematic

-- Bounds are (0, the size of the axis - 1)
-- Color is an octal string that formats an 8-bit color as RRRGGGBB. Max value is 773. IMPORTANT:
-- 000 is the default value in schematics and is treated as EMPTY, not BLACK. All other values make
-- a color. For example, '073' is light blue.
function Schematic:Set(x: number, y: number, z: number, color: string)

-- Bounds are (0, the size of the axis - 1)
function Schematic:Fill(x1: number, y1: number, z1: number, x2: number, y2: number, z2: number, color: string)

DO NOT GENERATE AN EXPLANATION- ONLY CODE. Your response will not be shown to the user, only the \
result of the code you produce will be apparent. The code *must* end with a return statement that \
designates which schematic to be generated."#;

#[derive(Debug, Deserialize)]
struct Response {
    choices: Vec<ResponseMessage>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Message {
    role: String,
    content: String,
}

pub async fn build(api_key: &str, prompt: &str) -> Result<Schematic, NlpError> {
    let response = generate_code(api_key, prompt).await
        .map_err(|e| NlpError::OpenAi(e))?;
    let message = &response.choices[0].message.content;
    tracing::info!(preprocessed_code = message);
    let code = match message.strip_prefix("```lua") {
        Some(str) => str[..str.len() - 3].to_owned(),
        None => message.to_owned(),
    };

    let lua = Lua::new_with(StdLib::MATH);
    lua.context(|ctx| {
        let schematic_ctor = ctx.create_function(|_, (x_size, y_size, z_size): (u8, u8, u8)| {
            Ok(Schematic::new(x_size, y_size, z_size))
        })?;
        ctx.globals().set("Schematic", schematic_ctor)?;

        Ok(ctx.load(&code).eval()?)
    }).map_err(|e| NlpError::Lua(e))
}

async fn generate_code(api_key: &str, prompt: &str) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client.post("https://api.openai.com/v1/chat/completions")
        .json(&json!({
            "model": "gpt-4",
            "messages": [
                { "role": "system", "content": SYSTEM_MESSAGE },
                { "role": "user", "content": prompt },
            ],
            "max_tokens": 512,
            "temperature": 0.0,
        }))
        .header("Authorization", format!("Bearer {}", api_key))
        .send().await?
        .json::<Response>().await
}

pub enum NlpError {
    OpenAi(reqwest::Error),
    Lua(rlua::Error),
}

impl fmt::Display for NlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAi(e) => write!(f, "OpenAI request failed: {}", e),
            Self::Lua(e) => write!(f, "error executing Lua: {}", e),
        }
    }
}

impl rlua::UserData for Schematic {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut("Set", |
            _,
            schematic,
            (x, y, z, color_str): (_, _, _, String)
        | {
            let color = match Color::try_from_octal_string(&color_str) {
                Ok(c) => c,
                Err(_) => return Err(RuntimeError(format!("color \"{}\" is invalid", color_str))),
            };

            match schematic.set(x, y, z, color) {
                Some(_) => Ok(()),
                None => Err(RuntimeError(format!("{}, {}, {} is out of bounds", x, y, z))),
            }
        });

        methods.add_method_mut("Fill", |
            _,
            schematic,
            (x1, y1, z1, x2, y2, z2, color_str): (_, _, _, _, _, _, String)
        | {
            let color = match Color::try_from_octal_string(&color_str) {
                Ok(c) => c,
                Err(_) => return Err(RuntimeError(format!("color \"{}\" is invalid", color_str))),
            };
    
            match schematic.fill(x1, y1, z1, x2, y2, z2, color) {
                Some(_) => Ok(()),
                None => Err(RuntimeError(format!(
                    "fill from {}, {}, {} to {}, {}, {} overlaps an out-of-bounds area",
                    x1, y1, z1, x2, y2, z2,
                ))),
            }
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
