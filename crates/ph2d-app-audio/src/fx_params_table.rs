//! The rack's effect table: one row per effect (name · parameter specs ·
//! constructor · arming knobs). Split out of `fx_params.rs` to keep that file
//! under the HR-18 600-LOC shell cap; the mapping/formatting logic lives there.
//!
//! Adding an effect is one entry here plus one `Effect` variant. There is nothing
//! to keep in sync: the name the panel paints, the sliders it shows and the DSP the
//! shell builds all come out of the same row.

use ph2d_audio_edit::{Effect, TailEffect};
use ph2d_i18n::TextKey;

use super::fx_param_specs::*;
use super::fx_params::{FxCommand, FxKind};

/// The effects the selector cycles, in order — grouped tone → dynamics → character
/// → space → modulation, the way a rack is laid out. Each row's `params` points at a
/// spec array in the sibling [`super::fx_param_specs`].
pub static KINDS: [FxKind; 42] = [
    FxKind {
        id: "low_pass",
        name: TextKey::new("audio.fx.kind.low_pass"),
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
        id: "high_pass",
        name: TextKey::new("audio.fx.kind.high_pass"),
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
        id: "peak_eq",
        name: TextKey::new("audio.fx.kind.peak_eq"),
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
        id: "low_shelf",
        name: TextKey::new("audio.fx.kind.low_shelf"),
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
        id: "high_shelf",
        name: TextKey::new("audio.fx.kind.high_shelf"),
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
        id: "de_hum",
        name: TextKey::new("audio.fx.kind.de_hum"),
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
        id: "compress",
        name: TextKey::new("audio.fx.kind.compress"),
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
        id: "multiband",
        name: TextKey::new("audio.fx.kind.multiband"),
        params: &MULTIBAND,
        arms: &[1], // Ratio — the same arm as Compress, because it is the same compressor.
        // Sits next to Compress on purpose: it is the one to reach for when the kick is
        // ducking the cymbals, and reading the two rows side by side is what says so.
        build: |v| {
            FxCommand::Plain(Effect::Multiband {
                threshold: v[0],
                ratio: v[1],
                attack_secs: v[2],
                release_secs: v[3],
            })
        },
    },
    FxKind {
        // One knob (Ratio) spans gentle downward expander → hard noise gate — the effect is
        // literally both, so the label says both (`gate()` doc in `ph2d-audio-edit`). Presets
        // saved as "Gate" still resolve via the legacy alias in `kind_by_name`.
        id: "gate_expander",
        name: TextKey::new("audio.fx.kind.gate_expander"),
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
        id: "de_esser",
        name: TextKey::new("audio.fx.kind.de_esser"),
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
        id: "de_plosive",
        name: TextKey::new("audio.fx.kind.de_plosive"),
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
        id: "de_click",
        name: TextKey::new("audio.fx.kind.de_click"),
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
        id: "de_clip",
        name: TextKey::new("audio.fx.kind.de_clip"),
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
        id: "limiter",
        name: TextKey::new("audio.fx.kind.limiter"),
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
        id: "leveler",
        name: TextKey::new("audio.fx.kind.leveler"),
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
        id: "transient",
        name: TextKey::new("audio.fx.kind.transient"),
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
        id: "saturate",
        name: TextKey::new("audio.fx.kind.saturate"),
        params: &SATURATE,
        arms: &[0], // Drive
        build: |v| FxCommand::Plain(Effect::Saturate { drive: v[0] }),
    },
    FxKind {
        id: "distortion",
        name: TextKey::new("audio.fx.kind.distortion"),
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
        id: "bitcrush",
        name: TextKey::new("audio.fx.kind.bitcrush"),
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
        id: "widen",
        name: TextKey::new("audio.fx.kind.widen"),
        params: &WIDEN,
        arms: &[0], // Width
        build: |v| FxCommand::Plain(Effect::StereoWidth { width: v[0] }),
    },
    FxKind {
        id: "haas",
        name: TextKey::new("audio.fx.kind.haas"),
        params: &HAAS,
        arms: &[0], // Delay
        build: |v| FxCommand::Plain(Effect::Haas { delay_ms: v[0] }),
    },
    FxKind {
        id: "exciter",
        name: TextKey::new("audio.fx.kind.exciter"),
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
        id: "reverb",
        name: TextKey::new("audio.fx.kind.reverb"),
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
        id: "conv_reverb",
        name: TextKey::new("audio.fx.kind.conv_reverb"),
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
        id: "echo",
        name: TextKey::new("audio.fx.kind.echo"),
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
        id: "ping_pong",
        name: TextKey::new("audio.fx.kind.ping_pong"),
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
        id: "comb",
        name: TextKey::new("audio.fx.kind.comb"),
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
        id: "chorus",
        name: TextKey::new("audio.fx.kind.chorus"),
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
        id: "flanger",
        name: TextKey::new("audio.fx.kind.flanger"),
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
        id: "vibrato",
        name: TextKey::new("audio.fx.kind.vibrato"),
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
        id: "phaser",
        name: TextKey::new("audio.fx.kind.phaser"),
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
        id: "auto_wah",
        name: TextKey::new("audio.fx.kind.auto_wah"),
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
        id: "tremolo",
        name: TextKey::new("audio.fx.kind.tremolo"),
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
        id: "auto_pan",
        name: TextKey::new("audio.fx.kind.auto_pan"),
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
        id: "trance_gate",
        name: TextKey::new("audio.fx.kind.trance_gate"),
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
        id: "doubler",
        name: TextKey::new("audio.fx.kind.doubler"),
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
        id: "ring_mod",
        name: TextKey::new("audio.fx.kind.ring_mod"),
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
        id: "pitch_shift",
        name: TextKey::new("audio.fx.kind.pitch_shift"),
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
        id: "formant_shift",
        name: TextKey::new("audio.fx.kind.formant_shift"),
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
        id: "vocoder",
        name: TextKey::new("audio.fx.kind.vocoder"),
        params: &VOCODER,
        arms: &[3], // Mix
        // Sits with the voice transforms: it is the one that throws the excitation away and
        // keeps only the vocal tract. Breath at 0 is the robot, at 1 the whisper.
        build: |v| {
            FxCommand::Plain(Effect::Vocoder {
                carrier_hz: v[0],
                bands: v[1] as u32,
                breath: v[2],
                mix: v[3],
            })
        },
    },
    FxKind {
        id: "granular",
        name: TextKey::new("audio.fx.kind.granular"),
        params: &GRANULAR,
        arms: &[3], // Mix
        build: |v| {
            FxCommand::Plain(Effect::Granular {
                grain_ms: v[0],
                scatter: v[1],
                pitch_st: v[2],
                mix: v[3],
            })
        },
    },
    FxKind {
        id: "harmonizer",
        name: TextKey::new("audio.fx.kind.harmonizer"),
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
