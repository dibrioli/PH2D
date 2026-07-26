//! The **reverse** verbs on a [`Track`] — `reverse_about` (whole track) and
//! `reverse_keys` (a selection, the AE *Time-Reverse Keyframes* crown jewel).
//! Split from `track.rs` under the workspace LOC cap. A CHILD module of `track`,
//! so it reaches `Track`'s private fields and `resort`/`invalidate_cursor` (a
//! descendant sees its ancestor's privates); it co-locates the two verbs that
//! share the "a key's `Interp` describes its OUTGOING segment" mirror rule.

use super::{KeyId, Track};
use crate::curve::Interp;
use crate::time::RationalTime;

impl Track {
    /// **Play this track backwards inside `[0, span]`**: every key's time becomes
    /// `span − t`, and every segment's shape is mirrored with it.
    ///
    /// The subtle half is that a key's `Interp` describes the segment **leaving**
    /// it, so reversal moves each interp to a different key: the segment between
    /// keys `i` and `i+1` is, reversed, the one leaving key `n−2−i`. Mirror the
    /// times and leave the interps in place and the curve keeps its old
    /// accelerations while the values run backwards — every ease-out becomes an
    /// ease-in, which is precisely the thing reversing is meant to preserve. The
    /// LAST key has no outgoing segment, so its interp is not data: it is dropped,
    /// and the new last key inherits the same non-role.
    ///
    /// Ids ride with their keys ([`KeyId`] names a key, not a position), and so do
    /// the roving flags — a key whose time is derived is still that key afterwards.
    pub fn reverse_about(&mut self, span: f64) {
        if self.keys.is_empty() {
            return;
        }
        let n = self.keys.len();
        for k in &mut self.keys {
            k.t = RationalTime::from_seconds(span - k.t.to_seconds());
        }
        // Hand each segment's shape to the key that will own it after the flip. The
        // walk is over the OLD order, so read the interps out first.
        let old: Vec<Interp> = self.keys.iter().map(|k| k.interp).collect();
        for i in 0..n {
            // Key `i` ends up at position `n-1-i`; the segment leaving it there is
            // the OLD segment leaving key `n-2-(n-1-i)` = `i-1`, played backwards.
            self.keys[i].interp = if i == 0 {
                // The new LAST key: no outgoing segment. Keep it inert.
                Interp::Linear
            } else {
                old[i - 1].reversed()
            };
        }
        self.keys.reverse();
        self.ids.reverse();
        self.roving.reverse();
        // The times were mirrored, so the reverse above already restores sort order
        // — but a `span` that does not bracket every key (a key past the clip's end)
        // would not. Re-sort rather than assume the caller measured well.
        self.resort();
        self.invalidate_cursor();
    }

    /// **Time-reverse the SELECTED keys about `pivot_seconds`** — the AE
    /// *Time-Reverse Keyframes* verb, scoped to a selection instead of the whole
    /// track (which is [`Track::reverse_about`]).
    ///
    /// Each selected key's time becomes `2·pivot − t` (so a pivot at the centre of
    /// the selected span maps the set onto itself, reversed), and the interps
    /// mirror the same way [`reverse_about`] does: a key's `Interp` describes the
    /// segment *leaving* it, so the segment leaving the key that lands at selected
    /// position `j` after the flip is the OLD segment leaving selected position
    /// `j−1`, played backwards. It is the exact restriction of `reverse_about` to
    /// the selected subsequence — with a full-track selection the two are identical.
    ///
    /// ⚠️ **Boundary:** the formerly-earliest selected key becomes the latest, and
    /// its outgoing segment (toward whatever key now follows it, selected or not)
    /// is reset to `Interp::Linear` — the same inert role `reverse_about` gives its
    /// new last key. On a contiguous run this touches one boundary segment; it is
    /// the honest cost of reversing a subset, not the whole curve.
    ///
    /// Ids and roving flags ride with their keys ([`resort`] sorts all three in
    /// lockstep), so a reversed key keeps its identity and its derived-time status.
    /// A selection of fewer than two keys has nothing to reverse (no-op).
    pub fn reverse_keys(&mut self, ids: &[KeyId], pivot_seconds: f64) {
        let mut sel: Vec<usize> = ids.iter().filter_map(|&id| self.index_of(id)).collect();
        sel.sort_unstable();
        sel.dedup();
        if sel.len() < 2 {
            return;
        }
        // Read the selected keys' outgoing interps in selected order BEFORE moving.
        let old: Vec<Interp> = sel.iter().map(|&i| self.keys[i].interp).collect();
        for &i in &sel {
            let t = self.keys[i].t.to_seconds();
            self.keys[i].t = RationalTime::from_seconds(2.0 * pivot_seconds - t);
        }
        for (j, &i) in sel.iter().enumerate() {
            // Same rule as `reverse_about`, restricted to the selected sequence:
            // the key formerly at selected position `j` ends up carrying the OLD
            // segment leaving `j−1`, reversed; the formerly-earliest goes inert.
            self.keys[i].interp = if j == 0 {
                Interp::Linear
            } else {
                old[j - 1].reversed()
            };
        }
        self.resort();
        self.invalidate_cursor();
    }
}
