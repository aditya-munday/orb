use crate::canvas::{BlendMode, Canvas};
use crate::color::Color;
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct ParticleGroup {
    pub speed_range: (f32, f32),
    pub size_range: (f32, f32),
    pub particle_count: usize,
    pub opacity_range: (f32, f32),
    pub blur: f32,
}

struct Rng(u32);

impl Rng {
    fn new(seed: u32) -> Self {
        Rng(seed | 1)
    }
    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    fn unit(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

fn keyframe(times: &[f32], values: &[f32], t: f32) -> f32 {
    if t <= times[0] {
        return values[0];
    }
    for i in 0..times.len() - 1 {
        if t <= times[i + 1] {
            let span = times[i + 1] - times[i];
            let f = if span <= 1e-6 {
                0.0
            } else {
                (t - times[i]) / span
            };
            return values[i] + (values[i + 1] - values[i]) * f;
        }
    }
    *values.last().unwrap()
}

struct Particle {
    phase: f32,
    lifetime: f32,
    x0: f32,
    y0: f32,
    vx: f32,
    vy: f32,
    alpha_values: [f32; 4],
    scale_values: [f32; 4],
}

fn build_group(group: &ParticleGroup, size: f32, seed: u32) -> Vec<Particle> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(group.particle_count);
    for _ in 0..group.particle_count {
        let lifetime = 2.0 + rng.range(-0.5, 0.5);
        let x0 = rng.range(-size / 2.0, size / 2.0);
        let y0 = rng.range(-size / 2.0, size / 2.0);
        let angle = PI / 2.0 + rng.range(-PI / 6.0, PI / 6.0);
        let speed = rng.range(group.speed_range.0, group.speed_range.1);

        let vx = speed * angle.cos();
        let vy = -speed * angle.sin();
        let alpha = [
            0.0,
            rng.range(group.opacity_range.0, group.opacity_range.1),
            rng.range(group.opacity_range.0, group.opacity_range.1),
            rng.range(group.opacity_range.0, group.opacity_range.1),
        ];
        let (lo, hi) = group.size_range;
        let scale = [lo * 0.7, hi * 0.9, hi, lo * 0.8];
        out.push(Particle {
            phase: rng.unit(),
            lifetime,
            x0,
            y0,
            vx,
            vy,
            alpha_values: alpha,
            scale_values: scale,
        });
    }
    out
}

pub fn render_group(group: &ParticleGroup, size: usize, t: f32, color: Color, seed: u32) -> Canvas {
    let s = size as f32;
    let center = s / 2.0;
    let particles = build_group(group, s, seed);
    let texture_radius = 4.0;
    let mut canvas = Canvas::new(size, size);

    for p in &particles {
        let age = (t + p.phase * p.lifetime) % p.lifetime;
        let u = age / p.lifetime;
        let alpha = keyframe(&[0.0, 0.2, 0.8, 1.0], &p.alpha_values, u);
        let scale = keyframe(&[0.0, 0.4, 0.7, 1.0], &p.scale_values, u);

        let x = center + p.x0 + p.vx * age;
        let y = center + p.y0 + p.vy * age + 0.5 * (-20.0) * age * age;
        let r = texture_radius * scale;
        if r > 0.1 && alpha > 0.001 {
            canvas.fill_circle(x, y, r, color.with_alpha(color.a * alpha));
        }
    }

    if group.blur > 0.0 {
        canvas.blur(group.blur);
    }
    canvas
}

pub fn render(size: usize, color: Color, t: f32) -> Canvas {
    let group_a = ParticleGroup {
        speed_range: (10.0, 20.0),
        size_range: (0.5, 1.0),
        particle_count: 10,
        opacity_range: (0.0, 0.3),
        blur: 1.0,
    };
    let group_b = ParticleGroup {
        speed_range: (20.0, 30.0),
        size_range: (0.2, 1.0),
        particle_count: 10,
        opacity_range: (0.3, 0.8),
        blur: 0.0,
    };
    let mut canvas = render_group(&group_a, size, t, color, 0x9E3779B9);
    let b = render_group(&group_b, size, t, color, 0x85EBCA6B);
    canvas.composite(&b, BlendMode::Normal);
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyframe_hits_endpoints() {
        let times = [0.0, 0.2, 0.8, 1.0];
        let values = [0.0, 1.0, 1.0, 0.5];
        assert_eq!(keyframe(&times, &values, 0.0), 0.0);
        assert_eq!(keyframe(&times, &values, 0.2), 1.0);
        assert_eq!(keyframe(&times, &values, 1.0), 0.5);
    }

    #[test]
    fn particles_are_deterministic() {
        let a = render(64, Color::WHITE, 1.0);
        let b = render(64, Color::WHITE, 1.0);
        assert_eq!(a.pixels, b.pixels);
    }

    #[test]
    fn particles_draw_something() {
        let c = render(128, Color::WHITE, 0.7);
        assert!(!c.is_empty(), "expected some particle pixels");
    }
}
