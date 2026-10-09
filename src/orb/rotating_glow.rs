use crate::canvas::{BlendMode, Canvas};
use crate::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationDirection {
    Clockwise,
    CounterClockwise,
}

impl RotationDirection {
    pub fn multiplier(self) -> f64 {
        match self {
            RotationDirection::Clockwise => 1.0,
            RotationDirection::CounterClockwise => -1.0,
        }
    }
}

pub fn render(size: usize, color: Color, rotation: f32) -> Canvas {
    let s = size as f32;
    let center = s / 2.0;
    let blur_r = s * 0.16;

    let mut mask = Canvas::new(size, size);
    mask.fill_circle(center, center, s / 2.0, Color::WHITE);
    mask.blur(blur_r);

    let mut cut = Canvas::new(size, size);
    cut.fill_circle(center, center + s * 0.31, s * 1.31 / 2.0, Color::WHITE);
    cut.blur(blur_r);
    mask.composite(&cut, BlendMode::DestinationOut);

    let mut glow = Canvas::new(size, size);
    for y in 0..size {
        for x in 0..size {
            let a = mask.get(x, y)[3];
            glow.set(x, y, [color.r, color.g, color.b, color.a * a]);
        }
    }

    glow.rotated(center, center, rotation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crescent_is_mostly_top_and_center_mask_applies() {
        let c = render(128, Color::WHITE, 0.0);

        assert!(c.get(64, 64)[3] < 0.2, "center alpha {}", c.get(64, 64)[3]);

        let top = c.get(64, 20)[3];
        assert!(top > 0.3, "top alpha {top}");
    }

    #[test]
    fn direction_multiplier() {
        assert_eq!(RotationDirection::Clockwise.multiplier(), 1.0);
        assert_eq!(RotationDirection::CounterClockwise.multiplier(), -1.0);
    }
}
