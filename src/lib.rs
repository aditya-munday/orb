pub mod canvas;
pub mod color;
pub mod orb;
pub mod orb_config;
pub mod overlay;
pub mod tts;

pub use canvas::{BlendMode, Canvas};
pub use color::{gradient_at, Color};
pub use orb::{OrbView, RotationDirection};
pub use orb_config::{all_presets, OrbConfiguration};
pub use overlay::{OrbOverlay, OverlaySettings, Rect, SegmentPlacement};
pub use tts::{TtsReactive, TtsSettings};

pub fn encode_png(canvas: &Canvas) -> Result<Vec<u8>, png::EncodingError> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, canvas.width as u32, canvas.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&canvas.to_rgba8())?;
    }
    Ok(out)
}

pub fn on_background(canvas: &Canvas, background: Color) -> Canvas {
    let mut base = Canvas::new(canvas.width, canvas.height);
    base.pixels
        .fill([background.r, background.g, background.b, 1.0]);
    base.composite(canvas, BlendMode::Normal);
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_encodes_with_signature() {
        let orb = OrbView::default();
        let frame = orb.render_at(64, 0.0);
        let png = encode_png(&frame).unwrap();
        assert_eq!(
            &png[0..8],
            &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn on_background_is_opaque() {
        let orb = OrbView::default();
        let frame = on_background(&orb.render_at(32, 0.0), Color::BLACK);
        assert!(frame.pixels.iter().all(|p| p[3] == 1.0));
    }
}
