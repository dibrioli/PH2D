//! The rack's effect table: one row per effect (name · parameter specs ·
//! constructor · arming knobs). Split out of `fx_params.rs` to keep that file
//! under the HR-18 600-LOC shell cap; the mapping/formatting logic lives there.
//!
//! Adding an effect is one entry here plus one `Effect` variant. There is nothing
//! to keep in sync: the name the panel paints, the sliders it shows and the DSP the
//! shell builds all come out of the same row.

use ph2d_audio_edit::{Effect, TailEffect};

use super::fx_params::{FxCommand, FxKind, FxParamSpec, spec};

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
// Depth is the arm (0 = 0 dB cut = pass-through). Freq covers both mains standards.
static DE_HUM: [FxParamSpec; 3] = [
    spec("Freq", 50.0, 60.0, false, 50.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
    spec("Harmonics", 1.0, 8.0, false, 4.0, "", true),
];
static COMPRESS: [FxParamSpec; 4] = [
    spec("Threshold", 0.01, 1.0, true, 0.3, "", false),
    // Neutral at 1:1 — no reduction at all (make-up is peak-preserving, so it
    // collapses to unity too).
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    spec("Attack", 0.001, 0.2, true, 0.005, "s", false),
    spec("Release", 0.01, 1.0, true, 0.1, "s", false),
];
static GATE: [FxParamSpec; 4] = [
    spec("Threshold", 0.001, 0.5, true, 0.05, "", false),
    // Neutral at 1:1 — `(level/threshold)^0` is unity at every level.
    spec("Ratio", 1.0, 20.0, false, 1.0, "x", false),
    // On a gate, Attack OPENS and Release CLOSES (a compressor's are the other way).
    spec("Attack", 0.0005, 0.05, true, 0.002, "s", false),
    spec("Release", 0.005, 1.0, true, 0.1, "s", false),
];
static DE_ESSER: [FxParamSpec; 4] = [
    spec("Freq", 2_000.0, 12_000.0, true, 6_000.0, "Hz", false),
    spec("Threshold", 0.005, 0.5, true, 0.05, "", false),
    // Neutral at 1:1 — a 0 dB shelf.
    spec("Ratio", 1.0, 10.0, false, 1.0, "x", false),
    // Attack is fixed fast inside the effect: sibilance is a transient.
    spec("Release", 0.005, 0.3, true, 0.05, "s", false),
];
// Mirror of the de-esser on the low "pop" band. Neutral at 1:1 (a 0 dB low-shelf).
static DE_PLOSIVE: [FxParamSpec; 4] = [
    spec("Freq", 60.0, 250.0, true, 120.0, "Hz", false),
    spec("Threshold", 0.005, 0.5, true, 0.05, "", false),
    spec("Ratio", 1.0, 10.0, false, 1.0, "x", false),
    // Attack is fixed fast inside the effect: a plosive is a transient.
    spec("Release", 0.01, 0.4, true, 0.08, "s", false),
];
static LIMITER: [FxParamSpec; 2] = [
    // Neutral at the TOP: a ceiling at 0 dBFS has nothing to catch. −1 dBTP is the
    // mastering convention; that is one notch down from neutral.
    spec("Ceiling", -12.0, 0.0, false, 0.0, "dB", false),
    // Doubles as the look-ahead: the gain dips this far ahead of every peak.
    spec("Release", 0.002, 0.2, true, 0.02, "s", false),
];
// Amount is the arm (0 = unity gain = pass-through). Speed maps log (a time param).
static LEVELER: [FxParamSpec; 3] = [
    spec("Target", -30.0, -6.0, false, -18.0, "dB", false),
    spec("Amount", 0.0, 1.0, false, 0.0, "", false),
    spec("Speed", 0.05, 2.0, true, 0.5, "s", false),
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
// The four modulation effects. Each is neutral at Mix (or Depth) 0 — fully dry.
static CHORUS: [FxParamSpec; 3] = [
    spec("Rate", 0.05, 8.0, true, 1.0, "Hz", false),
    spec("Depth", 0.5, 15.0, false, 5.0, "ms", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
static FLANGER: [FxParamSpec; 4] = [
    spec("Rate", 0.05, 8.0, true, 0.4, "Hz", false),
    spec("Depth", 0.2, 8.0, false, 3.0, "ms", false),
    spec("Feedback", 0.0, 0.95, false, 0.4, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
static PHASER: [FxParamSpec; 3] = [
    spec("Rate", 0.05, 8.0, true, 0.5, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.8, "", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Depth (not a Mix) is the arm: at 0 the gain stays at 1.0.
static TREMOLO: [FxParamSpec; 2] = [
    spec("Rate", 0.1, 16.0, true, 5.0, "Hz", false),
    spec("Depth", 0.0, 1.0, false, 0.0, "", false),
];
// Mix is the arm (0 = dry = pass-through). Freq is the carrier, log-swept across the
// low-mid band where the robot/metallic character lives.
static RING_MOD: [FxParamSpec; 2] = [
    spec("Freq", 20.0, 4_000.0, true, 500.0, "Hz", false),
    spec("Mix", 0.0, 1.0, false, 0.0, "", false),
];
// Semitones is the arm — neutral at 0 (linear so 0 lands exactly at slider centre).
// Mix stays fully wet by default (the pitched voice), pull down to blend a detune.
static PITCH_SHIFT: [FxParamSpec; 2] = [
    spec("Semitones", -12.0, 12.0, false, 0.0, "st", false),
    spec("Mix", 0.0, 1.0, false, 1.0, "", false),
];

/// The effects the selector cycles, in order — grouped tone → dynamics → character
/// → space, the way a rack is laid out.
pub(crate) static KINDS: [FxKind; 23] = [
    FxKind {
        name: "Low-Pass",
        params: &LOW_PASS,
        arms: &[0], // Cutoff
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
        arms: &[0], // Cutoff
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
        arms: &[2], // Gain
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
        arms: &[2], // Gain
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
        arms: &[2], // Gain
        build: |v| {
            FxCommand::Plain(Effect::HighShelf {
                freq: v[0],
                q: v[1],
                gain_db: v[2],
            })
        },
    },
    FxKind {
        name: "De-Hum",
        params: &DE_HUM,
        arms: &[1], // Depth
        build: |v| {
            FxCommand::Plain(Effect::DeHum {
                freq: v[0],
                depth: v[1],
                harmonics: v[2] as u32,
            })
        },
    },
    FxKind {
        name: "Compress",
        params: &COMPRESS,
        arms: &[1], // Ratio
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
        name: "Gate",
        params: &GATE,
        arms: &[1], // Ratio
        build: |v| {
            FxCommand::Plain(Effect::Gate {
                threshold: v[0],
                ratio: v[1],
                attack_secs: v[2],
                release_secs: v[3],
            })
        },
    },
    FxKind {
        name: "De-Esser",
        params: &DE_ESSER,
        arms: &[2], // Ratio
        build: |v| {
            FxCommand::Plain(Effect::DeEss {
                freq: v[0],
                threshold: v[1],
                ratio: v[2],
                release_secs: v[3],
            })
        },
    },
    FxKind {
        name: "De-Plosive",
        params: &DE_PLOSIVE,
        arms: &[2], // Ratio
        build: |v| {
            FxCommand::Plain(Effect::DePlosive {
                freq: v[0],
                threshold: v[1],
                ratio: v[2],
                release_secs: v[3],
            })
        },
    },
    FxKind {
        name: "Limiter",
        params: &LIMITER,
        arms: &[0], // Ceiling
        build: |v| {
            FxCommand::Plain(Effect::Limiter {
                ceiling_db: v[0],
                release_secs: v[1],
            })
        },
    },
    FxKind {
        name: "Leveler",
        params: &LEVELER,
        arms: &[1], // Amount
        build: |v| {
            FxCommand::Plain(Effect::Leveler {
                target_db: v[0],
                amount: v[1],
                speed_secs: v[2],
            })
        },
    },
    FxKind {
        name: "Saturate",
        params: &SATURATE,
        arms: &[0], // Drive
        build: |v| FxCommand::Plain(Effect::Saturate { drive: v[0] }),
    },
    FxKind {
        name: "Bitcrush",
        params: &BITCRUSH,
        arms: &[0, 1], // Bits or Downsample
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
        arms: &[0], // Width
        build: |v| FxCommand::Plain(Effect::StereoWidth { width: v[0] }),
    },
    FxKind {
        name: "Reverb",
        params: &REVERB,
        arms: &[2], // Mix
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
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Tail(TailEffect::Delay {
                time_secs: v[0],
                feedback: v[1],
                mix: v[2],
                tail_secs: v[3],
            })
        },
    },
    // Modulation group.
    FxKind {
        name: "Chorus",
        params: &CHORUS,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Chorus {
                rate: v[0],
                depth_ms: v[1],
                mix: v[2],
            })
        },
    },
    FxKind {
        name: "Flanger",
        params: &FLANGER,
        arms: &[3], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Flanger {
                rate: v[0],
                depth_ms: v[1],
                feedback: v[2],
                mix: v[3],
            })
        },
    },
    FxKind {
        name: "Phaser",
        params: &PHASER,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Phaser {
                rate: v[0],
                depth: v[1],
                mix: v[2],
            })
        },
    },
    FxKind {
        name: "Tremolo",
        params: &TREMOLO,
        arms: &[1], // Depth
        build: |v| {
            FxCommand::Plain(Effect::Tremolo {
                rate: v[0],
                depth: v[1],
            })
        },
    },
    FxKind {
        name: "Ring Mod",
        params: &RING_MOD,
        arms: &[1], // Mix
        build: |v| {
            FxCommand::Plain(Effect::RingMod {
                freq: v[0],
                mix: v[1],
            })
        },
    },
    FxKind {
        name: "Pitch Shift",
        params: &PITCH_SHIFT,
        arms: &[0], // Semitones
        build: |v| {
            FxCommand::Plain(Effect::PitchShift {
                semitones: v[0],
                mix: v[1],
            })
        },
    },
];
