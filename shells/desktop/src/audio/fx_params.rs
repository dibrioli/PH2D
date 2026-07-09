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
//! [`KINDS`] carries the display name, the parameter specs and the constructor
//! **together**. They used to be three parallel `match kind { .. }` arms, which
//! meant inserting an effect in the middle silently re-indexed one of them: the
//! rack would name "Compress", show its sliders, and build a Saturate.

use ph2d_audio_edit::{Effect, TailEffect};

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
// The three EQ bands share a neutral: 0 dB of gain is a flat filter.
static PEAK_EQ: [FxParamSpec; 3] = [
    spec("Freq", 20.0, 20_000.0, true, 1_000.0, "Hz", false),
    spec("Q", 0.1, 8.0, false, 1.0, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
static LOW_SHELF: [FxParamSpec; 3] = [
    spec("Freq", 20.0, 2_000.0, true, 200.0, "Hz", false),
    spec("Q", 0.1, 2.0, false, 0.707, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
static HIGH_SHELF: [FxParamSpec; 3] = [
    spec("Freq", 1_000.0, 20_000.0, true, 6_000.0, "Hz", false),
    spec("Q", 0.1, 2.0, false, 0.707, "", false),
    spec("Gain", -18.0, 18.0, false, 0.0, "dB", false),
];
static COMPRESS: [FxParamSpec; 4] = [
    spec("Threshold", 0.01, 1.0, true, 0.3, "", false),
    // Neutral at 1:1 — no reduction at all (make-up is peak-preserving, so it
    // collapses to unity too).
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    spec("Attack", 0.001, 0.2, true, 0.005, "s", false),
    spec("Release", 0.01, 1.0, true, 0.1, "s", false),
];
static LIMITER: [FxParamSpec; 2] = [
    // Neutral at the TOP: a ceiling at 0 dBFS has nothing to catch. −1 dBTP is the
    // mastering convention; that is one notch down from neutral.
    spec("Ceiling", -12.0, 0.0, false, 0.0, "dB", false),
    // Doubles as the look-ahead: the gain dips this far ahead of every peak.
    spec("Release", 0.002, 0.2, true, 0.02, "s", false),
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

/// The effects the selector cycles, in order — grouped tone → dynamics → character
/// → space, the way a rack is laid out.
pub(crate) static KINDS: [FxKind; 12] = [
    FxKind {
        name: "Low-Pass",
        params: &LOW_PASS,
        build: |v| {
            FxCommand::Plain(Effect::LowPass {
                cutoff: v[0],
                q: v[1],
            })
        },
    },
    FxKind {
        name: "High-Pass",
        params: &HIGH_PASS,
        build: |v| {
            FxCommand::Plain(Effect::HighPass {
                cutoff: v[0],
                q: v[1],
            })
        },
    },
    FxKind {
        name: "Peak EQ",
        params: &PEAK_EQ,
        build: |v| {
            FxCommand::Plain(Effect::Peak {
                freq: v[0],
                q: v[1],
                gain_db: v[2],
            })
        },
    },
    FxKind {
        name: "Low Shelf",
        params: &LOW_SHELF,
        build: |v| {
            FxCommand::Plain(Effect::LowShelf {
                freq: v[0],
                q: v[1],
                gain_db: v[2],
            })
        },
    },
    FxKind {
        name: "High Shelf",
        params: &HIGH_SHELF,
        build: |v| {
            FxCommand::Plain(Effect::HighShelf {
                freq: v[0],
                q: v[1],
                gain_db: v[2],
            })
        },
    },
    FxKind {
        name: "Compress",
        params: &COMPRESS,
        // Make-up is automatic and peak-preserving inside the effect: raising the
        // ratio must not raise the waveform's amplitude.
        build: |v| {
            FxCommand::Plain(Effect::Compress {
                threshold: v[0],
                ratio: v[1],
                attack_secs: v[2],
                release_secs: v[3],
            })
        },
    },
    FxKind {
        name: "Limiter",
        params: &LIMITER,
        build: |v| {
            FxCommand::Plain(Effect::Limiter {
                ceiling_db: v[0],
                release_secs: v[1],
            })
        },
    },
    FxKind {
        name: "Saturate",
        params: &SATURATE,
        build: |v| FxCommand::Plain(Effect::Saturate { drive: v[0] }),
    },
    FxKind {
        name: "Bitcrush",
        params: &BITCRUSH,
        build: |v| {
            FxCommand::Plain(Effect::Bitcrush {
                bits: v[0] as u32,
                downsample: v[1] as u32,
            })
        },
    },
    FxKind {
        name: "Widen",
        params: &WIDEN,
        build: |v| FxCommand::Plain(Effect::StereoWidth { width: v[0] }),
    },
    FxKind {
        name: "Reverb",
        params: &REVERB,
        build: |v| {
            FxCommand::Tail(TailEffect::Reverb {
                room_size: v[0],
                damp: v[1],
                mix: v[2],
                tail_secs: v[3],
            })
        },
    },
    FxKind {
        name: "Echo",
        params: &ECHO,
        build: |v| {
            FxCommand::Tail(TailEffect::Delay {
                time_secs: v[0],
                feedback: v[1],
                mix: v[2],
                tail_secs: v[3],
            })
        },
    },
];

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

/// Every kind's neutral defaults, in `KINDS` order — published to the panel so a
/// fresh (or reset) chain stage is seeded transparent without knowing any DSP range.
pub(crate) fn all_default_norms() -> Vec<[f32; MAX_FX_PARAMS]> {
    (0..KINDS.len()).map(default_norms).collect()
}

/// Render one real value for the panel readout.
fn format_value(s: &FxParamSpec, v: f32) -> String {
    match s.unit {
        "Hz" if v >= 1_000.0 => format!("{:.1} kHz", v / 1_000.0),
        "Hz" => format!("{v:.0} Hz"),
        "s" if v < 1.0 => format!("{:.0} ms", v * 1_000.0),
        "s" => format!("{v:.2} s"),
        "dB" => format!("{v:+.1} dB"),
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
        }
        assert!(build(KINDS.len(), &[0.0; MAX_FX_PARAMS]).is_none());
    }

    /// The names the panel paints must line up with the effects the shell builds.
    /// They live in one row now, so this pins the *ordering* the user sees.
    #[test]
    fn the_kind_table_is_the_rack_layout() {
        let names = kind_names();
        assert_eq!(names.len(), KINDS.len());
        assert_eq!(names[0], "Low-Pass");
        assert_eq!(names[6], "Limiter");
        assert_eq!(names[names.len() - 1], "Echo");
        assert_eq!(all_default_norms().len(), KINDS.len());
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
        let cutoff = &LOW_PASS[0]; // 20 Hz .. 20 kHz, log
        // The midpoint of a log sweep is the geometric mean (~632 Hz), not 10 kHz.
        let mid = norm_to_real(cutoff, 0.5);
        assert!((mid - 632.0).abs() < 5.0, "log midpoint was {mid}");
        assert_eq!(norm_to_real(cutoff, 0.0), 20.0);
        assert!((norm_to_real(cutoff, 1.0) - 20_000.0).abs() < 1.0);
    }

    /// A band-spread, off-centre, stereo-divergent signal: a filter, an EQ band, a
    /// compressor, a limiter, a crusher or an M/S tweak all leave a mark on it.
    fn probe() -> SampleData {
        let samples: Vec<f32> = (0..2_000)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                0.4 * (t * 220.0).sin() + 0.3 * (t * 9_000.0).sin() + 0.05
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

    /// ...and the bypass must not swallow a real edit: nudge one parameter off
    /// neutral and the audio has to move. Guards a `is_bypass` that is too eager,
    /// and a `build` wired to the wrong variant (a Compress spec feeding a Saturate
    /// would still be "neutral" here and the audio would never change).
    #[test]
    fn a_nudge_off_neutral_changes_the_audio() {
        let d = probe();
        for (kind, k) in KINDS.iter().enumerate() {
            // Move EVERY parameter a visible step off neutral, toward the middle of
            // its travel. Moving only one would not arm the EQ bands: their Freq and
            // Q do nothing while Gain sits at 0 dB.
            let mut norms = default_norms(kind);
            for slot in norms.iter_mut().take(k.params.len()) {
                *slot = if *slot > 0.5 {
                    *slot - 0.35
                } else {
                    *slot + 0.35
                };
            }
            let clip = EditClip::new(d.clone());
            let out = match build(kind, &norms).expect("builds") {
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
                "{} did nothing off its neutral point",
                k.name
            );
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
