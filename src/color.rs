/// Represents 8-bit color in the format RRRGGGBB
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u8);

impl Color {
    /// Converts the color to 3 floats corresponding to red, green, and blue
    /// ranging from 0.0 to 1.0
    pub fn to_rgb_normalized(self) -> [f32; 3] {
        let r = (self.0 >> 5) as f32 / 7.0;
        let g = ((self.0 >> 2) & 0b111) as f32 / 7.0;
        let b = (self.0 & 0b11) as f32 / 3.0;
        [r, g, b]
    }

    pub fn try_from_octal_string(s: &str) -> Result<Color, InvalidColorOctal> {
        let bytes = s.as_bytes();
        if bytes.len() != 3 {
            return Err(InvalidColorOctal::InvalidLength(bytes.len()))
        }

        let mut values = [0u8; 3];

        for i in 0..3 {
            let value = bytes[i].wrapping_sub(b'0');
            let max = if i == 2 { 4 } else { 8 };
            if value >= max {
                return Err(InvalidColorOctal::InvalidDigit { index: i, value: bytes[i] as char })
            }
            values[i] = value;
        }

        Ok(Color((values[0] << 5) | (values[1] << 2) | values[2]))
    }
}

#[derive(Debug, PartialEq)]
pub enum InvalidColorOctal {
    InvalidLength(usize),
    InvalidDigit{ index: usize, value: char },
}

#[cfg(test)]
mod tests {
    use crate::color::InvalidColorOctal;

    use super::Color;

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Color>(), 1);
    }

    #[test]
    fn test_to_rgb_normalized() {
        assert_eq!(Color(0b00100101).to_rgb_normalized(), [1f32/7., 1f32/7., 1f32/3.]);
        assert_eq!(Color(0b01001101).to_rgb_normalized(), [2./7., 3./7., 1./3.]);
        assert_eq!(Color(0b00100110).to_rgb_normalized(), [1./7., 1./7., 2./3.]);
        assert_eq!(Color(0b11111111).to_rgb_normalized(), [1., 1., 1.]);
    }

    #[test]
    fn test_from_octal_str() {
        assert_eq!(Color::try_from_octal_string("000"), Ok(Color(0o000)));
        assert_eq!(Color::try_from_octal_string("123"), Ok(Color(0b00101011)));
        assert_eq!(Color::try_from_octal_string("773"), Ok(Color(0b11111111)));
        assert_eq!(Color::try_from_octal_string("234"), Err(InvalidColorOctal::InvalidDigit { index: 2, value: '4' }));
        assert_eq!(Color::try_from_octal_string("283"), Err(InvalidColorOctal::InvalidDigit { index: 1, value: '8' }));
        assert_eq!(Color::try_from_octal_string("77a"), Err(InvalidColorOctal::InvalidDigit { index: 2, value: 'a' }));
        assert_eq!(Color::try_from_octal_string("!73"), Err(InvalidColorOctal::InvalidDigit { index: 0, value: '!' }));
        assert_eq!(Color::try_from_octal_string("77\u{00A7}"), Err(InvalidColorOctal::InvalidLength(4)));
        assert_eq!(Color::try_from_octal_string(" 000"), Err(InvalidColorOctal::InvalidLength(4)));
        assert_eq!(Color::try_from_octal_string("000 "), Err(InvalidColorOctal::InvalidLength(4)));
        assert_eq!(Color::try_from_octal_string("0000"), Err(InvalidColorOctal::InvalidLength(4)));
    }
}
