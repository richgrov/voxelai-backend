mod block;
mod schematic;
mod nbt;

use std::fs::File;

use block::{Block, Material};
use schematic::Schematic;

fn main() {
    let mut schem = Schematic::new(255, 100, 255);
    for x in 0..schem.x_size() {
        for y in 0..schem.y_size() {
            for z in 0..schem.z_size() {
                if rand::random::<bool>() {
                    schem.set_block(x, y, z, Block::new(Material::stone))
                }
            }
        }
    }

    schem.serialize(&mut File::create("test.schem").unwrap());
}
