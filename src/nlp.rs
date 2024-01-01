use std::fmt;

use rlua::Error::RuntimeError;
use rlua::{Lua, StdLib};
use serde::Deserialize;
use serde_json::json;

use crate::{color::Color, schematic::Schematic};

const SYSTEM_MESSAGE: &str = r#"You are a program that generates voxel art based on a prompt. You \
generate Lua code which is executed in a sandbox to construct the mesh. \
The Lua API is as follows:

-- Creates a schematic
-- Max size along any axis is 128
function Schematic(xSize: number, ySize: number, zSize: number): Schematic

-- Bounds are (0, the size of the axis - 1)
-- Color is a 6-digit hex string without the #.
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
    let code = generate_code(api_key, prompt).await?;

    let lua = Lua::new_with(StdLib::MATH);
    lua.context(|ctx| {
        let schematic_ctor = ctx.create_function(|_, (x_size, y_size, z_size): (u8, u8, u8)| {
            if x_size > 128 || y_size > 128 || z_size > 128 {
                return Err(RuntimeError(format!(
                    "schematic size {}x{}x{} too big",
                    x_size, y_size, z_size
                )));
            }
            Ok(Schematic::new(x_size, y_size, z_size))
        })?;
        ctx.globals().set("Schematic", schematic_ctor)?;

        Ok(ctx.load(&code).eval()?)
    })
    .map_err(|e| NlpError::Lua(e))
}

async fn generate_code(api_key: &str, prompt: &str) -> Result<String, NlpError> {
    let response_str = invoke_openai(api_key, prompt)
        .await
        .map_err(|e| NlpError::Network(e))?;
    tracing::info!(response = response_str);

    let response_json: Response =
        serde_json::from_str(&response_str).map_err(|e| NlpError::Deserialize(e, response_str))?;

    let message = &response_json.choices[0].message.content;

    Ok(match message.strip_prefix("```lua") {
        Some(str) => str[..str.len() - 3].to_owned(),
        None => message.to_owned(),
    })
}

async fn invoke_openai(api_key: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .post("https://api.openai.com/v1/chat/completions")
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
        .send()
        .await?
        .text()
        .await
}

pub enum NlpError {
    Network(reqwest::Error),
    Deserialize(serde_json::Error, String),
    Lua(rlua::Error),
}

impl fmt::Display for NlpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(e) => write!(f, "network request failed: {}", e),
            Self::Deserialize(e, src) => {
                write!(f, "deserialization failed: {}, original: {}", e, src)
            }
            Self::Lua(e) => write!(f, "error executing Lua: {}", e),
        }
    }
}

impl rlua::UserData for Schematic {
    fn add_methods<'lua, T: rlua::UserDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method_mut(
            "Set",
            |_, schematic, (x, y, z, color_str): (_, _, _, String)| {
                let color = match Color::try_from_hex_string(&color_str) {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(RuntimeError(format!("color \"{}\" is invalid", color_str)))
                    }
                };

                match schematic.set(x, y, z, color) {
                    Some(_) => Ok(()),
                    None => Err(RuntimeError(format!(
                        "{}, {}, {} is out of bounds",
                        x, y, z
                    ))),
                }
            },
        );

        methods.add_method_mut(
            "Fill",
            |_, schematic, (x1, y1, z1, x2, y2, z2, color_str): (_, _, _, _, _, _, String)| {
                let color = match Color::try_from_hex_string(&color_str) {
                    Ok(c) => c,
                    Err(_) => {
                        return Err(RuntimeError(format!("color \"{}\" is invalid", color_str)))
                    }
                };

                match schematic.fill(x1, y1, z1, x2, y2, z2, color) {
                    Some(_) => Ok(()),
                    None => Err(RuntimeError(format!(
                        "fill from {}, {}, {} to {}, {}, {} overlaps an out-of-bounds area",
                        x1, y1, z1, x2, y2, z2,
                    ))),
                }
            },
        );

        methods.add_method("xSize", |_, schematic, ()| Ok(schematic.x_size()));

        methods.add_method("ySize", |_, schematic, ()| Ok(schematic.y_size()));

        methods.add_method("zSize", |_, schematic, ()| Ok(schematic.z_size()));
    }
}
