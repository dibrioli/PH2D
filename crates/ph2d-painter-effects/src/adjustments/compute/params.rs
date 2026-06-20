//! UI param plumbing — the generic slider / toggle / segment accessors the layers
//! panel renders (and their inverse setters) + the UI↔param range constants. The
//! per-family bespoke editors (Curves / Gradient Map / Channel Mixer / Selective
//! Color) live with their families; this is the shared rack. Split out of the
//! former monolithic `compute.rs` (pure move).

use super::*;

/// The slider-editable params of an adjustment, as `(label, value01)` in slot
/// order — what the layers panel renders as a labeled slider per slot, and the
/// inverse of [`set_adjustment_slider_param`]. Kinds with bespoke controls
/// (Curves, Gradient Map, …) return empty here and get their own UI later.
#[must_use]
pub fn adjustment_slider_params(params: &AdjustmentParams) -> Vec<(&'static str, f32)> {
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => vec![
            ("Hue", p.h.clamp(0.0, 1.0)),
            ("Sat", (p.s + 1.0) * 0.5),
            ("Bright", (p.b + 1.0) * 0.5),
        ],
        AdjustmentParams::BrightnessContrast(p) => vec![
            ("Bright", (p.brightness + 1.0) * 0.5),
            ("Contrast", (p.contrast + 1.0) * 0.5),
        ],
        // Exposure: EV -4..4, Offset -0.5..0.5, Gamma -0.9..0.9 (all centered).
        AdjustmentParams::Exposure(p) => vec![
            ("Expo", (p.exposure_ev + 4.0) / 8.0),
            ("Offset", p.offset + 0.5),
            ("Gamma", (p.gamma_correction + 0.9) / 1.8),
        ],
        AdjustmentParams::Vibrance(p) => vec![
            ("Vib", (p.vibrance + 1.0) * 0.5),
            ("Sat", (p.saturation + 1.0) * 0.5),
        ],
        // Posterize levels 2..=32; Threshold cutoff 0..=255.
        AdjustmentParams::Posterize(p) => {
            vec![("Levels", (p.levels.clamp(2, 32) as f32 - 2.0) / 30.0)]
        }
        AdjustmentParams::Threshold(p) => vec![("Level", p.threshold as f32 / 255.0)],
        // Photo Filter: Temperature -1..1 (centered; 0.5 = neutral) + Density 0..1.
        // `preserve_luminosity` is a toggle (see `adjustment_toggle_params`).
        AdjustmentParams::PhotoFilter(p) => vec![
            ("Temp", (p.temperature.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("Density", p.density.clamp(0.0, 1.0)),
        ],
        // Color Balance: the 3 bipolar color-axis shifts (centered; 0.5 = neutral).
        // `scope` is a segment (see `adjustment_segment_params`) and
        // `preserve_luminosity` a toggle (see `adjustment_toggle_params`).
        AdjustmentParams::ColorBalance(p) => vec![
            ("C/R", (p.cyan_red.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("M/G", (p.magenta_green.clamp(-1.0, 1.0) + 1.0) * 0.5),
            ("Y/B", (p.yellow_blue.clamp(-1.0, 1.0) + 1.0) * 0.5),
        ],
        // Black & White: 6 per-hue weights (`-2..3 → 0..1`); the Tint toggle adds
        // the Hue + Tint-amount sliders (slots 6/7) only while a tint is set.
        AdjustmentParams::BlackAndWhite(p) => {
            let weight = |v: f32| (v.clamp(-2.0, 3.0) + 2.0) / 5.0;
            let mut v = vec![
                ("Reds", weight(p.reds)),
                ("Yellows", weight(p.yellows)),
                ("Greens", weight(p.greens)),
                ("Cyans", weight(p.cyans)),
                ("Blues", weight(p.blues)),
                ("Magentas", weight(p.magentas)),
            ];
            if let Some(t) = p.tint_color {
                v.push(("Hue", t.h.rem_euclid(360.0) / 360.0));
                v.push(("Tint", p.tint_amount.clamp(0.0, 1.0)));
            }
            v
        }
        // Gradient Map has a bespoke N-stop editor (preview bar + draggable stops
        // + the SELECTED stop's RGB sliders, see `gradient_stop_color_params`), so
        // it exposes no generic slider rack here.
        // Levels maps cleanly onto the generic slider rack (5 ≤ 6 slots): input
        // black/gamma/white + output black/white. (Curves needs the bespoke
        // curve canvas — handoff §4 — so it returns no generic sliders.)
        AdjustmentParams::Levels(p) => vec![
            ("Black", p.black_point.clamp(0.0, 1.0)),
            ("Gamma", levels_gamma_to_slider(p.gamma)),
            ("White", p.white_point.clamp(0.0, 1.0)),
            ("Out Lo", p.output_black.clamp(0.0, 1.0)),
            ("Out Hi", p.output_white.clamp(0.0, 1.0)),
        ],
        // W4 spatial mesh — the GPU pass-graph kinds. Radius/Distance 0..SPATIAL_PX,
        // Angle a full turn, Sharpen amount 0..2 (unsharp coef). Ranges are the
        // inverse of `set_adjustment_slider_param`.
        AdjustmentParams::GaussianBlur(p) => {
            vec![("Radius", (p.radius / SPATIAL_PX_MAX).clamp(0.0, 1.0))]
        }
        AdjustmentParams::MotionBlur(p) => vec![
            ("Distance", (p.distance / SPATIAL_PX_MAX).clamp(0.0, 1.0)),
            ("Angle", angle_to_slider(p.angle)),
        ],
        AdjustmentParams::Sharpen(p) => vec![
            ("Amount", (p.amount / SHARPEN_AMOUNT_MAX).clamp(0.0, 1.0)),
            ("Radius", (p.radius / SHARPEN_RADIUS_MAX).clamp(0.0, 1.0)),
        ],
        // Chromatic Aberration: 3 bipolar per-channel shifts (centered; 0.5 =
        // none). `falloff_center` is RESERVED (no slider — linear-radial model).
        AdjustmentParams::ChromaticAberration(p) => vec![
            ("Red", shift_to_slider(p.red_shift)),
            ("Green", shift_to_slider(p.green_shift)),
            ("Blue", shift_to_slider(p.blue_shift)),
        ],
        // Noise amount 0..1; the distribution (`kind`) is a segment + `mono` a
        // toggle (see the respective accessors).
        AdjustmentParams::Noise(p) => vec![("Amount", p.amount.clamp(0.0, 1.0))],
        // Halftone dot size 1..HALFTONE_DOT_MAX px + screen angle; the cell shape
        // is a segment.
        AdjustmentParams::Halftone(p) => vec![
            (
                "Dot Size",
                ((p.dot_size - 1.0) / (HALFTONE_DOT_MAX - 1.0)).clamp(0.0, 1.0),
            ),
            ("Angle", angle_to_slider(p.angle)),
        ],
        // Color Lookup: scrub the built-in look (handle = preset index, quantized
        // on the preset grid) + the intensity. A named popover is a UI follow-up.
        AdjustmentParams::ColorLookupLut(p) => vec![
            ("Look", preset_to_slider(p.lut_3d.0)),
            ("Amount", p.intensity.clamp(0.0, 1.0)),
        ],
        // Bloom: bright-pass threshold + glow intensity (0..2) + blur radius +
        // soft-knee falloff.
        AdjustmentParams::Bloom(p) => vec![
            ("Threshold", p.threshold.clamp(0.0, 1.0)),
            (
                "Intensity",
                (p.intensity / BLOOM_INTENSITY_MAX).clamp(0.0, 1.0),
            ),
            ("Radius", (p.radius / SPATIAL_PX_MAX).clamp(0.0, 1.0)),
            ("Falloff", p.falloff.clamp(0.0, 1.0)),
        ],
        // Shadows/Highlights: 8 params — shadows (amount/width/radius), highlights
        // (amount/width/radius), color correction (bipolar), midtone contrast
        // (bipolar). Fills the 8-slot generic rack exactly.
        AdjustmentParams::ShadowsHighlights(p) => vec![
            ("Shad Amt", p.shadows_amount.clamp(0.0, 1.0)),
            ("Shad Wid", p.shadows_tonal_width.clamp(0.0, 1.0)),
            (
                "Shad Rad",
                (p.shadows_radius / SPATIAL_PX_MAX).clamp(0.0, 1.0),
            ),
            ("High Amt", p.highlights_amount.clamp(0.0, 1.0)),
            ("High Wid", p.highlights_tonal_width.clamp(0.0, 1.0)),
            (
                "High Rad",
                (p.highlights_radius / SPATIAL_PX_MAX).clamp(0.0, 1.0),
            ),
            ("Color", (p.color_correction.clamp(-1.0, 1.0) + 1.0) * 0.5),
            (
                "Contrast",
                (p.midtone_contrast.clamp(-1.0, 1.0) + 1.0) * 0.5,
            ),
        ],
        _ => Vec::new(),
    }
}

/// Max bloom glow intensity exposed on the slider (the additive-glow multiplier).
const BLOOM_INTENSITY_MAX: f32 = 2.0;

/// Built-in Color Lookup preset index → `0..1` slider (quantized on the preset
/// grid; index 0 = None at the far left).
#[inline]
fn preset_to_slider(handle: u64) -> f32 {
    let last = (super::lut::LUT_PRESET_COUNT - 1).max(1) as f32;
    (handle.min(super::lut::LUT_PRESET_COUNT as u64 - 1) as f32 / last).clamp(0.0, 1.0)
}

/// `0..1` slider → built-in Color Lookup preset index (inverse of
/// [`preset_to_slider`]; snaps to the nearest preset).
#[inline]
fn slider_to_preset(v: f32) -> u64 {
    let last = (super::lut::LUT_PRESET_COUNT - 1) as f32;
    (v.clamp(0.0, 1.0) * last).round() as u64
}

// ── W4 spatial/coordinate slider ranges (UI ↔ param mapping, single source) ──
//
// The panel slider thumb is a normalized `0..1`; these constants are the physical
// extents `adjustment_slider_params` / `set_adjustment_slider_param` map to/from.

/// Max blur radius / motion distance exposed on the slider (px). The kernel cap
/// (`MAX_BLUR_HALF = 256`) is far past any interactive use; 100 px is a generous
/// editable range with a usable thumb resolution.
const SPATIAL_PX_MAX: f32 = 100.0;
/// Max unsharp-mask amount (the `base + amount·(base−blur)` coefficient).
const SHARPEN_AMOUNT_MAX: f32 = 2.0;
/// Max sharpen blur radius (px) — sharpening uses a small support.
const SHARPEN_RADIUS_MAX: f32 = 20.0;
/// Half-range of a chromatic-aberration per-channel shift (px at the canvas
/// corner); the slider is bipolar `−MAX..+MAX` (0.5 = no shift).
const CHROMA_SHIFT_MAX: f32 = 10.0;
/// Max halftone cell size (px); the minimum is 1 px (a single-pixel screen).
const HALFTONE_DOT_MAX: f32 = 32.0;

/// Radians angle → `0..1` slider (one full turn).
#[inline]
fn angle_to_slider(angle: f32) -> f32 {
    angle.rem_euclid(std::f32::consts::TAU) / std::f32::consts::TAU
}

/// `0..1` slider → radians (inverse of [`angle_to_slider`]).
#[inline]
fn slider_to_angle(v: f32) -> f32 {
    v * std::f32::consts::TAU
}

/// Bipolar px shift `−MAX..+MAX` → `0..1` slider (0.5 = no shift).
#[inline]
fn shift_to_slider(shift: f32) -> f32 {
    ((shift / CHROMA_SHIFT_MAX).clamp(-1.0, 1.0) + 1.0) * 0.5
}

/// `0..1` slider → bipolar px shift (inverse of [`shift_to_slider`]).
#[inline]
fn slider_to_shift(v: f32) -> f32 {
    (v * 2.0 - 1.0) * CHROMA_SHIFT_MAX
}

/// Set slider `slot` of an adjustment from a normalized `0..1` value (inverse of
/// [`adjustment_slider_params`]). Out-of-range slots / non-slider kinds no-op.
pub fn set_adjustment_slider_param(params: &mut AdjustmentParams, slot: usize, value01: f32) {
    let v = value01.clamp(0.0, 1.0);
    match params {
        AdjustmentParams::HueSaturationBrightness(p) => match slot {
            0 => p.h = v,             // 0..1 turns
            1 => p.s = v * 2.0 - 1.0, // -1..1
            2 => p.b = v * 2.0 - 1.0, // -1..1
            _ => {}
        },
        AdjustmentParams::BrightnessContrast(p) => match slot {
            0 => p.brightness = v * 2.0 - 1.0,
            1 => p.contrast = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Exposure(p) => match slot {
            0 => p.exposure_ev = v * 8.0 - 4.0,      // -4..4 EV
            1 => p.offset = v - 0.5,                 // -0.5..0.5
            2 => p.gamma_correction = v * 1.8 - 0.9, // -0.9..0.9 (effective γ 0.1..1.9)
            _ => {}
        },
        AdjustmentParams::Vibrance(p) => match slot {
            0 => p.vibrance = v * 2.0 - 1.0,
            1 => p.saturation = v * 2.0 - 1.0,
            _ => {}
        },
        AdjustmentParams::Posterize(p) if slot == 0 => {
            p.levels = (2.0 + v * 30.0).round().clamp(2.0, 32.0) as u8;
        }
        AdjustmentParams::Threshold(p) if slot == 0 => {
            p.threshold = (v * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        AdjustmentParams::PhotoFilter(p) => match slot {
            0 => p.temperature = v * 2.0 - 1.0, // -1..1
            1 => p.density = v,                 // 0..1
            _ => {}
        },
        AdjustmentParams::ColorBalance(p) => match slot {
            0 => p.cyan_red = v * 2.0 - 1.0,      // -1..1
            1 => p.magenta_green = v * 2.0 - 1.0, // -1..1
            2 => p.yellow_blue = v * 2.0 - 1.0,   // -1..1
            _ => {}
        },
        AdjustmentParams::BlackAndWhite(p) => {
            let weight = v * 5.0 - 2.0; // 0..1 → -2..3
            match slot {
                0 => p.reds = weight,
                1 => p.yellows = weight,
                2 => p.greens = weight,
                3 => p.cyans = weight,
                4 => p.blues = weight,
                5 => p.magentas = weight,
                // Tint Hue / amount (only meaningful while a tint is set).
                6 => {
                    if let Some(t) = &mut p.tint_color {
                        t.h = v * 360.0;
                    }
                }
                7 => p.tint_amount = v,
                _ => {}
            }
        }
        AdjustmentParams::Levels(p) => match slot {
            0 => p.black_point = v,
            1 => p.gamma = levels_slider_to_gamma(v),
            2 => p.white_point = v,
            3 => p.output_black = v,
            4 => p.output_white = v,
            _ => {}
        },
        // W4 spatial mesh — inverse of `adjustment_slider_params`.
        AdjustmentParams::GaussianBlur(p) if slot == 0 => p.radius = v * SPATIAL_PX_MAX,
        AdjustmentParams::MotionBlur(p) => match slot {
            0 => p.distance = v * SPATIAL_PX_MAX,
            1 => p.angle = slider_to_angle(v),
            _ => {}
        },
        AdjustmentParams::Sharpen(p) => match slot {
            0 => p.amount = v * SHARPEN_AMOUNT_MAX,
            1 => p.radius = v * SHARPEN_RADIUS_MAX,
            _ => {}
        },
        AdjustmentParams::ChromaticAberration(p) => match slot {
            0 => p.red_shift = slider_to_shift(v),
            1 => p.green_shift = slider_to_shift(v),
            2 => p.blue_shift = slider_to_shift(v),
            _ => {}
        },
        AdjustmentParams::Noise(p) if slot == 0 => p.amount = v,
        AdjustmentParams::Halftone(p) => match slot {
            0 => p.dot_size = 1.0 + v * (HALFTONE_DOT_MAX - 1.0),
            1 => p.angle = slider_to_angle(v),
            _ => {}
        },
        AdjustmentParams::ColorLookupLut(p) => match slot {
            0 => p.lut_3d = LutHandle(slider_to_preset(v)),
            1 => p.intensity = v,
            _ => {}
        },
        AdjustmentParams::Bloom(p) => match slot {
            0 => p.threshold = v,
            1 => p.intensity = v * BLOOM_INTENSITY_MAX,
            2 => p.radius = v * SPATIAL_PX_MAX,
            3 => p.falloff = v,
            _ => {}
        },
        AdjustmentParams::ShadowsHighlights(p) => match slot {
            0 => p.shadows_amount = v,
            1 => p.shadows_tonal_width = v,
            2 => p.shadows_radius = v * SPATIAL_PX_MAX,
            3 => p.highlights_amount = v,
            4 => p.highlights_tonal_width = v,
            5 => p.highlights_radius = v * SPATIAL_PX_MAX,
            6 => p.color_correction = v * 2.0 - 1.0, // -1..1
            7 => p.midtone_contrast = v * 2.0 - 1.0, // -1..1
            _ => {}
        },
        // Gradient Map has a bespoke editor (its stop colors go through
        // `set_gradient_stop_color_param`), so no generic slider slot here.
        // Curves has no generic sliders — its bespoke editor drives free 2-D point
        // drags through `PainterTool::set_curve_point` (W4 §3), not this slot path.
        _ => {}
    }
}

/// The boolean (toggle) params of an adjustment, as `(label, on)` in slot order
/// — the toggle-rack twin of [`adjustment_slider_params`]. The layers panel
/// renders each as a small switch row under the slider rack; the inverse setter
/// is [`set_adjustment_toggle_param`]. Kinds with no toggles return empty. Adding
/// a toggle-bearing kind needs ZERO panel change (the rack iterates this).
#[must_use]
pub fn adjustment_toggle_params(params: &AdjustmentParams) -> Vec<(&'static str, bool)> {
    match params {
        AdjustmentParams::PhotoFilter(p) => vec![("Preserve Lum.", p.preserve_luminosity)],
        AdjustmentParams::ColorBalance(p) => vec![("Preserve Lum.", p.preserve_luminosity)],
        AdjustmentParams::ChannelMixer(p) => vec![("Monochrome", p.monochromatic)],
        AdjustmentParams::BlackAndWhite(p) => vec![("Tint", p.tint_color.is_some())],
        // Noise: monochrome = one luma grain vs independent per-channel. (Sharpen's
        // `mask_edges` is intentionally NOT exposed yet — its gate is a deferred
        // joint CPU+GPU follow-up, so a toggle here would be a no-op affordance.)
        AdjustmentParams::Noise(p) => vec![("Monochrome", p.monochromatic)],
        _ => Vec::new(),
    }
}

/// Set toggle `slot` of an adjustment (inverse of [`adjustment_toggle_params`]).
/// Out-of-range slots / non-toggle kinds no-op.
pub fn set_adjustment_toggle_param(params: &mut AdjustmentParams, slot: usize, on: bool) {
    match params {
        AdjustmentParams::PhotoFilter(p) if slot == 0 => p.preserve_luminosity = on,
        AdjustmentParams::ColorBalance(p) if slot == 0 => p.preserve_luminosity = on,
        AdjustmentParams::ChannelMixer(p) if slot == 0 => p.monochromatic = on,
        AdjustmentParams::Noise(p) if slot == 0 => p.monochromatic = on,
        // Black & White Tint: enabling seeds a classic warm sepia + a visible
        // amount (so the toggle has an immediate effect); the Hue/Tint sliders
        // then refine it. Disabling drops the tint back to a plain grayscale.
        AdjustmentParams::BlackAndWhite(p) if slot == 0 => {
            if on {
                p.tint_color = Some(OklchColor::opaque(0.7, 0.1, 70.0));
                if p.tint_amount == 0.0 {
                    p.tint_amount = 0.5;
                }
            } else {
                p.tint_color = None;
            }
        }
        _ => {}
    }
}

/// The single segmented (1-of-N, N ≤ 3) param of an adjustment, as
/// `(options, selected)` — what the layers panel renders as a segment-button row
/// (mirror of the Curves channel tabs). The inverse setter is
/// [`set_adjustment_segment_param`]. Kinds with no segmented param return `None`.
/// Currently the Color-Balance tonal range (Shadows / Midtones / Highlights).
#[must_use]
pub fn adjustment_segment_params(params: &AdjustmentParams) -> Option<(Vec<&'static str>, usize)> {
    match params {
        AdjustmentParams::ColorBalance(p) => Some((
            vec!["Shadows", "Midtones", "Highlights"],
            match p.scope {
                ToneScope::Shadows => 0,
                ToneScope::Midtones => 1,
                ToneScope::Highlights => 2,
            },
        )),
        AdjustmentParams::GradientMap(p) => Some((
            vec!["Linear", "Smooth"],
            match p.interpolation {
                GradientInterp::Linear => 0,
                GradientInterp::Smooth => 1,
            },
        )),
        AdjustmentParams::SelectiveColor(p) => Some((
            vec!["Relative", "Absolute"],
            match p.method {
                SelectiveMethod::Relative => 0,
                SelectiveMethod::Absolute => 1,
            },
        )),
        // W4 — Noise distribution + Halftone cell shape.
        AdjustmentParams::Noise(p) => Some((
            vec!["Gaussian", "Uniform"],
            match p.kind {
                NoiseKind::Gaussian => 0,
                NoiseKind::Uniform => 1,
            },
        )),
        AdjustmentParams::Halftone(p) => Some((
            vec!["Dot", "Line", "Circle"],
            match p.shape {
                HalftoneShape::Dot => 0,
                HalftoneShape::Line => 1,
                HalftoneShape::Circle => 2,
            },
        )),
        _ => None,
    }
}

/// Select option `option` of an adjustment's segmented param (inverse of
/// [`adjustment_segment_params`]). Out-of-range options clamp to the nearest
/// valid; non-segmented kinds no-op.
pub fn set_adjustment_segment_param(params: &mut AdjustmentParams, option: usize) {
    match params {
        AdjustmentParams::ColorBalance(p) => {
            p.scope = match option {
                0 => ToneScope::Shadows,
                2 => ToneScope::Highlights,
                _ => ToneScope::Midtones,
            };
        }
        AdjustmentParams::GradientMap(p) => {
            p.interpolation = if option == 1 {
                GradientInterp::Smooth
            } else {
                GradientInterp::Linear
            };
        }
        AdjustmentParams::SelectiveColor(p) => {
            p.method = if option == 1 {
                SelectiveMethod::Absolute
            } else {
                SelectiveMethod::Relative
            };
        }
        AdjustmentParams::Noise(p) => {
            p.kind = if option == 1 {
                NoiseKind::Uniform
            } else {
                NoiseKind::Gaussian
            };
        }
        AdjustmentParams::Halftone(p) => {
            p.shape = match option {
                1 => HalftoneShape::Line,
                2 => HalftoneShape::Circle,
                _ => HalftoneShape::Dot,
            };
        }
        _ => {}
    }
}
