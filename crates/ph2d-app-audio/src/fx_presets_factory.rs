//! **Os PRESETS DE FÁBRICA do rack** — os dados; as leis (resolver, gravar, ler) ficam em
//! [`super::fx_presets`].
//!
//! ⚠️ Cortado de `fx_presets.rs` em 2026-09-16, quando a migração para a tabela de strings o levou
//! a 732 linhas (tecto 700): o próprio cabeçalho dele já nomeava esta cura desde 631 — *«a factory-
//! preset DATA table — the proper fix is to split the `static` presets into a sibling `data`
//! module»*. Cada estágio nomeia o efeito pelo `FxKind::id` estável e cada sobreposição o parâmetro
//! pela `const` de chave; os nomes dos presets são chaves `audio.fx.preset.*`.

use ph2d_i18n::TextKey;

use super::fx_param_keys as p;

/// One parameter override in a factory preset, in **real DSP units**, keyed by the
/// parameter's display label (`"Cutoff"`, `"Ratio"`, …). Unlisted params keep the
/// effect's neutral default, so a preset only spells out what it actually changes.
pub(crate) struct Ovr {
    pub(crate) label: TextKey,
    pub(crate) value: f32,
}

const fn ovr(label: TextKey, value: f32) -> Ovr {
    Ovr { label, value }
}

/// One stage of a factory preset: an effect (by its stable `FxKind::id`) + its overrides.
pub(crate) struct FStage {
    pub(crate) effect: &'static str,
    pub(crate) params: &'static [Ovr],
}

const fn stage(effect: &'static str, params: &'static [Ovr]) -> FStage {
    FStage { effect, params }
}

/// A named factory chain.
pub(crate) struct Preset {
    pub(crate) name: TextKey,
    pub(crate) stages: &'static [FStage],
}

// The curated set. Each is ≤ MAX_FX_STAGES stages and does something real at these
// values — `factory_presets_are_audible` proves none resolves to an all-neutral
// (silent) chain. Effect and label names are checked by `every_factory_stage_resolves`.
static VOICE_CLEANUP: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 90.0)]),
    stage("de_esser", &[ovr(p::FREQ, 6_500.0), ovr(p::RATIO, 4.0)]),
    stage("compress", &[ovr(p::THRESHOLD, 0.2), ovr(p::RATIO, 3.0)]),
    stage("limiter", &[ovr(p::CEILING, -1.0)]),
];
static PODCAST: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 80.0)]),
    stage("compress", &[ovr(p::THRESHOLD, 0.15), ovr(p::RATIO, 4.0)]),
    stage("high_shelf", &[ovr(p::FREQ, 6_000.0), ovr(p::GAIN, 3.0)]),
    stage("limiter", &[ovr(p::CEILING, -1.5)]),
];
static TELEPHONE: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 400.0)]),
    stage("low_pass", &[ovr(p::CUTOFF, 3_400.0)]),
    stage("saturate", &[ovr(p::DRIVE, 5.0)]),
    stage("bitcrush", &[ovr(p::BITS, 10.0)]),
];
// A squad radio: band-limited harder than a phone, with a honking mid resonance (a
// cheap comms speaker), squashed flat by its AGC, and grit from the transmitter. No
// bitcrush — that reads as digital; a radio reads as gritty and compressed.
static RADIO: [FStage; 5] = [
    stage("high_pass", &[ovr(p::CUTOFF, 400.0)]),
    stage("low_pass", &[ovr(p::CUTOFF, 3_000.0)]),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 1_800.0), ovr(p::Q, 2.0), ovr(p::GAIN, 8.0)],
    ),
    stage("compress", &[ovr(p::THRESHOLD, 0.08), ovr(p::RATIO, 8.0)]),
    stage("saturate", &[ovr(p::DRIVE, 7.0)]),
];
// Inside a helmet (spacesuit / pilot / diving): a boxy low-mid resonance from the
// enclosed shell, muffled highs the shell absorbs, and a tight short reverb — the
// reflection of the tiny cavity is what actually sells "helmet".
static HELMET: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 120.0)]),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 500.0), ovr(p::Q, 2.5), ovr(p::GAIN, 9.0)],
    ),
    stage("low_pass", &[ovr(p::CUTOFF, 4_000.0)]),
    stage(
        "reverb",
        &[
            ovr(p::ROOM, 0.3),
            ovr(p::DAMP, 0.6),
            ovr(p::MIX, 0.28),
            ovr(p::TAIL, 0.4),
        ],
    ),
];
static LO_FI: [FStage; 3] = [
    stage("bitcrush", &[ovr(p::BITS, 8.0), ovr(p::DOWNSAMPLE, 2.0)]),
    stage("low_pass", &[ovr(p::CUTOFF, 6_000.0)]),
    stage("widen", &[ovr(p::WIDTH, 1.3)]),
];
static MASTER_BUS: [FStage; 4] = [
    stage("low_shelf", &[ovr(p::FREQ, 120.0), ovr(p::GAIN, -1.5)]),
    stage("peak_eq", &[ovr(p::FREQ, 3_000.0), ovr(p::GAIN, 2.0)]),
    stage("compress", &[ovr(p::THRESHOLD, 0.4), ovr(p::RATIO, 2.0)]),
    stage("limiter", &[ovr(p::CEILING, -1.0)]),
];
static WIDE_AND_BRIGHT: [FStage; 3] = [
    stage("high_shelf", &[ovr(p::FREQ, 8_000.0), ovr(p::GAIN, 3.0)]),
    stage("widen", &[ovr(p::WIDTH, 1.4)]),
    stage("limiter", &[ovr(p::CEILING, -1.0)]),
];
static GATE_TIGHTEN: [FStage; 2] = [
    stage(
        "gate_expander",
        &[ovr(p::THRESHOLD, 0.04), ovr(p::RATIO, 8.0)],
    ),
    stage("compress", &[ovr(p::THRESHOLD, 0.3), ovr(p::RATIO, 3.0)]),
];
// Character presets that lean on the newer effects (ring mod, distortion, exciter,
// trance gate, ping-pong, chorus, vibrato, auto-pan).
static ROBOT: [FStage; 3] = [
    stage("ring_mod", &[ovr(p::FREQ, 200.0), ovr(p::MIX, 0.7)]),
    stage("distortion", &[ovr(p::DRIVE, 0.4), ovr(p::TONE, 0.5)]),
    stage("bitcrush", &[ovr(p::BITS, 8.0)]),
];
static MEGAPHONE: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 500.0)]),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 1_800.0), ovr(p::Q, 3.0), ovr(p::GAIN, 8.0)],
    ),
    stage("distortion", &[ovr(p::DRIVE, 0.5), ovr(p::TONE, 0.6)]),
    stage("low_pass", &[ovr(p::CUTOFF, 4_000.0)]),
];
static UNDERWATER: [FStage; 3] = [
    stage("low_pass", &[ovr(p::CUTOFF, 1_200.0)]),
    stage("chorus", &[ovr(p::RATE, 0.5), ovr(p::MIX, 0.6)]),
    stage(
        "reverb",
        &[ovr(p::ROOM, 0.6), ovr(p::MIX, 0.4), ovr(p::TAIL, 3.0)],
    ),
];
static SCI_FI_COMM: [FStage; 3] = [
    stage("ring_mod", &[ovr(p::FREQ, 1_200.0), ovr(p::MIX, 0.3)]),
    stage("trance_gate", &[ovr(p::RATE, 10.0), ovr(p::DEPTH, 0.7)]),
    stage(
        "ping_pong",
        &[ovr(p::TIME, 0.18), ovr(p::FEEDBACK, 0.5), ovr(p::MIX, 0.4)],
    ),
];
static AIR: [FStage; 2] = [
    stage("exciter", &[ovr(p::FREQ, 4_000.0), ovr(p::AMOUNT, 0.6)]),
    stage("high_shelf", &[ovr(p::FREQ, 10_000.0), ovr(p::GAIN, 3.0)]),
];
static WOBBLE: [FStage; 2] = [
    stage("vibrato", &[ovr(p::RATE, 5.0), ovr(p::DEPTH, 4.0)]),
    stage("auto_pan", &[ovr(p::RATE, 2.0), ovr(p::DEPTH, 0.7)]),
];
// The voice chain the W4 plan asked for as a preset rather than an effect: the five
// filters already exist, so "EQ for a voice" is data, not DSP. Roll off the rumble,
// scoop the boxiness a close mic adds, lift the presence band the consonants live in,
// and open the air on top.
static VOICE_EQ: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 80.0)]),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 300.0), ovr(p::Q, 1.2), ovr(p::GAIN, -4.0)],
    ),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 3_000.0), ovr(p::Q, 1.0), ovr(p::GAIN, 3.0)],
    ),
    stage("high_shelf", &[ovr(p::FREQ, 10_000.0), ovr(p::GAIN, 3.0)]),
];
// Breath with no body behind it: cut the chest, exaggerate the air the exciter makes,
// and squash the dynamics flat so every consonant sits right against the ear.
// The vocoder's two ends, as presets rather than rows -- which is the whole point of `Breath`.
// "Robot" (above) is the CHEAP robot: ring mod + grit, metallic and inharmonic. This is the other
// one, the Kraftwerk one: the voice's formants carried by a sawtooth, so the words survive and the
// pitch is the machine's.
static VOCODER_ROBOT: [FStage; 2] = [
    stage(
        "vocoder",
        &[
            ovr(p::CARRIER, 110.0),
            ovr(p::BANDS, 20.0),
            ovr(p::MIX, 1.0),
        ],
    ),
    stage("high_shelf", &[ovr(p::FREQ, 6_000.0), ovr(p::GAIN, 4.0)]),
];
// ...and with a NOISE carrier the same engine whispers, because unvoiced excitation through a
// vocal tract is what whispering physically is. (The EQ-built "Whisper" below is the other
// approach -- it thins and excites a voice that is still phonating; this one actually unvoices it.)
static VOCODER_WHISPER: [FStage; 2] = [
    stage(
        "vocoder",
        &[ovr(p::BANDS, 24.0), ovr(p::BREATH, 1.0), ovr(p::MIX, 1.0)],
    ),
    stage("high_pass", &[ovr(p::CUTOFF, 200.0)]),
];
static WHISPER: [FStage; 4] = [
    stage("high_pass", &[ovr(p::CUTOFF, 250.0)]),
    stage("exciter", &[ovr(p::FREQ, 4_000.0), ovr(p::AMOUNT, 0.9)]),
    stage("compress", &[ovr(p::THRESHOLD, 0.05), ovr(p::RATIO, 8.0)]),
    stage("high_shelf", &[ovr(p::FREQ, 9_000.0), ovr(p::GAIN, 8.0)]),
];
// The other end of the same axis: everything forward and strained. Hard compression
// brings the whole take to the front, the 2.5 kHz shout band is where a raised voice
// actually gets loud, and a little grit is the throat giving out.
static SHOUT: [FStage; 4] = [
    stage("compress", &[ovr(p::THRESHOLD, 0.06), ovr(p::RATIO, 8.0)]),
    stage(
        "peak_eq",
        &[ovr(p::FREQ, 2_500.0), ovr(p::Q, 1.5), ovr(p::GAIN, 7.0)],
    ),
    stage("distortion", &[ovr(p::DRIVE, 0.25), ovr(p::TONE, 0.6)]),
    stage("limiter", &[ovr(p::CEILING, -1.0)]),
];
// The restoration chain — and the one-click demo of the de-clicker: take out the
// ticks, then the mains buzz under them, then the sibilance the repair leaves standing.
static RESTORE: [FStage; 4] = [
    stage(
        "de_click",
        &[ovr(p::SENSITIVITY, 0.6), ovr(p::WIDTH, 0.001)],
    ),
    stage("de_hum", &[ovr(p::FREQ, 50.0), ovr(p::DEPTH, 0.8)]),
    stage("de_esser", &[ovr(p::FREQ, 6_500.0), ovr(p::RATIO, 3.0)]),
    stage("limiter", &[ovr(p::CEILING, -1.0)]),
];
// The formant shifter's demo, and the thing no pitch shifter can fake: the same
// performance, at the same pitch, out of a much bigger head. Shift the vocal tract
// down, add the chest the bigger body would have, and glue it.
static GIANT: [FStage; 3] = [
    stage("formant_shift", &[ovr(p::SHIFT, -7.0), ovr(p::MIX, 1.0)]),
    stage("low_shelf", &[ovr(p::FREQ, 200.0), ovr(p::GAIN, 4.0)]),
    stage("compress", &[ovr(p::THRESHOLD, 0.2), ovr(p::RATIO, 3.0)]),
];
// The harmonizer's demo: one take, three notes, one room.
static CHOIR: [FStage; 2] = [
    stage(
        "harmonizer",
        &[ovr(p::VOICE_1, 4.0), ovr(p::VOICE_2, 7.0), ovr(p::MIX, 0.6)],
    ),
    stage(
        "reverb",
        &[ovr(p::ROOM, 0.7), ovr(p::MIX, 0.3), ovr(p::TAIL, 2.5)],
    ),
];

pub(crate) static FACTORY: [Preset; 23] = [
    Preset {
        name: TextKey::new("audio.fx.preset.voice_cleanup"),
        stages: &VOICE_CLEANUP,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.podcast"),
        stages: &PODCAST,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.telephone"),
        stages: &TELEPHONE,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.radio"),
        stages: &RADIO,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.helmet"),
        stages: &HELMET,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.lo_fi"),
        stages: &LO_FI,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.master_bus"),
        stages: &MASTER_BUS,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.wide_and_bright"),
        stages: &WIDE_AND_BRIGHT,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.gate_glue"),
        stages: &GATE_TIGHTEN,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.robot"),
        stages: &ROBOT,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.vocoder"),
        stages: &VOCODER_ROBOT,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.vocoder_whisper"),
        stages: &VOCODER_WHISPER,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.megaphone"),
        stages: &MEGAPHONE,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.underwater"),
        stages: &UNDERWATER,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.sci_fi_comm"),
        stages: &SCI_FI_COMM,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.air"),
        stages: &AIR,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.wobble"),
        stages: &WOBBLE,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.voice_eq"),
        stages: &VOICE_EQ,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.whisper"),
        stages: &WHISPER,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.shout"),
        stages: &SHOUT,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.restore"),
        stages: &RESTORE,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.giant"),
        stages: &GIANT,
    },
    Preset {
        name: TextKey::new("audio.fx.preset.choir"),
        stages: &CHOIR,
    },
];
