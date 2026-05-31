//! Editor hero — composes the `02-editor-main` mockup
//! ([`docs/design/screens/02-editor-main.html`]) into a single
//! `paint_hero_screen` call.
//!
//! Layout regions (all in viewport-relative pixels):
//!
//! ```text
//! ┌──────────────────────────────────────────────────┐
//! │            TopBar  (h≈40, full width inset 14)   │
//! ├────┬──────────────────────────────────┬──────────┤
//! │ R  │                                  │          │
//! │ a  │            CANVAS                │  Hier    │
//! │ i  │                                  │  (fixed) │
//! │ l  │                                  │          │
//! │ 56 │  + Inspector overlay (left:84)   │          │
//! ├────┴──────────────────────────────────┴──────────┤
//! │           BottomHUD (centered pill)              │
//! └──────────────────────────────────────────────────┘
//! ```
//!
//! Region painters live in sibling sub-modules
//! ([`canvas`], [`topbar`], [`left_rail`], [`bottom_hud`],
//! [`selection`]). Inspector + Hierarchy panels live in their own
//! crates (`ph2d-panel-inspector`, `ph2d-panel-hierarchy`) per
//! ADR-0029 Phase C.1/C.2. Shared layout constants + small helpers
//! in [`style`]; stable `NodeId`s in [`ids`]. Hardcoded mockup
//! content stays in [`fixture`] until a
//! pilot project picks the entity model.

pub mod bottom_hud;
pub mod canvas;
pub mod chrome;
pub mod color_picker_demo;
pub mod context_menu_overlay;
pub mod fixture;
// Wave 6+7 Phase 2: hero ids promoted to ph2d-editor-core so dispatch
// and panel crates can reach them without depending back on hero. The
// `screens::hero::ids` path continues to resolve via this re-export.
pub use crate::ids;
pub mod left_rail;
pub mod pre_populate;
pub mod selection;
pub mod state;
pub mod style;
pub mod topbar;

pub use state::{GizmoStateGroup, GridState, ImageEditState, ViewState};

pub use bottom_hud::{BottomHudStats, paint_bottom_hud};
pub use canvas::{paint_canvas_bg, paint_drop_overlay};
pub use color_picker_demo::paint_blender_picker_demo;
pub use left_rail::paint_left_rail;
pub use selection::paint_selection_overlay;
pub use style::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
pub use topbar::paint_top_bar;

use crate::interaction::{
    HitIndex, WidgetEvent, WidgetStore, dispatch_pointer, dispatch_pointer_with_text,
};
use crate::zones::Rect;
use bumpalo::Bump;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_host::{KeyEvent, PointerEvent};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

// ADR-0029 Phase D: `HeroLayout` collapsed — single canonical definition
// lives in `crate::screens::layout`. Re-exported here so legacy paths
// (`crate::screens::hero::HeroLayout`, `super::HeroLayout` from sibling
// painters) keep resolving.
pub use crate::screens::layout::HeroLayout;

/// Selection state surfaced by the hero (drives the marquee + tag).
#[derive(Clone, Debug, Default)]
pub struct HeroSelection {
    pub label: String,
    pub kind: String,
    pub world_pos: (f32, f32),
}

#[derive(Debug)]
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    /// Text rendering strategy — orthogonal to `theme`. `Default`
    /// preserva o visual histórico; `Crisp` aplica snap-X + boost
    /// de FontWeight por faixa de tamanho. Persistência: runtime-only
    /// (não save). Toggle via `Settings ▸ Text rendering ▸ ...`.
    pub text_rendering: ph2d_tokens::TextRendering,
    pub selection: Option<HeroSelection>,
    /// Per-widget interactive state (hover/press/focus). Pre-populated
    /// at construction; mutated in-place by [`HeroScreen::handle_pointer`].
    pub store: WidgetStore,
    /// Per-frame hit-test index. Cleared at the start of each
    /// `paint_hero_screen` call and re-populated as painters emit
    /// geometry.
    pub hit_index: HitIndex,
    /// Outbound action queue (Wave 2.5 PR 11.8). Replaces the
    /// `pending_X: Option<T>` scatter-pattern with a strongly-typed
    /// FIFO of [`crate::action_bus::EditorAction`]. Hero pushes from
    /// inside [`HeroScreen::apply_event`]; shell drains once per frame
    /// via `hero.bus.drain()`. Migration is incremental — variants
    /// land one at a time as `pending_X` fields fold into the bus.
    pub bus: crate::action_bus::ActionBus,
    /// Wave 5 stage B: view-state flags — mirror toggle + stats HUD /
    /// widget gallery / grid overlay visibility + gallery rect.
    pub view: ViewState,
    /// ADR-0029 Phase C.1: per-panel visibility map keyed by
    /// [`crate::panel::Panel::ID`]. Host-side persistence replaces
    /// the legacy `hero.inspector.visible` field; left-rail toggles
    /// plus panel-close affordances mutate this map; orchestrator
    /// reads it to publish chrome rects. `BTreeMap` (not `HashMap`)
    /// per HR-5: bit-determinism rules out non-fixed hashers.
    pub panel_visibility: std::collections::BTreeMap<&'static str, bool>,
    /// Wave 5 stage B: image-edit subsystem state — TopBar Image-Tools
    /// mode flag + undo-availability signal from host.
    pub image_edit: ImageEditState,
    /// Wave 5 stage B: canvas gizmo state — selection + per-frame view
    /// + in-progress drag.
    pub gizmo: GizmoStateGroup,
    /// Wave 5 stage B: grid subsystem state — per-frame projection view
    /// + paint config + snap state (overlay + per-kind config).
    pub grid: GridState,
    /// M14.4b.bis: set by the VIEW button (`TOOL_HOME`) when its
    /// cycle lands on the "Zero" mode, signaling the host to reset
    /// `Camera2d` to its default (`center=(0,0)`, `height_world=10`).
    /// The shell polls this flag after `paint_hero_screen` and
    /// clears it after acting.
    pub camera_reset_pending: bool,
    /// M14.4c: set by the "Import…" context-menu entry
    /// (`CTX_MENU_IMPORT`). The shell polls this flag, opens the
    /// native file picker, and processes any selected images
    /// (PNG/WEBP/JPEG). Cleared by the shell after handling.
    pub import_requested: bool,
    /// Project-level configuration (px/meter, future global toggles).
    /// Edited via the TopBar Settings cluster; read by the shell
    /// during image import to convert source-pixel dimensions to
    /// world meters.
    pub project: crate::project::ProjectSettings,
    /// M14.4e: when the OS is hovering external files over the
    /// window, the host pushes the `(paths, cursor_px)` tuple here so
    /// the canvas painter can render a "Drop to import" overlay
    /// (translucent blue band + caption with file count + first name).
    /// Cleared on `on_file_hover_cancel` or after `on_file_drop` is
    /// processed.
    pub dragging_files: Option<(Vec<std::path::PathBuf>, (f32, f32))>,
    /// M14.4g Telemetry Phase A: real render statistics surfaced in
    /// the bottom HUD. Host assigns directly (`hero.stats = ...`)
    /// once per frame; painter reads them in `paint_bottom_hud`.
    pub stats: BottomHudStats,
    /// Most recent viewport rect — written each frame at the top of
    /// [`paint_hero_screen`]. Chrome event handlers in `chrome/` read
    /// it to make smart layout decisions (e.g. cascade submenus flip
    /// to the left of their parent when the right edge is reached).
    /// Defaults to a zero rect until the first paint.
    pub last_viewport: Rect,
    // Wave 2.5 PR 11.8c: 6 hierarchy fields migrated to the bus.
    //   pending_visibility_toggle → EditorAction::HierToggleVisibility { row }
    //   pending_reparent          → EditorAction::HierReparent(HierReparentIntent)
    //   pending_duplicate         → EditorAction::HierDuplicate { row }
    //   pending_delete            → EditorAction::HierDelete { row }
    //   pending_reset_transform   → EditorAction::HierResetTransform { row }
    //   pending_add_child         → EditorAction::HierAddChild { row }
    // Each push happens in `apply_event` (dispatcher event for
    // visibility/reparent, CTX_MENU_HIER_* for menu actions); the
    // shell drains via `hero.bus.drain()` + filter-and-replace,
    // resolves NodeId → Entity via `HeroLive::bridge`, and runs the
    // ECS mutation.
    //
    // Wave 2.5 PR 11.8c: `pending_hierarchy_row_click` migrated to
    // `bus.push(EditorAction::HierRowClick { row })`. Same drain
    // semantics: shell resolves row NodeId → sim entity via the
    // bridge and updates `gizmo.selection` so the canvas gizmo
    // follows the hierarchy click. Live (ECS) mode only.
    // Wave 2.5 PR 11.8d: `pending_view_focus` migrated to
    // `bus.push(EditorAction::SetViewFocus { kind })`. Raised by
    // the F/Home key, the VIEW button on the left rail (TOOL_HOME
    // cycles Selected/Camera/All), and double-click on a live row
    // (always Selected).
    // Wave 2.5 PR 11.8c: rename intents migrated to the bus.
    //   pending_rename_seed   → EditorAction::HierRenameSeed { row }
    //   pending_rename_commit → EditorAction::HierRenameCommit { row, new_name }
    // Wave 2.5 PR 11.8b1-3: image-edit + bgremoval + reimport intents
    // all live on the bus. ADR-0040 TG-A/B/C genericized the per-tool
    // variants into ActivateTool / OneShotImageOp / ToolPanelEvent /
    // CancelActiveTool; the non-tool variants (Reimport, UndoImageEdit)
    // stayed as-is.
    // Wave 2.5 PR 11.8d: inspector edits live on the bus
    // (InspectorTransformEdit / InspectorVisibilityEdit /
    //  InspectorNameEdit / InspectorSpriteSourceChange variants).
    //
    // Wave 5 stage B: 21 flat state fields moved into the 6 sub-state
    // groups declared above (`view`, `inspector`, `hierarchy`,
    // `image_edit`, `gizmo`, `grid`). Read access uses the structural
    // path (`hero.inspector.sprite`, `hero.view.ui_mirrored`, etc.).
    // Snapshot types `InspectorSpriteInfo` / `InspectorTransformInfo` /
    // `InspectorVisibilityInfo` / `InspectorNameInfo` keep their
    // definitions in this file (re-exported by `screens::hero` for the
    // crate-wide import surface; `state.rs` re-imports them from here).
}

/// Snapshot of the selected sprite's editor-facing fields. Host
/// rebuilds this each frame from `gizmo_selection` + SimWorld;
/// inspector renders read-only display + a Reimport button.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorSpriteInfo {
    /// Entity bits (= same shape `gizmo_selection` carries).
    pub entity_bits: u64,
    /// Display label — entity's `Name` component, or
    /// `Entity_{hex_bits}` when nameless.
    pub name: String,
    /// World-space size in meters at the current Transform scale.
    pub world_size: [f32; 2],
    /// Which storage strategy backs the sprite (Atlas / Hand-packed
    /// / Individual). Surfaced as a read-only display for now;
    /// switching strategies is M14.5 follow-up.
    pub source_kind: InspectorSpriteSource,
    /// Source-image dimensions (pixels). `None` for procedural /
    /// generated sprites that don't trace back to an `AssetId`.
    pub source_pixels: Option<(u32, u32)>,
    /// `true` when Reimport is meaningful — the entity's source
    /// resolves to an `AssetId` we can re-decode at the new px/m.
    pub can_reimport: bool,
    /// Logical horizontal flip (mirrors sampled U; survives reparenting).
    /// Editable via the Render Source / Sprite Sheet Flip toggles (W2).
    pub flip_x: bool,
    /// Logical vertical flip (mirrors sampled V).
    pub flip_y: bool,
    /// Final opacity multiplier `[0, 1]` (Color & Tint section). Renders
    /// today via `RenderInstance.opacity`.
    pub opacity: f32,
    /// Silhouette mode — texel RGB ignored, tint RGB fills. Renders today
    /// via `flip_uv` bit 2.
    pub tint_fill: bool,
    /// Sprite-sheet columns (`>= 1`). Renders today: the extract slices
    /// the atlas rect into an hframes×vframes grid.
    pub hframes: u32,
    /// Sprite-sheet rows (`>= 1`).
    pub vframes: u32,
    /// Active sheet frame index (`< hframes*vframes`).
    pub frame: u32,
    /// Inherited modulate (`Sprite::tint`, cascades to children).
    /// Linear RGBA `[0, 1]`. Edited via the Color & Tint section's
    /// Tint swatch (W2.T2.7); renders today via `RenderInstance.tint`.
    pub tint: [f32; 4],
    /// Local modulate (`Sprite::self_tint`, does NOT cascade). Linear
    /// RGBA `[0, 1]`. Edited via the Self Tint swatch; multiplies
    /// `tint` for this sprite only (Godot `self_modulate` semantics).
    pub self_tint: [f32; 4],
    /// Per-corner tint `[TL, TR, BL, BR]` — a 4-stop bilinear gradient.
    /// Each entry linear RGBA `[0, 1]`; default WHITE (no gradient).
    /// Edited via the Per-corner 2×2 swatch grid (W2.T2.7); renders via
    /// the shader's `@location(9..12)` per-corner attributes.
    pub per_corner_tint: [[f32; 4]; 4],
    /// Region sampling on/off (`Sprite::region_enabled`). When `true`,
    /// the sprite samples only `region_rect` of its source. Edited in the
    /// Render Source section (W2.T2.4); renders via the extract sub-UV.
    pub region_enabled: bool,
    /// Region sub-rect `[x, y, w, h]` in SOURCE pixels
    /// (`Sprite::region_rect`). Only meaningful when `region_enabled`.
    pub region_rect: [f32; 4],
    /// Region filter clip (`Sprite::region_filter_clip`) — clamps the
    /// sampler to `region_rect` (half-texel inset) to stop atlas bleed.
    pub region_filter_clip: bool,
    /// Origin mode (`Sprite::centered`): `true` (default) = quad center;
    /// `false` = texture top-left + `offset`. Edited in the Sprite Sheet
    /// section (W2.T2.6); renders via `Sprite::resolve_anchor`.
    pub centered: bool,
    /// Intrinsic image offset in pixels (`Sprite::offset`), applied after
    /// `centered`. Renders via `Sprite::resolve_anchor` (px → local m).
    pub offset: [f32; 2],
    /// Number of sprites in the active selection (primary + extras).
    /// `1` for a single selection; `> 1` enables BulkSelect (T2.0): edits
    /// apply to all, and diverging fields show as "Mixed" via [`mixed`].
    ///
    /// [`mixed`]: InspectorSpriteInfo::mixed
    pub selected_count: usize,
    /// Per-field "values diverge across the selection" flags (BulkSelect).
    /// All `false` for a single selection. A `true` flag makes the field
    /// render its Mixed affordance (checkbox → Indeterminate, NumberInput
    /// → blank) so editing it doesn't silently stomp the diverging values.
    pub mixed: InspectorSpriteMixed,
}

/// BulkSelect (T2.0): which editable `Sprite` fields diverge across a
/// multi-selection. Computed by the host each frame (compare every
/// selected sprite against the primary). Default = nothing mixed (the
/// single-selection case). The Inspector reads these to show "Mixed"
/// affordances instead of a misleading single value.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorSpriteMixed {
    pub flip_x: bool,
    pub flip_y: bool,
    pub tint_fill: bool,
    pub centered: bool,
    pub region_enabled: bool,
    pub region_filter_clip: bool,
    pub opacity: bool,
    pub hframes: bool,
    pub vframes: bool,
    pub frame: bool,
    pub offset_x: bool,
    pub offset_y: bool,
    pub region_x: bool,
    pub region_y: bool,
    pub region_w: bool,
    pub region_h: bool,
    pub tint: bool,
    pub self_tint: bool,
    /// Any of the 4 per-corner tints diverge.
    pub per_corner: bool,
}

/// A single editable `Sprite` field, dispatched Inspector → shell as
/// [`EditorAction::InspectorSpriteEdit`]. The shell reads the entity's
/// current `Sprite`, applies the one field (clamping where the schema
/// requires), and commits the whole struct via
/// `EditorCommand::SetComponent` — the same write path the Transform
/// commit uses. Payloads are primitives so this enum stays free of any
/// `ph2d-render` dependency (editor-core must not depend on the renderer).
///
/// Variants are added as each W2 section lands; only the ones a wired
/// section emits are produced today (Flip in W2.T2.4/T2.5). The full set
/// is declared up front so the action contract is stable.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SpriteFieldEdit {
    /// Logical horizontal flip.
    FlipX(bool),
    /// Logical vertical flip.
    FlipY(bool),
    /// `false` = top-left origin + `offset` applies; `true` = centered.
    Centered(bool),
    /// Intrinsic-pixel offset (whole vector) applied after `centered`.
    Offset([f32; 2]),
    /// Intrinsic-pixel offset X only — leaves Y untouched. The Inspector
    /// emits this (not [`Offset`]) so editing one axis on a multi-selection
    /// can't stomp a diverging Y (BulkSelect, audit D-1).
    ///
    /// [`Offset`]: SpriteFieldEdit::Offset
    OffsetX(f32),
    /// Intrinsic-pixel offset Y only — leaves X untouched. See [`OffsetX`].
    ///
    /// [`OffsetX`]: SpriteFieldEdit::OffsetX
    OffsetY(f32),
    /// Sprite-sheet columns (`>= 1`; clamped at the commit boundary).
    Hframes(u32),
    /// Sprite-sheet rows (`>= 1`).
    Vframes(u32),
    /// Active frame index (`< hframes * vframes`; clamped).
    Frame(u32),
    /// Region (sub-rect) sampling on/off.
    RegionEnabled(bool),
    /// Region rect `[x, y, w, h]` in source pixels (whole vector).
    RegionRect([f32; 4]),
    /// Region rect X only — leaves Y/W/H untouched (BulkSelect, audit D-1).
    RegionX(f32),
    /// Region rect Y only.
    RegionY(f32),
    /// Region rect W only (`>= 0`; clamped at commit).
    RegionW(f32),
    /// Region rect H only (`>= 0`; clamped at commit).
    RegionH(f32),
    /// Clamp the sampler to the region (atlas-bleed guard).
    RegionFilterClip(bool),
    /// Inherited modulate (cascades to children).
    Tint([f32; 4]),
    /// Local modulate (does NOT cascade).
    SelfTint([f32; 4]),
    /// Silhouette mode — texel RGB ignored, tint RGB fills.
    TintFill(bool),
    /// Final opacity multiplier (`[0, 1]`; clamped).
    Opacity(f32),
    /// Per-corner bilinear tint `[TL, TR, BL, BR]`.
    PerCornerTint([[f32; 4]; 4]),
}

/// Snapshot of the selected entity's W3 ordering/sorting components
/// published to the Inspector §7 (Ordering / Sorting). Every field is
/// *optional* (the components are presence-overrides, spec §02): `None`
/// / `false` markers mean "component absent → pipeline default". Raw
/// primitives keep editor-core loose-coupled from `ph2d-ecs`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorOrderingInfo {
    pub entity_bits: u64,
    /// `ZIndexOverride` — `None` = absent (DFS counter). `Some(v)` =
    /// forced Z (spec §3.7: "Z Index: —" vs explicit).
    pub z_index: Option<i32>,
    /// `ZAsRelative.0` — only meaningful when `z_index.is_some()`.
    pub z_as_relative: bool,
    /// `ShowBehindParent` marker present.
    pub show_behind_parent: bool,
    /// `SortingLayer.0.0` (LayerId index); default-layer index when absent.
    pub sorting_layer: u8,
    /// `OrderInLayer.0`.
    pub order_in_layer: i32,
    /// `YSort.enabled` (false when the component is absent).
    pub y_sort_enabled: bool,
    /// `YSort.sort_point` as a tag: 0 Center · 1 Pivot · 2 Custom.
    pub y_sort_point: u8,
    /// `YSort.axis` (only meaningful when `y_sort_point == 2`).
    pub y_sort_axis: [f32; 2],
    /// `SortingGroup` present.
    pub sorting_group: bool,
    /// `SortingGroup.sort_at_root` (only meaningful when `sorting_group`).
    pub sort_at_root: bool,
    /// `TopLevel` marker present.
    pub top_level: bool,
    pub selected_count: usize,
    pub mixed: InspectorOrderingMixed,
}

/// BulkSelect (T2.0) divergence flags for the §7 ordering fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorOrderingMixed {
    pub z_index: bool,
    pub z_as_relative: bool,
    pub show_behind_parent: bool,
    pub sorting_layer: bool,
    pub order_in_layer: bool,
    pub y_sort_enabled: bool,
    pub y_sort_point: bool,
    pub y_sort_axis: bool,
    pub sorting_group: bool,
    pub sort_at_root: bool,
    pub top_level: bool,
}

/// A single editable §7 ordering field, dispatched Inspector → shell as
/// [`EditorAction::InspectorOrderingEdit`]. Unlike [`SpriteFieldEdit`]
/// (which mutates the always-present `Sprite`), each variant maps to an
/// *optional* ECS component: the shell reads the component-or-default,
/// applies the edit, and commits via `EditorCommand::SetComponent`
/// (insert/update) or `EditorCommand::RemoveComponent` (detach). The
/// full set is declared up front so the action contract is stable; only
/// wired controls emit today (spec §3.7).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OrderingFieldEdit {
    /// `Some(v)` attaches/updates `ZIndexOverride(v)` (clamped to
    /// ±i32::MAX/2 at commit); `None` detaches it (back to DFS).
    ZIndex(Option<i32>),
    /// `ZAsRelative(b)` (attaches the component if absent).
    ZAsRelative(bool),
    /// Toggle the `ShowBehindParent` marker (insert / remove).
    ShowBehindParent(bool),
    /// `SortingLayer(LayerId(idx))`.
    SortingLayer(u8),
    /// `OrderInLayer(v)`.
    OrderInLayer(i32),
    /// `YSort.enabled` (read-modify-write the YSort component).
    YSortEnabled(bool),
    /// `YSort.sort_point` tag: 0 Center · 1 Pivot · 2 Custom.
    YSortPoint(u8),
    /// `YSort.axis`.
    YSortAxis([f32; 2]),
    /// Toggle `SortingGroup` presence (insert default / remove).
    SortingGroup(bool),
    /// `SortingGroup.sort_at_root` (attaches the component if absent).
    SortAtRoot(bool),
    /// Toggle the `TopLevel` marker (insert / remove).
    TopLevel(bool),
}

/// Snapshot of the selected entity's W3 §9 sampling components
/// (`TextureFilter`/`TextureRepeat`). Tags are the
/// `ph2d_ecs::FilterMode`/`RepeatMode` discriminants; `0` = `Inherit`
/// (component absent or explicitly Inherit). Raw primitives keep
/// editor-core loose-coupled from `ph2d-ecs`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorSamplingInfo {
    pub entity_bits: u64,
    pub filter_tag: u8,
    pub repeat_tag: u8,
    /// `UvTransform.scale` (W3 tiling; `[1,1]` = no tiling).
    pub uv_scale: [f32; 2],
    /// `UvTransform.offset` (W3 scroll; `[0,0]` = none).
    pub uv_offset: [f32; 2],
    pub selected_count: usize,
    pub mixed: InspectorSamplingMixed,
}

/// BulkSelect divergence flags for the §9 sampling fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorSamplingMixed {
    pub filter: bool,
    pub repeat: bool,
    pub uv_scale: bool,
    pub uv_offset: bool,
}

/// A single editable §9 sampling field, dispatched as
/// [`EditorAction::InspectorSamplingEdit`]. Filter/Repeat map to the
/// optional `TextureFilter`/`TextureRepeat` (tag `0` = detach); UV
/// scale/offset map to the optional `UvTransform`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SamplingFieldEdit {
    /// `FilterMode` tag (`0 Inherit … 6 LinearAniso`).
    Filter(u8),
    /// `RepeatMode` tag (`0 Inherit · 1 Disabled · 2 Enabled · 3 Mirror`).
    Repeat(u8),
    /// `UvTransform.scale.x` (W3 tiling). Read-modify-write the component.
    UvScaleX(f32),
    UvScaleY(f32),
    /// `UvTransform.offset.x` (W3 scroll).
    UvOffsetX(f32),
    UvOffsetY(f32),
}

/// W3 §8 Visibility-section snapshot (the collapsible section body, NOT
/// the always-on "Visible" toggle — that stays [`InspectorVisibilityInfo`]).
/// Mirrors the optional ECS components `VisibilityLayer` / `ClipChildren`
/// / `MaskInteraction` / `OnScreenEnabler`; editor-core stays loose-coupled
/// from `ph2d-ecs` so the shell maps tags ↔ enums at the boundary.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorVisibilitySectionInfo {
    pub entity_bits: u64,
    /// `VisibilityLayer` bitmask; absent component → `u32::MAX` (all 32
    /// layers, the canonical "visible to every camera" default).
    pub layer_mask: u32,
    /// `ClipChildren.mode` tag (`0 Disabled · 1 ClipOnly · 2 ClipAndDraw`).
    pub clip_mode: u8,
    /// `MaskInteraction.mode` tag (`0 None · 1 VisibleInside · 2 VisibleOutside`).
    pub mask_mode: u8,
    /// `MaskInteraction.alpha_cutoff` (`[0,1]`; shown when mask != None).
    pub alpha_cutoff: f32,
    /// `Mask2D` present? (this sprite is a mask SOURCE).
    pub mask_source: bool,
    /// `OnScreenEnabler` present?
    pub on_screen: bool,
    /// `OnScreenEnabler.rect` `[x, y, w, h]` world meters (shown when on).
    pub rect: [f32; 4],
    pub selected_count: usize,
    pub mixed: InspectorVisibilityMixed,
}

/// BulkSelect divergence flags for the §8 visibility-section fields.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct InspectorVisibilityMixed {
    pub layer_mask: bool,
    pub clip_mode: bool,
    pub mask_mode: bool,
    pub alpha_cutoff: bool,
    pub mask_source: bool,
    pub on_screen: bool,
    pub rect: bool,
}

/// A single editable §8 visibility-section field, dispatched as
/// [`EditorAction::InspectorVisibilitySectionEdit`]. Each maps to an
/// optional ECS component (presence = override): a `0`/`Disabled`/`None`
/// tag or an all-layers/false value detaches it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VisibilityFieldEdit {
    /// Toggle `VisibilityLayer` bit `n` (`0..32`) on/off.
    LayerBit(u8, bool),
    /// `ClipChildren.mode` tag (`0` detaches → no clip).
    ClipMode(u8),
    /// `MaskInteraction.mode` tag (`0` detaches → ignores masks).
    MaskMode(u8),
    /// `MaskInteraction.alpha_cutoff` (read-modify-write, keeps mode).
    AlphaCutoff(f32),
    /// `Mask2D` present? — make this sprite a mask source (`false` detaches).
    MaskSource(bool),
    /// `OnScreenEnabler` present? (`false` detaches.)
    OnScreen(bool),
    /// `OnScreenEnabler.rect` components (read-modify-write).
    RectX(f32),
    RectY(f32),
    RectW(f32),
    RectH(f32),
}

/// Mirror of `ph2d_render::SpriteSource` that doesn't depend on
/// the renderer crate. Stays small (1 enum tag + opt u32) so the
/// `Inspector*` struct is cheap to clone per frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InspectorSpriteSource {
    Atlas { key: u32 },
    Individual { texture_id: u32 },
    HandPacked,
}

/// Snapshot of the selected entity's local `Transform` published to
/// the Inspector. Mirrors the canonical [`ph2d_ecs::Transform`]
/// fields as raw arrays so the editor crate stays loose-coupled to
/// glam types and the snapshot stays cheap to clone.
///
/// Lifecycle: host writes this when the selection changes (or a
/// gizmo-driven external mutation lands); inspector seeds its
/// NumberInput buffers from it; commits flow back through
/// [`HeroScreen::pending_transform_edit`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InspectorTransformInfo {
    /// Entity bits — same shape `gizmo_selection` carries.
    pub entity_bits: u64,
    /// Local-space translation (meters). 2D-only by design — see
    /// SKILL §3 "Não é engine 3D" and ADR-0025.
    pub translation: [f32; 2],
    /// Local-space rotation in radians. The inspector renders this
    /// as degrees for UX parity with Unity/Godot/Blender; conversion
    /// happens at the paint/commit boundary via
    /// `f32::to_degrees`/`to_radians` (HR-5 bit-deterministic).
    pub rotation_rad: f32,
    /// Local-space scale (unitless). Identity = `[1.0, 1.0]`.
    pub scale: [f32; 2],
    /// Local-space skew `[skew_x, skew_y]` in radians (ADR-0025
    /// amendment-1). The inspector renders these as degrees like
    /// `rotation_rad`; conversion happens at the paint/commit boundary.
    /// Identity = `[0.0, 0.0]`. Authoring values are clamped to
    /// `Transform::SKEW_LIMIT` at the ECS-commit boundary.
    pub skew_rad: [f32; 2],
}

/// M14.D: snapshot of the selected entity's `Visibility` state.
///
/// The Inspector renders this as a single checkbox above the
/// Transform section, mirroring the eye toggle in the Hierarchy
/// panel (M14.6 A). Both surfaces drive the same underlying
/// `ph2d_ecs::Visibility { hidden: bool }` component via
/// `EditorCommand::SetComponent`.
///
/// `visible == true` ↔ no `Visibility` component OR
/// `Visibility { hidden: false }`. Absence-equals-visible is the
/// canonical invariant ([`ph2d_ecs::visibility`]); on commit, the
/// host always writes an explicit `Visibility { hidden: ... }` so
/// the round-trip is unambiguous.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InspectorVisibilityInfo {
    pub entity_bits: u64,
    pub visible: bool,
}

/// M14.E: snapshot of the selected entity's `Name` component.
///
/// Mirrors the M14.A / M14.D snapshot pattern. The Inspector renders
/// this as the editable name field at the very top of the panel body
/// (above the Visibility row + Transform section). Loose-coupled
/// (the editor crate doesn't depend on `ph2d-ecs::Name`); the shell
/// converts to/from `Name(String)` at the boundary.
///
/// `name` is the human-readable label. The host falls back to
/// `format!("Entity_{hex_bits}")` when the entity has no `Name`
/// component yet, so the field is always non-empty for a selected
/// entity (matches the existing `InspectorSpriteInfo::name` shape).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorNameInfo {
    pub entity_bits: u64,
    pub name: String,
}

/// M14.C: which sprite render-source strategy the user requested via
/// the Render Source segmented switcher in the Inspector. The host
/// translates this into the actual renderer + ECS mutations.
///
/// The variants intentionally drop the inner `key` / `texture_id`
/// fields of [`InspectorSpriteSource`] — the user is asking for a
/// *kind* change; the host picks (or allocates) the new identifier.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RequestedSpriteStrategy {
    Atlas,
    Individual,
    HandPacked,
}

/// Which framing action the VIEW button (TOOL_HOME) + F/Home key
/// should run when the user triggers a reframe.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ViewFocusKind {
    /// Focus on the currently selected sprite. Falls back to (0,0)
    /// when no selection. Doesn't change zoom.
    Selected,
    /// Focus the active camera. No camera-object exists yet, so the
    /// host pans to (0,0). Doesn't change zoom.
    Camera,
    /// Frame all sprites in the scene. Walks PresentWorld for every
    /// `(GlobalTransform, RenderInstance)` and adjusts both center +
    /// height_world so they all fit (with a 10% padding margin).
    /// Empty scene falls back to (0,0) + default zoom.
    All,
}

/// M14.6B host-side reparent intent. Mirrors the
/// `WidgetEvent::HierReparent` payload one-to-one. `new_parent =
/// None` is a root-level drop; `before = None` means "append at end
/// of siblings" (or, when `new_parent` is also `None`, "end of root
/// list").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HierReparentIntent {
    pub dragged: NodeId,
    pub new_parent: Option<NodeId>,
    pub before: Option<NodeId>,
    /// M14.7 polish: when set, the host inserts `dragged` AFTER this
    /// target sibling. Mirrors the `WidgetEvent::HierReparent.after`
    /// field. Mutually exclusive with `before` — only one resolution
    /// fires per drop.
    pub after: Option<NodeId>,
}

impl HeroScreen {
    pub fn new(id: NodeId) -> Self {
        // Wave 8 Phase 1: `HeroScreen::new` is a pure constructor. The
        // host (or the test harness) installs `PANEL_REGISTRY` BEFORE
        // the first `HeroScreen::new` call — production binaries via
        // `ph2d_panel_registry_init::register_all_panels()` (which
        // honors `panel-*` cargo features), tests via
        // `crate::test_support::ensure_panel_registry()`. The previous
        // auto-install here silently neutralized those features at
        // runtime (audit B1).
        let mut store = WidgetStore::with_capacity(64);
        Self::pre_populate_store(&mut store);
        Self {
            id,
            theme: Theme::Forge,
            text_rendering: ph2d_tokens::TextRendering::Default,
            selection: Some(fixture::default_selection()),
            store,
            hit_index: HitIndex::new(),
            bus: crate::action_bus::ActionBus::new(),
            // Wave 5 stage B: 21 flat fields grouped into 6 sub-state
            // structs. Inspector + Hierarchy visible by default; stats
            // HUD + grid overlay visible; everything else off / None.
            view: ViewState {
                ui_mirrored: false,
                stats_visible: true,
                grid_visible: true,
            },
            panel_visibility: default_panel_visibility(),
            image_edit: ImageEditState::default(),
            gizmo: GizmoStateGroup::default(),
            grid: GridState::default(),
            camera_reset_pending: false,
            import_requested: false,
            project: crate::project::ProjectSettings::default(),
            dragging_files: None,
            stats: BottomHudStats::default(),
            last_viewport: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// ADR-0029 Phase C.1 panel-visibility accessor. Mirrors the
    /// `PanelHostInternal::panel_visible` impl below so editor-core
    /// code paths (orchestrator chrome publish, left-rail toggle)
    /// can read without dyn-dispatching through the trait.
    pub fn is_panel_visible(&self, id: &str) -> bool {
        self.panel_visibility.get(id).copied().unwrap_or(false)
    }

    /// Pre-populate the [`WidgetStore`] by delegating to each
    /// region's `populate` function. Each region owns its ids;
    /// adding a widget means editing only that region's file.
    fn pre_populate_store(store: &mut WidgetStore) {
        topbar::populate(store);
        left_rail::populate(store);
        pre_populate::populate_shared(store);
        // ADR-0029 Phase C.4: every in-tree panel (Inspector,
        // Hierarchy, Widget Gallery, Grid Snap) registers its
        // widgets via `Panel::populate`. The legacy
        // `crate::grid_snap::populate` is now an empty stub.
        if let Some(mtx) = crate::panel::PANEL_REGISTRY.get() {
            let guard = mtx.lock().expect("PANEL_REGISTRY mutex poisoned");
            for panel in guard.panels() {
                panel.populate(store);
            }
        }
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn selection(mut self, sel: Option<HeroSelection>) -> Self {
        self.selection = sel;
        self
    }

    /// Inject the host's per-frame grid projection (ADR-0025 M14.4b).
    /// Pass `None` to suppress the grid even when `grid_visible` is
    /// true — useful while the host is between scenes and no
    /// camera is established.
    pub fn set_grid_view(&mut self, view: Option<crate::grid::GridView>) {
        self.grid.view = view;
    }

    /// Mutable access to the grid configuration (spacing, colors,
    /// stroke widths). Changes apply on the next paint.
    pub fn grid_config_mut(&mut self) -> &mut crate::grid::GridConfig {
        &mut self.grid.config
    }

    pub fn handle_pointer<'frame>(
        &mut self,
        event: PointerEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        dispatch_pointer(&mut self.store, &self.hit_index, event, arena)
    }

    /// Like [`Self::handle_pointer`] but threads a live `TextSystem`
    /// so click→caret mapping snaps to the nearest glyph boundary
    /// instead of the `font_size * APPROX_ADVANCE_RATIO` heuristic.
    /// The shell calls this from its winit handler where it already
    /// owns the `TextSystem` for paint; pixel-perfect caret placement
    /// on text widgets requires this path.
    pub fn handle_pointer_with_text<'frame>(
        &mut self,
        event: PointerEvent,
        text_system: &mut TextSystem,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        dispatch_pointer_with_text(
            &mut self.store,
            &self.hit_index,
            event,
            Some(text_system),
            arena,
        )
    }

    pub fn handle_key<'frame>(
        &mut self,
        event: KeyEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_key(&mut self.store, event, arena)
    }

    /// Forward a printable character into the focused widget's
    /// editing buffer (`TextInput.text` / `Combobox.query` /
    /// `NumberInput.buffer`). Filters by widget kind: NumberInput
    /// only accepts `[0-9.eE+-]`; TextInput/Combobox accept anything
    /// non-control.
    pub fn handle_text_input<'frame>(
        &mut self,
        ch: char,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_text_input(&mut self.store, ch, arena)
    }

    /// Forward a wheel/trackpad scroll event into the dispatch.
    /// Painters publish their panel rects each frame via
    /// `WidgetStore::set_panel_rect`, so the wheel dispatch knows
    /// which panel sits under the cursor and applies the delta.
    pub fn handle_wheel<'frame>(
        &mut self,
        event: ph2d_host::WheelEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_wheel(&mut self.store, event, arena)
    }

    /// Translate a [`WidgetEvent`] from the dispatcher into a
    /// hero-level state mutation. Walks each region's
    /// `apply_event` in z-order; first region that consumes the
    /// event wins. Returns true iff some region consumed it.
    pub fn apply_event(&mut self, event: WidgetEvent) -> bool {
        // ADR-0029 Phase D: legacy fn-pointer dispatch deleted — every
        // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
        // typed `Panel<State>`. Walk only the typed registry.
        // Tripartite outcome semantics (audit B2 + A4): Consumed stops
        // iteration entirely (returns `true`); Observed records a side
        // effect but continues; Ignored is a no-op.
        let mut observed = false;
        let consumed = crate::panel::with_registry_opt(|reg| {
            for panel in reg.panels_mut() {
                match panel.apply_event(self, event) {
                    crate::panel::EventOutcome::Consumed => return true,
                    crate::panel::EventOutcome::Observed => observed = true,
                    crate::panel::EventOutcome::Ignored => {}
                }
            }
            false
        })
        .unwrap_or(false);
        if consumed {
            return true;
        }
        // ADR-0029 Phase C.1: host-level showcase event handler —
        // covers `CTX_MENU_OUTLINE_*`, `CTX_MENU_CREATE_NOTE`,
        // `SECTION_IDS`, `SECTION_COLOR_IDS`, radio/tab/tree pin
        // clicks. Shared across the live Inspector (when typed
        // panel is installed) and the Widget Gallery (legacy);
        // running at host level means the gallery keeps working
        // when the typed Inspector is absent.
        if crate::widget::showcase::apply_showcase_event(&mut self.store, event) {
            return true;
        }
        // Wave 9 Eixo A.1: chrome affordances split per file under
        // `chrome/` — theme menu, radius presets, view toggles, rail
        // panel/tool toggles, file menu, Settings cascades, scene
        // picker, image-edit actions. Adding a new chrome affordance
        // = drop a new `chrome/<feature>.rs` + one line in
        // `chrome::dispatch_all`. Multi-agent parallel work no longer
        // collides on this function.
        if chrome::dispatch_all(self, event) {
            return true;
        }
        if topbar::apply_event(&mut self.store, event) {
            return true;
        }
        if left_rail::apply_event(&mut self.store, event) {
            return true;
        }
        // Wave 8 Phase 4: return `observed` so a panel that did a
        // side-effect via `EventOutcome::Observed` (e.g. hierarchy
        // Blur(HIER_RENAME_INPUT) commits) propagates as "handled"
        // even when no chrome region consumed.
        observed
    }

    pub fn build_a11y(&self, viewport: Rect) -> Node {
        NodeBuilder::new(Role::Window)
            .label("PH2D editor")
            .bounds(
                viewport.x as f64,
                viewport.y as f64,
                viewport.w as f64,
                viewport.h as f64,
            )
            .build()
    }
}

/// Shared entry-path for rename mode (right-click "Rename..." +
/// long-press). Wipes any leftover text from a prior rename session
/// (Cancel / Blur paths don't necessarily clear), reinstalls the
/// TextInput state as `Focused`, and parks focus on the field. The
/// host's `pending_rename_seed` drain fills the buffer with the
/// entity's current `Name` on the next frame.
///
/// Side-table safety: `HIER_RENAME_INPUT` has no associated
/// `widget_color` / `panel_z` / `panel_scroll` / `tooltip` entries,
/// so the force-overwrite `store.register` (vs `register_if_absent`)
/// only resets buffer / caret / state — the intended effect.
pub fn open_rename_public(store: &mut crate::interaction::WidgetStore) {
    open_rename(store)
}

fn open_rename(store: &mut crate::interaction::WidgetStore) {
    store.register(
        ids::HIER_RENAME_INPUT,
        crate::interaction::InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::HIER_RENAME_INPUT));
}

/// Top-level hero paint orchestrator. Clears + re-populates the
/// hit-index, then walks each region painter in z-order
/// (canvas → selection overlay → chrome → HUD).
pub fn paint_hero_screen(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // Publish the user-picked radius scale to the thread-local read
    // by `paint::fill_rounded_rect` / `stroke_rounded_rect`. Set
    // every frame so it stays in sync with the topbar's radius menu.
    crate::paint::set_radius_scale(hero.store.radius_scale());
    // Same pattern for the text-rendering strategy — read by
    // `paint_text*` via the `paint::text_rendering()` thread-local.
    crate::paint::set_text_rendering(hero.text_rendering);
    // Stash the viewport so chrome event handlers in `chrome/` can
    // make smart layout decisions (cascade submenu side-flip etc.).
    hero.last_viewport = viewport;

    // Rail width follows the user's Themes-menu rail-button-size
    // preset (Small / Medium / Large; default Small). Switching size
    // shifts Inspector/Hierarchy x-positions accordingly.
    let rail_w = hero.store.rail_button_size().rail_width_px();
    let mut layout =
        HeroLayout::for_viewport_mirrored_with_rail_w(viewport, hero.view.ui_mirrored, rail_w);
    // Apply user-driven panel drag offsets to the Inspector +
    // Hierarchy rects. The offsets live on the WidgetStore's
    // `blender_picker_offset` side-table (panel-agnostic — the
    // dispatch's BlenderHitKind::DragHandle path stores the
    // offset under the `parent` NodeId regardless of widget kind).
    // Clamp helper lives in `style::clamp_panel_rect` so the floating
    // panel thunks (widget gallery, grid snap) share the same math.
    let insp_off = hero.store.blender_picker_offset(ids::INSP_PANEL);
    let hier_off = hero.store.blender_picker_offset(ids::HIER_PANEL);
    let insp_resize = hero.store.panel_resize_delta(ids::INSP_PANEL);
    let hier_resize = hero.store.panel_resize_delta(ids::HIER_PANEL);
    let (insp_rect, insp_clamped_off, insp_clamped_resize) =
        style::clamp_panel_rect(layout.inspector, insp_off, insp_resize, viewport);
    let (hier_rect, hier_clamped_off, hier_clamped_resize) =
        style::clamp_panel_rect(layout.hierarchy, hier_off, hier_resize, viewport);
    layout.inspector = insp_rect;
    layout.hierarchy = hier_rect;
    // Image-tool panels (BgRemoval, Padding, CEQ, Upscale, Equalize
    // Sizes) share the right-dock slot with Inspector. Mirror the
    // resized + dragged rect so they paint at the same position and
    // size when active. The handles inside those panels parent to
    // INSP_PANEL too (single dock-slot persistence — resizing CEQ
    // also resizes the Inspector when the user switches back).
    layout.bgremoval = insp_rect;
    layout.padding = insp_rect;
    // W2.T2.1 Day-7 follow-up: Painter sidebar shares Inspector slot too
    // (single dock-slot persistence). Sem este propagação, drag/resize não
    // afetavam o painter_sidebar visualmente + rect publicado divergia do
    // que dispatch hit-test usava → click vazava pra canvas atrás.
    layout.painter_sidebar = insp_rect;
    if (insp_clamped_off.0 - insp_off.0).abs() > f32::EPSILON
        || (insp_clamped_off.1 - insp_off.1).abs() > f32::EPSILON
    {
        hero.store.set_blender_picker_offset(
            ids::INSP_PANEL,
            insp_clamped_off.0,
            insp_clamped_off.1,
        );
    }
    if (hier_clamped_off.0 - hier_off.0).abs() > f32::EPSILON
        || (hier_clamped_off.1 - hier_off.1).abs() > f32::EPSILON
    {
        hero.store.set_blender_picker_offset(
            ids::HIER_PANEL,
            hier_clamped_off.0,
            hier_clamped_off.1,
        );
    }
    if (insp_clamped_resize.0 - insp_resize.0).abs() > f32::EPSILON
        || (insp_clamped_resize.1 - insp_resize.1).abs() > f32::EPSILON
    {
        hero.store.set_panel_resize_delta(
            ids::INSP_PANEL,
            insp_clamped_resize.0,
            insp_clamped_resize.1,
        );
    }
    if (hier_clamped_resize.0 - hier_resize.0).abs() > f32::EPSILON
        || (hier_clamped_resize.1 - hier_resize.1).abs() > f32::EPSILON
    {
        hero.store.set_panel_resize_delta(
            ids::HIER_PANEL,
            hier_clamped_resize.0,
            hier_clamped_resize.1,
        );
    }
    hero.hit_index.clear_for_frame();

    // M14.5: in live mode (`grid_view` published) the compositor pass
    // shows `game_rt` underneath wherever vello_rt has α=0, so we
    // **skip** the opaque canvas Bg1 fill. Chrome panels (BgElev,
    // panels, topbar) paint their own backdrops — verified in the
    // M14.5 audit. Fixture mode keeps the canvas tint so mockup
    // screenshots stay theme-correct.
    if hero.grid.view.is_none() {
        paint_canvas_bg(&layout, scene, hero.theme);
    }
    // M14.4b: world-space grid overlay. Painted between the canvas
    // background and the selection marquee so the marquee remains
    // legible over the grid. Skipped when toggle is off or host
    // hasn't published a camera view. We substitute the layout's
    // computed canvas rect into the view so the host doesn't have
    // to mirror layout math — it only owns camera + window dims.
    //
    // Layer-order toggle (2026-05-15): the compositor currently
    // composes `game_rt_ldr` UNDER `vello_intermediate` in a single
    // pass — chrome (including the grid) always lands on top of
    // sprites. Real "behind" rendering needs a second Vello
    // intermediate + a 3-layer compositor shader (TODO follow-up).
    // For now we approximate by halving the grid's effective opacity
    // when `grid_in_front == false`, which reads as "the grid is
    // farther / underneath" without changing the compositing path.
    if hero.view.grid_visible
        && let Some(view) = hero.grid.view
    {
        let view = crate::grid::GridView {
            canvas: layout.canvas,
            ..view
        };
        let mut state_for_paint = hero.grid.snap_state.clone();
        if !state_for_paint.grid_in_front {
            state_for_paint.opacity *= 0.4; // LITERAL-PX-OK: grid behind-canvas dim ratio (visual effect)
        }
        crate::grid_snap::render::paint(scene, &view, &state_for_paint);
    }
    // M14.4c: the legacy mockup selection marquee draws a fixed-size
    // dashed rect at the CANVAS center in screen pixels — it has no
    // world-space coupling and so doesn't follow pan/zoom. Skip it
    // when a `grid_view` is published (live ECS mode) so we don't
    // mislead users into thinking the marquee tracks an entity.
    // Fixture mode keeps the placeholder marquee for the mockup
    // screenshots.
    if hero.grid.view.is_none()
        && let Some(sel) = hero.selection.as_ref()
    {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
    }
    // M14.7 B: live-mode sprite gizmo. The host publishes a
    // `gizmo_view` carrying the selected sprite's world-space bbox +
    // current camera; the painter projects to screen pixels with the
    // same math the grid uses (so the gizmo and grid stay aligned
    // across pan/zoom).
    if let Some(view) = hero.gizmo.view {
        crate::gizmo::paint_sprite_gizmo(scene, &view, hero.theme, &mut hero.hit_index);
    }
    paint_top_bar(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        hero.image_edit.mode_on,
    );
    paint_left_rail(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    // Publish Inspector + Hierarchy panel rects so wheel-event
    // dispatch can route to them. Both are static (no drag offset).
    // When a panel is hidden via its left-rail toggle we DROP the
    // published rect so dispatch's "inside panel" tests don't match
    // a stale geometry.
    if hero.is_panel_visible("inspector") {
        hero.store.set_panel_rect(ids::INSP_PANEL, layout.inspector);
    } else {
        hero.store.clear_panel_rect(ids::INSP_PANEL);
    }
    if hero.is_panel_visible("hierarchy") {
        hero.store.set_panel_rect(ids::HIER_PANEL, layout.hierarchy);
    } else {
        hero.store.clear_panel_rect(ids::HIER_PANEL);
    }
    // Mirror the global picker's current value into the target
    // widget's `widget_colors` slot before either panel paints so
    // color circles inside the Inspector see this frame's value.
    if let Some(target) = hero.store.picker_target()
        && let Some((value, _, _, _)) = hero.store.blender_picker(ids::INSP_BLENDER_PICKER)
    {
        hero.store.set_widget_color(target, value.rgba);
        // Mirror Grid-Settings swatch edits back into the grid_snap
        // state so the canvas overlay re-paints with the new color.
        if target == crate::grid_snap::ids::GS_COLOR_PICKER {
            hero.grid.snap_state.color_rgba = value.rgba;
        }
    }
    // ADR-0029 Phase C.2: Hierarchy migrated to a typed Panel — selection
    // label is read via `host.selection()` inside the panel's `paint`;
    // live entries and rename-target live in panel-owned thread-local /
    // typed `HierarchyState` respectively. No host-side publish needed.
    //
    // Publish the picker's outer rect so dispatch's "is the click
    // inside the picker?" test can reason about its bounds.
    if hero.store.picker_target().is_some()
        && let Some(picker_rect) = color_picker_demo::current_picker_rect(&layout, &hero.store)
    {
        hero.store
            .set_panel_rect(ids::INSP_BLENDER_PICKER, picker_rect);
    } else {
        hero.store.clear_panel_rect(ids::INSP_BLENDER_PICKER);
    }

    // Wave 5 stage D — paint each panel via the PanelRegistry in
    // z-order. Bottom-first, so the panel most recently clicked /
    // dragged / opened sits on top. Panels that haven't been touched
    // yet inherit a default order at the bottom (fallback list below
    // also covers floating panels that have their own panel rects:
    // GAL_PANEL + GS_PANEL).
    //
    // INSP_BLENDER_PICKER is intentionally NOT in the panel
    // registry — it's painted out-of-band AFTER every floating panel
    // (see `paint_blender_picker_demo` below) so it sits on top of
    // every other panel regardless of z order.
    //
    // Each manifest's `paint_fn` owns its full per-frame logic:
    // visibility check + lazy default rect + drag/resize clamp +
    // chrome publish + actual paint + content_h publish + scroll
    // clamp + stale-rect cleanup on hide. Adding a new panel needs
    // zero edits to this iteration — drop `PANEL_MANIFEST` in the
    // panel module + 1 line in `panel_registry::PANEL_REGISTRY`.
    let mut z_order: Vec<ph2d_a11y::NodeId> = hero.store.panel_z_order().to_vec();
    for &fallback in &[
        ids::HIER_PANEL,
        ids::INSP_PANEL,
        ids::BGR_PANEL,
        ids::PAD_PANEL,
        ids::CEQ_PANEL,
        ids::EQS_PANEL,
        ids::UPS_PANEL,
        ids::PAINTER_SIDEBAR_PANEL,
        ids::INSP_BLENDER_PICKER,
        ids::GAL_PANEL,
        crate::grid_snap::ids::GS_PANEL,
    ] {
        if !z_order.contains(&fallback) {
            z_order.push(fallback);
        }
    }
    // ADR-0029 Phase D: legacy fn-pointer dispatch deleted. Every
    // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
    // typed `Panel<State>`. The z-order walk resolves each id to its
    // typed entry; ids that don't match (e.g. `INSP_BLENDER_PICKER`,
    // painted out-of-band below) are silently skipped.
    crate::panel::with_registry_opt(|reg| {
        for panel_id in z_order {
            if let Some(idx) = reg.find_by_panel_node_id(panel_id) {
                // Hit barrier: register the panel rect BEFORE the
                // widgets inside `panel.paint()` so the gizmo's hit
                // rects (registered earlier this frame) don't bleed
                // through the panel surface. `HitIndex::hit()` walks
                // back-to-front, so internal panel widgets registered
                // by `paint()` below still outrank this barrier — only
                // empty panel area falls back to it. Enio 2026-05-25:
                // "alças do gizmo da sprite podem ser acessadas
                // através dos painéis. Isso não pode acontecer."
                if let Some(panel_rect) = hero.store.panel_rect(panel_id) {
                    hero.hit_index.register(panel_id, panel_rect);
                }
                let mut typed_ctx = crate::panel::PaintCtx {
                    host: hero,
                    layout: &layout,
                    viewport,
                    scene,
                    text_system,
                };
                reg.panels_mut()[idx].paint(&mut typed_ctx);
            }
        }
    });
    // hero/scene/text_system unborrowed for the
    // rest of paint_hero_screen (bottom HUD, picker overlay, tooltip,
    // context menu, drop overlay).
    if hero.view.stats_visible {
        paint_bottom_hud(&layout, scene, text_system, hero.theme, hero.stats);
    }
    // BlenderColorPicker — painted AFTER every floating panel
    // (Inspector, Hierarchy, Widget Gallery, Grid Settings) so it
    // never sits visually behind one of them. The painter is a no-op
    // when `picker_target` is None.
    if hero.store.picker_target().is_some() {
        color_picker_demo::paint_blender_picker_demo(
            &layout,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
        );
    }
    // Tooltip overlay on top of all chrome (Phase 3 polish).
    topbar::paint_hover_tooltip(scene, text_system, hero.theme, &hero.hit_index, &hero.store);
    // Context menu overlay — last so the floating menu sits above
    // every panel, including the floating BlenderColorPicker.
    context_menu_overlay::paint_context_menu_overlay(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.project,
        viewport,
    );
    // M14.4e: file-drop overlay sits above EVERY layer (chrome,
    // tooltips, context menus) so the user always sees the "Drop to
    // import" hint while the OS drag is active.
    if let Some((paths, cursor)) = hero.dragging_files.as_ref() {
        paint_drop_overlay(&layout, paths, *cursor, scene, text_system, hero.theme);
    }
}

/// ADR-0029 Phase B.3 — `PanelHostInternal` is the
/// `#[doc(hidden)] pub` trait surface that the four in-tree panels
/// consume in Phase C. The initial impl exposes only the minimal
/// foundation (theme + project + widget store + hit index); the
/// remaining ~25-30 accessors (selection, gizmo, grid, view, …)
/// land alongside each panel's migration in Phase C as they're
/// actually needed.
impl crate::panel::PanelHost for HeroScreen {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn project(&self) -> &crate::project::ProjectSettings {
        &self.project
    }
}

impl crate::panel::PanelHostInternal for HeroScreen {
    fn store(&self) -> &WidgetStore {
        &self.store
    }

    fn store_mut(&mut self) -> &mut WidgetStore {
        &mut self.store
    }

    fn hit_index_mut(&mut self) -> &mut HitIndex {
        &mut self.hit_index
    }

    fn store_and_hit_index_mut(&mut self) -> (&WidgetStore, &mut HitIndex) {
        (&self.store, &mut self.hit_index)
    }

    fn bus(&self) -> &crate::action_bus::ActionBus {
        &self.bus
    }

    fn bus_mut(&mut self) -> &mut crate::action_bus::ActionBus {
        &mut self.bus
    }

    fn selection(&self) -> Option<&HeroSelection> {
        self.selection.as_ref()
    }

    fn selection_mut(&mut self) -> &mut Option<HeroSelection> {
        &mut self.selection
    }

    fn panel_visible(&self, id: &str) -> bool {
        self.is_panel_visible(id)
    }

    fn set_panel_visible(&mut self, id: &str, value: bool) {
        // Use the canonical interned id when one matches a known
        // panel so the HashMap lookup is keyed by `&'static str`.
        let key = canonical_panel_id(id).unwrap_or_else(|| {
            // Fall back to leaking — unknown panels are rare (3rd
            // party / future migrations); a single allocation per
            // unique id is acceptable for the unstable internal tier.
            Box::leak(id.to_string().into_boxed_str()) as &'static str
        });
        self.panel_visibility.insert(key, value);
    }

    fn grid_snap_state(&self) -> &crate::grid_snap::GridSnapState {
        &self.grid.snap_state
    }

    fn grid_snap_state_mut(&mut self) -> &mut crate::grid_snap::GridSnapState {
        &mut self.grid.snap_state
    }

    fn store_and_grid_snap_state_mut(
        &mut self,
    ) -> (&WidgetStore, &mut crate::grid_snap::GridSnapState) {
        (&self.store, &mut self.grid.snap_state)
    }

    fn grid_snap_panel_rect(&self) -> Option<crate::zones::Rect> {
        self.grid.snap_state.panel_rect
    }

    fn set_grid_snap_panel_rect(&mut self, rect: Option<crate::zones::Rect>) {
        self.grid.snap_state.panel_rect = rect;
    }
}

/// Build the default per-panel visibility map for a fresh
/// `HeroScreen`. Inspector + Hierarchy visible by default; floating
/// panels (Widget Gallery, Grid Snap) hidden.
fn default_panel_visibility() -> std::collections::BTreeMap<&'static str, bool> {
    let mut map = std::collections::BTreeMap::new();
    map.insert("inspector", true);
    map.insert("hierarchy", true);
    map.insert("widget_gallery", false);
    map.insert("grid_snap", false);
    map
}

/// Canonical `&'static str` for known panel ids — keeps the
/// visibility HashMap keys stable across calls without leaking.
fn canonical_panel_id(id: &str) -> Option<&'static str> {
    match id {
        "inspector" => Some("inspector"),
        "hierarchy" => Some("hierarchy"),
        "widget_gallery" => Some("widget_gallery"),
        "grid_snap" => Some("grid_snap"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
