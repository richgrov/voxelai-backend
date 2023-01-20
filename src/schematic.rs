use std::io::Write;

use crate::block::{Block, Material};
use crate::nbt::NbtWriter;

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

    pub fn set_block(&mut self, x: u8, y: u8, z: u8, block: Block) {
        debug_assert!(x < self.x_size, "x={} >= x_size={}", x, self.x_size);
        debug_assert!(y < self.y_size, "y={} >= y_size={}", y, self.y_size);
        debug_assert!(z < self.z_size, "z={} >= z_size={}", z, self.z_size);

        let index = (y as usize * self.z_size as usize + z as usize) * self.x_size as usize + x as usize;
        self.blocks[index] = block;
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
