//! Parametric descriptors for the Audio Editor's effects rack (W3 blocks 3a/3b/4).
//!
//! The panel is UI-only: it owns nothing but a **normalized 0..1 slider per
//! parameter** and an index into [`KINDS`]. This module is the shell-side source of
//! truth that maps those normals onto real DSP units, formats them for display, and
//! finally builds the `ph2d_audio_edit` effect to apply.
//!
//! Frequencies and times map **logarithmically** — a linear cutoff slider spends
//! most of its travel above 10 kHz, where nothing useful happens.
//!
//! Every effect's **defaults are its neutral point**: selecting an effect leaves
//! the audio byte-identical until the user turns a knob.
//!
//! ## One row per effect
//!
//! [`KINDS`] (in the sibling `fx_params_table`) carries the display name, the
//! parameter specs and the constructor **together**. They used to be three parallel
//! `match kind { .. }` arms, which meant inserting an effect in the middle silently
//! re-indexed one of them: the rack would name "Compress", show its sliders, and
//! build a Saturate.

use ph2d_audio_edit::{Effect, TailEffect};

use super::fx_params_table::KINDS;

/// Parameter sliders the rack exposes. Mirrors `ph2d_panel_audio_editor::MAX_FX_PARAMS`.
pub(crate) const MAX_FX_PARAMS: usize = 4;

/// How a normalized 0..1 slider maps onto one real DSP parameter.
pub(crate) struct FxParamSpec {
    /// Shown left of the slider.
    pub label: &'static str,
    /// Real-unit bounds. `min > 0` is required when `log`.
    pub min: f32,
    pub max: f32,
    /// Map the slider logarithmically (frequency, time).
    pub log: bool,
    /// Default in REAL units — the effect's **neutral** value. At its defaults an
    /// effect is a byte-identical no-op, so selecting it changes nothing until the
    /// user turns a knob.
    pub default: f32,
    /// Drives the display formatting: `Hz`, `s`, `dB`, `x`, or `""`.
    pub unit: &'static str,
    /// Round to a whole number (bit depth, decimation factor).
    pub integral: bool,
}

/// One effect the selector can pick: its name, its parameters, and how to build it
/// from those parameters **in real units**. Keeping the three in one row is what
/// makes a mis-indexed rack impossible.
pub(crate) struct FxKind {
    pub name: &'static str,
    pub params: &'static [FxParamSpec],
    /// `v[i]` is parameter `i` already mapped to real units.
    pub build: fn(v: &[f32; MAX_FX_PARAMS]) -> FxCommand,
    /// Indices of the **arming knobs**: the parameters whose neutral values are what
    /// `Effect::is_bypass` watches. Turning any of them off its default is what wakes
    /// the effect up; every other knob is inert until then (a 0 dB EQ band ignores
    /// its Freq and Q). Usually one — Bitcrush has two, since either a lower bit
    /// depth or any decimation makes it audible.
    ///
    /// Nothing at runtime reads this: it exists so the tests can hold `is_bypass`
    /// honest from the outside. Get it wrong in either direction and
    /// `turning_an_arming_knob_wakes_the_effect_up` or
    /// `the_other_knobs_do_nothing_while_the_effect_is_neutral` says so.
    #[allow(dead_code)]
    pub arms: &'static [usize],
}

/// Shorthand for a spec (keeps the table in `fx_params_table` readable).
pub(super) const fn spec(
    label: &'static str,
    min: f32,
    max: f32,
    log: bool,
    default: f32,
    unit: &'static str,
    integral: bool,
) -> FxParamSpec {
    FxParamSpec {
        label,
        min,
        max,
        log,
        default,
        unit,
        integral,
    }
}

/// Display names, in `KINDS` order — published to the panel each frame.
pub(crate) fn kind_names() -> Vec<&'static str> {
    KINDS.iter().map(|k| k.name).collect()
}

/// The parameters of effect `kind` (empty when the index is out of range).
pub(crate) fn params_for(kind: usize) -> &'static [FxParamSpec] {
    KINDS.get(kind).map(|k| k.params).unwrap_or(&[])
}

/// Slider normal (0..1) → real DSP value.
///
/// The endpoints are returned **exactly**: several effects have their neutral
/// point at a bound (Low-Pass at max cutoff, High-Pass at min, Limiter at max
/// ceiling) and `exp(ln(x))` drifts by an ulp — `20_000.0` came back as
/// `19_999.998`, which is enough to miss the bypass check and make "no effect"
/// filter the audio.
pub(crate) fn norm_to_real(s: &FxParamSpec, norm: f32) -> f32 {
    let n = norm.clamp(0.0, 1.0);
    if n <= 0.0 {
        return s.min;
    }
    if n >= 1.0 {
        return s.max;
    }
    let v = if s.log {
        (s.min.ln() + n * (s.max.ln() - s.min.ln())).exp()
    } else {
        s.min + n * (s.max - s.min)
    };
    if s.integral { v.round() } else { v }
}

/// Real DSP value → slider normal (0..1). Inverse of [`norm_to_real`].
pub(crate) fn real_to_norm(s: &FxParamSpec, real: f32) -> f32 {
    let v = real.clamp(s.min, s.max);
    let n = if s.log {
        (v.ln() - s.min.ln()) / (s.max.ln() - s.min.ln())
    } else {
        (v - s.min) / (s.max - s.min)
    };
    n.clamp(0.0, 1.0)
}

/// The normalized slider positions of effect `kind`'s **neutral** point — where
/// it is a byte-identical no-op. Unused slots stay at `0.0` (the panel hides them).
pub(crate) fn default_norms(kind: usize) -> [f32; MAX_FX_PARAMS] {
    let mut out = [0.0; MAX_FX_PARAMS];
    for (slot, s) in out.iter_mut().zip(params_for(kind)) {
        *slot = real_to_norm(s, s.default);
    }
    out
}

/// Every kind's neutral defaults, in `KINDS` order — published to the panel so a
/// fresh (or reset) chain stage is seeded transparent without knowing any DSP range.
pub(crate) fn all_default_norms() -> Vec<[f32; MAX_FX_PARAMS]> {
    (0..KINDS.len()).map(default_norms).collect()
}

/// Render one real value for the panel readout.
///
/// Sub-unit values keep a decimal, or `{:.0}` rounds them to a **misleading zero**:
/// an LFO Rate of 0.05 Hz (the slowest sweep, not a stopped one) and a Gate Attack
/// of 0.5 ms both read "0" without it, which looks like the slider does nothing over
/// its lower travel (found by Enio, 2026-07-09).
fn format_value(s: &FxParamSpec, v: f32) -> String {
    match s.unit {
        "Hz" if v >= 1_000.0 => format!("{:.1} kHz", v / 1_000.0),
        "Hz" if v < 1.0 => format!("{v:.2} Hz"),
        "Hz" => format!("{v:.0} Hz"),
        "s" if v < 0.001 => format!("{:.1} ms", v * 1_000.0),
        "s" if v < 1.0 => format!("{:.0} ms", v * 1_000.0),
        "s" => format!("{v:.2} s"),
        // Already in milliseconds (a modulation depth), not seconds.
        "ms" => format!("{v:.1} ms"),
        "dB" => format!("{v:+.1} dB"),
        // Semitones, signed like dB: "+7.0 st" reads as a shift, "7.00" as a coefficient.
        "st" => format!("{v:+.1} st"),
        "x" if s.integral => format!("{v:.0}\u{d7}"),
        "x" => format!("{v:.2}\u{d7}"),
        _ if s.integral => format!("{v:.0}"),
        _ => format!("{v:.2}"),
    }
}

/// `(label, formatted value)` per parameter of `kind`, at the current slider
/// positions — exactly what the panel paints. Length = that effect's param count.
pub(crate) fn views(kind: usize, norms: &[f32; MAX_FX_PARAMS]) -> Vec<(String, String)> {
    params_for(kind)
        .iter()
        .zip(norms)
        .map(|(s, &n)| (s.label.to_string(), format_value(s, norm_to_real(s, n))))
        .collect()
}

/// The effect built from `kind` + slider positions, tagged by which splice family
/// it needs — the caller routes `Plain` to `apply_effect` and `Tail` to
/// `apply_tail_effect` (a tail effect run through the length-preserving splice
/// would have its ring-out silently truncated).
pub(crate) enum FxCommand {
    /// Length-preserving.
    Plain(Effect),
    /// Tail-extending.
    Tail(TailEffect),
}

impl FxCommand {
    /// Whether this effect is sitting on its neutral point and would return the
    /// audio byte-identical. A chain skips these instead of rendering them, so a
    /// rack full of fresh stages costs nothing.
    pub(crate) fn is_bypass(&self) -> bool {
        match self {
            FxCommand::Plain(fx) => fx.is_bypass(),
            FxCommand::Tail(fx) => fx.is_bypass(),
        }
    }
}

/// Build the effect for `kind` at the current slider positions.
/// Whether `kind` is the effect that needs an impulse response.
///
/// **Derived, not declared.** The alternative was a `needs_ir: bool` column on all 39 rows of
/// the kind table — 38 of them `false`, to describe one. Ask the table what it builds instead:
/// if the answer is a convolution, it wants a room.
pub(crate) fn needs_ir(kind: usize) -> bool {
    matches!(
        build(kind, &default_norms(kind)),
        Some(FxCommand::Tail(TailEffect::Convolution { .. }))
    )
}

pub(crate) fn build(kind: usize, norms: &[f32; MAX_FX_PARAMS]) -> Option<FxCommand> {
    let k = KINDS.get(kind)?;
    let mut v = [0.0f32; MAX_FX_PARAMS];
    for (slot, (s, &n)) in v.iter_mut().zip(k.params.iter().zip(norms)) {
        *slot = norm_to_real(s, n);
    }
    Some((k.build)(&v))
}

#[cfg(test)]
mod tests;
