use crate::canvas::{BlendMode, Canvas};
use crate::color::Color;

pub fn render(size: usize, outline_colors: &[Color]) -> Canvas {
    let s = size as f32;
    let center = s / 2.0;
    let r = center - 1.0;

    let mut out = Canvas::new(size, size);

    let strokes: [(f32, f32); 3] = [(8.0, 32.0), (4.0, 12.0), (1.0, 4.0)];
    for (width, blur) in strokes {
        let mut layer = Canvas::new(size, size);
        layer.stroke_circle_gradient(center, center, r, width, outline_colors);
        layer.blur(blur);
        out.composite(&layer, BlendMode::PlusLighter);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rim_is_bright_and_interior_is_dim() {
        let c = render(128, &[Color::WHITE, Color::CLEAR]);
        let rim = c.get(64, 4)[3];
        let inner = c.get(64, 60)[3];
        assert!(rim > inner, "rim {rim} should exceed inner {inner}");
    }
}
