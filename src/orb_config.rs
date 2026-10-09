use crate::color::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct OrbConfiguration {
    pub background_colors: Vec<Color>,
    pub glow_color: Color,
    pub particle_color: Color,
    pub core_glow_intensity: f64,
    pub show_background: bool,
    pub show_wavy_blobs: bool,
    pub show_particles: bool,
    pub show_glow_effects: bool,
    pub show_shadow: bool,
    pub speed: f64,
    pub render_scale: f64,
}

impl OrbConfiguration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        background_colors: Vec<Color>,
        glow_color: Color,
        particle_color: Color,
        core_glow_intensity: f64,
        show_background: bool,
        show_wavy_blobs: bool,
        show_particles: bool,
        show_glow_effects: bool,
        show_shadow: bool,
        speed: f64,
    ) -> Self {
        OrbConfiguration {
            background_colors,
            glow_color,
            particle_color,
            core_glow_intensity,
            show_background,
            show_wavy_blobs,
            show_particles,
            show_glow_effects,
            show_shadow,
            speed,
            render_scale: 1.0,
        }
    }

    pub fn background_colors(mut self, colors: Vec<Color>) -> Self {
        self.background_colors = colors;
        self
    }

    pub fn glow_color(mut self, color: Color) -> Self {
        self.glow_color = color;
        self
    }

    pub fn particle_color(mut self, color: Color) -> Self {
        self.particle_color = color;
        self
    }

    pub fn core_glow_intensity(mut self, v: f64) -> Self {
        self.core_glow_intensity = v;
        self
    }

    pub fn show_background(mut self, v: bool) -> Self {
        self.show_background = v;
        self
    }

    pub fn show_wavy_blobs(mut self, v: bool) -> Self {
        self.show_wavy_blobs = v;
        self
    }

    pub fn show_particles(mut self, v: bool) -> Self {
        self.show_particles = v;
        self
    }

    pub fn show_glow_effects(mut self, v: bool) -> Self {
        self.show_glow_effects = v;
        self
    }

    pub fn show_shadow(mut self, v: bool) -> Self {
        self.show_shadow = v;
        self
    }

    pub fn speed(mut self, v: f64) -> Self {
        self.speed = v;
        self
    }

    pub fn render_scale(mut self, v: f64) -> Self {
        self.render_scale = v.clamp(0.25, 1.0);
        self
    }
}

impl Default for OrbConfiguration {
    fn default() -> Self {
        OrbConfiguration {
            background_colors: vec![Color::GREEN, Color::BLUE, Color::PINK],
            glow_color: Color::WHITE,
            particle_color: Color::WHITE,
            core_glow_intensity: 1.0,
            show_background: true,
            show_wavy_blobs: true,
            show_particles: true,
            show_glow_effects: true,
            show_shadow: true,
            speed: 60.0,
            render_scale: 1.0,
        }
    }
}

impl OrbConfiguration {
    pub fn mystic() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::PURPLE, Color::BLUE, Color::INDIGO])
            .glow_color(Color::PURPLE)
            .core_glow_intensity(1.2)
    }

    pub fn nature() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::GREEN, Color::MINT, Color::TEAL])
            .glow_color(Color::GREEN)
            .speed(45.0)
    }

    pub fn sunset() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::ORANGE, Color::RED, Color::PINK])
            .glow_color(Color::ORANGE)
            .core_glow_intensity(0.8)
    }

    pub fn ocean() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::BLUE, Color::CYAN, Color::TEAL])
            .glow_color(Color::CYAN)
            .speed(75.0)
    }

    pub fn minimal() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::GRAY, Color::WHITE])
            .glow_color(Color::WHITE)
            .show_wavy_blobs(false)
            .show_particles(false)
            .speed(30.0)
    }

    pub fn cosmic() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::PURPLE, Color::PINK, Color::BLUE])
            .glow_color(Color::WHITE)
            .core_glow_intensity(1.5)
            .speed(90.0)
    }

    pub fn fire() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::RED, Color::ORANGE, Color::YELLOW])
            .glow_color(Color::ORANGE)
            .core_glow_intensity(1.3)
            .speed(80.0)
    }

    pub fn arctic() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::CYAN, Color::WHITE, Color::BLUE])
            .glow_color(Color::WHITE)
            .core_glow_intensity(0.75)
            .show_particles(true)
            .speed(40.0)
    }

    pub fn shadow() -> Self {
        OrbConfiguration::default()
            .background_colors(vec![Color::BLACK, Color::GRAY])
            .glow_color(Color::GRAY)
            .core_glow_intensity(0.7)
            .show_particles(false)
    }
}

pub fn all_presets() -> Vec<(&'static str, OrbConfiguration)> {
    vec![
        ("default", OrbConfiguration::default()),
        ("mystic", OrbConfiguration::mystic()),
        ("nature", OrbConfiguration::nature()),
        ("sunset", OrbConfiguration::sunset()),
        ("ocean", OrbConfiguration::ocean()),
        ("minimal", OrbConfiguration::minimal()),
        ("cosmic", OrbConfiguration::cosmic()),
        ("fire", OrbConfiguration::fire()),
        ("arctic", OrbConfiguration::arctic()),
        ("shadow", OrbConfiguration::shadow()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_configuration_is_correct() {
        let c = OrbConfiguration::default();
        assert_eq!(
            c.background_colors,
            vec![Color::GREEN, Color::BLUE, Color::PINK]
        );
        assert_eq!(c.glow_color, Color::WHITE);
        assert_eq!(c.particle_color, Color::WHITE);
        assert_eq!(c.core_glow_intensity, 1.0);
        assert!(c.show_background && c.show_wavy_blobs && c.show_particles);
        assert!(c.show_glow_effects && c.show_shadow);
        assert_eq!(c.speed, 60.0);
    }

    #[test]
    fn all_presets_are_constructible() {
        let presets = all_presets();
        assert_eq!(presets.len(), 10);
        assert!(presets.iter().all(|(_, c)| c.speed > 0.0));
    }
}
