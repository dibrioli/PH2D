//! Editor Action Bus — outbound intent queue from the hero screen.
//!
//! Wave 2.5 PR 11.8 foundation. Replaces the per-frame `hero.pending_X`
//! drain pattern with a single FIFO queue of strongly-typed
//! [`EditorAction`] variants. The shell drains the bus once per frame
//! and dispatches each action through a single `apply_editor_action`
//! match arm instead of 20 hand-written drain blocks scattered across
//! [`shells/desktop/src/main.rs`](shells/desktop/src/main.rs) and
//! [`shells/desktop/src/hero_intents.rs`](shells/desktop/src/hero_intents.rs).
//!
//! ## Why a queue instead of `Option<T>` fields
//!
//! Each `hero.pending_X: Option<T>` represented an at-most-one intent
//! per frame. The shell did `if let Some(v) = hero.pending_X.take() { ... }`
//! at ~20 sites in `render_frame()`. That pattern grew main.rs to
//! 2421 LOC and hero_intents.rs to 696 LOC — both currently carrying
//! `// ph2d-loc-cap:` exceptions (Wave 2 PR 11.9).
//!
//! With the bus, each push is a structurally-typed enum variant
//! carrying its payload. Drain is one `match` over a `Vec` instead of
//! 20 conditionals over scattered fields. Migration is incremental —
//! each `pending_X` field that lifts into [`EditorAction`] takes
//! `~10-20 LOC` out of main.rs and shrinks the HR-18 exception window.
//!
//! ## Determinism
//!
//! Actions drain in push order. The hero pushes from within
//! [`HeroScreen::apply_event`] which itself runs once per pointer/key
//! event; the shell drains after the per-frame `apply_event` cascade.
//! Per-event ordering is preserved (HR-5).
//!
//! ## Scope of this commit (foundation only)
//!
//! - Defines [`EditorAction`] enum (3 representative variants —
//!   `Trim` / `MakeSquare` / `Bgremoval`) covering the image-edit
//!   action row Wave 2 PR 11.4 already wired via Registry.
//! - Defines [`ActionBus`] with `push` / `drain` / `is_empty` / `len`.
//! - Comprehensive unit tests pinning the contract.
//!
//! Migration of the remaining `pending_X` fields lands in follow-up
//! PRs (11.8b — image-edit, 11.8c — hierarchy, 11.8d — inspector).
//! Each migration removes that field from `HeroScreen` and the
//! corresponding drain block from `main.rs` / `hero_intents.rs`.

/// One outbound intent from the editor to the shell. Variants are
/// added incrementally as `pending_X` fields migrate into the bus.
/// Each variant carries enough payload that the shell can dispatch
/// without re-reading `HeroScreen` state.
///
/// **Invariant:** every variant is `Copy` or holds owned data — never
/// borrows from `HeroScreen`. The bus must be drainable after the
/// per-frame `apply_event` cascade returns its `&mut self` borrow.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum EditorAction {
    /// Trim transparency from the entity's sprite source.
    /// Payload: `entity.to_bits()`. Hero raises this when the user
    /// clicks `IMAGE_ACTION_TRIM`; shell drains via the
    /// `trim_transparency` algorithm + Individual texture rebind.
    Trim { entity_bits: u64 },

    /// Pad the entity's sprite source to a square. Payload:
    /// `entity.to_bits()`. Mirror of [`Self::Trim`] for the
    /// `IMAGE_ACTION_MAKE_SQUARE` action pill.
    MakeSquare { entity_bits: u64 },

    /// Activate the stateful BgRemoval tool. No payload — the shell
    /// pulls the active selection from `HeroScreen::gizmo_selection`
    /// when dispatching. Raised by clicking
    /// `IMAGE_ACTION_BGREMOVAL`.
    ActivateBgRemoval,
}

/// FIFO queue of [`EditorAction`]s. Held on `HeroScreen` as a single
/// `bus: ActionBus` field replacing the 20 scattered `pending_X`
/// `Option`s. Cleared by the shell once per frame after drain.
#[derive(Debug, Default)]
pub struct ActionBus {
    queue: Vec<EditorAction>,
}

impl ActionBus {
    /// Construct an empty bus. Equivalent to `Default::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `action` to the back of the queue.
    pub fn push(&mut self, action: EditorAction) {
        self.queue.push(action);
    }

    /// Drain every pending action. The bus is empty after this call.
    /// Returns an iterator the shell consumes via a single `match`.
    pub fn drain(&mut self) -> std::vec::Drain<'_, EditorAction> {
        self.queue.drain(..)
    }

    /// True iff no actions are queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Number of pending actions.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Discard any queued actions without dispatching. Used by tests
    /// + reset paths; production code should always `drain`.
    #[cfg(any(test, debug_assertions))]
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_bus_is_empty() {
        let bus = ActionBus::new();
        assert!(bus.is_empty());
        assert_eq!(bus.len(), 0);
    }

    #[test]
    fn push_then_len_grows() {
        let mut bus = ActionBus::new();
        bus.push(EditorAction::Trim { entity_bits: 42 });
        assert_eq!(bus.len(), 1);
        bus.push(EditorAction::MakeSquare { entity_bits: 99 });
        assert_eq!(bus.len(), 2);
    }

    #[test]
    fn drain_returns_actions_in_push_order_and_empties() {
        // HR-5: actions drain in the exact push order. The shell
        // relies on this for the gizmo's drag-then-release sequence
        // and similar paired-intent cases.
        let mut bus = ActionBus::new();
        bus.push(EditorAction::Trim { entity_bits: 1 });
        bus.push(EditorAction::ActivateBgRemoval);
        bus.push(EditorAction::MakeSquare { entity_bits: 2 });
        let drained: Vec<_> = bus.drain().collect();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0], EditorAction::Trim { entity_bits: 1 });
        assert_eq!(drained[1], EditorAction::ActivateBgRemoval);
        assert_eq!(drained[2], EditorAction::MakeSquare { entity_bits: 2 });
        assert!(bus.is_empty(), "bus must be empty after drain");
    }

    #[test]
    fn drain_on_empty_bus_returns_zero_items() {
        let mut bus = ActionBus::new();
        let drained: Vec<_> = bus.drain().collect();
        assert!(drained.is_empty());
        assert!(bus.is_empty());
    }

    #[test]
    fn push_after_drain_starts_fresh_sequence() {
        let mut bus = ActionBus::new();
        bus.push(EditorAction::Trim { entity_bits: 1 });
        let _ = bus.drain().count();
        bus.push(EditorAction::MakeSquare { entity_bits: 2 });
        assert_eq!(bus.len(), 1);
        let drained: Vec<_> = bus.drain().collect();
        assert_eq!(drained, vec![EditorAction::MakeSquare { entity_bits: 2 }]);
    }

    #[test]
    fn clear_empties_without_dispatching() {
        let mut bus = ActionBus::new();
        bus.push(EditorAction::Trim { entity_bits: 1 });
        bus.push(EditorAction::ActivateBgRemoval);
        bus.clear();
        assert!(bus.is_empty());
        // Subsequent drain yields nothing.
        let drained: Vec<_> = bus.drain().collect();
        assert!(drained.is_empty());
    }

    #[test]
    fn editor_action_is_clone_and_partial_eq() {
        // Variants must implement these for test-side equality checks
        // and the shell's `match` clone scenarios. Locking via a
        // structural check so adding a non-Clone payload field fails
        // here loudly.
        fn assert_clone_partialeq<T: Clone + PartialEq>() {}
        assert_clone_partialeq::<EditorAction>();
    }
}
