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
//! rest of `ph2d-audio-edit` this runs on the control thread, so the jitter math may
//! use `exp2` freely (HR-5 is RT-only).

/// How the [`VariationPicker`] chooses the next variation from the enabled entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickStrategy {
    /// Weighted-random each time (may repeat back-to-back).
    Random,
    /// Cycle through the enabled entries in order (weights ignored — the order is
    /// the point). Round-robin.
    Sequence,
    /// Weighted-random, but never the same entry twice in a row when ≥2 are enabled
    /// (avoid-repeat). The natural default for footsteps.
    Shuffle,
}

impl PickStrategy {
    /// The strategies in cycle order (the panel's `◀ name ▶` selector walks this).
    pub const ALL: [PickStrategy; 3] = [
        PickStrategy::Random,
        PickStrategy::Sequence,
        PickStrategy::Shuffle,
    ];

    /// Display / manifest name.
    pub fn name(self) -> &'static str {
        match self {
            PickStrategy::Random => "Random",
            PickStrategy::Sequence => "Sequence",
            PickStrategy::Shuffle => "Shuffle",
        }
    }

    /// Parse a manifest/display name back to a strategy (unknown → `None`).
    pub fn from_name(s: &str) -> Option<PickStrategy> {
        PickStrategy::ALL.into_iter().find(|k| k.name() == s)
    }

    /// The strategy `delta` steps away in [`PickStrategy::ALL`] order (wrapping).
    pub fn cycled(self, delta: i32) -> PickStrategy {
        let n = PickStrategy::ALL.len() as i32;
        let i = PickStrategy::ALL
            .iter()
            .position(|k| *k == self)
            .unwrap_or(0) as i32;
        let j = (i + delta).rem_euclid(n) as usize;
        PickStrategy::ALL[j]
    }
}

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

/// Inclusive weight range. A weight below the floor is treated as the floor when
/// summing, so a weighted draw never divides by zero.
pub const WEIGHT_RANGE: (f32, f32) = (0.01, 16.0);
/// Maximum jitter, in semitones / decibels, for the container-level randomisers.
pub const MAX_JITTER: f32 = 24.0;

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
    /// Indices of the entries eligible for a pick (enabled), in list order.
    fn enabled_indices(&self) -> Vec<usize> {
        (0..self.entries.len())
            .filter(|&i| self.entries[i].enabled)
            .collect()
    }
}

/// A per-play pitch/gain offset applied to the chosen variation before it sounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jitter {
    /// Playback-rate multiplier (`1.0` = original pitch), from the pitch randomiser.
    pub pitch: f32,
    /// Linear gain (`1.0` = unity), from the gain randomiser.
    pub gain: f32,
}

impl Jitter {
    /// The identity offset (no jitter).
    pub const NONE: Jitter = Jitter {
        pitch: 1.0,
        gain: 1.0,
    };
}

/// Selection state for a [`VariationSet`]: a deterministic PRNG plus the little
/// bookkeeping the Sequence / Shuffle strategies need (the round-robin cursor and the
/// last index, so consecutive picks differ). Deterministic per session — repeated
/// picks feel random, and tests can assert exact behaviour.
#[derive(Debug, Clone)]
pub struct VariationPicker {
    rng: u64,
    /// Round-robin cursor into the *enabled* list (Sequence).
    seq: usize,
    /// The last picked entry index (Shuffle avoid-repeat).
    last: Option<usize>,
}

impl Default for VariationPicker {
    fn default() -> Self {
        // A fixed non-zero seed: deterministic, and never the degenerate all-zero
        // splitmix64 state.
        VariationPicker::with_seed(0x9E37_79B9_7F4A_7C15)
    }
}

impl VariationPicker {
    /// A picker seeded explicitly (tests pin the sequence with this).
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: seed,
            seq: 0,
            last: None,
        }
    }

    /// Advance the splitmix64 state and return the next 64-bit value.
    fn next_u64(&mut self) -> u64 {
        // splitmix64 — a transcendental-free integer PRNG (same family the painter's
        // per-dab jitter uses); good distribution, cheap, deterministic.
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform `f32` in `[0, 1)`.
    fn next_unit(&mut self) -> f32 {
        // Top 24 bits → the f32 mantissa's worth of resolution.
        const SCALE: f32 = 1.0 / (1u32 << 24) as f32;
        ((self.next_u64() >> 40) as u32) as f32 * SCALE
    }

    /// A uniform `f32` in `[-1, 1)`.
    fn next_bipolar(&mut self) -> f32 {
        self.next_unit() * 2.0 - 1.0
    }

    /// Pick the index of the next variation to play, or `None` when nothing is
    /// enabled. Honours the set's [`PickStrategy`]; the returned index is always an
    /// enabled entry.
    pub fn pick(&mut self, set: &VariationSet) -> Option<usize> {
        let enabled = set.enabled_indices();
        if enabled.is_empty() {
            self.last = None;
            return None;
        }
        if enabled.len() == 1 {
            self.last = Some(enabled[0]);
            self.seq = 0;
            return Some(enabled[0]);
        }
        let chosen = match set.strategy {
            PickStrategy::Sequence => {
                let i = self.seq % enabled.len();
                self.seq = (i + 1) % enabled.len();
                enabled[i]
            }
            PickStrategy::Random => self.weighted(set, &enabled),
            PickStrategy::Shuffle => {
                // Weighted, but re-draw off the previous pick (bounded tries so a set
                // whose weight is concentrated on `last` still terminates).
                let mut pick = self.weighted(set, &enabled);
                let mut tries = 0;
                while Some(pick) == self.last && tries < SHUFFLE_MAX_RETRIES {
                    pick = self.weighted(set, &enabled);
                    tries += 1;
                }
                pick
            }
        };
        self.last = Some(chosen);
        Some(chosen)
    }

    /// A weight-proportional draw over `enabled` (falls back to uniform if the total
    /// weight is non-positive).
    fn weighted(&mut self, set: &VariationSet, enabled: &[usize]) -> usize {
        let total: f32 = enabled
            .iter()
            .map(|&i| set.entries[i].weight.clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1))
            .sum();
        if total <= 0.0 {
            return enabled[(self.next_u64() as usize) % enabled.len()];
        }
        let mut r = self.next_unit() * total;
        for &i in enabled {
            let w = set.entries[i].weight.clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1);
            if r < w {
                return i;
            }
            r -= w;
        }
        *enabled.last().unwrap() // rounding guard
    }

    /// Draw a per-play pitch/gain offset from the container's jitter ranges. Two
    /// independent bipolar draws → `pitch = 2^(±st/12)`, `gain = 10^(±dB/20)`. Both
    /// reduce to a single `exp2` (`10^x = 2^(x·log2 10)`).
    pub fn jitter(&mut self, set: &VariationSet) -> Jitter {
        let st = set.pitch_jitter_semitones.clamp(0.0, MAX_JITTER) * self.next_bipolar();
        let db = set.gain_jitter_db.clamp(0.0, MAX_JITTER) * self.next_bipolar();
        Jitter {
            pitch: (st / SEMITONES_PER_OCTAVE).exp2(),
            gain: (db * DB_TO_EXP2).exp2(),
        }
    }
}

/// Retry budget for Shuffle's avoid-repeat re-draw.
const SHUFFLE_MAX_RETRIES: usize = 8;
/// Semitones in an octave (pitch-ratio math).
const SEMITONES_PER_OCTAVE: f32 = 12.0;
/// dB → base-2 exponent for a linear gain: `10^(dB/20) = 2^(dB · log2(10) / 20)`, so
/// the multiplier is `log2(10)/20`. Keeps the gain math on `exp2` (no `powf(10, …)`).
const DB_TO_EXP2: f32 = std::f32::consts::LOG2_10 / 20.0;

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
