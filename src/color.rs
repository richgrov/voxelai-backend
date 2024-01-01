/// 24-bit True color
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn to_rgb_normalized(self) -> [f32; 3] {
        [
            self.0 as f32 / u8::MAX as f32,
            self.1 as f32 / u8::MAX as f32,
            self.2 as f32 / u8::MAX as f32,
        ]
    }

    pub fn try_from_hex_string(s: &str) -> Result<Color, InvalidColorHex> {
        if s.len() != 6 {
            return Err(InvalidColorHex(s.to_owned()));
        }

        let mut values = [0u8; 3];

        for i in 0..3 {
            values[i] = match u8::from_str_radix(&s[i * 2..i * 2 + 2], 16) {
                Ok(b) => b,
                Err(_) => return Err(InvalidColorHex(s.to_owned())),
            };
        }

        Ok(Color(values[0], values[1], values[2]))
    }
}

#[derive(Debug, PartialEq)]
pub struct InvalidColorHex(String);

#[cfg(test)]
mod tests {
    use crate::color::InvalidColorHex;

    use super::Color;

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<Color>(), 3);
    }

    #[test]
    fn test_to_rgb_normalized() {
        assert_eq!(Color(0, 0, 0).to_rgb_normalized(), [0., 0., 0.]);
        assert_eq!(Color(255, 0, 255).to_rgb_normalized(), [1., 0., 1.]);
        assert_eq!(
            Color(85, 64, 1).to_rgb_normalized(),
            [85. / 255., 64. / 255., 1. / 255.]
        );
        assert_eq!(
            Color(38, 76, 55).to_rgb_normalized(),
            [38. / 255., 76. / 255., 55. / 255.]
        );
    }

    #[test]
    fn test_from_octal_str() {
        assert_eq!(Color::try_from_hex_string("000000"), Ok(Color(0, 0, 0)));
        assert_eq!(
            Color::try_from_hex_string("123ABC"),
            Ok(Color(0x12, 0x3A, 0xBC))
        );
        assert_eq!(
            Color::try_from_hex_string("def789"),
            Ok(Color(0xDE, 0xF7, 0x89))
        );

        let errors = [
            "00000",
            "bcdefg",
            " 123ab",
            "cde56 ",
            "890 ff",
            "77\u{00A7}777",
        ];
        for err in errors {
            assert_eq!(
                Color::try_from_hex_string(err),
                Err(InvalidColorHex(err.to_owned()))
            );
        }
    }
}
