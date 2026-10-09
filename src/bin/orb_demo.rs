use std::fs;
use std::io::BufWriter;
use std::path::Path;
use std::process::ExitCode;

use orb::{encode_png, on_background, Color, OrbOverlay, OrbView, OverlaySettings};

struct Args {
    preset: String,
    size: usize,
    seconds: f32,
    fps: u32,
    time: Option<f32>,
    gif: Option<String>,
    png: Option<String>,
    frames_dir: Option<String>,
    background: Color,
    ascii: bool,
    list: bool,
    overlay: bool,
    screen: (f32, f32),
    margin: f32,
    tts: bool,
    scale: f64,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            preset: "default".into(),
            size: 256,
            seconds: 3.0,
            fps: 12,
            time: None,
            gif: None,
            png: None,
            frames_dir: None,
            background: Color::BLACK,
            ascii: false,
            list: false,
            overlay: false,
            screen: (1200.0, 700.0),
            margin: 16.0,
            tts: false,
            scale: 1.0,
        }
    }
}

fn parse_color(s: &str) -> Result<Color, String> {
    Color::from_hex(s).ok_or_else(|| format!("invalid color: {s}"))
}

fn parse_screen(s: &str) -> Result<(f32, f32), String> {
    let (w, h) = s
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("expected WIDTHxHEIGHT, got {s}"))?;
    let w: f32 = w.parse().map_err(|e| format!("{e}"))?;
    let h: f32 = h.parse().map_err(|e| format!("{e}"))?;
    Ok((w, h))
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        macro_rules! val {
            () => {
                it.next()
                    .ok_or_else(|| format!("missing value for {arg}"))?
            };
        }
        match arg.as_str() {
            "--preset" | "-p" => a.preset = val!(),
            "--size" | "-s" => a.size = val!().parse().map_err(|e| format!("{e}"))?,
            "--seconds" | "-t" => a.seconds = val!().parse().map_err(|e| format!("{e}"))?,
            "--fps" => a.fps = val!().parse().map_err(|e| format!("{e}"))?,
            "--time" => a.time = Some(val!().parse().map_err(|e| format!("{e}"))?),
            "--gif" => a.gif = Some(val!()),
            "--png" => a.png = Some(val!()),
            "--frames" => a.frames_dir = Some(val!()),
            "--bg" => a.background = parse_color(&val!())?,
            "--ascii" => a.ascii = true,
            "--list" => a.list = true,
            "--overlay" => a.overlay = true,
            "--screen" => a.screen = parse_screen(&val!())?,
            "--margin" => a.margin = val!().parse().map_err(|e| format!("{e}"))?,
            "--tts" => a.tts = true,
            "--scale" => a.scale = val!().parse().map_err(|e| format!("{e}"))?,
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(a)
}

fn print_help() {
    println!(
        "orb-demo — render the Rust Orb\n\n\
         USAGE:\n  orb-demo [OPTIONS]\n\n\
         OPTIONS:\n\
         \x20 -p, --preset <NAME>   Preset: default, mystic, nature, sunset, ocean,\n\
         \x20                      minimal, cosmic, fire, arctic, shadow [default: default]\n\
         \x20 -s, --size <PX>       Render size in pixels [default: 256]\n\
         \x20 -t, --seconds <S>     Animation length for GIF/frames [default: 3]\n\
         \x20     --fps <N>         Frames per second for GIF/frames [default: 12]\n\
         \x20     --time <S>        Render a single still at time S seconds\n\
         \x20     --gif <PATH>      Write an animated GIF\n\
         \x20     --png <PATH>      Write a single PNG (uses --time, else t=0)\n\
         \x20     --frames <DIR>    Write numbered PNG frames\n\
         \x20     --bg <#RRGGBB>    Background color [default: #000000]\n\
         \x20     --overlay         Transparent overlay mode for app integration\n\
         \x20     --screen <WxH>    Host screen size for overlay [default: 1200x700]\n\
         \x20     --margin <PX>     Overlay inset from segment edges [default: 16]\n\
         \x20     --tts             Simulate speech loudness driving the orb\n\
         \x20     --scale <F>       Internal render scale 0.25-1.0; lower is\n\
         \x20                       faster for real-time overlay use [default: 1.0]\n\
         \x20     --ascii           Print an ASCII preview to stdout\n\
         \x20     --list            List available presets and exit\n\
         \x20 -h, --help            Show this help"
    );
}

fn speak_level(t: f32) -> f32 {
    let gate = if (t % 2.2) < 1.6 { 1.0 } else { 0.0 };
    let s = 0.55 + 0.30 * (t * 7.0).sin() + 0.15 * (t * 13.0).sin();
    (s * gate).clamp(0.0, 1.0)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}\n");
            print_help();
            return ExitCode::FAILURE;
        }
    };

    if args.list {
        for (name, cfg) in orb::all_presets() {
            println!(
                "{:<8} glow={:?} speed={} core={}",
                name, cfg.glow_color, cfg.speed, cfg.core_glow_intensity
            );
        }
        return ExitCode::SUCCESS;
    }

    validate_preset(&args.preset);

    if args.overlay {
        return run_overlay(&args);
    }
    run_orb(&args)
}

fn run_overlay(args: &Args) -> ExitCode {
    let settings = OverlaySettings {
        margin: args.margin,
        ..OverlaySettings::default()
    };
    let mut view = OrbView::preset(&args.preset);
    view.configuration_mut().render_scale = args.scale.clamp(0.25, 1.0);
    let mut overlay = OrbOverlay::new(view, settings);
    overlay.resize(args.screen.0, args.screen.1);
    overlay.update(0.0, 1.0 / args.fps as f32);

    let frame_count = if args.time.is_some() {
        1
    } else {
        ((args.seconds * args.fps as f32).ceil() as usize).max(1)
    };

    let dt = 1.0 / args.fps as f32;
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let t = if let Some(single) = args.time {
            single
        } else {
            i as f32 / args.fps as f32
        };
        if args.time.is_some() && args.tts {
            for k in 0..60 {
                let warm_t = (t - (60 - k) as f32 * dt).max(0.0);
                overlay.update(speak_level(warm_t), dt);
            }
        }
        let level = if args.tts { speak_level(t) } else { 0.0 };
        overlay.update(level, dt);
        let transparent = overlay.render(t as f64);
        let out = if args.gif.is_some() || args.frames_dir.is_some() {
            on_background(&transparent, args.background)
        } else {
            transparent
        };
        frames.push(out);
    }

    let seg = overlay.segment();
    let b = overlay.orb_box();
    println!(
        "Overlay '{}' | segment x={:.0} y={:.0} w={:.0} h={:.0} | orb x={:.0} y={:.0} d={:.0}",
        args.preset, seg.x, seg.y, seg.w, seg.h, b.x, b.y, b.w
    );

    emit_outputs(args, frames)
}

fn run_orb(args: &Args) -> ExitCode {
    let mut view = OrbView::preset(&args.preset);
    view.configuration_mut().render_scale = args.scale.clamp(0.25, 1.0);

    println!(
        "Rendering orb '{}' at {}px ({} frame(s))...",
        args.preset,
        args.size,
        if args.time.is_some() {
            1
        } else {
            (args.seconds * args.fps as f32).ceil() as u32
        }
    );

    if let Some(time) = args.time {
        let frame = on_background(&view.render_at(args.size, time as f64), args.background);
        if let Some(path) = &args.png {
            if let Err(e) = write_png(&frame, path) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
            println!("Wrote {path}");
        }
        if args.ascii {
            print_ascii(&frame);
        }
        if args.png.is_none() && !args.ascii {
            let path = "orb.png";
            if let Err(e) = write_png(&frame, path) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
            println!("Wrote {path}");
        }
        return ExitCode::SUCCESS;
    }

    let frame_count = ((args.seconds * args.fps as f32).ceil() as usize).max(1);
    let mut frames = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let t = i as f64 / args.fps as f64;
        frames.push(on_background(
            &view.render_at(args.size, t),
            args.background,
        ));
    }

    emit_outputs(args, frames)
}

fn emit_outputs(args: &Args, frames: Vec<orb::Canvas>) -> ExitCode {
    if let Some(dir) = &args.frames_dir {
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!("error creating {dir}: {e}");
            return ExitCode::FAILURE;
        }
        for (i, frame) in frames.iter().enumerate() {
            let path = format!("{dir}/frame_{i:04}.png");
            if let Err(e) = write_png(frame, &path) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
        println!("Wrote {} frames to {dir}/", frames.len());
    }

    if let Some(path) = &args.gif {
        if let Err(e) = write_gif(&frames, path, args.fps) {
            eprintln!("error writing {path}: {e}");
            return ExitCode::FAILURE;
        }
        println!("Wrote {path}");
    }

    if args.ascii {
        if let Some(last) = frames.last() {
            print_ascii(last);
        }
    }

    if args.gif.is_none() && args.frames_dir.is_none() && !args.ascii {
        if let Some(first) = frames.first() {
            let path = args.png.as_deref().unwrap_or("orb.png");
            if let Err(e) = write_png(first, path) {
                eprintln!("error writing {path}: {e}");
                return ExitCode::FAILURE;
            }
            println!("Wrote {path}");
        }
    }

    if args.time.is_none() {
        println!("Done.");
    }
    ExitCode::SUCCESS
}

fn validate_preset(name: &str) {
    let known = orb::all_presets().iter().any(|(n, _)| *n == name);
    if !known {
        eprintln!("warning: unknown preset '{name}', using default");
    }
}

fn ensure_parent(path: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

fn write_png(frame: &orb::Canvas, path: &str) -> std::io::Result<()> {
    ensure_parent(path)?;
    let bytes = encode_png(frame).map_err(std::io::Error::other)?;
    fs::write(path, bytes)
}

fn write_gif(frames: &[orb::Canvas], path: &str, fps: u32) -> std::io::Result<()> {
    use orb::Canvas;
    if frames.is_empty() {
        return Ok(());
    }
    ensure_parent(path)?;
    let canvas: &Canvas = &frames[0];
    let (w, h) = (canvas.width as u16, canvas.height as u16);
    let file = std::fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let mut encoder = gif::Encoder::new(writer, w, h, &[]).map_err(std::io::Error::other)?;
    encoder
        .set_repeat(gif::Repeat::Infinite)
        .map_err(std::io::Error::other)?;
    let delay_cs = (100.0 / fps as f32).round().max(2.0) as u16;
    for frame in frames {
        let mut rgba = frame.to_rgba8();
        let mut gif_frame = gif::Frame::from_rgba_speed(w, h, &mut rgba, 10);
        gif_frame.delay = delay_cs;
        encoder
            .write_frame(&gif_frame)
            .map_err(std::io::Error::other)?;
    }
    Ok(())
}

fn print_ascii(frame: &orb::Canvas) {
    const RAMP: &[u8] = b" .:-=+*#%@";
    let cols = 64usize;
    let rows = 32usize;
    let mut out = String::new();
    for ry in 0..rows {
        for rx in 0..cols {
            let x = rx * frame.width / cols;
            let y = ry * frame.height / rows;
            let p = frame.get(x.min(frame.width - 1), y.min(frame.height - 1));
            let lum = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            let idx = (lum.clamp(0.0, 1.0) * (RAMP.len() - 1) as f32).round() as usize;
            out.push(RAMP[idx] as char);
        }
        out.push('\n');
    }
    print!("{out}");
}
