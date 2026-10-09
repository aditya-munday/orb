use minifb::{Key, Window, WindowOptions};
use orb::{on_background, Color, OrbView};

fn main() {
    let mut size = 420usize;
    let mut preset = "default".to_string();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--size" | "-s" => {
                if let Some(v) = args.next().and_then(|v| v.parse().ok()) {
                    size = v;
                }
            }
            "--preset" | "-p" => {
                if let Some(v) = args.next() {
                    preset = v;
                }
            }
            _ => {}
        }
    }

    let presets: Vec<String> = orb::all_presets()
        .into_iter()
        .map(|(n, _)| n.to_string())
        .collect();

    let mut window = Window::new("Orb — Rust", size, size, WindowOptions::default())
        .expect("failed to open window");
    window.set_target_fps(60);

    let internal = size.min(220);

    let start = std::time::Instant::now();
    let mut buffer = vec![0u32; size * size];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (i, key) in [
            Key::Key1,
            Key::Key2,
            Key::Key3,
            Key::Key4,
            Key::Key5,
            Key::Key6,
            Key::Key7,
            Key::Key8,
            Key::Key9,
            Key::Key0,
        ]
        .iter()
        .enumerate()
        {
            if window.is_key_down(*key) {
                if let Some(name) = presets.get(i) {
                    preset = name.clone();
                }
            }
        }

        let view = OrbView::preset(&preset);
        let t = start.elapsed().as_secs_f64();
        let frame = on_background(&view.render_at(internal, t), Color::BLACK);
        for y in 0..size {
            let sy = (y * internal / size).min(internal - 1);
            for x in 0..size {
                let sx = (x * internal / size).min(internal - 1);
                let p = frame.get(sx, sy);
                let r = (p[0].clamp(0.0, 1.0) * 255.0) as u32;
                let g = (p[1].clamp(0.0, 1.0) * 255.0) as u32;
                let b = (p[2].clamp(0.0, 1.0) * 255.0) as u32;
                buffer[y * size + x] = (r << 16) | (g << 8) | b;
            }
        }
        window.set_title(&format!("Orb — {preset} (t={t:.1}s)"));
        window.update_with_buffer(&buffer, size, size).unwrap();
    }
}
