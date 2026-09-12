//! The rack's gates — the five that hold the whole thing up, plus the pinned layout.
//!
//! A child module so `fx_params.rs` stays under the shell's 600-LOC cap (HR-18). Split, never
//! an allowlist: the cap exists to keep a file readable, and an exception would only move the
//! problem onto whoever reads it next.

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
            "Multiband",
            "Gate / Expander",
            "De-Esser",
            "De-Plosive",
            "De-Click",
            "De-Clip",
            "Limiter",
            "Leveler",
            "Transient",
            "Saturate",
            "Distortion",
            "Bitcrush",
            "Widen",
            "Haas",
            "Exciter",
            "Reverb",
            "Conv Reverb",
            "Echo",
            "Ping-Pong",
            "Comb",
            "Chorus",
            "Flanger",
            "Vibrato",
            "Phaser",
            "Auto-Wah",
            "Tremolo",
            "Auto-Pan",
            "Trance Gate",
            "Doubler",
            "Ring Mod",
            "Pitch Shift",
            "Formant Shift",
            "Vocoder",
            "Granular",
            "Harmonizer",
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
///
/// **And it carries a click** — a few frames of full-scale garbage, mid-buffer.
/// A *repair* tool only leaves a mark on damage, so undamaged audio cannot test
/// one: without this, `turning_an_arming_knob_wakes_the_effect_up` would be asking
/// the de-clicker to prove itself on a signal with nothing wrong with it, and the
/// only way to pass would be to smear audio that was never broken — the exact bug
/// its own `the_undamaged_audio_survives_untouched` forbids.
fn probe() -> SampleData {
    let tau = std::f32::consts::TAU;
    let mut samples: Vec<f32> = (0..4_800)
        .map(|i| {
            let t = i as f32 / 48_000.0;
            0.4 * (tau * 220.0 * t).sin() + 0.3 * (tau * 9_000.0 * t).sin() + 0.05
        })
        .collect();
    // A click: six alternating samples, for De-Click's residual detector.
    for (n, s) in samples.iter_mut().skip(2_400).take(6).enumerate() {
        *s = if n % 2 == 0 { 0.95 } else { -0.95 };
    }
    // A CLIPPED plateau, for De-Clip: 20 interleaved samples pinned at the ceiling =
    // 10 consecutive frames per channel, flat to the bit. The click above will not do:
    // its samples alternate sign, so per channel it is only three frames long — and a
    // three-frame plateau is what the crest of a clean bass note looks like, which is
    // exactly what De-Clip's detector now (correctly) refuses to touch (audit
    // 2026-07-12). A probe for a restoration tool has to carry the damage it restores.
    for s in samples.iter_mut().skip(3_000).take(20) {
        *s = 0.97;
    }
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
                clip.render_tail_effect(&fx)
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
/// A tiny synthetic room for the rack gates — the Convolution Reverb's precondition.
fn seed_room() {
    let ir: Vec<f32> = (0..2_000)
        .map(|i| {
            let t = i as f32 / 2_000.0;
            ((i * 37 % 101) as f32 / 50.0 - 1.0) * (1.0 - t) * (1.0 - t)
        })
        .collect();
    super::super::editor::ir::set(ir, 1, 48_000, "test-room");
}

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
    // The Convolution Reverb needs a ROOM as well as a wet Mix — with no impulse response
    // it is bypassed by design, whatever the knob says (there is nothing to convolve with,
    // and convolving with an empty buffer would silence the clip rather than reverberate
    // it). It is the one effect in the rack whose arming knob is not sufficient on its own,
    // so the probe hands it a room: a short decaying burst, which is what a room is.
    seed_room();
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
                    clip.render_tail_effect(&fx)
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

/// **The transcription check for ADR-0117 D2** — run it before the refactor, run it after, diff.
///
/// D2 changes how every effect *builds* its output buffer (one allocation, instead of a `Vec` that
/// is then copied into an `Arc`). It must not change a single sample any of them **computes**. The
/// arithmetic is untouched by construction — the same loop, in the same order, writing into a
/// different container — so the only real risk across 69 sites is a **slip of the hand**, and a
/// before/after diff catches that completely.
///
/// Deliberately a printout and not a pinned golden: the editor's DSP uses transcendentals
/// (`tanh`/`sin`/`exp` — HR-5 does not apply off the RT thread), and those can differ in the last
/// ulp between platform libms. Pinned hashes would be a CI landmine on the linux/macOS/windows
/// matrix, red for a reason that has nothing to do with the code.
///
/// ```text
/// cargo test -p ph2d-desktop --release --lib audio::fx_params::tests::the_rack_fingerprint \
///   -- --nocapture
/// ```
#[test]
fn the_rack_fingerprint() {
    /// FNV-1a over the sample BITS — a sign flip on a zero is a real difference, and a NaN must
    /// compare equal to itself.
    fn digest(samples: &[f32]) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for s in samples {
            h ^= u64::from(s.to_bits());
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    let d = probe();
    seed_room();
    println!(
        "\n=== ADR-0117 rack fingerprint ({} effects) ===",
        KINDS.len()
    );
    for (kind, k) in KINDS.iter().enumerate() {
        // The first arming knob, at the far end of its travel: the setting the rack's own
        // "the arm wakes it" gate already trusts to make each effect audible.
        let arm = k.arms.first().copied().unwrap_or(0);
        let clip = EditClip::new(d.clone());
        let out = match build(kind, &norms_with(kind, arm)).expect("builds") {
            FxCommand::Plain(fx) => clip.render_effect(fx),
            FxCommand::Tail(fx) => clip.render_tail_effect(&fx),
        };
        println!(
            "{:<24} {:>7} frames  {:016x}",
            k.name,
            out.frame_count(),
            digest(out.samples())
        );
    }
    println!();
}

/// **Exactly one effect wants a room, and it is the right one.**
///
/// `needs_ir` is DERIVED (ask the table what it builds) rather than declared in a column,
/// which is what keeps the other 38 rows from carrying a `false` to describe one. The price
/// of deriving is that a mistake is silent: the Load IR button would simply appear on the
/// wrong effect, or on none — and a Convolution Reverb with no way to load a room is a
/// reverb that can never be switched on.
#[test]
fn exactly_one_effect_wants_a_room() {
    let wanting: Vec<&str> = KINDS
        .iter()
        .enumerate()
        .filter(|(k, _)| needs_ir(*k))
        .map(|(_, k)| k.name)
        .collect();
    assert_eq!(
        wanting,
        ["Conv Reverb"],
        "the Load IR button follows this list, and it is wrong"
    );
}

/// **No room = bypass, however wet the Mix.** Convolving with an empty buffer would not
/// reverberate the clip, it would SILENCE it — so the effect refuses instead of pretending.
/// This is the rack-level half of what `conv.rs` proves in the DSP.
#[test]
fn a_convolution_with_no_room_is_bypassed() {
    let kind = KINDS.iter().position(|k| k.name == "Conv Reverb").unwrap();
    super::super::editor::ir::set(Vec::new(), 1, 48_000, "");
    let mut norms = default_norms(kind);
    norms[0] = 1.0; // fully wet
    let cmd = build(kind, &norms).expect("builds");
    assert!(
        cmd.is_bypass(),
        "a fully-wet convolution with NO impulse response is not bypassed — it would \
             convolve with nothing and silence the clip"
    );
    // …and with a room, the same knob wakes it. (Otherwise the assertion above would be
    // satisfied by an effect that is simply always dead.)
    seed_room();
    assert!(!build(kind, &norms).expect("builds").is_bypass());
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
