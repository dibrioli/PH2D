//! [`BrushSpec::default`] — the brush the app boots with, and the baseline every "byte-identical"
//! claim in the Painter is measured against.
//!
//! Split out of `spec.rs` (the workspace LOC cap). It earns its own file: this is not merely a `Default`
//! impl, it is the DEFINITION of the neutral brush — the one whose stroke must be indistinguishable
//! from a build in which Impasto, the watercolor optics and the material never existed. Every default
//! here is load-bearing, and several of them are load-bearing in a way a reader would not guess (the
//! neutral `impasto_roughness` is the geometric midpoint that reproduces the old hard-coded exponent,
//! to the float).

use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::falloff_curve::FalloffCurve;
use crate::height::{DepthSource, DrawTo};
use crate::spec::BrushSpec;
use crate::stroke_method::{JitterUnit, StrokeMethod};
use crate::symmetry::SymmetrySettings;
use crate::texture::TextureSettings;

impl Default for BrushSpec {
    /// A soft round black brush, matching Blender's default "TexDraw": smooth falloff, full
    /// strength/flow, 10% spacing.
    fn default() -> Self {
        Self {
            radius_px: 25.0,
            hardness: 0.0,
            strength: 1.0,
            flow: 1.0,
            spacing: 0.10,
            blend: BrushBlend::Mix,
            falloff: Falloff::Smooth,
            jitter: 0.0,
            color: [0.0, 0.0, 0.0],
            custom_falloff: FalloffCurve::default(),
            stroke_method: StrokeMethod::Space,
            space_attenuation: false, // Adjust Strength off by default (Enio 2026-06-24)
            accumulate: false,
            dash_ratio: 1.0,
            dash_samples: 20,
            jitter_unit: JitterUnit::Brush,
            jitter_absolute_px: 0.0,
            input_samples: 1,
            stabilizer: 0.5,
            airbrush_rate_s: 0.1,
            edge_to_edge: false,
            texture: TextureSettings::default(),
            grain_depth: 1.0,
            shape: TextureSettings::default(),
            dab_flatten: 0.0,
            dab_angle_deg: 0,
            color_jitter_enabled: false,
            color_jitter_hue: 0.0,
            color_jitter_sat: 0.0,
            color_jitter_val: 0.0,
            jitter_scale: 0.0,
            jitter_rotate: 0.0,
            jitter_spacing: 0.0,
            symmetry: SymmetrySettings::default(),
            // Watercolor: the `watercolor` gate (OFF) is what guarantees a byte-identical default
            // brush — so the params carry sensible *when-enabled* values, not neutral zeros, and
            // toggling "Wet edges" on shows an effect immediately.
            watercolor: false,
            edge_gain: 1.5,
            edge_spread: 7.0,
            granulation: 0.3,
            pigment: false,
            pigment_mix: 0.5,
            // Render-path optics (wet_edges defaults); inert unless `watercolor` is on.
            fill: 0.12,
            depth: 1.2,
            // Pigment body: lifts light-valued pigments so they deposit at their hue (not near-invisible
            // over white). Inert unless `watercolor` is on → a plain brush is byte-identical regardless.
            opacity: 0.4,
            warp: 6.0,
            wet_smudge: 0.0,   // off → byte-identical (the smear path is skipped)
            wet_rewet: 0.0,    // off → byte-identical (the rewet path is skipped)
            wet_charge: 1.0,   // full fresh paint → mixer skipped → byte-identical
            wet_dilution: 0.0, // full-strength deposit → byte-identical
            wet_pull: 0.0,     // no colour carry (inert unless charge < 1)
            // Paper slot inactive by default (the render-path falls back to its built-in paper noise);
            // granulation follows the paper's tooth until the artist points it at the Grain slot map.
            paper: TextureSettings::default(),
            granulation_use_paper: true,
            paper_depth: 1.0,
            watercolor_shape_auto: true, // built-in feather silhouette (byte-identical default)
            // Impasto: the `impasto` gate (OFF) is what guarantees the byte-identical default — the
            // params below carry sensible *when-enabled* values (a visible ridge the moment the
            // artist ticks the box), not neutral zeros. `impasto_off_is_byte_identical` locks that.
            impasto: false,
            // Enio's dialled-in defaults (2026-07-12, after the smoke): thick paint (Depth 1) whose
            // relief OBEYS the falloff (Body 0 — the rounded ridge he asked back for), settled soft
            // (Smoothing 1). They are the artist's numbers, not the engine's: the `impasto` gate below
            // is what keeps the brush byte-identical until he ticks the box.
            impasto_depth: 1.0,
            impasto_source: DepthSource::Uniform,
            impasto_draw_to: DrawTo::ColorAndDepth,
            impasto_smoothing: 1.0,
            impasto_body: 0.0,
            impasto_plow: 0.0, // o padrão do Smear é arrastar a COR e deixar o corpo onde está
            impasto_push: 0.0, // sem deslocamento: um traço empilha sobre o que já estava (byte-idêntico)
            // O material NEUTRO — o passe de luz de antes deste módulo, à risca. `roughness: 0.5` cai
            // EXATAMENTE no expoente 24 que estava cravado (a média geométrica de 6 e 96), então um
            // pincel default é byte-idêntico ao build pré-material. `shine: 0.7` era o default global.
            impasto_shine: 0.7,
            impasto_roughness: crate::material::Material::NEUTRAL.roughness,
            impasto_metallic: crate::material::Material::NEUTRAL.metallic,
            impasto_wax: crate::material::Material::NEUTRAL.wax,
        }
    }
}
