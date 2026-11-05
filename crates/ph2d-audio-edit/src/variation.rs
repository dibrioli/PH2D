//! Variation containers (W6 asset-prep) — a game-audio **random / sequence
//! container** (the Wwise Random/Sequence Container, FMOD Multi Instrument). A set
//! of clips that, on each trigger, yields **one** variation picked by a strategy,
//! with per-play pitch/gain randomisation and per-entry weights. This is what kills
//! the robotic repetition of footsteps, gunshots and impacts.
//!
//! This module is the **pure model**: the entries (a path + weight + enabled flag),
//! the strategy, the container-level jitter ranges, the [`VariationPicker`] that
//! implements selection, and a text **manifest** (`serialize`/`parse`) so a set
//! round-trips to a file the editor saves and a future game runtime reads.
//!
//! It holds **no audio** — the shell owns the decoded [`ph2d_audio::SampleData`] for
//! each entry (keyed by index) and plays the pick through the preview voice. Like the
//! rest of `ph2d-audio-edit` this runs on the control thread.

pub use ph2d_sound_set::{Jitter, MAX_JITTER, PickStrategy, WEIGHT_RANGE};

/// One clip in a variation set. The `path` doubles as its display label; `weight`
/// biases the weighted strategies (Random / Shuffle); a disabled entry stays in the
/// set but is never picked.
#[derive(Debug, Clone, PartialEq)]
pub struct Variation {
    /// Source file path (also the label — the panel shows the file stem).
    pub path: String,
    /// Relative pick weight for the weighted strategies (clamped to [`WEIGHT_RANGE`]).
    pub weight: f32,
    /// Whether this entry participates in picks.
    pub enabled: bool,
}

impl Variation {
    /// A unit-weight, enabled entry for `path`.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            weight: 1.0,
            enabled: true,
        }
    }
}

/// A variation container: the entries, the pick strategy, and container-level
/// per-play randomisation (symmetric `±` in semitones / dB). The jitter applies to
/// **whichever** entry plays, exactly like a Wwise container's pitch/volume
/// randomiser.
#[derive(Debug, Clone, PartialEq)]
pub struct VariationSet {
    /// The clips, in list order.
    pub entries: Vec<Variation>,
    /// How the next variation is chosen.
    pub strategy: PickStrategy,
    /// Per-play pitch randomisation, `± semitones` (0 = none).
    pub pitch_jitter_semitones: f32,
    /// Per-play gain randomisation, `± dB` (0 = none).
    pub gain_jitter_db: f32,
}

impl Default for VariationSet {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            strategy: PickStrategy::Shuffle,
            pitch_jitter_semitones: 0.0,
            gain_jitter_db: 0.0,
        }
    }
}

impl VariationSet {
    fn candidates(&self) -> Vec<ph2d_sound_set::Candidate> {
        self.entries
            .iter()
            .map(|e| ph2d_sound_set::Candidate {
                weight: e.weight,
                enabled: e.enabled,
            })
            .collect()
    }
}

/// O sorteio de um conjunto: a lei é a de [`ph2d_sound_set::Picker`], a mesma do som de cena. O manifesto
/// guarda caminhos porque é um arquivo do editor; o `SoundSet` do objeto guarda nomes do projeto.
#[derive(Debug, Clone, Default)]
pub struct VariationPicker(ph2d_sound_set::Picker);

impl VariationPicker {
    /// A picker seeded explicitly (tests pin the sequence with this).
    pub fn with_seed(seed: u64) -> Self {
        Self(ph2d_sound_set::Picker::with_seed(seed))
    }

    /// Pick the index of the next variation to play, or `None` when nothing is enabled.
    pub fn pick(&mut self, set: &VariationSet) -> Option<usize> {
        self.0.pick(
            &set.candidates(),
            set.strategy,
            set.strategy.no_repeat_of_the_editor(),
        )
    }

    /// Draw a per-play pitch/gain offset from the container's jitter ranges.
    pub fn jitter(&mut self, set: &VariationSet) -> Jitter {
        self.0
            .jitter(set.pitch_jitter_semitones, set.gain_jitter_db)
    }
}

// ---------------------------------------------------------------------------------
// Manifest — a tolerant text format so a set round-trips to a file (the editor's
// Save/Load) and a future game runtime can read it. Keyed by content, never by
// index; unknown / blank / `#` lines are skipped so an older file still loads.
// ---------------------------------------------------------------------------------

/// Serialise a set to the manifest text (round-trips through [`parse`]).
pub fn serialize(set: &VariationSet) -> String {
    let mut out = String::from("# PH2D variation set\n");
    out.push_str(&format!("strategy {}\n", set.strategy.name()));
    out.push_str(&format!("pitch {:.3}\n", set.pitch_jitter_semitones));
    out.push_str(&format!("gain {:.3}\n", set.gain_jitter_db));
    for e in &set.entries {
        let on = if e.enabled { "on" } else { "off" };
        // `path | weight | on|off` — path first so it may contain spaces.
        out.push_str(&format!("entry | {} | {:.3} | {}\n", e.path, e.weight, on));
    }
    out
}

/// Parse the manifest text back to a set. Tolerant: unknown keywords and malformed
/// lines are skipped; a file with no recognised header keeps the defaults.
pub fn parse(text: &str) -> VariationSet {
    let mut set = VariationSet::default();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("entry ") {
            // `| path | weight | on|off`
            let fields: Vec<&str> = rest.split('|').map(str::trim).collect();
            // fields[0] is empty (the leading `|`); path/weight/state follow.
            if fields.len() >= 4 {
                let path = fields[1].to_string();
                if path.is_empty() {
                    continue;
                }
                let weight = fields[2]
                    .parse::<f32>()
                    .unwrap_or(1.0)
                    .clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1);
                let enabled = fields[3] != "off";
                set.entries.push(Variation {
                    path,
                    weight,
                    enabled,
                });
            }
        } else if let Some(name) = line.strip_prefix("strategy ")
            && let Some(s) = PickStrategy::from_name(name.trim())
        {
            set.strategy = s;
        } else if let Some(v) = line.strip_prefix("pitch ")
            && let Ok(f) = v.trim().parse::<f32>()
        {
            set.pitch_jitter_semitones = f.clamp(0.0, MAX_JITTER);
        } else if let Some(v) = line.strip_prefix("gain ")
            && let Ok(f) = v.trim().parse::<f32>()
        {
            set.gain_jitter_db = f.clamp(0.0, MAX_JITTER);
        }
    }
    set
}

/// Compare two file names "naturally" — digit runs compare by numeric value, so
/// `step_2` sorts before `step_10` (plain lexicographic would put `_10` first). Used
/// by the folder import so an unpadded `_1..NN` set still lands in order (Sequence
/// depends on it). Case-sensitive on the non-digit runs; leading zeros don't matter.
pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (mut ai, mut bi) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (ai.peek().copied(), bi.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(ca), Some(cb)) => {
                if ca.is_ascii_digit() && cb.is_ascii_digit() {
                    // Compare whole digit runs by numeric value (strip leading zeros).
                    let na: String = take_digits(&mut ai);
                    let nb: String = take_digits(&mut bi);
                    let (ta, tb) = (na.trim_start_matches('0'), nb.trim_start_matches('0'));
                    let ord = ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb));
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    // Equal numeric value → the shorter original (fewer leading zeros)
                    // sorts first, a stable tiebreak.
                    let ord = na.len().cmp(&nb.len());
                    if ord != Ordering::Equal {
                        return ord;
                    }
                } else {
                    let ord = ca.cmp(&cb);
                    if ord != Ordering::Equal {
                        return ord;
                    }
                    ai.next();
                    bi.next();
                }
            }
        }
    }
}

/// Consume and return the leading run of ASCII digits from `it`.
fn take_digits(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        if c.is_ascii_digit() {
            s.push(c);
            it.next();
        } else {
            break;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_of(paths: &[&str]) -> VariationSet {
        VariationSet {
            entries: paths.iter().map(|p| Variation::new(*p)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn empty_set_picks_nothing() {
        let mut p = VariationPicker::default();
        assert_eq!(p.pick(&VariationSet::default()), None);
    }

    #[test]
    fn sequence_cycles_the_enabled_entries_in_order() {
        let mut set = set_of(&["a", "b", "c"]);
        set.strategy = PickStrategy::Sequence;
        let mut p = VariationPicker::default();
        let seq: Vec<usize> = (0..6).map(|_| p.pick(&set).unwrap()).collect();
        assert_eq!(
            seq,
            vec![0, 1, 2, 0, 1, 2],
            "round-robin must wrap in order"
        );
    }

    #[test]
    fn sequence_skips_disabled_entries() {
        let mut set = set_of(&["a", "b", "c"]);
        set.strategy = PickStrategy::Sequence;
        set.entries[1].enabled = false; // only a, c
        let mut p = VariationPicker::default();
        let seq: Vec<usize> = (0..4).map(|_| p.pick(&set).unwrap()).collect();
        assert_eq!(seq, vec![0, 2, 0, 2], "disabled entry must never appear");
    }

    #[test]
    fn shuffle_never_repeats_back_to_back() {
        let mut set = set_of(&["a", "b", "c", "d"]);
        set.strategy = PickStrategy::Shuffle;
        let mut p = VariationPicker::default();
        let mut prev = None;
        for _ in 0..500 {
            let i = p.pick(&set).unwrap();
            assert_ne!(Some(i), prev, "Shuffle repeated a pick immediately");
            prev = Some(i);
        }
    }

    #[test]
    fn single_enabled_entry_repeats_without_looping_forever() {
        // With one entry Shuffle cannot avoid a repeat — it must still terminate and
        // return that entry, not spin on the retry loop.
        let mut set = set_of(&["only"]);
        set.strategy = PickStrategy::Shuffle;
        let mut p = VariationPicker::default();
        for _ in 0..10 {
            assert_eq!(p.pick(&set), Some(0));
        }
    }

    #[test]
    fn weights_bias_the_random_distribution() {
        let mut set = set_of(&["rare", "common"]);
        set.strategy = PickStrategy::Random;
        set.entries[0].weight = 1.0;
        set.entries[1].weight = 9.0; // ~90% of picks
        let mut p = VariationPicker::default();
        let mut common = 0;
        const N: usize = 10_000;
        for _ in 0..N {
            if p.pick(&set) == Some(1) {
                common += 1;
            }
        }
        let frac = common as f32 / N as f32;
        assert!(
            (0.85..0.95).contains(&frac),
            "weighted draw off target: {frac} (want ~0.9)"
        );
    }

    #[test]
    fn no_jitter_is_the_identity() {
        let set = set_of(&["a"]);
        let mut p = VariationPicker::default();
        let j = p.jitter(&set);
        assert_eq!(j.pitch, 1.0);
        assert_eq!(j.gain, 1.0);
    }

    #[test]
    fn jitter_stays_within_the_declared_range() {
        let mut set = set_of(&["a"]);
        set.pitch_jitter_semitones = 12.0; // ±1 octave → pitch in [0.5, 2]
        set.gain_jitter_db = 6.0; // ±6 dB → gain in [~0.501, ~1.995]
        let mut p = VariationPicker::default();
        for _ in 0..1000 {
            let j = p.jitter(&set);
            assert!(
                (0.5 - 1e-4..=2.0 + 1e-4).contains(&j.pitch),
                "pitch {} out of ±1 octave",
                j.pitch
            );
            assert!(
                (0.5..=2.0).contains(&j.gain),
                "gain {} out of ±6 dB",
                j.gain
            );
        }
    }

    #[test]
    fn a_disabled_entry_is_never_picked() {
        // Three takes; the middle one is the one you are unsure about.
        let mut set = VariationSet::default();
        for n in ["a.wav", "b.wav", "c.wav"] {
            set.entries.push(Variation::new(n));
        }
        set.entries[1].enabled = false;

        // Every strategy, many draws: the disabled entry must never come up. (Random would
        // otherwise hit it ~1/3 of the time, so a single draw would prove nothing.)
        for strategy in [
            PickStrategy::Random,
            PickStrategy::Sequence,
            PickStrategy::Shuffle,
        ] {
            set.strategy = strategy;
            let mut picker = VariationPicker::with_seed(0xABCD);
            for _ in 0..200 {
                let i = picker.pick(&set).expect("two entries are enabled");
                assert_ne!(
                    i,
                    1,
                    "{}: picked the DISABLED entry — the toggle would be decorative",
                    strategy.name()
                );
            }
        }
    }

    /// …and disabling everything leaves nothing to pick, rather than picking a disabled one.
    #[test]
    fn a_set_with_nothing_enabled_picks_nothing() {
        let mut set = VariationSet::default();
        set.entries.push(Variation::new("a.wav"));
        set.entries[0].enabled = false;
        let mut picker = VariationPicker::with_seed(1);
        assert!(picker.pick(&set).is_none());
    }

    #[test]
    fn manifest_round_trips() {
        let set = VariationSet {
            entries: vec![
                Variation {
                    path: "sfx/foot 01.wav".into(), // a space in the path
                    weight: 2.5,
                    enabled: true,
                },
                Variation {
                    path: "sfx/foot_02.wav".into(),
                    weight: 1.0,
                    enabled: false,
                },
            ],
            strategy: PickStrategy::Sequence,
            pitch_jitter_semitones: 3.0,
            gain_jitter_db: 1.5,
        };
        let back = parse(&serialize(&set));
        assert_eq!(back, set, "manifest did not round-trip exactly");
    }

    #[test]
    fn parse_skips_junk_and_unknown_lines() {
        let text = "\
# a comment
strategy Nonsense
strategy Random
pitch not-a-number
pitch 4.0
entry | ok.wav | 1.0 | on
entry | | 1.0 | on
garbage line
entry | bad-weight.wav | xyz | off
";
        let set = parse(text);
        assert_eq!(set.strategy, PickStrategy::Random, "second strategy wins");
        assert_eq!(set.pitch_jitter_semitones, 4.0);
        assert_eq!(set.entries.len(), 2, "empty-path entry must be skipped");
        assert_eq!(set.entries[0].path, "ok.wav");
        assert_eq!(set.entries[1].path, "bad-weight.wav");
        assert_eq!(set.entries[1].weight, 1.0, "bad weight falls back to 1.0");
        assert!(!set.entries[1].enabled);
    }

    #[test]
    fn strategy_cycles_and_wraps() {
        assert_eq!(PickStrategy::Random.cycled(1), PickStrategy::Sequence);
        assert_eq!(PickStrategy::Random.cycled(-1), PickStrategy::Shuffle);
        assert_eq!(PickStrategy::Shuffle.cycled(1), PickStrategy::Random);
    }

    #[test]
    fn natural_sort_orders_numeric_suffixes() {
        let mut v = vec![
            "step_10.wav",
            "step_2.wav",
            "step_1.wav",
            "step_20.wav",
            "step_3.wav",
        ];
        v.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            v,
            vec![
                "step_1.wav",
                "step_2.wav",
                "step_3.wav",
                "step_10.wav",
                "step_20.wav"
            ],
            "unpadded numbers must sort by value, not lexically"
        );
        // Zero-padded names sort the same way, and a shorter run wins ties.
        let mut z = vec!["a_09", "a_10", "a_1", "a_01"];
        z.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(z, vec!["a_1", "a_01", "a_09", "a_10"]);
    }
}
