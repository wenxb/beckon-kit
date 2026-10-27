use gpui::{FontWeight, Hsla, Pixels, SharedString, px, rgb};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Appearance {
    Light,
    #[default]
    Dark,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Density {
    Compact,
    #[default]
    Comfortable,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Preferences {
    pub reduced_transparency: bool,
    pub increased_contrast: bool,
}

/// Shared frosted material for every opted-in component in this theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrostedMaterial {
    /// Gaussian sigma in logical pixels, in (0, 64].
    pub blur_sigma: Pixels,
    /// Surface tint opacity in [0, 1]; independent of the surface color's alpha.
    pub tint_opacity: f32,
}

impl FrostedMaterial {
    pub fn for_appearance(appearance: Appearance) -> Self {
        Self {
            blur_sigma: px(16.0),
            tint_opacity: match appearance {
                Appearance::Light => 0.95,
                Appearance::Dark => 0.91,
            },
        }
    }

    pub(crate) fn is_valid(self) -> bool {
        self.blur_sigma.as_f32().is_finite()
            && self.blur_sigma > px(0.0)
            && self.blur_sigma <= px(64.0)
            && self.tint_opacity.is_finite()
            && (0.0..=1.0).contains(&self.tint_opacity)
    }
}

/// Neutral structure and independent semantic states; no application brand color.
#[derive(Debug, Clone)]
pub struct Colors {
    pub canvas: Hsla,
    pub surface: Hsla,
    pub elevated: Hsla,
    pub text: Hsla,
    pub secondary: Hsla,
    pub disabled: Hsla,
    pub border: Hsla,
    /// Subtle outer window edge, independent of control boundaries and focus rings.
    pub window_border: Hsla,
    pub hover: Hsla,
    pub pressed: Hsla,
    pub primary: Hsla,
    pub on_primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_pressed: Hsla,
    pub info: Hsla,
    pub on_info: Hsla,
    pub info_hover: Hsla,
    pub info_pressed: Hsla,
    pub selection: Hsla,
    pub danger: Hsla,
    pub on_danger: Hsla,
    pub success: Hsla,
    pub on_success: Hsla,
    pub warning: Hsla,
    pub focus: Hsla,
    pub shadow: Hsla,
    pub highlight: Hsla,
}

impl Colors {
    pub fn for_appearance(appearance: Appearance) -> Self {
        match appearance {
            Appearance::Light => Self {
                canvas: rgb(0xf4f4f5).into(),
                surface: rgb(0xffffff).into(),
                elevated: rgb(0xf4f4f5).into(),
                text: rgb(0x18181b).into(),
                secondary: rgb(0x606069).into(),
                disabled: rgb(0x8e8e98).into(),
                border: rgb(0xdcdce1).into(),
                window_border: rgb(0xdcdce1).into(),
                hover: rgb(0xeaeaec).into(),
                pressed: rgb(0xdcdce1).into(),
                primary: rgb(0x202024).into(),
                on_primary: rgb(0xffffff).into(),
                primary_hover: rgb(0x36363d).into(),
                primary_pressed: rgb(0x09090b).into(),
                info: rgb(0x1d4ed8).into(),
                on_info: rgb(0xffffff).into(),
                info_hover: rgb(0x1e40af).into(),
                info_pressed: rgb(0x1e3a8a).into(),
                selection: rgb(0x1d4ed8).opacity(0.12).into(),
                danger: rgb(0xdc2626).into(),
                on_danger: rgb(0xffffff).into(),
                success: rgb(0x15803d).into(),
                on_success: rgb(0xffffff).into(),
                warning: rgb(0xa16207).into(),
                focus: rgb(0x1d4ed8).into(),
                shadow: rgb(0x09090b).opacity(0.08).into(),
                highlight: rgb(0xffffff).opacity(0.8).into(),
            },
            Appearance::Dark => Self {
                canvas: rgb(0x141416).into(),
                surface: rgb(0x1c1c1f).into(),
                elevated: rgb(0x28282d).into(),
                text: rgb(0xf4f4f5).into(),
                secondary: rgb(0xb0b0b8).into(),
                disabled: rgb(0x71717a).into(),
                border: rgb(0x3f3f46).into(),
                window_border: rgb(0x323236).into(),
                hover: rgb(0x323238).into(),
                pressed: rgb(0x3f3f46).into(),
                // Neutral primary reverses contrast, not hue, on dark surfaces.
                primary: rgb(0xe4e4e7).into(),
                on_primary: rgb(0x18181b).into(),
                primary_hover: rgb(0xf4f4f5).into(),
                primary_pressed: rgb(0xc4c4cc).into(),
                info: rgb(0x60a5fa).into(),
                on_info: rgb(0x101f36).into(),
                info_hover: rgb(0x93c5fd).into(),
                info_pressed: rgb(0x3b82f6).into(),
                selection: rgb(0x60a5fa).opacity(0.12).into(),
                danger: rgb(0xf87171).into(),
                on_danger: rgb(0x300a0e).into(),
                success: rgb(0x4ade80).into(),
                on_success: rgb(0x092514).into(),
                warning: rgb(0xfacc15).into(),
                focus: rgb(0x60a5fa).into(),
                shadow: rgb(0x000000).opacity(0.28).into(),
                highlight: rgb(0xffffff).opacity(0.10).into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub unit: f32,
    pub control_height: f32,
    pub control_padding: f32,
    pub control_radius: f32,
    pub surface_radius: f32,
    pub surface_padding: f32,
    pub focus_width: f32,
}

impl Metrics {
    pub fn for_density(density: Density) -> Self {
        Self {
            unit: 4.0,
            control_height: match density {
                Density::Compact => 28.0,
                Density::Comfortable => 34.0,
            },
            control_padding: 12.0,
            control_radius: 6.0,
            surface_radius: 12.0,
            surface_padding: 20.0,
            focus_width: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Typography {
    pub family: SharedString,
    pub body: f32,
    pub caption: f32,
    pub title: f32,
    pub line_height: f32,
    pub control_weight: FontWeight,
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            family: ".SystemUIFont".into(),
            body: 13.0,
            caption: 11.0,
            title: 22.0,
            line_height: 1.5,
            control_weight: FontWeight::MEDIUM,
        }
    }
}
