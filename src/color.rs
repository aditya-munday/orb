#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b, a: 1.0 }
    }

    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Color::rgba8(r, g, b, 255)
    }

    pub fn from_hex(hex: &str) -> Option<Color> {
        let h = hex.strip_prefix('#').unwrap_or(hex);
        let parse = |s: &str| u8::from_str_radix(s, 16).ok();
        match h.len() {
            6 => Some(Color::rgb8(
                parse(&h[0..2])?,
                parse(&h[2..4])?,
                parse(&h[4..6])?,
            )),
            8 => Some(Color::rgba8(
                parse(&h[0..2])?,
                parse(&h[2..4])?,
                parse(&h[4..6])?,
                parse(&h[6..8])?,
            )),
            _ => None,
        }
    }

    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const CLEAR: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    pub const RED: Color = Color::rgb8(255, 59, 48);
    pub const ORANGE: Color = Color::rgb8(255, 149, 0);
    pub const YELLOW: Color = Color::rgb8(255, 204, 0);
    pub const GREEN: Color = Color::rgb8(52, 199, 89);
    pub const MINT: Color = Color::rgb8(0, 199, 190);
    pub const TEAL: Color = Color::rgb8(48, 176, 199);
    pub const CYAN: Color = Color::rgb8(50, 173, 230);
    pub const BLUE: Color = Color::rgb8(0, 122, 255);
    pub const INDIGO: Color = Color::rgb8(88, 86, 214);
    pub const PURPLE: Color = Color::rgb8(175, 82, 222);
    pub const PINK: Color = Color::rgb8(255, 45, 85);
    pub const GRAY: Color = Color::rgb8(142, 142, 147);

    pub fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    pub fn opacity(self, factor: f32) -> Color {
        Color {
            a: self.a * factor,
            ..self
        }
    }

    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    pub fn premult(self) -> [f32; 4] {
        [self.r * self.a, self.g * self.a, self.b * self.a, self.a]
    }

    pub fn to_rgba8(self) -> [u8; 4] {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        [c(self.r), c(self.g), c(self.b), c(self.a)]
    }
}

pub fn gradient_at(colors: &[Color], t: f32) -> Color {
    match colors.len() {
        0 => Color::CLEAR,
        1 => colors[0],
        n => {
            let t = t.clamp(0.0, 1.0) * (n - 1) as f32;
            let i = (t.floor() as usize).min(n - 2);
            colors[i].lerp(colors[i + 1], t - i as f32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing() {
        assert_eq!(Color::from_hex("#FF0000"), Some(Color::rgb8(255, 0, 0)));
        assert_eq!(Color::from_hex("00FF00").unwrap().g, 1.0);
        assert_eq!(Color::from_hex("#00000080").unwrap().a, 128.0 / 255.0);
        assert!(Color::from_hex("#xyz").is_none());
    }

    #[test]
    fn premultiply_and_opacity() {
        let c = Color::rgb(1.0, 1.0, 1.0).opacity(0.5);
        assert_eq!(c.premult(), [0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn gradient_interpolates() {
        let g = [Color::BLACK, Color::WHITE];
        assert_eq!(gradient_at(&g, 0.0), Color::BLACK);
        assert_eq!(gradient_at(&g, 1.0), Color::WHITE);
        assert!((gradient_at(&g, 0.5).r - 0.5).abs() < 1e-6);
    }
}
