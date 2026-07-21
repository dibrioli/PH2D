//! [`TimelineState`] — the full editor state the shell embeds in `AppGfx`:
//! the [`TimelineDoc`] plus panel-side selection, undo [`TimelineHistory`], and
//! transport-adjacent flags.
//!
//! Only the `doc` is undoable; selection + flags are panel state (never
//! snapshotted). Pan/zoom live on the panel itself (`Panel::State`), not here.

use ph2d_anim::{AnimTarget, KeyId};

use crate::clipboard::TimelineClipboard;
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

impl SelectedKey {
    /// Build from raw ids — lets a UI layer that carried the key's identity as
    /// primitives (e.g. a `TimelineHitKind::Key { target, key }` gesture)
    /// reconstruct the typed selection without naming the `ph2d-anim` types.
    #[must_use]
    pub const fn new(target_raw: u64, key_raw: u64) -> Self {
        Self {
            target: AnimTarget::new(target_raw),
            key: KeyId::new(key_raw),
        }
    }
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
    /// **Performing / record** armed (W5): while the transport is PLAYING,
    /// dragging a bound object records its pose live along the playhead — mocap
    /// by hand. Distinct from `auto_key`, which is inert during play precisely
    /// so the tocando animation cannot mint keys on its own: record fires ONLY
    /// with an active manipulation gesture, never the passive pose the curve is
    /// driving. Off by default — it is a modal, explicit mode.
    pub performing: bool,
    /// **Play drives the rigid simulation too** (ADR-0131).
    ///
    /// The transport is ONE clock with two consumers: the curves in this
    /// document, and the rapier world. Left implicit they run together, and
    /// that is a conflict rather than a feature — scrubbing to review an
    /// animation would also drop every dynamic body a little further, so the
    /// scene the artist is judging is never the scene they authored.
    ///
    /// So the two are separated at the transport, where the artist can see
    /// which one is armed. **Off by default:** this is an animation timeline,
    /// and the answer to "what does Play do?" has to be the same on the frame
    /// after a project loads as it was before it was saved — a simulation that
    /// starts itself is a scene that has already changed by the time it is
    /// looked at. Physics is opted INTO, per session.
    ///
    /// ⚠️ Disarming does not freeze the objects, it stops *simulating* them:
    /// a body whose pose the timeline drives (a baked one, W4) still follows
    /// its curves, because those are animation. That is the whole shape of
    /// Bake — it converts a simulation into an animation, and an animation is
    /// exactly what plays with this off.
    pub simulate_physics: bool,
}

impl Default for TimelineFlags {
    fn default() -> Self {
        Self {
            // Armed by default: this is an animation timeline — dragging a bound
            // sprite records a key at the playhead, the primary authoring gesture
            // (the transport pill shows the state and can disarm it).
            auto_key: true,
            frame_snap: true,
            // Record is modal and deliberate — never armed on its own.
            performing: false,
            // Physics is opted into: Play means "play my animation" until the
            // artist says otherwise. See the field docs.
            simulate_physics: false,
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
    /// Copied keys (panel state, not undoable, not serialized).
    pub clipboard: TimelineClipboard,
    /// **The view the panel last painted is the Keys tab** — the shell stamps this
    /// each frame from `ph2d_panel_timeline::state::keys_mode()`, before draining
    /// intents. It picks which of the active clip's two loops an edit (or a
    /// clip-switch sync) targets: the Keys-view clip-clock loop, or the Arrange
    /// timeline loop. Session state, like [`Self::flags`]; defaults to Arrange.
    pub keys_mode: bool,
    /// **How deep into the nesting the animator has walked** — outermost first, empty at the
    /// scene root (ADR-0133 §5).
    ///
    /// A PATH and not a single container, because the trail has to answer two things that a
    /// single index cannot: *where you came from* (entering B from inside A must still offer
    /// the way back to A) and *where you are* (the trail would otherwise read `Scene > B`
    /// while you stand in `A > B`, which is a wrong statement, not a terse one).
    ///
    /// Routing only ever needs the innermost — that is [`Self::edit_host`], DERIVED here
    /// rather than stamped beside the path, so the two can never disagree about where the
    /// animator is.
    ///
    /// Stamped by the shell each frame from `ph2d_panel_timeline::state::edit_path()`, before
    /// intents drain — the same channel [`Self::keys_mode`] rides, for the same reason: the
    /// panel knows where the animator is looking, and an edit has to land where they are
    /// looking. Session state; a document does not remember which container was open, exactly
    /// as Animate does not.
    pub edit_path: Vec<crate::EnterStep>,
}

impl TimelineState {
    /// **Which stack an edit lands in** — the innermost container of [`Self::edit_path`], or
    /// the document's own stack at the root.
    #[must_use]
    pub fn edit_host(&self) -> crate::nest::StackHost {
        self.edit_path
            .last()
            .map_or(crate::nest::StackHost::Document, |step| {
                crate::nest::StackHost::Container(step.container)
            })
    }

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
