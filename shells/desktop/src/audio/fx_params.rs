//! Parametric descriptors for the Audio Editor's effects rack (W3 block 3a).
//!
//! The panel is UI-only: it owns nothing but a **normalized 0..1 slider per
//! parameter** and an index into [`FX_KINDS`]. This module is the shell-side
//! source of truth that maps those normals onto real DSP units, formats them for
//! display, and finally builds the `ph2d_audio_edit` effect to apply.
//!
//! Frequencies and times map **logarithmically** — a linear cutoff slider spends
//! most of its travel above 10 kHz, where nothing useful happens.
//!
//! Every effect's **defaults are its neutral point**: selecting an effect leaves
//! the audio byte-identical until the user turns a knob.

use ph2d_audio_edit::{Effect, TailEffect};

/// Parameter sliders the rack exposes. Mirrors `ph2d_panel_audio_editor::MAX_FX_PARAMS`.
pub(crate) const MAX_FX_PARAMS: usize = 4;

/// The effects the selector cycles, in order. The panel shows `FX_KINDS[index]`.
pub(crate) const FX_KINDS: [&str; 8] = [
    "Low-Pass",
    "High-Pass",
    "Compress",
    "Saturate",
    "Bitcrush",
    "Widen",
    "Reverb",
    "Echo",
];

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
    /// Drives the display formatting: `Hz`, `s`, `x`, or `""`.
    pub unit: &'static str,
    /// Round to a whole number (bit depth, decimation factor).
    pub integral: bool,
}

/// Shorthand for a spec (keeps the tables below readable).
const fn spec(
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

// Every `default` below is the effect's NEUTRAL point: selecting an effect (or
// arrowing past it) must leave the audio byte-identical until the user turns
// something. `ph2d_audio_edit::{Effect, TailEffect}::is_bypass` mirrors these, and
// `every_effect_is_a_no_op_at_its_defaults` proves the two agree.
static LOW_PASS: [FxParamSpec; 2] = [
    // Neutral at the TOP of the range: nothing is filtered out.
    spec("Cutoff", 20.0, 20_000.0, true, 20_000.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 0.707, "", false),
];
static HIGH_PASS: [FxParamSpec; 2] = [
    // Neutral at the BOTTOM of the range.
    spec("Cutoff", 20.0, 20_000.0, true, 20.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 0.707, "", false),
];
static COMPRESS: [FxParamSpec; 4] = [
    spec("Threshold", 0.01, 1.0, true, 0.3, "", false),
    // Neutral at 1:1 — no reduction, and auto-makeup collapses to unity.
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    spec("Attack", 0.001, 0.2, true, 0.005, "s", false),
    spec("Release", 0.01, 1.0, true, 0.1, "s", false),
];
// Linear (not log) so the neutral point can sit at exactly 0.
static SATURATE: [FxParamSpec; 1] = [spec("Drive", 0.0, 12.0, false, 0.0, "x", false)];
static BITCRUSH: [FxParamSpec; 2] = [
    // Neutral = full depth, no decimation.
    spec("Bits", 1.0, 16.0, false, 16.0, "", true),
    spec("Downsample", 1.0, 32.0, false, 1.0, "x", true),
];
// Neutral at width 1.0 (mid/side passthrough), the middle of the range.
static WIDEN: [FxParamSpec; 1] = [spec("Width", 0.0, 2.0, false, 1.0, "x", false)];
static REVERB: [FxParamSpec; 4] = [
    spec("Room", 0.0, 1.0, false, 0.7, "", false),
    spec("Damp", 0.0, 1.0, false, 0.5, "", false),
    // Neutral: fully dry. A dry tail effect must not even ring out, or it would
    // lengthen the clip with silence — `tail_frames()` returns 0 when Mix is 0.
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
    // Freeverb's shortest comb is ~25 ms: a tail below that renders pure silence.
    spec("Tail", 0.1, 6.0, true, 2.5, "s", false),
];
static ECHO: [FxParamSpec; 4] = [
    // The dsp kit's delay line is one second long, so the tap must stay under it.
    spec("Time", 0.01, 0.99, true, 0.25, "s", false),
    spec("Feedback", 0.0, 0.95, false, 0.4, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
    spec("Tail", 0.1, 6.0, true, 2.0, "s", false),
];

/// The parameters of effect `kind` (empty when the index is out of range).
pub(crate) fn params_for(kind: usize) -> &'static [FxParamSpec] {
    match kind {
        0 => &LOW_PASS,
        1 => &HIGH_PASS,
        2 => &COMPRESS,
        3 => &SATURATE,
        4 => &BITCRUSH,
        5 => &WIDEN,
        6 => &REVERB,
        7 => &ECHO,
        _ => &[],
    }
}

/// Slider normal (0..1) → real DSP value.
///
/// The endpoints are returned **exactly**: several effects have their neutral
/// point at a bound (Low-Pass at max cutoff, High-Pass at min), and `exp(ln(x))`
/// drifts by an ulp — `20_000.0` came back as `19_999.998`, which is enough to
/// miss the bypass check and make "no effect" filter the audio.
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
fn real_to_norm(s: &FxParamSpec, real: f32) -> f32 {
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

/// Render one real value for the panel readout.
fn format_value(s: &FxParamSpec, v: f32) -> String {
    match s.unit {
        "Hz" if v >= 1_000.0 => format!("{:.1} kHz", v / 1_000.0),
        "Hz" => format!("{v:.0} Hz"),
        "s" if v < 1.0 => format!("{:.0} ms", v * 1_000.0),
        "s" => format!("{v:.2} s"),
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

/// Read parameter `i` of `kind` in real units.
fn real(kind: usize, norms: &[f32; MAX_FX_PARAMS], i: usize) -> f32 {
    let p = params_for(kind);
    match p.get(i) {
        Some(s) => norm_to_real(s, norms[i]),
        None => 0.0,
    }
}

/// Auto make-up gain for the compressor: the linear boost that brings a
/// full-scale signal back to unity after `ratio`:1 reduction above `threshold`.
/// Clamped so an extreme threshold can't detonate the level.
fn auto_makeup(threshold: f32, ratio: f32) -> f32 {
    let t = threshold.clamp(0.001, 1.0);
    let r = ratio.max(1.0);
    (1.0 / t).powf(1.0 - 1.0 / r).clamp(1.0, 4.0)
}

/// Build the effect for `kind` at the current slider positions.
pub(crate) fn build(kind: usize, norms: &[f32; MAX_FX_PARAMS]) -> Option<FxCommand> {
    let v = |i| real(kind, norms, i);
    Some(match kind {
        0 => FxCommand::Plain(Effect::LowPass {
            cutoff: v(0),
            q: v(1),
        }),
        1 => FxCommand::Plain(Effect::HighPass {
            cutoff: v(0),
            q: v(1),
        }),
        2 => {
            let (threshold, ratio) = (v(0), v(1));
            FxCommand::Plain(Effect::Compress {
                threshold,
                ratio,
                attack_secs: v(2),
                release_secs: v(3),
                makeup: auto_makeup(threshold, ratio),
            })
        }
        3 => FxCommand::Plain(Effect::Saturate { drive: v(0) }),
        4 => FxCommand::Plain(Effect::Bitcrush {
            bits: v(0) as u32,
            downsample: v(1) as u32,
        }),
        5 => FxCommand::Plain(Effect::StereoWidth { width: v(0) }),
        6 => FxCommand::Tail(TailEffect::Reverb {
            room_size: v(0),
            damp: v(1),
            mix: v(2),
            tail_secs: v(3),
        }),
        7 => FxCommand::Tail(TailEffect::Delay {
            time_secs: v(0),
            feedback: v(1),
            mix: v(2),
            tail_secs: v(3),
        }),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};
    use ph2d_audio_edit::EditClip;

    #[test]
    fn every_kind_has_a_spec_and_builds() {
        for (kind, name) in FX_KINDS.iter().enumerate() {
            let p = params_for(kind);
            assert!(!p.is_empty(), "{name} has no params");
            assert!(p.len() <= MAX_FX_PARAMS, "{name} exceeds the slider count");
            assert!(
                build(kind, &default_norms(kind)).is_some(),
                "{name} does not build"
            );
            // Log params need a positive lower bound or `ln` blows up.
            assert!(p.iter().all(|s| !s.log || s.min > 0.0));
        }
        assert!(build(FX_KINDS.len(), &[0.0; MAX_FX_PARAMS]).is_none());
    }

    #[test]
    fn defaults_round_trip_to_the_preset_values() {
        for (kind, name) in FX_KINDS.iter().enumerate() {
            let norms = default_norms(kind);
            for (i, s) in params_for(kind).iter().enumerate() {
                let back = norm_to_real(s, norms[i]);
                let tol = if s.integral {
                    0.5
                } else {
                    s.default.abs() * 1e-3 + 1e-6
                };
                assert!(
                    (back - s.default).abs() <= tol,
                    "{name} / {}: {back} != {}",
                    s.label,
                    s.default
                );
            }
        }
    }

    #[test]
    fn log_mapping_spends_travel_where_it_matters() {
        let cutoff = &LOW_PASS[0]; // 20 Hz .. 20 kHz, log
        // The midpoint of a log sweep is the geometric mean (~632 Hz), not 10 kHz.
        let mid = norm_to_real(cutoff, 0.5);
        assert!((mid - 632.0).abs() < 5.0, "log midpoint was {mid}");
        assert_eq!(norm_to_real(cutoff, 0.0), 20.0);
        assert!((norm_to_real(cutoff, 1.0) - 20_000.0).abs() < 1.0);
    }

    #[test]
    fn auto_makeup_is_bounded_and_rises_with_ratio() {
        assert_eq!(auto_makeup(1.0, 4.0), 1.0, "no reduction → no make-up");
        assert!(auto_makeup(0.3, 8.0) > auto_makeup(0.3, 2.0));
        assert!(auto_makeup(0.001, 20.0) <= 4.0, "clamped");
    }

    /// THE contract of the defaults: selecting an effect (or arrowing past it)
    /// must leave the audio **byte-identical** until the user turns something.
    /// Not "almost" — a filter at the top of its range still phase-shifts, a 1:1
    /// compressor still rounds, and a dry reverb would otherwise append a silent
    /// tail. Each effect therefore short-circuits at its neutral point.
    #[test]
    fn every_effect_is_a_no_op_at_its_defaults() {
        // A signal with content across the band, off-centre and stereo-divergent,
        // so a filter, a compressor, a crusher or an M/S tweak would all show.
        let samples: Vec<f32> = (0..2_000)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                0.4 * (t * 220.0).sin() + 0.3 * (t * 9_000.0).sin() + 0.05
            })
            .collect();
        let d = SampleData::from_interleaved(samples, AudioFormat::stereo(48_000));

        for (kind, name) in FX_KINDS.iter().enumerate() {
            let clip = EditClip::new(d.clone());
            let out = match build(kind, &default_norms(kind)).expect("every kind builds") {
                FxCommand::Plain(fx) => {
                    assert!(fx.is_bypass(), "{name}: defaults are not the neutral point");
                    clip.render_effect(fx)
                }
                FxCommand::Tail(fx) => {
                    assert!(fx.is_bypass(), "{name}: defaults are not the neutral point");
                    clip.render_tail_effect(fx)
                }
            };
            assert_eq!(
                out.frame_count(),
                clip.frame_count(),
                "{name} changed the clip length at its defaults"
            );
            assert_eq!(
                out.samples(),
                d.samples(),
                "{name} is not a byte-identical no-op at its defaults"
            );
        }
    }

    /// …and one nudge off neutral must actually do something, or the bypass
    /// short-circuit is swallowing real edits.
    #[test]
    fn a_nudge_off_neutral_changes_the_audio() {
        let d =
            SampleData::from_interleaved(vec![0.5, -0.4, 0.3, -0.2], AudioFormat::stereo(48_000));
        let clip = EditClip::new(d.clone());
        // Low-Pass: pull the cutoff well down from its neutral top.
        let mut norms = default_norms(0);
        norms[0] = 0.3;
        let FxCommand::Plain(fx) = build(0, &norms).unwrap() else {
            panic!("Low-Pass is length-preserving")
        };
        assert!(!fx.is_bypass());
        assert_ne!(clip.render_effect(fx).samples(), d.samples());

        // Reverb: raise Mix off zero → it rings out and grows the clip.
        let mut norms = default_norms(6);
        norms[2] = 0.5;
        let FxCommand::Tail(fx) = build(6, &norms).unwrap() else {
            panic!("Reverb is tail-extending")
        };
        assert!(!fx.is_bypass());
        assert!(clip.render_tail_effect(fx).frame_count() > clip.frame_count());
    }

    #[test]
    fn views_match_the_param_count_and_format_units() {
        let reverb = 6;
        let v = views(reverb, &default_norms(reverb));
        assert_eq!(v.len(), 4);
        assert_eq!(v[0].0, "Room");
        assert_eq!(v[3].1, "2.50 s", "tail formats as seconds");
        // Sub-second times read as milliseconds.
        let echo = 7;
        let e = views(echo, &default_norms(echo));
        assert_eq!(e[0].1, "250 ms");
        // Cutoff above 1 kHz reads in kHz.
        let lp = views(0, &default_norms(0));
        assert_eq!(lp[0].1, "20.0 kHz", "neutral cutoff sits at the top");
    }
}
