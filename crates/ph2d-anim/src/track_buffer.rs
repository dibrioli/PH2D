//! **Buffer Curves** (crown-jewels §5) — a byte-identical snapshot of a track's
//! whole curve (its keys, their ids, and the roving flags), so the graph editor
//! can Store the current curve, tune it, then Swap back to the stored one (and
//! back again) — the A/B toggle Unreal calls *Buffer Curves*.
//!
//! Split from `track.rs` under the workspace LOC cap; a CHILD module so its
//! `impl Track` reaches `Track`'s private key vectors and `invalidate_cursor`.
//!
//! The snapshot carries the **ids** too, not just the keys: the selection points
//! at keys by id, so a swap that kept the values but reissued ids would drop the
//! selection and break the graph the artist is looking at. Restore is therefore
//! exact — same keys, same ids, same roving — which is what makes the round-trip
//! (Store -> edit -> Swap) return the curve unchanged to the byte.

use super::{Key, KeyId, Track};

/// An opaque, byte-identical snapshot of a track's curve — keys, their ids, and
/// the roving flags. Produced by [`Track::snapshot_curve`] and applied by
/// [`Track::restore_curve`]. Session state (the graph editor's buffer); never
/// serialized.
#[derive(Debug, Clone)]
pub struct CurveSnapshot {
    keys: Vec<Key>,
    ids: Vec<KeyId>,
    roving: Vec<bool>,
}

impl Track {
    /// Snapshot the whole curve — keys, ids and roving flags — for the graph
    /// editor's buffer. Cheap `Vec` clones of plain `Copy` data; the track is
    /// left untouched.
    #[must_use]
    pub fn snapshot_curve(&self) -> CurveSnapshot {
        CurveSnapshot {
            keys: self.keys.clone(),
            ids: self.ids.clone(),
            roving: self.roving.clone(),
        }
    }

    /// Replace the curve wholesale with a snapshot — the exact keys, ids and
    /// roving it captured. Byte-identical to the track that produced it, so a
    /// Store -> edit -> Swap round-trip returns the original unchanged; and the
    /// ids survive, so the selection still points at real keys.
    pub fn restore_curve(&mut self, snap: &CurveSnapshot) {
        self.keys = snap.keys.clone();
        self.ids = snap.ids.clone();
        self.roving = snap.roving.clone();
        self.invalidate_cursor();
    }
}
