//! The per-`TextureKind` parameter declarations (labels + defaults) the panel exposes as sliders.
//! Split from `patterns.rs` (the samplers) for the LOC cap. The slot order here is the value order in
//! [`super::super::TextureSettings::params`]: slots `0`/`1` are the universal Contrast / Brightness
//! ([`super::apply_tone`]); slots `2..` are the kind's shape knobs, read by its sampler from
//! `&params[2..]`.

use crate::texture::TextureKind;

/// One tunable parameter a [`TextureKind`] exposes: its label (for the panel) and neutral default
/// (normalized `[0, 1]`). The slot index is the position in [`param_specs`] /
/// [`crate::texture::TextureSettings::params`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamSpec {
    /// English label shown next to the slider (HR-15).
    pub label: &'static str,
    /// Default value (normalized `[0, 1]`) assigned when the kind is selected.
    pub default: f32,
}

const CONTRAST: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.contrast",
    default: 0.5,
};
const BRIGHTNESS: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.brightness",
    default: 0.5,
};
const DETAIL: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.detail",
    default: 0.5,
};
const ROUGHNESS: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.roughness",
    default: 0.5,
};
const WARP: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.warp",
    default: 0.0,
};
const FREQUENCY: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.frequency",
    default: 0.35,
};
const SOFTNESS: ParamSpec = ParamSpec {
    label: "paint_brush.pattern_param.softness",
    default: 0.3,
};

/// The parameters a `kind` exposes, in slot order (see the module note). `None` exposes nothing.
#[must_use]
pub fn param_specs(kind: TextureKind) -> &'static [ParamSpec] {
    use TextureKind::*;
    /// Inline a kind-specific knob (`label`, `default`).
    macro_rules! p {
        ($l:literal, $d:expr) => {
            ParamSpec {
                label: $l,
                default: $d,
            }
        };
    }
    match kind {
        None => &[],
        Image => &[CONTRAST, BRIGHTNESS],
        // Fractal noise: octaves + persistence + domain warp.
        Noise => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.detail", 0.0),
            ROUGHNESS,
            WARP,
        ],
        Clouds | Grain => &[CONTRAST, BRIGHTNESS, DETAIL, ROUGHNESS, WARP],
        Stucci => &[
            CONTRAST,
            BRIGHTNESS,
            DETAIL,
            p!("paint_brush.pattern_param.depth", 0.5),
            WARP,
        ],
        Musgrave => &[
            CONTRAST,
            BRIGHTNESS,
            DETAIL,
            ROUGHNESS,
            p!("paint_brush.pattern_param.sharpness", 0.5),
        ],
        // Voronoi — the exemplar: cell jitter, smooth merge, distance metric, cells↔cracks.
        Voronoi => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.randomness", 1.0),
            p!("paint_brush.pattern_param.smoothness", 0.0),
            p!("paint_brush.pattern_param.metric", 0.0),
            p!("paint_brush.pattern_param.edges", 0.0),
        ],
        Marble => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.turbulence", 0.5),
            FREQUENCY,
        ],
        Wood => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.turbulence", 0.5),
            p!("paint_brush.pattern_param.rings", 0.35),
        ],
        DistortedNoise => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.distortion", 0.5),
            p!("paint_brush.pattern_param.detail", 0.0),
        ],
        Magic => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.distortion", 0.5),
            p!("paint_brush.pattern_param.complexity", 0.5),
        ],
        Checker => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.softness", 0.0),
        ],
        Stripes | Chevron => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.width", 0.5),
            FREQUENCY,
            SOFTNESS,
        ],
        Waves => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.width", 0.5),
            FREQUENCY,
            p!("paint_brush.pattern_param.ripple", 0.5),
        ],
        Gradient => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.curve", 0.5),
            p!("paint_brush.pattern_param.repeat", 0.0),
        ],
        Crosshatch | Grid => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.thickness", 0.4),
            FREQUENCY,
        ],
        Dots | Scales => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.radius", 0.5),
            p!("paint_brush.pattern_param.softness", 1.0),
            p!("paint_brush.pattern_param.randomness", 0.0),
        ],
        Bricks => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.gap", 0.4),
            p!("paint_brush.pattern_param.bond", 0.5),
        ],
        Weave => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.gap", 0.4),
            FREQUENCY,
        ],
        Hexagons => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.rim", 0.5),
            FREQUENCY,
        ],
        Diamonds | Triangles => &[
            CONTRAST,
            BRIGHTNESS,
            p!("paint_brush.pattern_param.softness", 0.0),
        ],
        // Watercolor papers: the preset (cold / rough / hot) IS the character; Contrast tunes the tooth
        // depth, Size x/y the scale + anisotropy, Angle the fibre orientation — so no redundant knobs.
        PaperCold | PaperRough | PaperHot => &[CONTRAST, BRIGHTNESS],
    }
}
