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

/// **O parâmetro `slot` de `kind` está INERTE com estes `params`?** — uma lei por agora: o *Roughness*
/// é a persistência ENTRE oitavas, e com o *Detail* numa oitava só (`octaves_from`) não há entre. O
/// Noise nasce assim (Detail `0`). O painel esmaece a linha quando ela responde `true` (doc 46 §2-3).
#[must_use]
pub fn param_inerte(kind: TextureKind, params: &[f32], slot: usize) -> bool {
    let specs = param_specs(kind);
    let Some(s) = specs.get(slot) else {
        return false;
    };
    let Some(detail) = specs.iter().position(|p| p.label == DETAIL.label) else {
        return false;
    };
    s.label == ROUGHNESS.label
        && params
            .get(detail)
            .is_some_and(|&d| super::octaves_from(d) == 1)
}

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
        // The engine's bristle knobs, no tone slots (Strength is the level): the slider order is
        // `ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS`; defaults = the Wet Round tile at the oil presets'
        // Strength 2 — at 1 its peaks (0,16–0,32) sit under the Impasto film tail (`height_film::W_TAIL`
        // 0,35) and the Impasto stroke is EMPTY (measured: 0 · Strength 1,5 → 43 688 · 2 → 192 835).
        Bristles => &[
            p!("paint_brush.pattern_param.bristle_count", 0.5),
            p!("paint_brush.pattern_param.bristle_strength", 0.5),
            p!("paint_brush.pattern_param.bristle_size", 36.0 / 76.0),
            p!("paint_brush.pattern_param.felt", 0.15),
        ],
    }
}

/// The scale a pattern parameter is SHOWN and TYPED in by the panel. Stored values stay normalized
/// `[0, 1]`; [`Escala::UNIT`] (the default) shows them as they are. The Bristles show the Wet Paint
/// Tuning's own units (Count 2000, Felt 0,030), range and step read from the engine's knob table — the
/// same numbers the Tuning showed, so moving a knob into the Shape loses nothing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escala {
    /// The value shown at `0`.
    pub min: f32,
    /// The value shown at `1`.
    pub max: f32,
    /// The stepper / scrub step, in shown units.
    pub step: f64,
}

impl Escala {
    /// The normalized track itself (`0..1`, step `0.01`).
    pub const UNIT: Self = Self {
        min: 0.0,
        max: 1.0,
        step: 0.01,
    };

    /// Decimals the shown value needs at this step (`10` → 0, `0.05` → 2, `0.001` → 3).
    #[must_use]
    pub fn decimals(self) -> usize {
        let mut d = 0;
        let mut s = self.step;
        while s < 0.999_999 && d < 6 {
            s *= 10.0;
            d += 1;
        }
        d
    }

    /// The shown value of the stored `u`.
    #[must_use]
    pub fn mostra(self, u: f32) -> f32 {
        self.min + u.clamp(0.0, 1.0) * (self.max - self.min)
    }

    /// The stored `[0, 1]` value of a typed/scrubbed `v` (clamped to the range).
    #[must_use]
    pub fn guarda(self, v: f32) -> f32 {
        ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

/// The [`Escala`] of `kind`'s parameter `slot`.
#[must_use]
pub fn param_escala(kind: TextureKind, slot: usize) -> Escala {
    use ph2d_wet_paint::brush::BRISTLE_UNIT_KNOBS;
    use ph2d_wet_paint::tuning::KNOB_DEFS;
    match (kind, BRISTLE_UNIT_KNOBS.get(slot)) {
        (TextureKind::Bristles, Some(&k)) => {
            let d = &KNOB_DEFS[k as usize];
            Escala {
                min: d.min as f32,
                max: d.max as f32,
                step: d.step,
            }
        }
        _ => Escala::UNIT,
    }
}
