use crate::canvas::Canvas;
use crate::color::Color;
use std::f32::consts::PI;

fn base_points() -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let angle = (i as f32 / 6.0) * 2.0 * PI;
        (0.5 + angle.cos() * 0.9, 0.5 + angle.sin() * 0.9)
    })
}

pub fn blob_points(size: f32, t: f32, loop_duration: f32) -> Vec<(f32, f32)> {
    let loop_duration = loop_duration.max(1e-4);
    let angle = ((t % loop_duration) / loop_duration) * 2.0 * PI;
    let center = size / 2.0;
    let radius = size * 0.45;
    base_points()
        .iter()
        .enumerate()
        .map(|(i, &(px, py))| {
            let phase = i as f32 * PI / 3.0;
            let x_off = (angle + phase).sin() * 0.15;
            let y_off = (angle + phase).cos() * 0.15;
            (
                (px - 0.5 + x_off) * radius + center,
                (py - 0.5 + y_off) * radius + center,
            )
        })
        .collect()
}

fn catmull_rom_polygon(pts: &[(f32, f32)], samples_per_seg: usize) -> Vec<(f32, f32)> {
    let n = pts.len();
    let mut out = Vec::with_capacity(n * samples_per_seg);
    for i in 0..n {
        let p0 = pts[(i + n - 1) % n];
        let p1 = pts[i];
        let p2 = pts[(i + 1) % n];
        let p3 = pts[(i + 2) % n];
        for s in 0..samples_per_seg {
            let t = s as f32 / samples_per_seg as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let x = 0.5
                * ((2.0 * p1.0)
                    + (-p0.0 + p2.0) * t
                    + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
                    + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
            let y = 0.5
                * ((2.0 * p1.1)
                    + (-p0.1 + p2.1) * t
                    + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
                    + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
            out.push((x, y));
        }
    }
    out
}

pub fn render(size: usize, color: Color, t: f32, loop_duration: f32) -> Canvas {
    let mut canvas = Canvas::new(size, size);
    let pts = blob_points(size as f32, t, loop_duration);
    let smooth = catmull_rom_polygon(&pts, 24);
    canvas.fill_polygon(&smooth, color);
    canvas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_stay_near_bounds() {
        for step in 0..20 {
            let t = step as f32 * 0.1;
            let pts = blob_points(100.0, t, 1.75);
            assert_eq!(pts.len(), 6);
            for (x, y) in pts {
                assert!(x > 0.0 && x < 100.0, "x {x}");
                assert!(y > 0.0 && y < 100.0, "y {y}");
            }
        }
    }

    #[test]
    fn blob_renders_opaque_center_and_transparent_corner() {
        let c = render(128, Color::WHITE, 0.0, 1.75);
        assert_eq!(c.get(0, 0)[3], 0.0);
        assert!(c.get(64, 64)[3] > 0.5, "center {}", c.get(64, 64)[3]);
    }

    #[test]
    fn loop_is_periodic() {
        let a = blob_points(100.0, 0.3, 2.0);
        let b = blob_points(100.0, 2.3, 2.0);
        for (p, q) in a.iter().zip(b.iter()) {
            assert!((p.0 - q.0).abs() < 1e-3 && (p.1 - q.1).abs() < 1e-3);
        }
    }
}
