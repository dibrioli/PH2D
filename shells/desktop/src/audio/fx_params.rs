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
pub(crate) fn build(kind: usize, norms: &[f32; MAX_FX_PARAMS]) -> Option<FxCommand> {
    let k = KINDS.get(kind)?;
    let mut v = [0.0f32; MAX_FX_PARAMS];
    for (slot, (s, &n)) in v.iter_mut().zip(k.params.iter().zip(norms)) {
        *slot = norm_to_real(s, n);
    }
    Some((k.build)(&v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};
    use ph2d_audio_edit::EditClip;

    #[test]
    fn every_kind_has_a_spec_and_builds() {
        for (kind, k) in KINDS.iter().enumerate() {
            let p = k.params;
            assert!(!p.is_empty(), "{} has no params", k.name);
            assert!(
                p.len() <= MAX_FX_PARAMS,
                "{} exceeds the slider count",
                k.name
            );
            assert!(
                build(kind, &default_norms(kind)).is_some(),
                "{} does not build",
                k.name
            );
            // Log params need a positive lower bound or `ln` blows up.
            assert!(p.iter().all(|s| !s.log || s.min > 0.0), "{}", k.name);
            // ...and a default inside its own bounds, or `real_to_norm` clamps it
            // somewhere that isn't the neutral point.
            assert!(
                p.iter().all(|s| s.default >= s.min && s.default <= s.max),
                "{} has a default outside its range",
                k.name
            );
            assert!(!k.arms.is_empty(), "{} has no arming knob", k.name);
            assert!(
                k.arms.iter().all(|&a| a < p.len()),
                "{}'s arming knob is out of range",
                k.name
            );
        }
        assert!(build(KINDS.len(), &[0.0; MAX_FX_PARAMS]).is_none());
    }

    /// The order the user arrows through: tone → dynamics → character → space. Pinned
    /// whole, because the panel's selector *is* this list and reordering it silently
    /// re-labels everyone's muscle memory.
    #[test]
    fn the_kind_table_is_the_rack_layout() {
        assert_eq!(
            kind_names(),
            [
                "Low-Pass",
                "High-Pass",
                "Peak EQ",
                "Low Shelf",
                "High Shelf",
                "De-Hum",
                "Compress",
                "Gate",
                "De-Esser",
                "De-Plosive",
                "Limiter",
                "Leveler",
                "Transient",
                "Saturate",
                "Bitcrush",
                "Widen",
                "Reverb",
                "Echo",
                "Ping-Pong",
                "Chorus",
                "Flanger",
                "Phaser",
                "Tremolo",
                "Auto-Pan",
                "Ring Mod",
                "Pitch Shift",
            ]
        );
        assert_eq!(all_default_norms().len(), KINDS.len());
    }

    /// No slider may read a bare "0" over any of its travel unless the real value is
    /// actually zero — a `{:.0}` that rounds 0.05 Hz to "0 Hz" makes the lower half of
    /// an LFO Rate slider look dead (Enio, 2026-07-09). Walk every parameter across its
    /// whole range and assert the readout only says a zero quantity when it means it.
    #[test]
    fn no_slider_reads_a_false_zero() {
        for (kind, k) in KINDS.iter().enumerate() {
            for (i, s) in k.params.iter().enumerate() {
                for step in 0..=20 {
                    let n = step as f32 / 20.0;
                    let mut norms = default_norms(kind);
                    norms[i] = n;
                    let shown = &views(kind, &norms)[i].1;
                    let real = norm_to_real(s, n);
                    // A readout whose numeric part parses to 0 is only honest when the
                    // real value rounds to zero at the shown precision.
                    let num: f32 = shown
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                        .parse()
                        .unwrap_or(0.0);
                    if num == 0.0 {
                        assert!(
                            real < 0.05 || (s.unit == "s" && real < 5e-5),
                            "{} / {}: shows {:?} but the value is {real}",
                            k.name,
                            s.label,
                            shown
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn defaults_round_trip_to_the_preset_values() {
        for (kind, k) in KINDS.iter().enumerate() {
            let norms = default_norms(kind);
            for (i, s) in k.params.iter().enumerate() {
                let back = norm_to_real(s, norms[i]);
                let tol = if s.integral {
                    0.5
                } else {
                    s.default.abs() * 1e-3 + 1e-6
                };
                assert!(
                    (back - s.default).abs() <= tol,
                    "{} / {}: {back} != {}",
                    k.name,
                    s.label,
                    s.default
                );
            }
        }
    }

    #[test]
    fn log_mapping_spends_travel_where_it_matters() {
        let cutoff = &params_for(0)[0]; // Low-Pass Cutoff: 20 Hz .. 20 kHz, log
        // The midpoint of a log sweep is the geometric mean (~632 Hz), not 10 kHz.
        let mid = norm_to_real(cutoff, 0.5);
        assert!((mid - 632.0).abs() < 5.0, "log midpoint was {mid}");
        assert_eq!(norm_to_real(cutoff, 0.0), 20.0);
        assert!((norm_to_real(cutoff, 1.0) - 20_000.0).abs() < 1.0);
    }

    /// A band-spread, off-centre, stereo-divergent signal: a filter, an EQ band, a
    /// compressor, a gate, a de-esser, a limiter, a crusher or an M/S tweak all leave
    /// a mark on it. It swings through zero, which is what a gate needs to bite.
    ///
    /// (`TAU * hz * t`, not `hz * t` — the old version's "220 Hz" was 35 Hz worth of
    /// radians over the whole buffer, so the signal never left a narrow band around
    /// its DC offset and a gate had nothing to close on.)
    fn probe() -> SampleData {
        let tau = std::f32::consts::TAU;
        let samples: Vec<f32> = (0..4_800)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                0.4 * (tau * 220.0 * t).sin() + 0.3 * (tau * 9_000.0 * t).sin() + 0.05
            })
            .collect();
        SampleData::from_interleaved(samples, AudioFormat::stereo(48_000))
    }

    /// THE contract of the defaults: selecting an effect (or arrowing past it)
    /// must leave the audio **byte-identical** until the user turns something.
    /// Not "almost" — a filter at the top of its range still phase-shifts, a 1:1
    /// compressor still rounds, and a dry reverb would otherwise append a silent
    /// tail. Each effect therefore short-circuits at its neutral point.
    #[test]
    fn every_effect_is_a_no_op_at_its_defaults() {
        let d = probe();
        for (kind, k) in KINDS.iter().enumerate() {
            let clip = EditClip::new(d.clone());
            let out = match build(kind, &default_norms(kind)).expect("every kind builds") {
                FxCommand::Plain(fx) => {
                    assert!(
                        fx.is_bypass(),
                        "{}: defaults are not the neutral point",
                        k.name
                    );
                    clip.render_effect(fx)
                }
                FxCommand::Tail(fx) => {
                    assert!(
                        fx.is_bypass(),
                        "{}: defaults are not the neutral point",
                        k.name
                    );
                    clip.render_tail_effect(fx)
                }
            };
            assert_eq!(
                out.frame_count(),
                clip.frame_count(),
                "{} changed the clip length at its defaults",
                k.name
            );
            assert_eq!(
                out.samples(),
                d.samples(),
                "{} is not a byte-identical no-op at its defaults",
                k.name
            );
        }
    }

    /// Turn one knob to the far end of its travel, leave the rest at their defaults.
    fn norms_with(kind: usize, i: usize) -> [f32; MAX_FX_PARAMS] {
        let mut norms = default_norms(kind);
        norms[i] = if norms[i] > 0.5 { 0.0 } else { 1.0 };
        norms
    }

    /// ...and the bypass must not swallow a real edit. Turn an **arming knob** to the
    /// far end of its travel and the audio has to move.
    ///
    /// Only the arming knobs, because that is the contract: a 0 dB EQ band ignores its
    /// Freq and Q, a 1:1 gate ignores its Threshold. Guards a `is_bypass` that is too
    /// eager, a `build` wired to the wrong variant, and an `arms` entry pointing at an
    /// inert parameter.
    #[test]
    fn turning_an_arming_knob_wakes_the_effect_up() {
        let d = probe();
        for (kind, k) in KINDS.iter().enumerate() {
            for &arm in k.arms {
                let clip = EditClip::new(d.clone());
                let out = match build(kind, &norms_with(kind, arm)).expect("builds") {
                    FxCommand::Plain(fx) => {
                        assert!(!fx.is_bypass(), "{} still reads as neutral", k.name);
                        clip.render_effect(fx)
                    }
                    FxCommand::Tail(fx) => {
                        assert!(!fx.is_bypass(), "{} still reads as neutral", k.name);
                        clip.render_tail_effect(fx)
                    }
                };
                assert_ne!(
                    out.samples(),
                    d.samples(),
                    "{}: turning {} did nothing",
                    k.name,
                    k.params[arm].label
                );
            }
        }
    }

    /// The rest of the knobs are inert while the arming ones sit at neutral. This is
    /// what lets the user browse an effect and sweep its Freq without hearing a thing
    /// until they decide to — and it is what makes `arms` an honest list rather than
    /// a guess.
    #[test]
    fn the_other_knobs_do_nothing_while_the_effect_is_neutral() {
        for (kind, k) in KINDS.iter().enumerate() {
            for i in 0..k.params.len() {
                if k.arms.contains(&i) {
                    continue;
                }
                assert!(
                    build(kind, &norms_with(kind, i))
                        .expect("builds")
                        .is_bypass(),
                    "{}: moving {} armed the effect — then it belongs in `arms`",
                    k.name,
                    k.params[i].label
                );
            }
        }
    }

    #[test]
    fn views_match_the_param_count_and_format_units() {
        let v = views(0, &default_norms(0)); // Low-Pass: Cutoff (Hz), Q
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].0, "Cutoff");
        assert!(v[0].1.ends_with("kHz"), "got {}", v[0].1);

        // The EQ bands read in dB, signed — a "+0.0 dB" that shows as "0.00" reads
        // like a raw coefficient.
        let peak = KINDS.iter().position(|k| k.name == "Peak EQ").unwrap();
        let v = views(peak, &default_norms(peak));
        assert_eq!(v.len(), 3);
        assert_eq!(v[2].1, "+0.0 dB", "the neutral gain must read as 0 dB");
    }
}
