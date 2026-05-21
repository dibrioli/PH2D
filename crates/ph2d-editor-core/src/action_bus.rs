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

    /// Reset the entity's sprite `Transform.scale` to 1:1 (preserving
    /// flip sign). Payload: `entity.to_bits()`. Mirror of [`Self::Trim`]
    /// for the `IMAGE_ACTION_REAL_SIZE` action pill; the shell drain
    /// applies `ph2d_tool_real_size::real_size_scale` to the ECS
    /// `Transform` (no pixel work, no texture rebind).
    RealSize { entity_bits: u64 },

    /// Activate the stateful BgRemoval tool. No payload — the shell
    /// pulls the active selection from `HeroScreen::gizmo_selection`
    /// when dispatching. Raised by clicking
    /// `IMAGE_ACTION_BGREMOVAL`.
    ActivateBgRemoval,

    /// Apply Background Removal at full resolution to the entity's
    /// sprite. Payload: `entity.to_bits()`. Raised by the shell when
    /// the `BgRemovalTool` panel's Apply Toggle fires (the tool sets
    /// `pending_apply = true` inside `handle_panel_event`, the shell
    /// pushes this variant with the current selection). Shell drain
    /// runs `BgRemovalTool::run_full_resolution` against the source
    /// RGBA and swaps `Sprite.source` to a fresh `Individual`
    /// texture. Gated at drain time on `bgremoval` being the active
    /// tool — if not active when drained, the action is pushed back
    /// onto the bus for the next frame (preserves the
    /// `pending_bgremoval` "stays intact across tool switches"
    /// contract).
    Bgremoval { entity_bits: u64 },

    /// One Background-Removal panel edit (mode / slider / Apply) routed
    /// from the typed `ph2d-panel-bgremoval` to the shell. The shell
    /// drains it and calls `BgRemovalTool::apply_ui_edit` against the
    /// active tool instance (the tool lives in the shell's
    /// `ToolRegistry`, unreachable from `HeroScreen`, so the panel can't
    /// mutate it directly — same bus round-trip as `ActivateBgRemoval`).
    /// On `BgRemovalUiEdit::Apply` the shell additionally pushes a
    /// [`Self::Bgremoval`] for the active selection to commit full-res.
    BgremovalUiEdit(crate::tools::bgremoval::BgRemovalUiEdit),

    /// Cancel Background Removal: abandon the live preview (no commit)
    /// and deactivate the tool so the panel hides and the Inspector
    /// returns. Raised by the panel's Cancel button. The shell switches
    /// the active tool back to the default (first-registered) tool.
    BgremovalCancel,

    /// Activate the stateful Padding tool. No payload — the shell pulls
    /// the active selection from `HeroScreen::gizmo_selection` when
    /// dispatching. Raised by clicking `IMAGE_ACTION_PADDING`. Mirror of
    /// [`Self::ActivateBgRemoval`].
    ActivatePadding,

    /// One Padding panel edit (one of the four signed per-edge fields or
    /// Apply) routed from the typed `ph2d-panel-padding` to the shell.
    /// The shell drains it and calls `PaddingTool::apply_ui_edit` against
    /// the active tool instance (the tool lives in the shell's
    /// `ToolRegistry`, unreachable from `HeroScreen`). Mirror of
    /// [`Self::BgremovalUiEdit`].
    PaddingUiEdit(crate::tools::padding::PaddingUiEdit),

    /// Cancel Padding: abandon the in-progress spec (no bake) and
    /// deactivate the tool so the panel hides and the Inspector returns.
    /// Raised by the panel's Cancel button. Mirror of
    /// [`Self::BgremovalCancel`].
    PaddingCancel,

    /// Apply Padding at full resolution to the entity's sprite. Payload:
    /// `entity.to_bits()`. Raised by the shell when the `PaddingTool`
    /// panel's Apply fires. Shell drain reads the source RGBA, runs
    /// `ph2d_tool_padding::add_padding` with the tool's signed per-edge
    /// spec, swaps `Sprite.source` to a fresh `Individual` texture, and
    /// reprojects the pivot. Mirror of [`Self::Bgremoval`].
    Padding { entity_bits: u64 },

    /// Re-decode the entity's sprite source asset at the current
    /// `ProjectSettings::pixels_per_meter` and write the recomputed
    /// world size back to `Sprite.size`. Payload: `entity.to_bits()`.
    /// Raised by the Inspector's "Reimport at current px/m" button
    /// (`INSP_RENDER_SOURCE_REIMPORT`). Texture itself unchanged;
    /// only `Sprite.size` is recomputed.
    Reimport { entity_bits: u64 },

    /// Undo the most recent image-edit (Trim Transparency / Make
    /// Square / Bg Removal). No payload — the shell owns the
    /// snapshot. Raised by clicking `TOOL_UNDO` on the LeftRail or
    /// pressing Cmd+Z / Ctrl+Z in the desktop shell. Single-level
    /// by design; the broader editor-undo system is M14.x scope.
    UndoImageEdit,

    /// Toggle the `Visibility` component on the entity backing the
    /// hierarchy row whose eye-icon was just clicked. Payload: the
    /// row's `NodeId`. The shell resolves NodeId → Entity via
    /// `HeroLive::bridge.entity_for(row)` and flips `Visibility.hidden`
    /// on `SimWorld`.
    HierToggleVisibility { row: ph2d_a11y::NodeId },

    /// Drag-and-drop reparent for a hierarchy row. Payload mirrors
    /// the `WidgetEvent::HierReparent` event one-to-one. `new_parent
    /// = None` is a root-level drop; `before`/`after` position the
    /// dragged entity relative to a target sibling. The shell
    /// resolves NodeIds → Entities via the bridge and runs
    /// `hero_intents::drain_reparent` which rebuilds the bevy_ecs
    /// `Children` ordering by re-inserting `ChildOf` in sequence.
    HierReparent(crate::screens::hero::HierReparentIntent),

    /// Duplicate the entity backing the hierarchy row. Payload: the
    /// row's `NodeId`. Shell copies Transform / Sprite / Name /
    /// ChildOf onto a freshly-spawned entity, suffixes the name
    /// with `_copy`, and toasts on success. Raised by the row's
    /// right-click → Duplicate menu entry.
    HierDuplicate { row: ph2d_a11y::NodeId },

    /// Despawn the entity backing the hierarchy row. Payload: the
    /// row's `NodeId`. Cascades through bevy_ecs 0.18's `ChildOf`
    /// relation, taking descendants with it. Also clears
    /// `gizmo_selection` if it pointed at the deleted entity.
    /// Raised by the row's right-click → Delete menu entry.
    HierDelete { row: ph2d_a11y::NodeId },

    /// Reset the entity's `Transform` to `Transform::IDENTITY`.
    /// Payload: the row's `NodeId`. Raised by the row's right-click
    /// → Reset Transform menu entry.
    HierResetTransform { row: ph2d_a11y::NodeId },

    /// Spawn a new child entity (identity transform, name "Child")
    /// under the hierarchy row. Payload: the parent row's `NodeId`.
    /// Raised by the row's right-click → Add Child menu entry.
    HierAddChild { row: ph2d_a11y::NodeId },

    /// Sync `gizmo_selection` to the entity backing the clicked
    /// hierarchy row — cross-panel selection sync from the
    /// hierarchy panel to the canvas gizmo. Payload: the row's
    /// `NodeId`. Live (ECS) mode only; fixture-only rows don't
    /// raise this.
    HierRowClick { row: ph2d_a11y::NodeId },

    /// One-shot seed of the rename TextInput buffer when inline-
    /// rename mode opens. Payload: the row's `NodeId`. Shell reads
    /// the entity's current `Name`, fills `HIER_RENAME_INPUT.text`,
    /// and selects all. Without the one-shot semantic, subsequent
    /// Backspace edits would get clobbered back to the original
    /// name on every frame. Raised by right-click → Rename and by
    /// long-press on the row.
    HierRenameSeed { row: ph2d_a11y::NodeId },

    /// Finalized rename commit (Enter / blur on the rename
    /// TextInput). Payload: the row's `NodeId` + the trimmed new
    /// name. Shell writes the new `Name` component on the entity
    /// and toasts confirmation, then clears the rename TextInput
    /// buffer. `String` owned-data payload is fine — `EditorAction`
    /// is `Clone` (not `Copy`); see the `editor_action_is_clone_and_partial_eq`
    /// test below.
    HierRenameCommit {
        row: ph2d_a11y::NodeId,
        new_name: String,
    },

    /// Reframe the camera. Payload: which mode to fire (Selected
    /// focuses the current `gizmo_selection`; Camera resets to the
    /// project's default view; All frames every sprite in the scene
    /// with a 10% padding). Raised by clicking `TOOL_HOME` on the
    /// LeftRail (which cycles the 3 modes) and by double-clicking
    /// a live hierarchy row (always `Selected`).
    SetViewFocus {
        kind: crate::screens::hero::ViewFocusKind,
    },

    /// Inspector → shell channel for `Transform` edits — the first
    /// end-to-end consumer of the editor command pipeline. Payload
    /// is the full snapshot the inspector built from its NumberInput
    /// buffers (entity_bits + translation/rotation/scale). Shell
    /// drains, builds a `ph2d_ecs::Transform` from the raw fields,
    /// and pushes a `EditorCommand::SetComponent` to its
    /// `EditorCommandQueue`. Raised by NumberInput commits (Enter /
    /// blur) on `INSP_TRANSFORM_*` and by the Reset Transform button.
    InspectorTransformEdit(crate::screens::hero::InspectorTransformInfo),

    /// Inspector → shell channel for `Visibility` commits. Payload:
    /// the POST-toggle snapshot `(entity_bits, visible)`. Shell drains
    /// and pushes a `EditorCommand::SetComponent` for
    /// `ph2d_ecs::Visibility` — same pipeline as
    /// [`Self::InspectorTransformEdit`]. Raised by flipping the
    /// `INSP_VISIBILITY_CHECK` checkbox.
    InspectorVisibilityEdit(crate::screens::hero::InspectorVisibilityInfo),

    /// Inspector → shell channel for `Sprite` source-strategy
    /// switches. Payload: `(entity_bits, requested_strategy)`.
    /// Shell does the actual swap: Atlas → Individual re-decodes
    /// the source asset via `atlas_asset_map` + `acquire_individual`;
    /// Individual → Atlas and HandPacked transitions surface a toast
    /// in v1. Raised by picking a different segment in the Render
    /// Source segmented switcher.
    InspectorSpriteSourceChange {
        entity_bits: u64,
        strategy: crate::screens::hero::RequestedSpriteStrategy,
    },

    /// Config → "Image filter" pick. Payload: the chosen
    /// [`ImageFilterMode`]. The hero already wrote
    /// `project.image_filter` (so the menu checkmark is correct on the
    /// next paint); this round-trips the change to the shell, which
    /// owns the GPU sampler state and calls
    /// `SpriteRenderer::set_filter_mode(mode)` to rebuild the atlas +
    /// individual samplers and their bind groups. The shell also stores
    /// the mode so the per-frame BG-Removal Vello preview picks the
    /// matching `peniko::ImageQuality`. Raised by clicking a row in the
    /// `SettingsFilterSubmenu`.
    SetImageFilter {
        mode: crate::project::ImageFilterMode,
    },

    /// Config → "Display" present-mode pick. `vsync = true` → `Fifo`
    /// (smooth, hardware-paced motion); `vsync = false` → `Immediate`
    /// (non-blocking, kills the mouse-move stutter at the cost of
    /// vsync pacing). The shell owns the swap chain and calls
    /// `SurfaceContext::set_present_mode`. Raised by clicking a row in
    /// the `SettingsDisplaySubmenu`.
    SetPresentMode { vsync: bool },

    /// Inspector → shell channel for entity-`Name` edits. Payload:
    /// the snapshot `(entity_bits, new_name)`. Shell drains and
    /// pushes a `EditorCommand::SetComponent` for `ph2d_ecs::Name`,
    /// same pipeline as Transform / Visibility. Raised by
    /// `TextChanged` on `INSP_ENTITY_NAME`.
    InspectorNameEdit(crate::screens::hero::InspectorNameInfo),
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

    /// Non-consuming iterator over queued actions. Used by editor-
    /// side guards that need to ask "does the bus already carry a
    /// variant of this kind?" without dispatching it — e.g.
    /// `inspector_sync` skips re-seeding the Inspector's name /
    /// visibility widgets when an unsent edit is already in flight,
    /// so a frame between push + drain doesn't clobber the user's
    /// in-progress UI state.
    pub fn iter(&self) -> std::slice::Iter<'_, EditorAction> {
        self.queue.iter()
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
