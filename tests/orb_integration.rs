use orb::{
    all_presets, encode_png, on_background, Canvas, Color, OrbConfiguration, OrbOverlay, OrbView,
    OverlaySettings, SegmentPlacement, TtsReactive,
};

#[test]
fn every_preset_renders_a_valid_circle() {
    for (name, cfg) in all_presets() {
        let view = OrbView::new(cfg);
        let c = view.render_at(72, 0.4);
        let center = c.get(36, 36);
        assert!(center[3] > 0.85, "preset {name}: center not opaque");

        assert!(c.get(1, 1)[3] < 0.15, "preset {name}: corner too opaque");
    }
}

#[test]
fn animation_is_continuous_and_changes() {
    let view = OrbView::default();
    let a = view.render_at(48, 0.0);
    let b = view.render_at(48, 0.25);
    let c = view.render_at(48, 0.5);
    assert_ne!(a.pixels, b.pixels);
    assert_ne!(b.pixels, c.pixels);

    assert_eq!(a.pixels, view.render_at(48, 0.0).pixels);
}

#[test]
fn size_scales_output() {
    let view = OrbView::ocean();
    let small = view.render_at(32, 0.1);
    let large = view.render_at(128, 0.1);
    assert_eq!((small.width, small.height), (32, 32));
    assert_eq!((large.width, large.height), (128, 128));
}

#[test]
fn toggles_isolate_layers() {
    let bg_only = OrbView::new(
        OrbConfiguration::default()
            .show_wavy_blobs(false)
            .show_particles(false)
            .show_glow_effects(false)
            .show_shadow(false),
    );
    let c = bg_only.render_at(64, 0.0);
    let p = c.get(32, 32);
    assert!(p[3] > 0.9);

    assert!(p[2] >= p[0], "expected blue channel dominant, got {p:?}");
}

#[test]
fn png_and_background_helpers() {
    let view = OrbView::fire();
    let frame = view.render_at(64, 0.3);
    let png = encode_png(&frame).unwrap();
    assert!(png.len() > 100);
    assert_eq!(&png[1..4], b"PNG");

    let opaque = on_background(&frame, Color::BLACK);
    assert!(opaque.pixels.iter().all(|p| (p[3] - 1.0).abs() < 1e-6));
}

#[test]
fn custom_configuration_is_respected() {
    let view = OrbView::new(
        OrbConfiguration::default()
            .background_colors(vec![Color::rgb8(10, 20, 30), Color::rgb8(40, 50, 60)])
            .glow_color(Color::CYAN)
            .speed(120.0),
    );
    let c = view.render_at(48, 1.0);
    assert_eq!((c.width, c.height), (48, 48));
}

#[test]
fn canvas_supports_public_construction() {
    let mut c = Canvas::new(8, 8);
    c.fill_circle(4.0, 4.0, 3.0, Color::WHITE);
    assert!(c.get(4, 4)[3] > 0.9);
}

#[test]
fn overlay_is_transparent_and_anchored_right() {
    let mut o = OrbOverlay::new(OrbView::preset("default"), OverlaySettings::default());
    o.resize(1200.0, 700.0);
    o.update(0.0, 1.0 / 60.0);
    let frame = o.render(0.0);
    assert_eq!((frame.width, frame.height), (1200, 700));
    assert_eq!(frame.get(0, 0)[3], 0.0);
    assert_eq!(frame.get(1199, 699)[3], 0.0);
    let b = o.orb_box();
    assert!(b.right() <= o.segment().right() + 1.0);
    assert!(b.x > 1200.0 * 10.0 / 12.0);
}

#[test]
fn overlay_tts_loudness_grows_the_orb() {
    let mut o = OrbOverlay::new(OrbView::preset("ocean"), OverlaySettings::default());
    o.resize(1200.0, 700.0);
    o.update(0.0, 1.0 / 60.0);
    let quiet = o.diameter();
    for _ in 0..120 {
        o.update(1.0, 1.0 / 60.0);
    }
    let loud = o.diameter();
    assert!(loud > quiet * 1.1, "quiet {quiet} loud {loud}");
}

#[test]
fn overlay_blend_into_preserves_host_pixels() {
    let mut o = OrbOverlay::new(OrbView::preset("mystic"), OverlaySettings::default());
    o.resize(600.0, 400.0);
    o.update(0.0, 1.0 / 60.0);
    let mut host = Canvas::new(600, 400);
    let before = host.get(300, 380);
    o.blend_into(&mut host, 0.2);
    assert_eq!(host.get(300, 380), before);
}

#[test]
fn tts_reactive_is_deterministic_and_monotonic() {
    let mut r = TtsReactive::default();
    let mut prev = 0.0;
    for _ in 0..30 {
        r.update(0.8, 1.0 / 60.0);
        assert!(r.level() >= prev - 1e-6);
        prev = r.level();
    }
    assert!(r.speed() > 1.0);
    assert!(r.scale() > 1.0);
}

#[test]
fn segment_placement_covers_requested_segments() {
    let p = SegmentPlacement::right_corner();
    let r = p.resolve(1200.0, 700.0, 0.0);
    assert!((r.width_ratio(1200.0) - 2.0 / 12.0).abs() < 1e-4);
    assert!((r.height_ratio(700.0) - 2.0 / 7.0).abs() < 1e-4);
}

trait RectRatio {
    fn width_ratio(&self, screen_w: f32) -> f32;
    fn height_ratio(&self, screen_h: f32) -> f32;
}

impl RectRatio for orb::Rect {
    fn width_ratio(&self, screen_w: f32) -> f32 {
        self.w / screen_w
    }
    fn height_ratio(&self, screen_h: f32) -> f32 {
        self.h / screen_h
    }
}
