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
//!
//! The factory DATA lives in the sibling [`super::fx_presets_factory`] (split 2026-09-16, the cure
//! this header had named since the file reached 631 lines).

use ph2d_panel_audio_editor::{FxStage, MAX_FX_PARAMS};

use super::fx_presets_factory::{FACTORY, FStage};

use super::fx_params::{default_norms, params_for, real_to_norm};
use super::fx_params_table::KINDS;

/// The factory preset names, in table order — published to the panel's selector.
pub fn factory_names() -> Vec<&'static str> {
    FACTORY.iter().map(|p| p.name.tr()).collect()
}

/// Look up an effect kind by its stable id, honouring the names older preset files carry so a
/// chain saved before an effect was renamed (or before the ids existed) still resolves, rather than
/// silently dropping the stage.
fn kind_by_name(name: &str) -> Option<usize> {
    let want = legacy_alias(name).unwrap_or(name);
    KINDS.iter().position(|k| k.id == want)
}

/// The names preset files on disk may carry, mapped to the stable id.
///
/// ⭐ **Until 2026-09-16 the file stored the ENGLISH DISPLAY NAME** (`v1`): the rack's names moved to
/// the string table (HR-15), and a translated name can no longer be an identity, so `v2` writes the
/// `FxKind::id`. Every v1 name stays readable here — as match PATTERNS, which is also why this list
/// is not "text in the source" to the HR-15 census. "Gate" is older still (it always did both; the
/// label just started saying so). A dropped entry here is a user's saved preset silently losing an
/// effect, so it is gated (`the_legacy_gate_name_still_resolves`, `every_v1_name_still_resolves`).
fn legacy_alias(name: &str) -> Option<&'static str> {
    match name {
        "Gate" => Some("gate_expander"),
        "Low-Pass" => Some("low_pass"),
        "High-Pass" => Some("high_pass"),
        "Peak EQ" => Some("peak_eq"),
        "Low Shelf" => Some("low_shelf"),
        "High Shelf" => Some("high_shelf"),
        "De-Hum" => Some("de_hum"),
        "Compress" => Some("compress"),
        "Multiband" => Some("multiband"),
        "Gate / Expander" => Some("gate_expander"),
        "De-Esser" => Some("de_esser"),
        "De-Plosive" => Some("de_plosive"),
        "De-Click" => Some("de_click"),
        "De-Clip" => Some("de_clip"),
        "Limiter" => Some("limiter"),
        "Leveler" => Some("leveler"),
        "Transient" => Some("transient"),
        "Saturate" => Some("saturate"),
        "Distortion" => Some("distortion"),
        "Bitcrush" => Some("bitcrush"),
        "Widen" => Some("widen"),
        "Haas" => Some("haas"),
        "Exciter" => Some("exciter"),
        "Reverb" => Some("reverb"),
        "Conv Reverb" => Some("conv_reverb"),
        "Echo" => Some("echo"),
        "Ping-Pong" => Some("ping_pong"),
        "Comb" => Some("comb"),
        "Chorus" => Some("chorus"),
        "Flanger" => Some("flanger"),
        "Vibrato" => Some("vibrato"),
        "Phaser" => Some("phaser"),
        "Auto-Wah" => Some("auto_wah"),
        "Tremolo" => Some("tremolo"),
        "Auto-Pan" => Some("auto_pan"),
        "Trance Gate" => Some("trance_gate"),
        "Doubler" => Some("doubler"),
        "Ring Mod" => Some("ring_mod"),
        "Pitch Shift" => Some("pitch_shift"),
        "Formant Shift" => Some("formant_shift"),
        "Vocoder" => Some("vocoder"),
        "Granular" => Some("granular"),
        "Harmonizer" => Some("harmonizer"),
        _ => None,
    }
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
pub fn factory_chain(idx: usize) -> Vec<FxStage> {
    FACTORY
        .get(idx)
        .map(|p| p.stages.iter().filter_map(resolve_stage).collect())
        .unwrap_or_default()
}

/// The header line every user preset file starts with, so `parse_chain` can reject a
/// file that is plainly not one of ours (and so a future format bump has a hook).
const HEADER: &str = "# PH2D audio chain v2";

/// Serialize a chain to the user-preset text format — one stage per line,
/// `effect_id | on|off | n0 n1 …`, keyed by the stable id so it survives a `KINDS` reorder and a
/// translated display name (v1 wrote the English name; [`legacy_alias`] still reads it).
/// Storing the normals verbatim makes the round-trip exact.
pub fn serialize_chain(chain: &[FxStage]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for st in chain {
        let name = KINDS.get(st.kind).map(|k| k.id).unwrap_or("unknown");
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
pub fn parse_chain(text: &str) -> Vec<FxStage> {
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
    use super::super::fx_param_keys as p;
    use super::*;
    use ph2d_audio::{AudioFormat, SampleData};
    use ph2d_audio_edit::EditClip;
    use ph2d_i18n::TextKey;

    /// Every factory stage must name a real effect and real params — a typo would
    /// silently drop the stage, and the preset would quietly do less than it says.
    #[test]
    fn every_factory_stage_resolves() {
        for preset in &FACTORY {
            assert!(
                preset.stages.len() <= ph2d_panel_audio_editor::MAX_FX_STAGES,
                "{} has more stages than the chain holds",
                preset.name.tr()
            );
            for fs in preset.stages {
                let kind = kind_by_name(fs.effect).unwrap_or_else(|| {
                    panic!("{}: unknown effect {:?}", preset.name.tr(), fs.effect)
                });
                let specs = params_for(kind);
                for o in fs.params {
                    assert!(
                        specs.iter().any(|s| s.label == o.label),
                        "{}: {} has no param {:?}",
                        preset.name.tr(),
                        fs.effect,
                        o.label
                    );
                }
            }
            assert_eq!(
                factory_chain(FACTORY.iter().position(|p| p.name == preset.name).unwrap()).len(),
                preset.stages.len(),
                "{} dropped a stage",
                preset.name.tr()
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
                        clip.render_tail_effect(&fx)
                    }
                    None => continue,
                };
                clip.commit_rendered(out);
            }
            assert_ne!(
                clip.data().samples(),
                d.samples(),
                "{} is a silent no-op",
                preset.name.tr()
            );
        }
    }

    /// Serialize → parse must round-trip a chain exactly, including a disabled stage
    /// and the effect's kind — what you save is what you load.
    #[test]
    fn user_preset_round_trips_including_disabled_stages() {
        // A hand-built chain: an enabled Peak EQ, a disabled Compress, an Echo.
        let mk = |name: &str, on: bool, over: &[(TextKey, f32)]| {
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
            mk("peak_eq", true, &[(p::GAIN, 6.0)]),
            mk("compress", false, &[(p::RATIO, 4.0)]),
            mk("echo", true, &[(p::MIX, 0.5)]),
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

    /// **The rename does not orphan old saves.** A preset written before "Gate" became
    /// "Gate / Expander" stores the string "Gate"; it must still resolve to the same effect, not
    /// silently drop the stage. The alias is the only thing standing between the rename and every
    /// user's saved gate turning into nothing.
    #[test]
    fn the_legacy_gate_name_still_resolves() {
        let now = kind_by_name("Gate / Expander").expect("the effect exists under its new name");
        assert_eq!(
            kind_by_name("Gate"),
            Some(now),
            "a preset saved as \"Gate\" no longer resolves -- the rename orphaned it"
        );
        // And a real preset line keyed by the old name parses to that effect.
        let text = format!("{HEADER}\nGate | on | 0.04 0.80\n");
        let chain = parse_chain(&text);
        assert_eq!(chain.len(), 1, "the legacy \"Gate\" line was dropped");
        assert_eq!(chain[0].kind, now);
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

    /// ⭐⭐ **Every name a v1 file could carry still resolves** (2026-09-16). v1 stored the ENGLISH
    /// DISPLAY NAME; the names moved to the string table, and v2 writes the stable id. A v1 name
    /// missing from [`legacy_alias`] is a user's saved chain silently losing that stage — so the
    /// list is checked against the display text itself, all 42.
    #[test]
    fn every_v1_name_still_resolves() {
        for (kind, k) in KINDS.iter().enumerate() {
            let v1 = k.name.tr();
            assert_eq!(
                kind_by_name(v1),
                Some(kind),
                "a v1 preset line naming {v1:?} no longer resolves to `{}`",
                k.id
            );
            assert_eq!(
                kind_by_name(k.id),
                Some(kind),
                "the id `{}` does not resolve",
                k.id
            );
        }
    }

    /// ⭐ **v2 writes the ID, never the display text** — a translated rack must save the same file.
    #[test]
    fn a_saved_chain_names_the_effect_by_its_id() {
        let kind = kind_by_name("conv_reverb").unwrap();
        let line = serialize_chain(&[FxStage {
            kind,
            norms: default_norms(kind),
            enabled: true,
        }]);
        assert!(line.starts_with("# PH2D audio chain v2\n"), "{line}");
        let first = line.lines().nth(1).unwrap();
        assert!(first.starts_with("conv_reverb | on |"), "{first}");
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
