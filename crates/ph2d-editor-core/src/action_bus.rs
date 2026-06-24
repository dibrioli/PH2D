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

/// Modifier-key context for a [`EditorAction::SelectSprite`] event
/// (Fase 0b — image-tools multi-select). The hero/panel side resolves
/// the OS keyboard modifier into this enum before pushing; the shell
/// dispatches the matching [`crate::screens::hero::GizmoStateGroup`]
/// API call. Stays a plain enum (no `bitflags`) — modifiers in PH2D
/// don't compose meaningfully (Shift+Cmd-click on the same element
/// has no defined semantics in this version).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SelectModifier {
    /// No modifier. Replaces the entire selection with the clicked
    /// sprite as the new primary. Most common path.
    #[default]
    Replace,
    /// Shift held. Adds the clicked sprite to the selection without
    /// dropping current sprites. If already selected, no-op (Shift
    /// re-click is idempotent; use [`Self::Toggle`] for off-on).
    Add,
    /// Cmd (macOS) / Ctrl (Linux/Windows) held. Toggles the clicked
    /// sprite in the selection — adds if absent, removes if present.
    /// Removing the primary promotes the oldest extra to primary.
    Toggle,
}

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
    /// Set the active tool by canonical id (the manifest's `id` field).
    /// Generic activation variant (ADR-0040 TG-A): replaces the
    /// per-tool `ActivateBgRemoval` / `ActivatePadding` etc. Shell
    /// drains by routing through `ToolRegistry::set_active`; the
    /// `mode_on` gate for image tools applies at the shell side, not
    /// in the variant. Adding a new modal tool does not require a new
    /// variant.
    ActivateTool { tool_id: &'static str },

    /// Apply a one-shot or stateful-bake image edit on `entity_bits`,
    /// dispatched by `tool_id` (the manifest id). Generic image-edit
    /// variant (ADR-0040 TG-A): replaces the per-tool `Trim` /
    /// `MakeSquare` / `RealSize` (stateless one-shots) and `Bgremoval`
    /// (stateful bake reading the active tool's pipeline state). The
    /// shell dispatches by `tool_id` to the matching drain function
    /// (which still reads tool-specific state from the active tool when
    /// needed — Padding/BgRemoval bake). The ordering / `mode_on` /
    /// "stays intact across tool switches" gating applies at the shell
    /// drain site, not in the variant. Adding a one-shot image op does
    /// not require a new variant.
    OneShotImageOp {
        tool_id: &'static str,
        entity_bits: u64,
    },

    /// Generic panel → tool event channel (ADR-0040 TG-B). Typed
    /// panels (`ph2d-panel-bgremoval`, `ph2d-panel-padding`, …) convert
    /// their `WidgetEvent`s into a tool-agnostic [`crate::tool::PanelEvent`]
    /// (carrying the widget's `NodeId` + payload) and push this variant;
    /// the shell drains by calling
    /// `tools.active_mut().map(|t| t.handle_panel_event(ev))`. The
    /// semantic mapping (slider id → `BgRemovalUiEdit::Tolerance(v)`)
    /// lives in the tool's `handle_panel_event`, not in the panel, so
    /// adding a panel-edit semantic does not require a new variant here.
    ToolPanelEvent(crate::tool::PanelEvent),

    /// Generic cancel of the active modal tool (ADR-0040 TG-B/TG-C).
    /// Shell drains by calling `tools.set_active(default)` and tearing
    /// down any tool-specific shell-side preview state. Raised by both
    /// panels' Cancel buttons (BgRemoval + Padding).
    CancelActiveTool,

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

    /// 2026-05-26 — user clicked the per-row lock icon. Shell drains
    /// and flips presence of `ph2d_ecs::Locked` on the row's entity
    /// (only the entity is locked; descendants remain editable).
    HierToggleLock { row: ph2d_a11y::NodeId },

    /// 2026-05-26 — user clicked the per-row "group lock" (folder)
    /// icon. Shell drains and flips presence of
    /// `ph2d_ecs::GroupedChildren` on the row's entity (descendants
    /// locked; the entity itself remains editable).
    HierToggleGroup { row: ph2d_a11y::NodeId },

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

    /// Composite the current multi-selection (≥ 2 sprites) into one new Individual-texture sprite at
    /// the union bbox, then despawn the originals. Payload: the right-clicked row's `NodeId` (visual
    /// anchor — the merge inherits its parent / z). Drain toasts when < 2 sprites are selected (no-op).
    HierMergeSprites { row: ph2d_a11y::NodeId },

    /// "Use as Brush Texture" — the shell resolves row → entity → pixels → `set_brush_texture_image`
    /// (Enio 2026-06-24); a non-image row toasts and no-ops.
    HierUseAsBrushTexture { row: ph2d_a11y::NodeId },

    /// Sync `gizmo_selection` to the entity backing the clicked
    /// hierarchy row — cross-panel selection sync from the
    /// hierarchy panel to the canvas gizmo. Payload: the row's
    /// `NodeId`. Live (ECS) mode only; fixture-only rows don't
    /// raise this.
    HierRowClick { row: ph2d_a11y::NodeId },

    /// Canvas multi-select event (Fase 0b). Raised by the desktop
    /// shell's canvas pick handler after resolving the click to its
    /// `entity_bits` and reading the OS modifier into a
    /// [`SelectModifier`]. Shell drains by routing through the
    /// matching `GizmoStateGroup` mutation: `Replace` →
    /// `replace_selection`, `Add` → `add_to_selection`, `Toggle` →
    /// `toggle_in_selection`. Hierarchy clicks use the row-keyed
    /// twin [`Self::HierSelectRow`] since the panel cannot resolve
    /// `NodeId → entity_bits` itself.
    SelectSprite {
        entity_bits: u64,
        modifier: SelectModifier,
    },

    /// Hierarchy-panel multi-select event (Fase 0b). Twin of
    /// [`Self::SelectSprite`] but keyed by the row's `NodeId` —
    /// `ph2d-panel-hierarchy` does not have a `NodeId → Entity`
    /// resolver, so the shell does the lookup via the live bridge
    /// before applying the same `GizmoStateGroup` mutation. Replaces
    /// the legacy [`Self::HierRowClick`] for new emitters; the legacy
    /// variant stays in the enum for now to avoid churning shell
    /// drain logic outside Fase 0e.
    HierSelectRow {
        row: ph2d_a11y::NodeId,
        modifier: SelectModifier,
    },

    /// Hierarchy-panel range select (Fase 0b). Shift-click on a live
    /// row with a primary already set: shell walks the hierarchy row
    /// order from the current primary's row to `row` and calls
    /// `add_to_selection` on every entity in between (inclusive). No-op
    /// when nothing is selected (no anchor). Canvas clicks have no
    /// natural linear order so they do not emit this variant.
    HierRangeSelect { row: ph2d_a11y::NodeId },

    /// Drop primary + extras in one go (Fase 0b). Raised by canvas
    /// click on an empty area without modifier, and by pressing Esc
    /// while a selection is active. Shell drains by calling
    /// `GizmoStateGroup::clear_all_selection`.
    ClearSelection,

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

    /// Inspector → shell channel for a §8 Visibility-SECTION field
    /// (VisibilityLayer bitmask / ClipChildren / MaskInteraction /
    /// OnScreenEnabler). Optional-component edit like
    /// [`Self::InspectorSamplingEdit`] (W3 §3.8). Distinct from
    /// [`Self::InspectorVisibilityEdit`], which is the always-on
    /// "Visible" toggle.
    InspectorVisibilitySectionEdit {
        entity_bits: u64,
        edit: crate::screens::hero::VisibilityFieldEdit,
    },

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

    /// Inspector → shell channel for a single editable `Sprite` field
    /// (flip, sprite-sheet grid, region, tint channels, opacity, …).
    /// The shell reads the entity's current `Sprite`, applies the one
    /// `edit` (clamping per the schema), and writes the whole struct via
    /// `EditorCommand::SetComponent` — same path as the Transform commit.
    /// Raised by the Render Source / Sprite Sheet / 9-Slice / Color & Tint
    /// sections (W2 Sprite Inspector v2).
    InspectorSpriteEdit {
        entity_bits: u64,
        edit: crate::screens::hero::SpriteFieldEdit,
    },

    /// Inspector → shell channel for a single editable §7 ordering field
    /// (Z Index, Sorting Layer, Y-Sort, Show Behind Parent, …). Unlike
    /// [`Self::InspectorSpriteEdit`], each maps to an *optional* ECS
    /// component: the shell reads-or-defaults, applies, and commits via
    /// `EditorCommand::SetComponent` (insert/update) or `RemoveComponent`
    /// (detach). Raised by the Ordering / Sorting section (W3 Sprite
    /// Inspector v2 §3.7).
    InspectorOrderingEdit {
        entity_bits: u64,
        edit: crate::screens::hero::OrderingFieldEdit,
    },

    /// Inspector → shell channel for a §9 sampling field (Texture Filter
    /// / Repeat). Optional-component edit like
    /// [`Self::InspectorOrderingEdit`] (W3 §3.9).
    InspectorSamplingEdit {
        entity_bits: u64,
        edit: crate::screens::hero::SamplingFieldEdit,
    },

    /// Inspector → shell channel for a §10 Material & Blend field (Blend
    /// Mode). Optional-component edit like [`Self::InspectorSamplingEdit`]
    /// (§3.10); tag `0` (Mix) detaches the `BlendMode` component.
    InspectorBlendEdit {
        entity_bits: u64,
        edit: crate::screens::hero::BlendFieldEdit,
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

    /// Settings → Anthropic API key (P4, ADR-0061). Carries the key the user
    /// entered/pasted in the API-key submenu's `TextInput`. The shell drains and
    /// persists it to the user config dir (`llm_vector::save_api_key`); the
    /// LLM-vector authoring feature resolves it lazily on the next generation.
    /// Raised by clicking `CTX_MENU_API_KEY_SAVE`.
    SetApiKey(String),

    /// LLM vector authoring (P4, ADR-0061). Carries the natural-language prompt
    /// the user entered in the vector-prompt dialog. The shell drains it and runs
    /// the LLM generation off-thread (`App::llm_vector.submit`), committing the
    /// resulting editable shape to the document. Raised by clicking
    /// `CTX_MENU_VECTOR_PROMPT_GENERATE`.
    GenerateVectorFromPrompt(String),
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
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 42,
        });
        assert_eq!(bus.len(), 1);
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 99,
        });
        assert_eq!(bus.len(), 2);
    }

    #[test]
    fn drain_returns_actions_in_push_order_and_empties() {
        // HR-5: actions drain in the exact push order. The shell
        // relies on this for the gizmo's drag-then-release sequence
        // and similar paired-intent cases.
        let mut bus = ActionBus::new();
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 1,
        });
        bus.push(EditorAction::ActivateTool {
            tool_id: "bgremoval",
        });
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 2,
        });
        let drained: Vec<_> = bus.drain().collect();
        assert_eq!(drained.len(), 3);
        assert_eq!(
            drained[0],
            EditorAction::OneShotImageOp {
                tool_id: "trim_transparency",
                entity_bits: 1
            }
        );
        assert_eq!(
            drained[1],
            EditorAction::ActivateTool {
                tool_id: "bgremoval"
            }
        );
        assert_eq!(
            drained[2],
            EditorAction::OneShotImageOp {
                tool_id: "make_square",
                entity_bits: 2
            }
        );
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
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 1,
        });
        let _ = bus.drain().count();
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "make_square",
            entity_bits: 2,
        });
        assert_eq!(bus.len(), 1);
        let drained: Vec<_> = bus.drain().collect();
        assert_eq!(
            drained,
            vec![EditorAction::OneShotImageOp {
                tool_id: "make_square",
                entity_bits: 2
            }]
        );
    }

    #[test]
    fn clear_empties_without_dispatching() {
        let mut bus = ActionBus::new();
        bus.push(EditorAction::OneShotImageOp {
            tool_id: "trim_transparency",
            entity_bits: 1,
        });
        bus.push(EditorAction::ActivateTool {
            tool_id: "bgremoval",
        });
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

    #[test]
    fn select_modifier_default_is_replace() {
        assert_eq!(SelectModifier::default(), SelectModifier::Replace);
    }

    #[test]
    fn selection_variants_round_trip_through_bus() {
        let row_a = ph2d_a11y::NodeId(1);
        let row_b = ph2d_a11y::NodeId(2);
        let mut bus = ActionBus::new();
        bus.push(EditorAction::SelectSprite {
            entity_bits: 0xAAAA,
            modifier: SelectModifier::Replace,
        });
        bus.push(EditorAction::HierSelectRow {
            row: row_a,
            modifier: SelectModifier::Add,
        });
        bus.push(EditorAction::HierSelectRow {
            row: row_b,
            modifier: SelectModifier::Toggle,
        });
        bus.push(EditorAction::HierRangeSelect { row: row_b });
        bus.push(EditorAction::ClearSelection);
        let drained: Vec<EditorAction> = bus.drain().collect();
        assert_eq!(
            drained,
            vec![
                EditorAction::SelectSprite {
                    entity_bits: 0xAAAA,
                    modifier: SelectModifier::Replace
                },
                EditorAction::HierSelectRow {
                    row: row_a,
                    modifier: SelectModifier::Add
                },
                EditorAction::HierSelectRow {
                    row: row_b,
                    modifier: SelectModifier::Toggle
                },
                EditorAction::HierRangeSelect { row: row_b },
                EditorAction::ClearSelection,
            ]
        );
    }
}
