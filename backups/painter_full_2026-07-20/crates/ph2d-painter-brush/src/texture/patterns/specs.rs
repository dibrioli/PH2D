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
    label: "Contrast",
    default: 0.5,
};
const BRIGHTNESS: ParamSpec = ParamSpec {
    label: "Brightness",
    default: 0.5,
};
const DETAIL: ParamSpec = ParamSpec {
    label: "Detail",
    default: 0.5,
};
const ROUGHNESS: ParamSpec = ParamSpec {
    label: "Roughness",
    default: 0.5,
};
const WARP: ParamSpec = ParamSpec {
    label: "Warp",
    default: 0.0,
};
const FREQUENCY: ParamSpec = ParamSpec {
    label: "Frequency",
    default: 0.35,
};
const SOFTNESS: ParamSpec = ParamSpec {
    label: "Softness",
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
        Noise => &[CONTRAST, BRIGHTNESS, p!("Detail", 0.0), ROUGHNESS, WARP],
        Clouds | Grain => &[CONTRAST, BRIGHTNESS, DETAIL, ROUGHNESS, WARP],
        Stucci => &[CONTRAST, BRIGHTNESS, DETAIL, p!("Depth", 0.5), WARP],
        Musgrave => &[
            CONTRAST,
            BRIGHTNESS,
            DETAIL,
            ROUGHNESS,
            p!("Sharpness", 0.5),
        ],
        // Voronoi — the exemplar: cell jitter, smooth merge, distance metric, cells↔cracks.
        Voronoi => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Randomness", 1.0),
            p!("Smoothness", 0.0),
            p!("Metric", 0.0),
            p!("Edges", 0.0),
        ],
        Marble => &[CONTRAST, BRIGHTNESS, p!("Turbulence", 0.5), FREQUENCY],
        Wood => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Turbulence", 0.5),
            p!("Rings", 0.35),
        ],
        DistortedNoise => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Distortion", 0.5),
            p!("Detail", 0.0),
        ],
        Magic => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Distortion", 0.5),
            p!("Complexity", 0.5),
        ],
        Checker => &[CONTRAST, BRIGHTNESS, p!("Softness", 0.0)],
        Stripes | Chevron => &[CONTRAST, BRIGHTNESS, p!("Width", 0.5), FREQUENCY, SOFTNESS],
        Waves => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Width", 0.5),
            FREQUENCY,
            p!("Ripple", 0.5),
        ],
        Gradient => &[CONTRAST, BRIGHTNESS, p!("Curve", 0.5), p!("Repeat", 0.0)],
        Crosshatch | Grid => &[CONTRAST, BRIGHTNESS, p!("Thickness", 0.4), FREQUENCY],
        Dots | Scales => &[
            CONTRAST,
            BRIGHTNESS,
            p!("Radius", 0.5),
            p!("Softness", 1.0),
            p!("Randomness", 0.0),
        ],
        Bricks => &[CONTRAST, BRIGHTNESS, p!("Gap", 0.4), p!("Bond", 0.5)],
        Weave => &[CONTRAST, BRIGHTNESS, p!("Gap", 0.4), FREQUENCY],
        Hexagons => &[CONTRAST, BRIGHTNESS, p!("Rim", 0.5), FREQUENCY],
        Diamonds | Triangles => &[CONTRAST, BRIGHTNESS, p!("Softness", 0.0)],
        // Watercolor papers: the preset (cold / rough / hot) IS the character; Contrast tunes the tooth
        // depth, Size x/y the scale + anisotropy, Angle the fibre orientation — so no redundant knobs.
        PaperCold | PaperRough | PaperHot => &[CONTRAST, BRIGHTNESS],
    }
}
