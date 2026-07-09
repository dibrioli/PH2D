//! [`TimelineState`] — the full editor state the shell embeds in `AppGfx`:
//! the [`TimelineDoc`] plus panel-side selection, undo [`TimelineHistory`], and
//! transport-adjacent flags.
//!
//! Only the `doc` is undoable; selection + flags are panel state (never
//! snapshotted). Pan/zoom live on the panel itself (`Panel::State`), not here.

use ph2d_anim::{AnimTarget, KeyId};

use crate::doc::TimelineDoc;
use crate::history::TimelineHistory;

/// One selected key in the active clip: which track (`target`) and which key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectedKey {
    /// The track's opaque target.
    pub target: AnimTarget,
    /// The stable key id within that track.
    pub key: KeyId,
}

/// The set of selected keys (dope-sheet / graph). Small, order-insensitive; a
/// `Vec` is fine at the key counts a hand-authored clip holds.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    keys: Vec<SelectedKey>,
}

impl Selection {
    /// An empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` if `key` is selected.
    #[must_use]
    pub fn contains(&self, key: SelectedKey) -> bool {
        self.keys.contains(&key)
    }

    /// Whether nothing is selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Number of selected keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// All selected keys.
    #[must_use]
    pub fn keys(&self) -> &[SelectedKey] {
        &self.keys
    }

    /// Replace the selection with a single key.
    pub fn set_single(&mut self, key: SelectedKey) {
        self.keys.clear();
        self.keys.push(key);
    }

    /// Toggle a key's membership (shift-click).
    pub fn toggle(&mut self, key: SelectedKey) {
        if let Some(i) = self.keys.iter().position(|k| *k == key) {
            self.keys.remove(i);
        } else {
            self.keys.push(key);
        }
    }

    /// Add a key if not already present (box-select accumulation).
    pub fn add(&mut self, key: SelectedKey) {
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// The selected key ids for one track (for a bulk op on that track).
    #[must_use]
    pub fn ids_for(&self, target: AnimTarget) -> Vec<KeyId> {
        self.keys
            .iter()
            .filter(|k| k.target == target)
            .map(|k| k.key)
            .collect()
    }
}

/// Panel/transport flags that are not part of the undoable document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineFlags {
    /// Auto-key armed: moving the object records a key at the playhead (W4).
    pub auto_key: bool,
    /// Snap edited/scrubbed times to whole display frames.
    pub frame_snap: bool,
}

impl Default for TimelineFlags {
    fn default() -> Self {
        Self {
            // Armed by default: this is an animation timeline — dragging a bound
            // sprite records a key at the playhead, the primary authoring gesture
            // (the transport pill shows the state and can disarm it).
            auto_key: true,
            frame_snap: true,
        }
    }
}

/// The editor state for the timeline: document + selection + history + flags.
#[derive(Debug, Default, Clone)]
pub struct TimelineState {
    /// The undoable document.
    pub doc: TimelineDoc,
    /// Current key selection (panel state, not undoable).
    pub selection: Selection,
    /// Undo/redo over the document.
    pub history: TimelineHistory,
    /// Transport/panel flags (not undoable).
    pub flags: TimelineFlags,
}

impl TimelineState {
    /// A fresh state around an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Undo one document step (returns `true` if something was undone). The
    /// selection is cleared, since undone keys may no longer exist.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.history.undo(&self.doc) {
            self.doc = prev;
            self.selection.clear();
            true
        } else {
            false
        }
    }

    /// Redo one document step (returns `true` if something was redone).
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.history.redo(&self.doc) {
            self.doc = next;
            self.selection.clear();
            true
        } else {
            false
        }
    }
}
