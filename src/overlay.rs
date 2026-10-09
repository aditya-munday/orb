use crate::canvas::Canvas;
use crate::orb::view::OrbView;
use crate::tts::TtsReactive;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Rect { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentPlacement {
    pub h_start: u32,

    pub h_end: u32,

    pub h_total: u32,

    pub v_start: u32,

    pub v_end: u32,

    pub v_total: u32,
}

impl SegmentPlacement {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        h_start: u32,
        h_end: u32,
        h_total: u32,
        v_start: u32,
        v_end: u32,
        v_total: u32,
    ) -> Self {
        SegmentPlacement {
            h_start,
            h_end,
            h_total,
            v_start,
            v_end,
            v_total,
        }
    }

    pub const fn right_corner() -> Self {
        SegmentPlacement::new(11, 12, 12, 2, 3, 7)
    }

    pub fn resolve(&self, screen_w: f32, screen_h: f32, margin: f32) -> Rect {
        let ht = self.h_total.max(1) as f32;
        let vt = self.v_total.max(1) as f32;
        let h0 = (self.h_start.saturating_sub(1)).min(self.h_total) as f32 / ht;
        let h1 = self.h_end.min(self.h_total) as f32 / ht;
        let v0 = (self.v_start.saturating_sub(1)).min(self.v_total) as f32 / vt;
        let v1 = self.v_end.min(self.v_total) as f32 / vt;
        let x = h0 * screen_w + margin;
        let y = v0 * screen_h + margin;
        let w = ((h1 - h0) * screen_w - 2.0 * margin).max(1.0);
        let h = ((v1 - v0) * screen_h - 2.0 * margin).max(1.0);
        Rect::new(x, y, w, h)
    }
}

#[derive(Clone, Debug)]
pub struct OverlaySettings {
    pub placement: SegmentPlacement,

    pub margin: f32,

    pub size_ratio: f32,

    pub base_size: Option<f32>,

    pub align_right: bool,

    pub align_top: bool,

    pub transparent_background: bool,

    pub max_pixels: usize,

    pub render_scale: f64,

    pub reactive: bool,

    pub breathe: f32,

    pub breathe_hz: f32,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        OverlaySettings {
            placement: SegmentPlacement::right_corner(),
            margin: 16.0,
            size_ratio: 0.86,
            base_size: None,
            align_right: true,
            align_top: true,
            transparent_background: true,
            max_pixels: 512,
            render_scale: 0.6,
            reactive: true,
            breathe: 0.025,
            breathe_hz: 0.4,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrbOverlay {
    pub view: OrbView,
    pub settings: OverlaySettings,
    pub tts: TtsReactive,
    screen_w: f32,
    screen_h: f32,
    segment: Rect,
    opacity: f32,
}

impl OrbOverlay {
    pub fn new(mut view: OrbView, settings: OverlaySettings) -> Self {
        view.configuration.render_scale = settings.render_scale.clamp(0.25, 1.0);
        let mut overlay = OrbOverlay {
            view,
            settings,
            tts: TtsReactive::default(),
            screen_w: 0.0,
            screen_h: 0.0,
            segment: Rect::new(0.0, 0.0, 1.0, 1.0),
            opacity: 1.0,
        };
        overlay.recompute();
        overlay
    }

    pub fn preset(name: &str) -> Self {
        OrbOverlay::new(OrbView::preset(name), OverlaySettings::default())
    }

    pub fn resize(&mut self, screen_w: f32, screen_h: f32) {
        self.screen_w = screen_w.max(0.0);
        self.screen_h = screen_h.max(0.0);
        self.recompute();
    }

    pub fn update(&mut self, level: f32, dt: f32) {
        self.tts.update(level, dt);
        if self.settings.reactive {
            self.view.set_speed(self.tts.speed());
            self.view.set_core_glow(self.tts.glow());
        }
        self.opacity = self.tts.opacity();
        self.recompute();
    }

    fn recompute(&mut self) {
        self.segment =
            self.settings
                .placement
                .resolve(self.screen_w, self.screen_h, self.settings.margin);
    }

    pub fn segment(&self) -> Rect {
        self.segment
    }

    pub fn diameter(&self) -> f32 {
        self.diameter_at(0.0)
    }

    pub fn diameter_at(&self, t: f64) -> f32 {
        let base = self.settings.base_size.unwrap_or_else(|| {
            self.segment.w.min(self.segment.h).max(1.0) * self.settings.size_ratio
        });
        let s = &self.settings;
        let breath = 1.0 + s.breathe * (t as f32 * s.breathe_hz * std::f32::consts::TAU).sin();
        (base * self.tts.scale() * breath).max(1.0)
    }

    pub fn orb_box(&self) -> Rect {
        self.orb_box_at(0.0)
    }

    pub fn orb_box_at(&self, t: f64) -> Rect {
        let d = self.diameter_at(t);
        let x = if self.settings.align_right {
            self.segment.right() - d
        } else {
            self.segment.center().0 - d / 2.0
        };
        let y = if self.settings.align_top {
            self.segment.y
        } else {
            self.segment.center().1 - d / 2.0
        };
        Rect::new(x, y, d, d)
    }

    fn render_orb(&self, t: f64) -> Canvas {
        let d = self.diameter_at(t);
        let res = (d.round() as usize).clamp(8, self.settings.max_pixels.max(8));
        let canvas = self.view.render_at(res, t);
        let out_px = d.round().max(1.0) as usize;
        if res == out_px {
            canvas
        } else {
            canvas.resized(out_px, out_px)
        }
    }

    pub fn render_region(&self, t: f64) -> (Rect, Canvas) {
        let mut layer = self.render_orb(t);
        if self.settings.reactive && self.opacity < 1.0 {
            layer = layer.scaled_alpha(self.opacity);
        }
        (self.orb_box_at(t), layer)
    }

    pub fn render(&self, t: f64) -> Canvas {
        let mut screen = Canvas::new(
            self.screen_w.round().max(0.0) as usize,
            self.screen_h.round().max(0.0) as usize,
        );
        let (box_, layer) = self.render_region(t);
        screen.composite_shifted(&layer, box_.x, box_.y, crate::canvas::BlendMode::Normal);
        screen
    }

    pub fn blend_into(&self, target: &mut Canvas, t: f64) {
        let (box_, layer) = self.render_region(t);
        target.composite_shifted(&layer, box_.x, box_.y, crate::canvas::BlendMode::Normal);
    }

    pub fn rgba8(&self, t: f64) -> Vec<u8> {
        self.render(t).to_rgba8()
    }

    pub fn rgba8_premultiplied(&self, t: f64) -> Vec<u8> {
        self.render(t).to_rgba8_premultiplied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_corner_resolves_to_expected_rect() {
        let p = SegmentPlacement::right_corner();
        let r = p.resolve(1200.0, 700.0, 0.0);
        assert!((r.x - 1000.0).abs() < 1e-2, "x {}", r.x);
        assert!((r.w - 200.0).abs() < 1e-2, "w {}", r.w);
        assert!((r.y - 100.0).abs() < 1e-3);
        assert!((r.h - 200.0).abs() < 1e-3);
    }

    #[test]
    fn segment_is_full_when_covering_all() {
        let p = SegmentPlacement::new(1, 12, 12, 1, 7, 7);
        let r = p.resolve(120.0, 70.0, 0.0);
        assert_eq!((r.x, r.y, r.w, r.h), (0.0, 0.0, 120.0, 70.0));
    }

    #[test]
    fn overlay_background_is_transparent() {
        let mut o = OrbOverlay::preset("default");
        o.resize(1200.0, 700.0);
        o.update(0.0, 1.0 / 60.0);
        let frame = o.render(0.0);
        assert_eq!(frame.get(0, 0)[3], 0.0);
        assert_eq!(frame.get(frame.width - 1, 0)[3], 0.0);
        assert_eq!(frame.get(0, frame.height - 1)[3], 0.0);
    }

    #[test]
    fn orb_lands_inside_segment_and_is_opaque_at_center() {
        let mut o = OrbOverlay::preset("default");
        o.resize(1200.0, 700.0);
        o.update(0.0, 1.0 / 60.0);
        let b = o.orb_box();
        let seg = o.segment();
        assert!(b.x >= seg.x - 2.0 && b.right() <= seg.right() + 2.0);
        assert!(b.y >= seg.y - 2.0 && b.bottom() <= seg.bottom() + 2.0);
        let (cx, cy) = b.center();
        let frame = o.render(0.0);
        let a = frame.get(cx as usize, cy as usize)[3];
        assert!(a > 0.9, "center alpha {a}");
    }

    #[test]
    fn tts_level_scales_orb_box_up() {
        let mut o = OrbOverlay::preset("default");
        o.resize(1200.0, 700.0);
        let quiet = o.orb_box();
        for _ in 0..120 {
            o.update(1.0, 1.0 / 60.0);
        }
        let loud = o.orb_box();
        assert!(loud.w > quiet.w, "{} vs {}", loud.w, quiet.w);
    }

    #[test]
    fn premultiplied_output_is_transparent_outside() {
        let mut o = OrbOverlay::preset("ocean");
        o.resize(800.0, 600.0);
        o.update(0.0, 1.0 / 60.0);
        let bytes = o.rgba8_premultiplied(0.0);
        assert_eq!(bytes.len(), 800 * 600 * 4);
        assert_eq!(bytes[3], 0);
        assert_eq!(bytes[0], 0);
    }

    #[test]
    fn blend_into_leaves_background_untouched() {
        let mut o = OrbOverlay::preset("fire");
        o.resize(600.0, 400.0);
        o.update(0.0, 1.0 / 60.0);
        let mut target = Canvas::new(600, 400);
        target.fill_circle(50.0, 350.0, 20.0, crate::color::Color::WHITE);
        let before = target.get(10, 10);
        o.blend_into(&mut target, 0.5);
        assert_eq!(target.get(10, 10), before);
        assert!(target.get(10, 10)[3] < 0.01);
    }
}
