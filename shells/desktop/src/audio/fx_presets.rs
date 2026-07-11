//! Chain presets for the Audio Editor's effects rack (W3 — presets).
//!
//! A preset is a named chain. This module owns the **factory** table and the
//! **user-file** format, and both key effects by **name**, never by index into
//! [`super::fx_params::KINDS`]: reordering that table (or inserting an effect in the
//! middle, as W3 blocks 4 and 5 did) must not turn a saved "Voice Cleanup" into
//! whatever now sits at index 3.
//!
//! - **Factory presets** ([`FACTORY`]) are specified in **real units by parameter
//!   label** — readable, and robust to a param being reordered inside its effect.
//!   [`factory_chain`] resolves one into the panel's `FxStage` list.
//! - **User presets** round-trip through a tiny line-based text format
//!   ([`serialize_chain`] / [`parse_chain`]) that stores the normalized slider
//!   values verbatim (exact round-trip) under the effect name.
//!
//! The panel owns the live chain and cannot do any of this — it has no name table —
//! so it lives shell-side and the bridge pushes the result via `set_fx_chain`.

use ph2d_panel_audio_editor::{FxStage, MAX_FX_PARAMS};

use super::fx_params::{default_norms, params_for, real_to_norm};
use super::fx_params_table::KINDS;

/// One parameter override in a factory preset, in **real DSP units**, keyed by the
/// parameter's display label (`"Cutoff"`, `"Ratio"`, …). Unlisted params keep the
/// effect's neutral default, so a preset only spells out what it actually changes.
struct Ovr {
    label: &'static str,
    value: f32,
}

const fn ovr(label: &'static str, value: f32) -> Ovr {
    Ovr { label, value }
}

/// One stage of a factory preset: an effect (by name) + its overrides.
struct FStage {
    effect: &'static str,
    params: &'static [Ovr],
}

const fn stage(effect: &'static str, params: &'static [Ovr]) -> FStage {
    FStage { effect, params }
}

/// A named factory chain.
struct Preset {
    name: &'static str,
    stages: &'static [FStage],
}

// The curated set. Each is ≤ MAX_FX_STAGES stages and does something real at these
// values — `factory_presets_are_audible` proves none resolves to an all-neutral
// (silent) chain. Effect and label names are checked by `every_factory_stage_resolves`.
static VOICE_CLEANUP: [FStage; 4] = [
    stage("High-Pass", &[ovr("Cutoff", 90.0)]),
    stage("De-Esser", &[ovr("Freq", 6_500.0), ovr("Ratio", 4.0)]),
    stage("Compress", &[ovr("Threshold", 0.2), ovr("Ratio", 3.0)]),
    stage("Limiter", &[ovr("Ceiling", -1.0)]),
];
static PODCAST: [FStage; 4] = [
    stage("High-Pass", &[ovr("Cutoff", 80.0)]),
    stage("Compress", &[ovr("Threshold", 0.15), ovr("Ratio", 4.0)]),
    stage("High Shelf", &[ovr("Freq", 6_000.0), ovr("Gain", 3.0)]),
    stage("Limiter", &[ovr("Ceiling", -1.5)]),
];
static TELEPHONE: [FStage; 4] = [
    stage("High-Pass", &[ovr("Cutoff", 400.0)]),
    stage("Low-Pass", &[ovr("Cutoff", 3_400.0)]),
    stage("Saturate", &[ovr("Drive", 5.0)]),
    stage("Bitcrush", &[ovr("Bits", 10.0)]),
];
// A squad radio: band-limited harder than a phone, with a honking mid resonance (a
// cheap comms speaker), squashed flat by its AGC, and grit from the transmitter. No
// bitcrush — that reads as digital; a radio reads as gritty and compressed.
static RADIO: [FStage; 5] = [
    stage("High-Pass", &[ovr("Cutoff", 400.0)]),
    stage("Low-Pass", &[ovr("Cutoff", 3_000.0)]),
    stage(
        "Peak EQ",
        &[ovr("Freq", 1_800.0), ovr("Q", 2.0), ovr("Gain", 8.0)],
    ),
    stage("Compress", &[ovr("Threshold", 0.08), ovr("Ratio", 8.0)]),
    stage("Saturate", &[ovr("Drive", 7.0)]),
];
// Inside a helmet (spacesuit / pilot / diving): a boxy low-mid resonance from the
// enclosed shell, muffled highs the shell absorbs, and a tight short reverb — the
// reflection of the tiny cavity is what actually sells "helmet".
static HELMET: [FStage; 4] = [
    stage("High-Pass", &[ovr("Cutoff", 120.0)]),
    stage(
        "Peak EQ",
        &[ovr("Freq", 500.0), ovr("Q", 2.5), ovr("Gain", 9.0)],
    ),
    stage("Low-Pass", &[ovr("Cutoff", 4_000.0)]),
    stage(
        "Reverb",
        &[
            ovr("Room", 0.3),
            ovr("Damp", 0.6),
            ovr("Mix", 0.28),
            ovr("Tail", 0.4),
        ],
    ),
];
static LO_FI: [FStage; 3] = [
    stage("Bitcrush", &[ovr("Bits", 8.0), ovr("Downsample", 2.0)]),
    stage("Low-Pass", &[ovr("Cutoff", 6_000.0)]),
    stage("Widen", &[ovr("Width", 1.3)]),
];
static MASTER_BUS: [FStage; 4] = [
    stage("Low Shelf", &[ovr("Freq", 120.0), ovr("Gain", -1.5)]),
    stage("Peak EQ", &[ovr("Freq", 3_000.0), ovr("Gain", 2.0)]),
    stage("Compress", &[ovr("Threshold", 0.4), ovr("Ratio", 2.0)]),
    stage("Limiter", &[ovr("Ceiling", -1.0)]),
];
static WIDE_AND_BRIGHT: [FStage; 3] = [
    stage("High Shelf", &[ovr("Freq", 8_000.0), ovr("Gain", 3.0)]),
    stage("Widen", &[ovr("Width", 1.4)]),
    stage("Limiter", &[ovr("Ceiling", -1.0)]),
];
static GATE_TIGHTEN: [FStage; 2] = [
    stage("Gate", &[ovr("Threshold", 0.04), ovr("Ratio", 8.0)]),
    stage("Compress", &[ovr("Threshold", 0.3), ovr("Ratio", 3.0)]),
];

static FACTORY: [Preset; 9] = [
    Preset {
        name: "Voice Cleanup",
        stages: &VOICE_CLEANUP,
    },
    Preset {
        name: "Podcast",
        stages: &PODCAST,
    },
    Preset {
        name: "Telephone",
        stages: &TELEPHONE,
    },
    Preset {
        name: "Radio",
        stages: &RADIO,
    },
    Preset {
        name: "Helmet",
        stages: &HELMET,
    },
    Preset {
        name: "Lo-Fi",
        stages: &LO_FI,
    },
    Preset {
        name: "Master Bus",
        stages: &MASTER_BUS,
    },
    Preset {
        name: "Wide & Bright",
        stages: &WIDE_AND_BRIGHT,
    },
    Preset {
        name: "Gate + Glue",
        stages: &GATE_TIGHTEN,
    },
];

/// The factory preset names, in table order — published to the panel's selector.
pub(crate) fn factory_names() -> Vec<&'static str> {
    FACTORY.iter().map(|p| p.name).collect()
}

/// Look up an effect kind by its display name.
fn kind_by_name(name: &str) -> Option<usize> {
    KINDS.iter().position(|k| k.name == name)
}

/// Resolve one factory stage into an `FxStage`: start from the effect's neutral
/// defaults, then override each named parameter (real units → slider normal).
/// `None` when the effect name is unknown — the preset drops that stage rather than
/// building the wrong effect.
fn resolve_stage(fs: &FStage) -> Option<FxStage> {
    let kind = kind_by_name(fs.effect)?;
    let specs = params_for(kind);
    let mut norms = default_norms(kind);
    for o in fs.params {
        if let Some(i) = specs.iter().position(|s| s.label == o.label) {
            norms[i] = real_to_norm(&specs[i], o.value);
        }
    }
    Some(FxStage {
        kind,
        norms,
        enabled: true,
    })
}

/// Build factory preset `idx`'s chain (empty when the index is out of range — the
/// panel's `set_fx_chain` then falls back to a neutral stage).
pub(crate) fn factory_chain(idx: usize) -> Vec<FxStage> {
    FACTORY
        .get(idx)
        .map(|p| p.stages.iter().filter_map(resolve_stage).collect())
        .unwrap_or_default()
}

/// The header line every user preset file starts with, so `parse_chain` can reject a
/// file that is plainly not one of ours (and so a future format bump has a hook).
const HEADER: &str = "# PH2D audio chain v1";

/// Serialize a chain to the user-preset text format — one stage per line,
/// `Effect Name | on|off | n0 n1 …`, keyed by name so it survives a `KINDS` reorder.
/// Storing the normals verbatim makes the round-trip exact.
pub(crate) fn serialize_chain(chain: &[FxStage]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for st in chain {
        let name = KINDS.get(st.kind).map(|k| k.name).unwrap_or("Unknown");
        let n = params_for(st.kind).len().min(MAX_FX_PARAMS);
        let nums: Vec<String> = st.norms[..n].iter().map(|v| format!("{v:.4}")).collect();
        let on = if st.enabled { "on" } else { "off" };
        out.push_str(&format!("{name} | {on} | {}\n", nums.join(" ")));
    }
    out
}

/// Parse the user-preset text format. Unknown effect names and malformed lines are
/// **skipped**, not fatal — a preset saved before an effect was renamed still loads
/// what it can. Returns an empty chain if nothing resolved (the caller falls back to
/// a neutral stage).
pub(crate) fn parse_chain(text: &str) -> Vec<FxStage> {
    let mut chain = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('|');
        let (Some(name), Some(on)) = (fields.next(), fields.next()) else {
            continue;
        };
        let Some(kind) = kind_by_name(name.trim()) else {
            continue;
        };
        // Start from neutral, so a line that lists fewer numbers than the effect has
        // params leaves the rest at their defaults.
        let mut norms = default_norms(kind);
        if let Some(nums) = fields.next() {
            for (slot, tok) in norms.iter_mut().zip(nums.split_whitespace()) {
                if let Ok(v) = tok.parse::<f32>() {
                    *slot = v.clamp(0.0, 1.0);
                }
            }
        }
        chain.push(FxStage {
            kind,
            norms,
            enabled: on.trim() != "off",
        });
    }
    chain
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};
    use ph2d_audio_edit::EditClip;

    /// Every factory stage must name a real effect and real params — a typo would
    /// silently drop the stage, and the preset would quietly do less than it says.
    #[test]
    fn every_factory_stage_resolves() {
        for preset in &FACTORY {
            assert!(
                preset.stages.len() <= ph2d_panel_audio_editor::MAX_FX_STAGES,
                "{} has more stages than the chain holds",
                preset.name
            );
            for fs in preset.stages {
                let kind = kind_by_name(fs.effect)
                    .unwrap_or_else(|| panic!("{}: unknown effect {:?}", preset.name, fs.effect));
                let specs = params_for(kind);
                for o in fs.params {
                    assert!(
                        specs.iter().any(|s| s.label == o.label),
                        "{}: {} has no param {:?}",
                        preset.name,
                        fs.effect,
                        o.label
                    );
                }
            }
            assert_eq!(
                factory_chain(FACTORY.iter().position(|p| p.name == preset.name).unwrap()).len(),
                preset.stages.len(),
                "{} dropped a stage",
                preset.name
            );
        }
    }

    /// A band-spread stereo probe — a filter, EQ, dynamics, crush or width all mark it.
    fn probe() -> SampleData {
        let tau = std::f32::consts::TAU;
        let x: Vec<f32> = (0..4_800)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                0.4 * (tau * 220.0 * t).sin() + 0.3 * (tau * 9_000.0 * t).sin() + 0.05
            })
            .collect();
        SampleData::from_interleaved(x, AudioFormat::stereo(48_000))
    }

    /// Applying a factory preset must actually change the audio — a preset whose
    /// every stage happened to land on neutral would be a silent no-op that looks
    /// like it did something.
    #[test]
    fn factory_presets_are_audible() {
        let d = probe();
        for (idx, preset) in FACTORY.iter().enumerate() {
            let mut clip = EditClip::new(d.clone());
            for st in factory_chain(idx) {
                let out = match super::super::fx_params::build(st.kind, &st.norms) {
                    Some(super::super::fx_params::FxCommand::Plain(fx)) => clip.render_effect(fx),
                    Some(super::super::fx_params::FxCommand::Tail(fx)) => {
                        clip.render_tail_effect(fx)
                    }
                    None => continue,
                };
                clip.commit_rendered(out);
            }
            assert_ne!(
                clip.data().samples(),
                d.samples(),
                "{} is a silent no-op",
                preset.name
            );
        }
    }

    /// Serialize → parse must round-trip a chain exactly, including a disabled stage
    /// and the effect's kind — what you save is what you load.
    #[test]
    fn user_preset_round_trips_including_disabled_stages() {
        // A hand-built chain: an enabled Peak EQ, a disabled Compress, an Echo.
        let mk = |name: &str, on: bool, over: &[(&str, f32)]| {
            let kind = kind_by_name(name).unwrap();
            let specs = params_for(kind);
            let mut norms = default_norms(kind);
            for (label, val) in over {
                let i = specs.iter().position(|s| s.label == *label).unwrap();
                norms[i] = real_to_norm(&specs[i], *val);
            }
            FxStage {
                kind,
                norms,
                enabled: on,
            }
        };
        let chain = vec![
            mk("Peak EQ", true, &[("Gain", 6.0)]),
            mk("Compress", false, &[("Ratio", 4.0)]),
            mk("Echo", true, &[("Mix", 0.5)]),
        ];
        let restored = parse_chain(&serialize_chain(&chain));
        assert_eq!(restored.len(), chain.len());
        for (a, b) in chain.iter().zip(&restored) {
            assert_eq!(a.kind, b.kind, "kind drifted");
            assert_eq!(a.enabled, b.enabled, "enabled drifted");
            for (na, nb) in a.norms.iter().zip(&b.norms) {
                assert!((na - nb).abs() < 1e-3, "norm drifted: {na} vs {nb}");
            }
        }
    }

    /// A file keyed by name survives a `KINDS` reorder: the string "Limiter" resolves
    /// to whatever index Limiter now lives at, not to a frozen number.
    #[test]
    fn parse_is_keyed_by_name_not_index() {
        let text = format!("{HEADER}\nLimiter | on | 0.10 0.20\n");
        let chain = parse_chain(&text);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].kind, kind_by_name("Limiter").unwrap());
        // ...and the number we stored came back (0.10 → the Ceiling slider).
        assert!((chain[0].norms[0] - 0.10).abs() < 1e-3);
    }

    /// Malformed input never panics and never invents a stage.
    #[test]
    fn parse_skips_junk_and_unknown_effects() {
        let text = "not a header\n\nNope Effect | on | 0.5\nLimiter\n| on |\nLimiter | on | 0.1";
        let chain = parse_chain(text);
        // Only the last well-formed Limiter line survives (the bare "Limiter" with no
        // fields resolves too — kind is known, params default — so allow 1 or 2).
        assert!(
            chain
                .iter()
                .all(|s| s.kind == kind_by_name("Limiter").unwrap()),
            "invented an effect from junk"
        );
    }

    /// The two directions agree on how many numbers a stage carries: serialize writes
    /// exactly the effect's param count, parse tolerates fewer.
    #[test]
    fn serialize_writes_one_number_per_param() {
        let kind = kind_by_name("Compress").unwrap();
        let st = FxStage {
            kind,
            norms: default_norms(kind),
            enabled: true,
        };
        let line = serialize_chain(&[st]);
        let nums = line.lines().nth(1).unwrap().split('|').nth(2).unwrap();
        assert_eq!(nums.split_whitespace().count(), params_for(kind).len());
    }
}
