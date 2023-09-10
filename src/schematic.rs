use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Write;

use gltf::json::accessor::GenericComponentType;
use gltf::json::validation::Checked;

use crate::color::Color;
use crate::lua_err;
use crate::scripting::LuaInit;

#[derive(Clone)]
pub struct Schematic {
    x_size: u8,
    y_size: u8,
    z_size: u8,
    blocks: Vec<Color>,
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    color: [f32; 3],
}

impl Schematic {
    const ABSENT: Color = Color(0);

    pub fn new(x_size: u8, y_size: u8, z_size: u8) -> Self {
        let capacity = x_size as usize * y_size as usize * z_size as usize;

        Schematic {
            x_size,
            y_size,
            z_size,
            blocks: vec![Color(0); capacity],
        }
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, color: Color) -> Result<(), rlua::Error> {
        let index = self.get_index(x, y, z)?;
        self.blocks[index] = color;
        Ok(())
    }

    fn get(&self, x: u8, y: u8, z: u8) -> Result<Color, rlua::Error> {
        let index = self.get_index(x, y, z)?;
        Ok(self.blocks[index])
    }

    pub fn fill(
        &mut self, x1: u8, y1: u8, z1: u8, x2: u8, y2: u8, z2: u8, block: Color
    ) -> Result<(), rlua::Error> {
        for x in x1..=x2 {
            for y in y1..=y2 {
                for z in z1..=z2 {
                    self.set(x, y, z, block)?;
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

    pub fn serialize<W: Write>(&self, w: &mut W) -> Result<(), Box<dyn std::error::Error>> {
        let mut vertices = Vec::new();
        vertices.extend_from_slice(&[
            Vertex {
                pos: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                pos: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                pos: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ]);

        let indices = vec![0u32, 1, 2];

        let vertices_bytes = bytemuck::cast_slice(&vertices);
        let indices_bytes = bytemuck::cast_slice(&indices);
        let buffer = [vertices_bytes, indices_bytes].concat();

        let mut min = [f32::MAX, f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN, f32::MIN];

        for vertex in &vertices {
            for i in 0..3 {
                min[i] = f32::min(min[i], vertex.pos[i]);
                max[i] = f32::max(max[i], vertex.pos[i]);
            }
        }

        let json = gltf::json::serialize::to_string(&gltf::json::Root {
            accessors: vec![
                gltf::json::Accessor {
                    buffer_view: Some(gltf::json::Index::new(0)),
                    byte_offset: Some(0),
                    count: vertices.len() as u32,
                    component_type: Checked::Valid(GenericComponentType(gltf::json::accessor::ComponentType::F32)),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Checked::Valid(gltf::json::accessor::Type::Vec3),
                    min: Some(Vec::from(min).into()),
                    max: Some(Vec::from(max).into()),
                    name: None,
                    normalized: false,
                    sparse: None,
                },
                gltf::json::Accessor {
                    buffer_view: Some(gltf::json::Index::new(0)),
                    byte_offset: Some((3 * std::mem::size_of::<f32>()) as u32),
                    count: vertices.len() as u32,
                    component_type: Checked::Valid(GenericComponentType(gltf::json::accessor::ComponentType::F32)),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Checked::Valid(gltf::json::accessor::Type::Vec3),
                    min: None,
                    max: None,
                    name: None,
                    normalized: false,
                    sparse: None,
                },
                gltf::json::Accessor {
                    buffer_view: Some(gltf::json::Index::new(1)),
                    byte_offset: Some(0),
                    count: indices.len() as u32,
                    component_type: Checked::Valid(GenericComponentType(gltf::json::accessor::ComponentType::U32)),
                    extensions: Default::default(),
                    extras: Default::default(),
                    type_: Checked::Valid(gltf::json::accessor::Type::Scalar),
                    min: None,
                    max: None,
                    name: None,
                    normalized: false,
                    sparse: None,
                },
            ],
            buffers: vec![gltf::json::Buffer {
                byte_length: buffer.len() as u32,
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                uri: None,
            }],
            buffer_views: vec![
                gltf::json::buffer::View {
                    buffer: gltf::json::Index::new(0),
                    byte_length: vertices_bytes.len() as u32,
                    byte_offset: None,
                    byte_stride: Some(std::mem::size_of::<Vertex>() as u32),
                    extensions: Default::default(),
                    extras: Default::default(),
                    name: None,
                    target: Some(Checked::Valid(gltf::json::buffer::Target::ArrayBuffer)),
                },
                gltf::json::buffer::View {
                    buffer: gltf::json::Index::new(0),
                    byte_length: indices_bytes.len() as u32,
                    byte_offset: Some(vertices_bytes.len() as u32),
                    byte_stride: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                    name: None,
                    target: Some(Checked::Valid(gltf::json::buffer::Target::ElementArrayBuffer)),
                },
            ],
            meshes: vec![gltf::json::Mesh {
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                primitives: vec![gltf::json::mesh::Primitive {
                    attributes: {
                        let mut map = BTreeMap::new();
                        map.insert(Checked::Valid(gltf::json::mesh::Semantic::Positions), gltf::json::Index::new(0));
                        map.insert(Checked::Valid(gltf::json::mesh::Semantic::Colors(0)), gltf::json::Index::new(1));
                        map
                    },
                    extensions: Default::default(),
                    extras: Default::default(),
                    indices: Some(gltf::json::Index::new(2)),
                    material: None,
                    mode: Checked::Valid(gltf::json::mesh::Mode::Triangles),
                    targets: None,
                }],
                weights: None,
            }],
            nodes: vec![gltf::json::Node {
                camera: None,
                children: None,
                extras: Default::default(),
                extensions: Default::default(),
                matrix: None,
                mesh: Some(gltf::json::Index::new(0)),
                name: None,
                rotation: None,
                scale: None,
                translation: None,
                skin: None,
                weights: None,
            }],
            scenes: vec![gltf::json::Scene {
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                nodes: vec![gltf::json::Index::new(0)],
            }],
            ..Default::default()
        })?;

        let glb = gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: json.len() as u32 + buffer.len() as u32,
            },
            bin: Some(Cow::Owned(buffer)),
            json: Cow::Owned(json.into_bytes()),
        };

        glb.to_writer(w)?;
        Ok(())
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
            (x, y, z, color_str): (_, _, _, String)
        | {
            let color = match Color::try_from_octal_string(&color_str) {
                Ok(c) => c,
                Err(_) => lua_err!("color \"{}\" is invalid", color_str),
            };

            schematic.set(x, y, z, color)
        });

        methods.add_method_mut("Fill", |
            _,
            schematic,
            (x1, y1, z1, x2, y2, z2, color_str): (_, _, _, _, _, _, String)
        | {
            let color = match Color::try_from_octal_string(&color_str) {
                Ok(c) => c,
                Err(_) => lua_err!("color \"{}\" is invalid", color_str),
            };
    
            schematic.fill(x1, y1, z1, x2, y2, z2, color)
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

#[cfg(test)]
mod tests {
    use crate::color::Color;

    use super::Schematic;

    #[test]
    fn test_coordinates() {
        let mut schem = Schematic::new(10, 10, 10);
        schem.set(0, 0, 0, Color(0o070)).unwrap();
        schem.set(9, 9, 9, Color(1)).unwrap();

        assert_eq!(schem.get(0, 0, 0).unwrap(), Color(0o070)); 
        assert_eq!(schem.get(9, 9, 9).unwrap(), Color(1)); 
    }

    #[test]
    fn test_fill() {
        let mut schem = Schematic::new(10, 10, 10);
        schem.fill(1, 2, 3, 7, 8, 9, Color(1)).unwrap();

        for x in 0..schem.x_size() {
            for y in 0..schem.y_size() {
                for z in 0..schem.z_size() {
                    let expected = if (1..=7).contains(&x) &&
                                      (2..=8).contains(&y) &&
                                      (3..=9).contains(&z) {
                        Color(1)
                    } else {
                        Color(0)
                    };

                    assert_eq!(schem.get(x, y, z).unwrap(), expected);
                }
            }
        }
    }
}
