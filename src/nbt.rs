use flate2::write::GzEncoder;
use std::io::Write;

#[repr(u8)]
#[allow(dead_code)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray
}

pub struct NbtWriter<W: Write> {
    w: GzEncoder<W>,
}

impl<W: Write> NbtWriter<W> {
    pub fn new(w: W) -> NbtWriter<W> {
        NbtWriter {
            w: GzEncoder::new(w, flate2::Compression::default()),
        }
    }

    fn write_tag(&mut self, value: TagType, name: &str) {
        self.w.write_all(&[value as u8]).unwrap();

        let name_len = name.len();
        self.w.write_all(&[(name_len >> 8) as u8, name_len as u8]).unwrap();
        self.w.write_all(name.as_bytes()).unwrap();
    }

    pub fn write_short(&mut self, name: &str, value: i16) {
        self.write_tag(TagType::Short, name);
        self.w.write_all(&[(value >> 8) as u8, value as u8]).unwrap();
    }

    pub fn write_byte_array(&mut self, name: &str, value: &[u8]) {
        self.write_tag(TagType::ByteArray, name);
        let len = value.len() as i32;
        self.w.write_all(&[
            (len >> 24) as u8,
            (len >> 16) as u8,
            (len >> 8) as u8,
            len as u8
        ]).unwrap();
        self.w.write_all(value).unwrap();
    }

    pub fn write_string(&mut self, name: &str, value: &str) {
        self.write_tag(TagType::String, name);

        let value_len = value.len() as u16;
        self.w.write_all(&[(value_len >> 8) as u8, value_len as u8]).unwrap();
        self.w.write_all(value.as_bytes()).unwrap();
    }

    pub fn begin_compound(&mut self, name: &str) {
        self.write_tag(TagType::Compound, name);
    }

    pub fn end_compound(&mut self) {
        self.w.write_all(&[TagType::End as u8]).unwrap();
    }
    
    pub fn finish(self) {
        self.w.finish().unwrap();
    }
}
