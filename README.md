# Orb

A customizable animated orb written in pure Rust, with a transparent overlay
for AI agent apps.

Orb renders a circular, glowing, animated object: a gradient body, rotating
glow layers, wobbling organic blobs, drifting particles, a bright core and a
glassy rim. It masks the result to a perfect circle and puts a layered drop
shadow behind it. There are no GUI or platform dependencies. Rendering happens
on the CPU into an RGBA frame buffer, so it runs headless anywhere Rust runs.

## Details

- Renders an animated orb at any size, for any point in time.
- Ten built-in presets: `default`, `mystic`, `nature`, `sunset`, `ocean`,
  `minimal`, `cosmic`, `fire`, `arctic`, `shadow`.
- Fully configurable through `OrbConfiguration`: gradient colors, glow and
  particle colors, core glow intensity, animation speed, per-layer on/off
  toggles and an internal render scale.
- Exports PNG stills, animated GIFs and an ASCII terminal preview.
- Optional live preview in a native window (`viewer` feature).
- Provides a transparent, segment-anchored overlay that grows and shrinks with
  speech / text-to-speech loudness, for AI agent apps. It only produces pixels,
  so the host app owns the platform overlay layer.
- Pure Rust, no Swift or macOS code; MIT licensed.

## Requirements

- Rust 1.80 or newer (edition 2021).
- No system libraries for the default build.
- The optional `viewer` feature needs a display server (X11 or Wayland on
  Linux).

Install Rust from <https://rustup.rs> if you do not have it.

## Run it locally

Clone the repository and run these commands from its root. All commands use
Cargo.

Render a single PNG:

```bash
cargo run --bin orb-demo -- --preset mystic --size 512 --time 0.7 --png orb.png
```

Render an animated GIF:

```bash
cargo run --bin orb-demo -- --preset cosmic --size 400 --seconds 4 --fps 20 --gif orb.gif
```

Print an ASCII preview in the terminal:

```bash
cargo run --bin orb-demo -- --preset fire --ascii --time 1.2
```

Render the transparent overlay with simulated speech loudness:

```bash
cargo run --bin orb-demo -- --overlay --tts --screen 1200x700 --preset cosmic --seconds 5 --fps 20 --gif overlay.gif
```

Lower the internal render scale for faster, real-time-friendly output:

```bash
cargo run --bin orb-demo -- --preset cosmic --size 256 --scale 0.6 --png orb.png
```

List the available presets:

```bash
cargo run --bin orb-demo -- --list
```

Show every command-line option:

```bash
cargo run --bin orb-demo -- --help
```

For faster output, build in release mode first:

```bash
cargo build --release
./target/release/orb-demo --preset ocean --size 512 --time 1.0 --png orb.png
```

Run the live animated window (needs a display). Press `1` through `0` to switch
presets and `Esc` to quit:

```bash
cargo run --features viewer --bin orb-viewer -- --preset ocean --size 420
```

Run the tests:

```bash
cargo test
```

## License

MIT. See [LICENSE](LICENSE).
