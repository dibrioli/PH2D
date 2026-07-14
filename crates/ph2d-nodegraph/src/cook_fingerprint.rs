//! **The reuse decision** — what a node's memo keys on.
//!
//! A sibling of `cook.rs`, not an inline module: the cook hit the workspace's 700-LOC cap, and a
//! cap is answered by SPLITTING ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]). The
//! fingerprint is its own responsibility anyway — it is the one place that has to know **every**
//! way a node's output can change, and every field in it was added because something changed and
//! the memo did not notice.

use super::BTreeMap;

/// What a node's reuse decision depends on: revisions of its forward inputs,
/// the playhead bits (if `Temporal`), and the tick (if it consumes a `pre`
/// edge, i.e. is sequential and must advance every tick). Playhead is stored
/// as `to_bits` so the key is a stable bitwise compare (no NaN-self-inequality,
/// no `-0.0`/`+0.0` aliasing).
#[derive(Clone, Default, PartialEq)]
pub(super) struct Fingerprint {
    pub(super) input_revs: Vec<u64>,
    pub(super) playhead: Option<u64>,
    pub(super) tick: Option<u64>,
    /// FNV-1a of WHICH params are driven and by whom (name + source). The driver's own
    /// revision already rides in [`input_revs`](Self::input_revs), so this is not about the
    /// value — it is about the WIRING: re-pointing a param to a different node whose output
    /// happens to be unchanged must still recompute, because the node now reads a different
    /// place.
    pub(super) param_sources: u64,
    /// FNV-1a of the revisions of the externals this node read LAST time (doc 65). An external
    /// is an input the graph does not describe: edit the drawn curve and the node must recompute,
    /// and nothing else in this fingerprint would notice. `0` for a node that has never cooked —
    /// which cooks, which is right.
    pub(super) externals: u64,
    /// FNV-1a of the node's per-instance param overrides (name + value bits).
    /// Folds edited params into the reuse decision: an override change must
    /// recompute, or a re-cook with the same `Cook` would return a stale,
    /// pre-edit stream. Manifest defaults are compile-time constant, so only
    /// overrides can change at runtime — hashing them suffices.
    pub(super) params: u64,
    /// FNV-1a of the node's per-node **text** param overrides (name + string
    /// bytes). Same reuse-gating role as [`params`](Self::params): an edited
    /// formula must recompute rather than return a stale, pre-edit stream.
    pub(super) text_params: u64,
}

/// FNV-1a over a node's overrides, in `BTreeMap` (deterministic) order. `None`
/// or empty → the FNV offset basis (a stable constant for "no overrides").
///
/// Each name is **length-prefixed** so the byte stream is unambiguous: without
/// it, `{"p": x, "q": y}` and `{"pemonq": y}` can flatten to the same bytes and
/// collide, which (since the fingerprint gates memo reuse) would return a stale,
/// pre-edit stream — a silent wrong result. The length prefix makes the
/// encoding injective, so distinct override sets always hash distinctly.
pub(super) fn params_fingerprint(overrides: Option<&BTreeMap<String, f32>>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    if let Some(map) = overrides {
        for (name, value) in map {
            mix(&(name.len() as u64).to_le_bytes());
            mix(name.as_bytes());
            mix(&value.to_bits().to_le_bytes());
        }
    }
    hash
}

/// FNV-1a over a node's **text** param overrides (the string channel), same
/// length-prefixed injective encoding as [`params_fingerprint`] so an edited
/// formula recomputes rather than returning a stale stream.
pub(super) fn text_params_fingerprint(overrides: Option<&BTreeMap<String, String>>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    if let Some(map) = overrides {
        for (name, value) in map {
            mix(&(name.len() as u64).to_le_bytes());
            mix(name.as_bytes());
            mix(&(value.len() as u64).to_le_bytes());
            mix(value.as_bytes());
        }
    }
    hash
}
