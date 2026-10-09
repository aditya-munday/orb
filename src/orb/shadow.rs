use crate::canvas::Canvas;
use crate::color::Color;

pub fn render(size: usize, colors: &[Color], radius: f32) -> Canvas {
    let s = size as f32;
    let center = s / 2.0;
    let mut canvas = Canvas::new(size, size);

    let mut back = Canvas::new(size, size);
    back.fill_gradient(colors, |_, y| 1.0 - y / s);
    back.mask_circle(center, center, s / 2.0, true);
    back = faded_and_shifted(&back, 0.3, radius * 0.75);
    back.blur(radius * 3.0);

    let mut front = Canvas::new(size, size);
    front.fill_gradient(colors, |_, y| 1.0 - y / s);
    front.mask_circle(center, center, s / 2.0, true);
    front = faded_and_shifted(&front, 0.5, radius * 0.5);
    front.blur(radius * 0.75);

    canvas.composite(&back, crate::canvas::BlendMode::Normal);
    canvas.composite(&front, crate::canvas::BlendMode::Normal);
    canvas
}

fn faded_and_shifted(src: &Canvas, opacity: f32, dy: f32) -> Canvas {
    let mut faded = Canvas::new(src.width, src.height);
    for y in 0..src.height {
        for x in 0..src.width {
            let p = src.get(x, y);
            faded.set(x, y, [p[0], p[1], p[2], p[3] * opacity]);
        }
    }
    let mut out = Canvas::new(src.width, src.height);
    out.composite_shifted(&faded, 0.0, dy, crate::canvas::BlendMode::Normal);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_below_orb_is_stronger() {
        let c = render(128, &[Color::PURPLE, Color::BLUE], 128.0 * 0.08);
        let below = c.get(64, 110)[3];
        let above = c.get(64, 18)[3];
        assert!(below > above, "below {below} above {above}");
    }
}
