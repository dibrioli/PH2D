//! The rack's effect table: one row per effect (name · parameter specs ·
//! constructor · arming knobs). Split out of `fx_params.rs` to keep that file
//! under the HR-18 600-LOC shell cap; the mapping/formatting logic lives there.
//!
//! Adding an effect is one entry here plus one `Effect` variant. There is nothing
//! to keep in sync: the name the panel paints, the sliders it shows and the DSP the
//! shell builds all come out of the same row.

use ph2d_audio_edit::{Effect, TailEffect};

use super::fx_param_specs::*;
use super::fx_params::{FxCommand, FxKind};

/// The effects the selector cycles, in order — grouped tone → dynamics → character
/// → space → modulation, the way a rack is laid out. Each row's `params` points at a
/// spec array in the sibling [`super::fx_param_specs`].
pub(crate) static KINDS: [FxKind; 39] = [
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
    // The rack's two restoration tools, next to the other "take the damage out" pair.
    // Both are gap-fillers over the same LSAR core: De-Click rebuilds what the signal could
    // not have meant, De-Clip rebuilds what it never got to write.
    FxKind {
        name: "De-Click",
        params: &DE_CLICK,
        arms: &[0], // Sensitivity
        build: |v| {
            FxCommand::Plain(Effect::DeClick {
                sensitivity: v[0],
                width_secs: v[1],
            })
        },
    },
    FxKind {
        name: "De-Clip",
        params: &DE_CLIP,
        arms: &[0], // Amount
        build: |v| {
            FxCommand::Plain(Effect::DeClip {
                threshold: v[1],
                amount: v[0],
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
        name: "Transient",
        params: &TRANSIENT,
        arms: &[0, 1], // Attack, Sustain
        build: |v| {
            FxCommand::Plain(Effect::Transient {
                attack: v[0],
                sustain: v[1],
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
        name: "Distortion",
        params: &DISTORTION,
        arms: &[0], // Drive
        build: |v| {
            FxCommand::Plain(Effect::Distortion {
                drive: v[0],
                tone: v[1],
            })
        },
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
        name: "Haas",
        params: &HAAS,
        arms: &[0], // Delay
        build: |v| FxCommand::Plain(Effect::Haas { delay_ms: v[0] }),
    },
    FxKind {
        name: "Exciter",
        params: &EXCITER,
        arms: &[1], // Amount
        build: |v| {
            FxCommand::Plain(Effect::Exciter {
                freq: v[0],
                amount: v[1],
            })
        },
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
    // The rack's second reverb, and a different KIND of thing: Freeverb above is a plausible
    // room, modelled. This one is a measured one — an impulse response IS a room, and the only
    // knob is how much of it you want. The IR itself is a resource, not a parameter: `build`
    // runs in the shell, so it bakes the loaded room into the effect value (see `editor::ir`).
    FxKind {
        name: "Conv Reverb",
        params: &CONV_REVERB,
        arms: &[0], // Mix
        build: |v| {
            FxCommand::Tail(TailEffect::Convolution {
                ir: super::editor::ir::samples(),
                ir_channels: super::editor::ir::channels(),
                ir_rate: super::editor::ir::rate(),
                mix: v[0],
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
    FxKind {
        name: "Ping-Pong",
        params: &PING_PONG,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Tail(TailEffect::PingPong {
                time_secs: v[0],
                feedback: v[1],
                mix: v[2],
                tail_secs: v[3],
            })
        },
    },
    FxKind {
        name: "Comb",
        params: &COMB,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Comb {
                freq: v[0],
                feedback: v[1],
                mix: v[2],
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
        name: "Vibrato",
        params: &VIBRATO,
        arms: &[1], // Depth
        build: |v| {
            FxCommand::Plain(Effect::Vibrato {
                rate: v[0],
                depth_ms: v[1],
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
        name: "Auto-Wah",
        params: &AUTO_WAH,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::AutoWah {
                base_freq: v[0],
                sens: v[1],
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
        name: "Auto-Pan",
        params: &AUTO_PAN,
        arms: &[1], // Depth
        build: |v| {
            FxCommand::Plain(Effect::AutoPan {
                rate: v[0],
                depth: v[1],
            })
        },
    },
    FxKind {
        name: "Trance Gate",
        params: &TRANCE_GATE,
        arms: &[1], // Depth
        build: |v| {
            FxCommand::Plain(Effect::TranceGate {
                rate: v[0],
                depth: v[1],
                smooth_secs: v[2],
            })
        },
    },
    FxKind {
        name: "Doubler",
        params: &DOUBLER,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Doubler {
                delay_ms: v[0],
                detune_ms: v[1],
                mix: v[2],
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
    // The voice group: what the pitch shifter cannot do alone.
    FxKind {
        name: "Formant Shift",
        params: &FORMANT_SHIFT,
        arms: &[0], // Shift
        build: |v| {
            FxCommand::Plain(Effect::FormantShift {
                semitones: v[0],
                mix: v[1],
            })
        },
    },
    FxKind {
        name: "Harmonizer",
        params: &HARMONIZER,
        arms: &[2], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Harmonizer {
                v1_st: v[0],
                v2_st: v[1],
                mix: v[2],
            })
        },
    },
];
