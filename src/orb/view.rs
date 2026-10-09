use crate::canvas::{BlendMode, Canvas};
use crate::color::Color;
use crate::orb::rotating_glow::RotationDirection;
use crate::orb::{inner_glow, particles, rotating_glow, shadow, wavy_blob};
use crate::orb_config::OrbConfiguration;

#[derive(Clone, Debug)]
pub struct OrbView {
    pub configuration: OrbConfiguration,
}

impl Default for OrbView {
    fn default() -> Self {
        OrbView::new(OrbConfiguration::default())
    }
}

impl OrbView {
    pub fn new(configuration: OrbConfiguration) -> Self {
        OrbView { configuration }
    }

    pub fn preset(name: &str) -> Self {
        let cfg = crate::orb_config::all_presets()
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, c)| c)
            .unwrap_or_default();
        OrbView::new(cfg)
    }

    pub fn mystic() -> Self {
        OrbView::new(OrbConfiguration::mystic())
    }
    pub fn nature() -> Self {
        OrbView::new(OrbConfiguration::nature())
    }
    pub fn sunset() -> Self {
        OrbView::new(OrbConfiguration::sunset())
    }
    pub fn ocean() -> Self {
        OrbView::new(OrbConfiguration::ocean())
    }
    pub fn minimal() -> Self {
        OrbView::new(OrbConfiguration::minimal())
    }
    pub fn cosmic() -> Self {
        OrbView::new(OrbConfiguration::cosmic())
    }
    pub fn fire() -> Self {
        OrbView::new(OrbConfiguration::fire())
    }
    pub fn arctic() -> Self {
        OrbView::new(OrbConfiguration::arctic())
    }
    pub fn shadow() -> Self {
        OrbView::new(OrbConfiguration::shadow())
    }

    pub fn configuration(&self) -> &OrbConfiguration {
        &self.configuration
    }

    pub fn configuration_mut(&mut self) -> &mut OrbConfiguration {
        &mut self.configuration
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.configuration.speed = speed;
    }

    pub fn set_core_glow(&mut self, intensity: f64) {
        self.configuration.core_glow_intensity = intensity;
    }

    pub fn set_glow_color(&mut self, color: Color) {
        self.configuration.glow_color = color;
    }

    pub fn render_at(&self, canvas_size: usize, t: f64) -> Canvas {
        let s = canvas_size as f32;
        let t = t as f32;
        let cfg = &self.configuration;

        let orb = self.render_orb(s, t);

        let mut scene = Canvas::new(canvas_size, canvas_size);
        if cfg.show_shadow {
            let shadow = if self.low_res() {
                shadow::render(
                    self.low_size(s),
                    &cfg.background_colors,
                    self.low_size(s) as f32 * 0.08,
                )
                .resized(canvas_size, canvas_size)
            } else {
                shadow::render(canvas_size, &cfg.background_colors, s * 0.08)
            };
            scene.composite(&shadow, BlendMode::Normal);
        }
        scene.composite(&orb, BlendMode::Normal);
        scene
    }

    fn low_res(&self) -> bool {
        self.configuration.render_scale < 0.999
    }

    fn low_size(&self, size: f32) -> usize {
        ((size * self.configuration.render_scale.clamp(0.25, 1.0) as f32).round() as usize).max(8)
    }

    fn render_orb(&self, size: f32, t: f32) -> Canvas {
        let cfg = &self.configuration;
        let n = size.max(1.0) as usize;

        let mut z = Canvas::new(n, n);

        if cfg.show_background {
            z.fill_gradient(&cfg.background_colors, |_, y| 1.0 - y / size);
        }

        let diffuse = self.low_res();
        let ds = if diffuse {
            size * cfg.render_scale as f32
        } else {
            size
        };

        {
            let group = self.base_depth_glows(ds, t);
            let group = if diffuse { group.resized(n, n) } else { group };
            z.composite(&group, BlendMode::Normal);
        }

        if cfg.show_wavy_blobs {
            let a = self.wavy_blob_one(ds, t);
            let a = if diffuse { a.resized(n, n) } else { a };
            z.composite(&a, BlendMode::PlusLighter);
            let b = self.wavy_blob_two(ds, t);
            let b = if diffuse { b.resized(n, n) } else { b };
            z.composite(&b, BlendMode::PlusLighter);
        }

        if cfg.show_glow_effects {
            let g = self.core_glow_effects(ds, t);
            let g = if diffuse { g.resized(n, n) } else { g };
            z.composite(&g, BlendMode::Normal);
        }

        if cfg.show_particles {
            let p = particles::render(n, cfg.particle_color, t);
            z.composite(&p, BlendMode::PlusLighter);
        }

        let inner = inner_glow::render(n, &[Color::WHITE, Color::CLEAR]);
        z.composite(&inner, BlendMode::Normal);

        z.mask_circle(size / 2.0, size / 2.0, size / 2.0, true);
        z
    }

    fn base_depth_glows(&self, size: f32, t: f32) -> Canvas {
        let cfg = &self.configuration;
        let speed = cfg.speed as f32;
        let n = size.max(1.0) as usize;

        let angle =
            speed * 0.75 * t * RotationDirection::CounterClockwise.multiplier() as f32 + 180.0;
        let mut glow1 = rotating_glow::render(n, cfg.glow_color, angle);
        glow1.blur(size * 0.06);

        let ring_geo = (size * 0.94 - 16.0).max(1.0);
        let ring_n = ring_geo as usize;
        let angle2 = speed * 0.25 * t * RotationDirection::Clockwise.multiplier() as f32 + 180.0;
        let mut ring = rotating_glow::render(ring_n, cfg.glow_color.opacity(0.5), angle2);
        ring.blur(size * 0.032);

        let mut group = Canvas::new(n, n);
        let dx = (size - ring_geo) / 2.0;
        group.composite_shifted(&ring, dx, dx, BlendMode::Normal);

        {
            use rayon::prelude::*;
            let w = n;
            group
                .pixels
                .par_chunks_mut(w)
                .enumerate()
                .for_each(|(y, row)| {
                    for (x, d) in row.iter_mut().enumerate() {
                        let s1 = glow1.get(x, y);
                        if s1[3] <= 0.0 {
                            continue;
                        }
                        *d = over_straight(*d, s1);
                    }
                });
        }
        group
    }

    fn wavy_blob_one(&self, size: f32, t: f32) -> Canvas {
        let cfg = &self.configuration;
        let speed = cfg.speed as f32;
        let n = size.max(1.0) as usize;

        let angle = speed * 1.5 * t * RotationDirection::Clockwise.multiplier() as f32;
        let mut glow = rotating_glow::render(n, Color::WHITE.opacity(0.75), angle);
        let mask = self.blob_mask(
            size,
            size * 1.875,
            0.0,
            size * 0.31,
            t,
            60.0 / speed * 1.75,
            0.0,
        );
        glow.multiply_alpha_mask(&mask);
        glow.blur(1.0);
        glow
    }

    fn wavy_blob_two(&self, size: f32, t: f32) -> Canvas {
        let cfg = &self.configuration;
        let speed = cfg.speed as f32;
        let n = size.max(1.0) as usize;

        let angle = speed * 0.75 * t * RotationDirection::CounterClockwise.multiplier() as f32;
        let mut glow = rotating_glow::render(n, Color::WHITE, angle);
        let mask = self.blob_mask(
            size,
            size * 1.25,
            0.0,
            -size * 0.31,
            t,
            60.0 / speed * 2.25,
            90.0,
        );
        glow.multiply_alpha_mask(&mask);
        apply_opacity(&mut glow, 0.5);
        glow.blur(1.0);
        glow
    }

    #[allow(clippy::too_many_arguments)]
    fn blob_mask(
        &self,
        orb_size: f32,
        geo_size: f32,
        dx: f32,
        dy: f32,
        t: f32,
        loop_duration: f32,
        rotation_deg: f32,
    ) -> Canvas {
        let n = orb_size.max(1.0) as usize;
        let mut layer =
            wavy_blob::render(geo_size.max(1.0) as usize, Color::WHITE, t, loop_duration);
        if rotation_deg != 0.0 {
            layer = layer.rotated(geo_size / 2.0, geo_size / 2.0, rotation_deg);
        }
        let place_x = (orb_size - geo_size) / 2.0 + dx;
        let place_y = (orb_size - geo_size) / 2.0 + dy;
        let mut mask = Canvas::new(n, n);
        mask.composite_shifted(&layer, place_x, place_y, BlendMode::Normal);
        mask
    }

    fn core_glow_effects(&self, size: f32, t: f32) -> Canvas {
        let cfg = &self.configuration;
        let speed = cfg.speed as f32;
        let inner = size * 0.84;
        let n = inner as usize;

        let mut out = Canvas::new(size as usize, size as usize);

        let a1 = speed * 3.0 * t * RotationDirection::Clockwise.multiplier() as f32;
        let mut g1 = rotating_glow::render(n, cfg.glow_color, a1);
        g1.blur(size * 0.08);
        apply_opacity(&mut g1, cfg.core_glow_intensity as f32);

        let a2 = speed * 2.3 * t * RotationDirection::Clockwise.multiplier() as f32;
        let mut g2 = rotating_glow::render(n, cfg.glow_color, a2);
        g2.blur(size * 0.06);
        apply_opacity(&mut g2, cfg.core_glow_intensity as f32);

        let off = (size - inner) / 2.0;
        out.composite_shifted(&g1, off, off, BlendMode::Normal);
        out.composite_shifted(&g2, off, off, BlendMode::PlusLighter);
        out
    }
}

fn apply_opacity(c: &mut Canvas, factor: f32) {
    for p in &mut c.pixels {
        p[3] *= factor;
    }
}

fn over_straight(d: [f32; 4], s: [f32; 4]) -> [f32; 4] {
    let sa = s[3];
    let oa = sa + d[3] * (1.0 - sa);
    if oa <= 1e-6 {
        return [0.0; 4];
    }
    [
        (s[0] * sa + d[0] * d[3] * (1.0 - sa)) / oa,
        (s[1] * sa + d[1] * d[3] * (1.0 - sa)) / oa,
        (s[2] * sa + d[2] * d[3] * (1.0 - sa)) / oa,
        oa,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_opaque_center_transparent_corner() {
        let orb = OrbView::default();
        let c = orb.render_at(96, 0.0);

        assert!(c.get(0, 0)[3] < 0.1, "corner alpha {}", c.get(0, 0)[3]);
        assert!(c.get(48, 48)[3] > 0.9, "center should be opaque");
    }

    #[test]
    fn configuration_toggles_change_output() {
        let base = OrbView::default();
        let plain = OrbView::new(
            OrbConfiguration::default()
                .show_background(false)
                .show_wavy_blobs(false)
                .show_particles(false)
                .show_glow_effects(false)
                .show_shadow(false),
        );
        assert_ne!(
            base.render_at(64, 0.3).pixels,
            plain.render_at(64, 0.3).pixels
        );
    }

    #[test]
    fn animation_moves_pixels_over_time() {
        let orb = OrbView::default();
        let a = orb.render_at(64, 0.0);
        let b = orb.render_at(64, 0.5);
        assert_ne!(a.pixels, b.pixels);
    }

    #[test]
    fn render_is_deterministic() {
        let orb = OrbView::mystic();
        assert_eq!(orb.render_at(64, 1.0).pixels, orb.render_at(64, 1.0).pixels);
    }

    fn mean_abs_diff(a: &Canvas, b: &Canvas) -> f64 {
        let mask = a.pixels.iter().map(|p| p[3] > 0.04).collect::<Vec<_>>();
        let (sum, count) = a
            .pixels
            .iter()
            .zip(&b.pixels)
            .zip(&mask)
            .filter(|(_, &m)| m)
            .fold((0.0f64, 0usize), |(s, n), ((pa, pb), _)| {
                let d = (0..4).map(|i| (pa[i] - pb[i]).abs() as f64).sum::<f64>() / 4.0;
                (s + d, n + 1)
            });
        if count == 0 {
            0.0
        } else {
            sum / count as f64
        }
    }

    #[test]
    fn reduced_render_scale_stays_visually_close() {
        let full = OrbView::new(OrbConfiguration::cosmic());
        let reduced = OrbView::new(OrbConfiguration::cosmic().render_scale(0.6));
        for t in [0.0, 0.37, 1.1] {
            let a = full.render_at(192, t);
            let b = reduced.render_at(192, t);
            let diff = mean_abs_diff(&a, &b);
            assert!(diff < 0.02, "t={t} mean abs diff {diff}");
        }
    }

    #[test]
    fn render_scale_is_clamped_to_valid_range() {
        assert_eq!(
            OrbConfiguration::default().render_scale(2.0).render_scale,
            1.0
        );
        assert_eq!(
            OrbConfiguration::default().render_scale(0.0).render_scale,
            0.25
        );
    }
}
