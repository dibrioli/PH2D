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
//! ([`canvas`], [`topbar`], [`left_rail`], [`inspector`],
//! [`hierarchy`], [`bottom_hud`], [`selection`]). Shared layout
//! constants + small helpers in [`style`]; stable `NodeId`s in
//! [`ids`]. Hardcoded mockup content stays in [`fixture`] until a
//! pilot project picks the entity model.

pub mod bottom_hud;
pub mod canvas;
pub mod color_picker_demo;
pub mod context_menu_overlay;
pub mod fixture;
pub mod hierarchy;
pub mod ids;
pub mod inspector;
pub mod inspector_sync;
pub mod left_rail;
pub mod selection;
pub mod style;
pub mod topbar;
pub mod widget_gallery;

pub use bottom_hud::{BottomHudStats, paint_bottom_hud};
pub use canvas::{paint_canvas_bg, paint_drop_overlay};
pub use color_picker_demo::paint_blender_picker_demo;
pub use hierarchy::{paint_hierarchy, set_live_component_count};
pub use inspector::paint_inspector;
pub use left_rail::paint_left_rail;
pub use selection::paint_selection_overlay;
pub use style::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
pub use topbar::paint_top_bar;

use crate::interaction::{
    HitIndex, InteractiveState, WidgetEvent, WidgetStore, dispatch_pointer,
    dispatch_pointer_with_text,
};
use crate::zones::Rect;
use bumpalo::Bump;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_host::{KeyEvent, PointerEvent};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Pre-computed sub-regions that the rest of the hero painters
/// consume. Built once per frame from a viewport rect — cheap.
#[derive(Copy, Clone, Debug)]
pub struct HeroLayout {
    pub viewport: Rect,
    pub top_bar: Rect,
    pub left_rail: Rect,
    pub inspector: Rect,
    pub hierarchy: Rect,
    pub bottom_hud: Rect,
    /// Visible canvas region (between rail/inspector on the left and
    /// hierarchy on the right, between TopBar and HUD vertically).
    /// The selection overlay positions itself relative to this rect.
    pub canvas: Rect,
}

impl HeroLayout {
    /// Default layout (mirrored = false): Hierarchy on the LEFT next
    /// to the rail, Inspector pinned to the RIGHT edge. The canvas
    /// sits between them. Pass `mirrored = true` to flip horizontally
    /// (Inspector left of canvas, Hierarchy right) — used by the
    /// "Mirror UI" theme-menu toggle.
    pub fn for_viewport(viewport: Rect) -> Self {
        Self::for_viewport_mirrored(viewport, false)
    }

    pub fn for_viewport_mirrored(viewport: Rect, mirrored: bool) -> Self {
        use style::{
            EDGE_PAD, HIERARCHY_W, HUD_BOTTOM_PAD, HUD_H, INSPECTOR_W, RAIL_W, TOPBAR_GAP, TOPBAR_H,
        };
        let top_bar = Rect::new(
            viewport.x + EDGE_PAD,
            viewport.y + EDGE_PAD,
            (viewport.w - EDGE_PAD * 2.0).max(0.0),
            TOPBAR_H,
        );
        let chrome_top = top_bar.y + top_bar.h + TOPBAR_GAP;
        let chrome_bot = viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H - 8.0;
        let chrome_h = (chrome_bot - chrome_top).max(0.0);

        // Rail is FLUSH with the viewport's left edge — the
        // sub-labels paint at `rail.x + LABEL_LEFT_PAD` so this
        // gives them an exact 3-px gap from the screen edge.
        let left_rail = Rect::new(viewport.x, chrome_top, RAIL_W, chrome_h);
        // Default panel sides (mirrored=false):
        //   - Hierarchy LEFT (just past the rail)
        //   - Inspector RIGHT (pinned to viewport edge)
        // Mirrored flips both.
        // Side panels sit just past the rail (now flush at viewport.x)
        // — `RAIL_W + EDGE_PAD` from the screen's left edge gives the
        // canonical breathing room.
        let (hierarchy_x, inspector_x) = if mirrored {
            (
                viewport.x + viewport.w - EDGE_PAD - HIERARCHY_W,
                viewport.x + RAIL_W + EDGE_PAD,
            )
        } else {
            (
                viewport.x + RAIL_W + EDGE_PAD,
                viewport.x + viewport.w - EDGE_PAD - INSPECTOR_W,
            )
        };
        let inspector = Rect::new(inspector_x, chrome_top, INSPECTOR_W, chrome_h.min(880.0));
        let hierarchy = Rect::new(hierarchy_x, chrome_top, HIERARCHY_W, chrome_h);
        // Canvas spans the gap between whichever panel is on the
        // left side of it and whichever is on the right.
        let (left_panel_right, right_panel_left) = if mirrored {
            (inspector.x + inspector.w, hierarchy.x)
        } else {
            (hierarchy.x + hierarchy.w, inspector.x)
        };
        // Canvas spans the FULL viewport — every other piece of
        // chrome (rail, top bar, side panels, bottom HUD) is a
        // floating overlay on top. Includes the area BELOW the
        // chrome bottom so the canvas tint reaches the screen's
        // bottom edge; the stats HUD floats above it.
        let _ = (left_panel_right, right_panel_left, chrome_bot);
        let canvas = Rect::new(viewport.x, viewport.y, viewport.w, viewport.h);

        let bottom_hud = Rect::new(
            viewport.x + (viewport.w - 480.0) * 0.5,
            viewport.y + viewport.h - HUD_BOTTOM_PAD - HUD_H,
            480.0,
            HUD_H,
        );

        Self {
            viewport,
            top_bar,
            left_rail,
            inspector,
            hierarchy,
            bottom_hud,
            canvas,
        }
    }
}

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
    pub selection: Option<HeroSelection>,
    /// Per-widget interactive state (hover/press/focus). Pre-populated
    /// at construction; mutated in-place by [`HeroScreen::handle_pointer`].
    pub store: WidgetStore,
    /// Per-frame hit-test index. Cleared at the start of each
    /// `paint_hero_screen` call and re-populated as painters emit
    /// geometry.
    pub hit_index: HitIndex,
    /// When `true`, the Inspector and Hierarchy panels swap sides
    /// (Inspector left, Hierarchy right). Toggled via the "Mirror
    /// UI" entry in the theme context menu. Defaults to `false` —
    /// Hierarchy left, Inspector right.
    pub ui_mirrored: bool,
    /// Visibility of the Inspector panel — toggled by the
    /// `RAIL_SHOW_INSPECTOR` button in the left rail.
    pub inspector_visible: bool,
    /// Visibility of the Hierarchy panel — toggled by the
    /// `RAIL_SHOW_HIERARCHY` button in the left rail.
    pub hierarchy_visible: bool,
    /// Visibility of the bottom statistics HUD — toggled by the
    /// "Show Statistics" entry in the theme context menu.
    pub stats_visible: bool,
    /// Whether the TopBar is in **Image Tools mode**. When `true`,
    /// the right-side clusters (Project / Play / Right / Settings)
    /// are hidden and replaced by an action row of image-editing
    /// pills (`[Trim Transparency]` in V1; more to follow). Toggled
    /// by clicks on the `TOPBAR_IMAGE_TOOLS` button — handled in
    /// [`HeroScreen::apply_event`] before the topbar's stub
    /// `apply_event` runs. Default `false`.
    pub image_tools_mode: bool,
    /// Visibility of the floating **Widget Gallery** panel — toggled
    /// by clicks on the `TOPBAR_WIDGET_GALLERY` palette button.
    /// Painted as an overlay on top of the canvas (NOT in the panel
    /// z-order list, since it doesn't dock). Default `false`.
    pub widget_gallery_visible: bool,
    /// Rect of the Widget Gallery panel in viewport pixels. Set on
    /// first toggle to a centered default; persisted across frames so
    /// dragging keeps the position. Width and height match the
    /// reference snapshot's Inspector dimensions so the showcase fits
    /// without scroll.
    pub widget_gallery_rect: Option<crate::zones::Rect>,
    /// Live-mode entity rows published by the host via
    /// [`HeroScreen::sync_from_hierarchy`] (ADR-0025 M14.4a).
    ///
    /// When `Some`, the hierarchy panel renders these entries instead
    /// of `fixture::hierarchy()`, and `apply_event` resolves click
    /// ids against this map. `None` keeps the fixture behavior (used
    /// by tests + the standalone hero demo).
    pub live_hierarchy_entries:
        Option<std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>>,
    /// World-space grid overlay toggle (ADR-0025 M14.4b). Default
    /// `true`. Toggled via the "Show Grid" context-menu entry and
    /// the `G` key.
    pub grid_visible: bool,
    /// Per-frame grid projection state. `None` means the host hasn't
    /// supplied a view yet → grid stays hidden even if
    /// `grid_visible` is `true`. Set each frame via
    /// [`HeroScreen::set_grid_view`].
    pub grid_view: Option<crate::grid::GridView>,
    /// Spacing + color config for the grid painter. Mutate via
    /// [`HeroScreen::grid_config_mut`] for project-level
    /// customization.
    pub grid_config: crate::grid::GridConfig,
    /// Grid-snap subsystem state — kind selector, per-kind config,
    /// snap policy, overlay display + opacity. Canonical source for
    /// the canvas grid overlay (paints via [`crate::grid_snap::render::paint`])
    /// and for snapping world positions (via
    /// [`crate::grid_snap::GridSnapState::snap_world`]).
    /// Panel opens/closes via `TOPBAR_GRID_SETTINGS`.
    pub grid_snap_state: crate::grid_snap::GridSnapState,
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
    /// M14.6A: row NodeId whose visibility eye-icon was just clicked.
    /// The host drains this each frame, resolves NodeId → Entity via
    /// the bridge, and flips the `Visibility` component on
    /// `SimWorld`. Cleared by `apply_event` after dispatch sets it
    /// when the host reads + applies the toggle.
    pub pending_visibility_toggle: Option<NodeId>,
    /// M14.6B: hierarchy drag-reparent intent emitted by the
    /// dispatcher when a DnD drop resolves. Same drain semantics as
    /// `pending_visibility_toggle`: host reads on the next frame,
    /// translates NodeIds → Entities via the bridge, then issues the
    /// matching `ChildOf` mutation on `SimWorld`. Carries only
    /// NodeIds — staying `Copy + Eq` keeps the field cheap to clear.
    pub pending_reparent: Option<HierReparentIntent>,
    /// M14.6 F: per-row context-menu action intents. Each is a
    /// `Some(row_node_id)` once the user picks the matching menu
    /// entry; the host drains and applies the matching ECS mutation,
    /// then re-snapshots the hierarchy on the next frame.
    pub pending_duplicate: Option<NodeId>,
    pub pending_delete: Option<NodeId>,
    pub pending_reset_transform: Option<NodeId>,
    pub pending_add_child: Option<NodeId>,
    /// M14.7 A: sim-entity bits of the sprite currently selected for
    /// gizmo manipulation. The host's canvas-click handler runs
    /// `pick_sprite_at_world` against PresentWorld and writes the
    /// result here; the gizmo painter (M14.7 B) and the inspector
    /// (M14.5) read it on the next frame. `None` = nothing selected
    /// (click landed on empty canvas, or the entity was just
    /// despawned).
    pub gizmo_selection: Option<u64>,
    /// M14.7 B: per-frame projection input for the gizmo painter.
    /// Host computes this from `selection_bbox_world(present,
    /// gizmo_selection)` + the current camera/window and pushes it
    /// here just before `paint_hero_screen`. `None` ⇒ no gizmo
    /// painted this frame (selection is empty, or the entity vanished).
    pub gizmo_view: Option<crate::gizmo::GizmoView>,
    /// M14.7 C: in-progress drag on the gizmo. Host's MouseInput
    /// handler fills this when a Mouse Down lands on a gizmo handle;
    /// the Move handler advances `cursor_screen`, calls
    /// [`crate::gizmo::compute_gizmo_transform`], and writes the
    /// result back to SimWorld; Up clears the field.
    pub gizmo_drag: Option<crate::gizmo::GizmoDragState>,
    /// M14.6 D: hierarchy-row click intent for cross-panel selection
    /// sync. When the user clicks a live row in the hierarchy panel,
    /// `apply_event` raises this; the host drains it on the next
    /// frame, resolves the row NodeId → sim entity via the bridge,
    /// and updates `gizmo_selection` so the canvas gizmo follows
    /// the hierarchy click.
    pub pending_hierarchy_row_click: Option<NodeId>,
    /// M14.7 polish: pending request to reframe the camera. Raised by
    /// the F/Home key or the VIEW button on the left rail; the host
    /// drains and updates `Camera2d::center` (and `height_world` for
    /// `All`) on the next frame.
    pub pending_view_focus: Option<ViewFocusKind>,
    /// M14.7 polish: row currently in inline-rename mode. The
    /// hierarchy painter replaces the row's name label with a
    /// TextInput when this matches; user typing flows through the
    /// usual TextInput dispatch. `None` = no row in rename.
    pub rename_target_row: Option<NodeId>,
    /// One-shot seed signal raised when rename mode opens. Host takes
    /// it on the next frame, fills `HIER_RENAME_INPUT.text` with the
    /// entity's current `Name`, and selects all. Without this flag,
    /// the host can't tell "rename just opened (seed once)" from
    /// "user typed Backspace and emptied the buffer (don't re-seed)"
    /// — the previous `buffer_empty` heuristic clobbered every
    /// keystroke once the field hit zero chars.
    pub pending_rename_seed: Option<NodeId>,
    /// Pending Name commit. Host drains on the next frame,
    /// resolves bridge NodeId → Entity, and writes the new `Name`
    /// component. `text` is the buffer contents at commit time.
    pub pending_rename_commit: Option<(NodeId, String)>,
    /// M14.5 inspector phase (6.4): pending request to re-import
    /// the currently-selected sprite at the current
    /// `project.pixels_per_meter`. Host drains, recomputes the
    /// sprite's world size from the source PNG dims / px-per-m, and
    /// updates the `Sprite` component. `Some(entity_bits)` keeps the
    /// host independent of `gizmo_selection`'s value at drain time
    /// (avoids races with a concurrent selection change).
    pub pending_reimport: Option<u64>,
    /// Pending request to apply Trim Transparency to the selected
    /// sprite. Raised by clicking `IMAGE_ACTION_TRIM` on the Image
    /// Tools action row while in `image_tools_mode`. Host drains,
    /// reads the sprite's atlas-source RGBA pixels via the asset_db,
    /// runs [`crate::trim_transparency`] (alpha threshold 0), and
    /// — when the result is `trimmed = true` — acquires a fresh
    /// `IndividualTextureStore` entry, repoints the sprite source
    /// to it, and rewrites `Sprite::size` to the new dims at the
    /// current `project.pixels_per_meter`. Source remains snapshot
    /// here (not read via `gizmo_selection` at drain time) so a
    /// concurrent selection change doesn't retarget the action.
    pub pending_trim_transparency: Option<u64>,
    /// Pending request to apply Make Square to the selected sprite.
    /// Raised by clicking `IMAGE_ACTION_MAKE_SQUARE` on the Image Tools
    /// action row. Host drains, reads the sprite's atlas-source RGBA,
    /// runs [`crate::make_square`], and — when the result is
    /// `made_square = true` — replaces the sprite's source pixels with
    /// the padded square, reprojects the pivot to preserve world
    /// position (formula: `(old_dim * old_pivot + offset) / new_size`),
    /// and pushes an "Make square" entry to the undo history. Source
    /// remains snapshot here (not read via `gizmo_selection` at drain
    /// time) so a concurrent selection change doesn't retarget the
    /// action — same contract as `pending_trim_transparency`.
    pub pending_make_square: Option<u64>,
    /// M14.5 inspector phase: snapshot of the selected sprite's
    /// data the host publishes each frame so `paint_inspector` can
    /// surface a "Render Source" section without crossing the
    /// ADR-0021 / HR-8 boundary into SimWorld directly. `None`
    /// when nothing is selected or the selection isn't a sprite.
    pub inspector_sprite: Option<InspectorSpriteInfo>,
    /// M14.A: snapshot of the selected entity's local `Transform`
    /// the host publishes when selection changes (or the gizmo drag
    /// mutates the transform externally). `None` when no entity is
    /// selected. The Inspector's Transform editor section reads this
    /// to seed its NumberInput buffers; subsequent live edits live
    /// in the [`WidgetStore`] until commit (HR-8 / ADR-0021: Inspector
    /// never reads SimWorld directly).
    pub inspector_transform: Option<InspectorTransformInfo>,
    /// Entity bits of the last selection that `inspector_transform`
    /// was populated for. When the current selection differs, the
    /// Inspector's `apply_event` path force-rewrites the 5 Transform
    /// NumberInput buffers so an in-progress edit on entity A doesn't
    /// silently apply to entity B after a selection switch.
    pub last_inspector_entity: Option<u64>,
    /// M14.A: editor → host channel for Transform edits. The
    /// inspector publishes the full snapshot (entity_bits +
    /// translation/rotation/scale) when a NumberInput commits
    /// (Enter / blur) or the Reset button fires; the shell drains
    /// this once per frame, builds an `ph2d_ecs::Transform` from
    /// the raw fields, and pushes a [`EditorCommand::SetComponent`]
    /// to its `EditorCommandQueue`. **First end-to-end consumer of
    /// the editor command pipeline** — every prior `pending_*` field
    /// bypassed the queue and mutated SimWorld directly.
    ///
    /// Re-uses [`InspectorTransformInfo`] so `ph2d-editor` stays
    /// decoupled from `ph2d-ecs`; the type-id resolution + glam
    /// conversion happens at the shell boundary.
    pub pending_transform_edit: Option<InspectorTransformInfo>,
    /// M14.D: snapshot of the selected entity's `Visibility` state,
    /// mirroring the eye toggle that already lives in the Hierarchy
    /// panel (M14.6 A). The Inspector renders a checkbox above the
    /// Transform section so the user can flip visibility from either
    /// surface. `None` when no entity is selected. Same HR-8 / ADR-0021
    /// boundary as Transform — the host bridges from SimWorld.
    pub inspector_visibility: Option<InspectorVisibilityInfo>,
    /// M14.D: editor → host channel for Visibility commits. The
    /// inspector publishes `(entity_bits, visible)` when the user
    /// flips the checkbox; the shell drains and pushes a
    /// `EditorCommand::SetComponent` for
    /// [`ph2d_ecs::Visibility`] to its `EditorCommandQueue` — same
    /// pipeline as `pending_transform_edit`.
    pub pending_visibility_edit: Option<InspectorVisibilityInfo>,
    /// M14.C: editor → host channel for Sprite source-strategy
    /// switches. The inspector publishes `(entity_bits, requested)`
    /// when the user picks a different strategy in the Render Source
    /// section's segmented switcher. The shell does the actual swap:
    /// Atlas → Individual re-decodes the source asset via
    /// `atlas_asset_map` and `acquire_individual`; Individual → Atlas
    /// and HandPacked transitions surface a toast in v1
    /// (renderer-side parity arrives in M14.C+).
    pub pending_sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
    /// M14.E: snapshot of the selected entity's `Name` component.
    /// Host publishes this per frame so the editable name field at
    /// the top of the Inspector body can seed its TextInput buffer.
    /// `None` when nothing is selected.
    pub inspector_name: Option<InspectorNameInfo>,
    /// M14.E: editor → host channel for entity-name edits. The
    /// inspector publishes the full snapshot (entity_bits + new name)
    /// on every `TextChanged` — `Option` coalescing means the shell
    /// drains at most once per frame, even when the user is typing
    /// fast. Drained via `EditorCommand::SetComponent` for
    /// `ph2d_ecs::Name`, same pipeline as Transform / Visibility.
    pub pending_name_edit: Option<InspectorNameInfo>,
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
        let mut store = WidgetStore::with_capacity(64);
        Self::pre_populate_store(&mut store);
        Self {
            id,
            theme: Theme::Forge,
            selection: Some(fixture::default_selection()),
            store,
            hit_index: HitIndex::new(),
            ui_mirrored: false,
            inspector_visible: true,
            hierarchy_visible: true,
            stats_visible: true,
            image_tools_mode: false,
            widget_gallery_visible: false,
            widget_gallery_rect: None,
            live_hierarchy_entries: None,
            grid_visible: true,
            grid_view: None,
            grid_config: crate::grid::GridConfig::default(),
            grid_snap_state: crate::grid_snap::GridSnapState::default(),
            camera_reset_pending: false,
            import_requested: false,
            project: crate::project::ProjectSettings::default(),
            dragging_files: None,
            stats: BottomHudStats::default(),
            pending_visibility_toggle: None,
            pending_reparent: None,
            pending_duplicate: None,
            pending_delete: None,
            pending_reset_transform: None,
            pending_add_child: None,
            gizmo_selection: None,
            gizmo_view: None,
            gizmo_drag: None,
            pending_hierarchy_row_click: None,
            pending_view_focus: None,
            rename_target_row: None,
            pending_rename_seed: None,
            pending_rename_commit: None,
            pending_reimport: None,
            pending_trim_transparency: None,
            pending_make_square: None,
            inspector_sprite: None,
            inspector_transform: None,
            last_inspector_entity: None,
            pending_transform_edit: None,
            inspector_visibility: None,
            pending_visibility_edit: None,
            pending_sprite_source_change: None,
            inspector_name: None,
            pending_name_edit: None,
        }
    }

    /// Pre-populate the [`WidgetStore`] by delegating to each
    /// region's `populate` function. Each region owns its ids;
    /// adding a widget means editing only that region's file.
    fn pre_populate_store(store: &mut WidgetStore) {
        topbar::populate(store);
        left_rail::populate(store);
        hierarchy::populate(store);
        inspector::populate(store);
        widget_gallery::populate(store);
        crate::grid_snap::populate(store);
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn selection(mut self, sel: Option<HeroSelection>) -> Self {
        self.selection = sel;
        self
    }

    /// Inject host-supplied live entity rows into the hierarchy panel
    /// (ADR-0025 M14.4a). Each call:
    ///
    /// 1. Re-registers the `ordered` `NodeId`s on the `WidgetStore`
    ///    as plain interactive rows (idempotent — repeat calls cost
    ///    nothing for ids already seen this session).
    /// 2. Replaces the `WidgetStore::init_hierarchy_order` list so
    ///    the painter iterates in the order the host supplies (the
    ///    bridge's `HierarchySnapshot` walk order = DFS root-first).
    /// 3. Stores `entries` so `paint_hero_screen` can publish them
    ///    to the hierarchy painter's thread-local before paint, and
    ///    so `apply_event` can resolve click ids back to entity
    ///    names without crossing the `bevy_ecs::World` boundary
    ///    (HR-8).
    ///
    /// Call once per frame from the host's `render_frame` loop
    /// before `paint_hero_screen`. Passing an empty `ordered` slice
    /// is valid (renders an empty hierarchy).
    pub fn sync_from_hierarchy(
        &mut self,
        ordered: &[NodeId],
        entries: std::collections::BTreeMap<NodeId, fixture::HierarchyEntity>,
    ) {
        hierarchy::repopulate(&mut self.store, ordered);
        self.live_hierarchy_entries = Some(entries);
    }

    /// Drop any host-supplied hierarchy state, reverting to the
    /// fixture data set in `hierarchy::populate`. The host calls
    /// this when leaving live-edit mode (e.g. user pressed
    /// `PH2D_HERO_LIVE` toggle off).
    pub fn clear_live_hierarchy(&mut self) {
        self.live_hierarchy_entries = None;
    }

    /// Inject the host's per-frame grid projection (ADR-0025 M14.4b).
    /// Pass `None` to suppress the grid even when `grid_visible` is
    /// true — useful while the host is between scenes and no
    /// camera is established.
    pub fn set_grid_view(&mut self, view: Option<crate::grid::GridView>) {
        self.grid_view = view;
    }

    /// Mutable access to the grid configuration (spacing, colors,
    /// stroke widths). Changes apply on the next paint.
    pub fn grid_config_mut(&mut self) -> &mut crate::grid::GridConfig {
        &mut self.grid_config
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
        // M14.6B: hierarchy drag-reparent. Dispatcher emits one
        // `HierReparent` per drop in addition to mutating the panel
        // store. Live (ECS) mode reads it via `pending_reparent` and
        // the host applies `ChildOf` accordingly. Fixture mode can
        // ignore it (the store mutation is already in place).
        if let WidgetEvent::HierReparent {
            dragged,
            new_parent,
            before,
            after,
        } = event
        {
            self.pending_reparent = Some(HierReparentIntent {
                dragged,
                new_parent,
                before,
                after,
            });
            return true;
        }
        // Theme + radius selector from the TopBar theme menu —
        // intercepted at the Hero level because `self.theme` lives
        // here, not on the WidgetStore.
        if let WidgetEvent::Click(id) = event {
            // M14.6A: hierarchy eye-toggle clicks arrive as a
            // companion NodeId with the EYE_TOGGLE_BIT set. Route
            // them to `pending_visibility_toggle` for the host to
            // drain, then short-circuit so the row's regular click
            // (selection / inspector focus) does NOT also fire.
            if let Some(row_id) = ids::hier_eye_companion_to_row(id) {
                self.pending_visibility_toggle = Some(row_id);
                return true;
            }
            // M14.6C: chevron click on a hierarchy parent row.
            // Toggles the panel's view-only collapse state — does
            // not touch the ECS `ChildOf` hierarchy.
            if let Some(row_id) = ids::hier_expand_companion_to_row(id) {
                self.store.toggle_hierarchy_collapsed(row_id);
                return true;
            }
            let new_theme = if id == ids::CTX_MENU_THEME_FORGE {
                Some(Theme::Forge)
            } else if id == ids::CTX_MENU_THEME_PAINT {
                Some(Theme::Workshop)
            } else if id == ids::CTX_MENU_THEME_SUNSTONE {
                Some(Theme::Sunstone)
            } else if id == ids::CTX_MENU_THEME_BLUEPRINT {
                Some(Theme::Blueprint)
            } else {
                None
            };
            if let Some(t) = new_theme {
                self.theme = t;
                self.store.close_context_menu();
                return true;
            }
            let new_radius_scale = if id == ids::CTX_MENU_RADIUS_SHARP {
                Some(0.2_f32)
            } else if id == ids::CTX_MENU_RADIUS_DEFAULT {
                Some(1.0_f32)
            } else if id == ids::CTX_MENU_RADIUS_ROUND {
                Some(1.6_f32)
            } else {
                None
            };
            if let Some(s) = new_radius_scale {
                self.store.set_radius_scale(s);
                self.store.close_context_menu();
                return true;
            }
            if id == ids::CTX_MENU_MIRROR_UI {
                self.ui_mirrored = !self.ui_mirrored;
                self.store.close_context_menu();
                return true;
            }
            if id == ids::CTX_MENU_SHOW_STATS {
                self.stats_visible = !self.stats_visible;
                self.store.close_context_menu();
                return true;
            }
            if id == ids::CTX_MENU_SHOW_GRID {
                self.grid_visible = !self.grid_visible;
                self.store.close_context_menu();
                return true;
            }
            // Rail compound toggles: SPACE flips Global↔Local, VIEW
            // cycles Selected → Camera → All. The face label is read
            // from the store every paint, so flipping the value here
            // is enough — the next frame renders the new label.
            if id == ids::TOOL_SPACE {
                let next = !self.store.tool_space_local();
                self.store.set_tool_space_local(next);
                return true;
            }
            if id == ids::TOOL_HOME {
                // M14.7 polish: 3-mode cycle (Selected → Camera →
                // All). Each click EXECUTES the current mode and
                // then advances the label so the user can chain
                // actions or see what's next.
                let current = self.store.tool_view_mode();
                let kind = match current {
                    1 => ViewFocusKind::Camera,
                    2 => ViewFocusKind::All,
                    _ => ViewFocusKind::Selected,
                };
                self.pending_view_focus = Some(kind);
                let next = (current + 1) % 3;
                self.store.set_tool_view_mode(next);
                return true;
            }
            // Transform tools are an EXCLUSIVE toggle group (a radio
            // group with no off-state): clicking any one activates
            // it and de-activates the others. Mirrors Blender / Unity
            // convention — only one transform tool is "current".
            const TRANSFORM_TOOLS: [ph2d_a11y::NodeId; 4] = [
                ids::TOOL_TRANSLATE,
                ids::TOOL_ROTATE,
                ids::TOOL_SCALE,
                ids::TOOL_PIVOT,
            ];
            if TRANSFORM_TOOLS.contains(&id) {
                for tool_id in TRANSFORM_TOOLS {
                    if let Some(crate::interaction::InteractiveState::Button { state }) =
                        self.store.get_mut(tool_id)
                    {
                        *state = if tool_id == id {
                            crate::widget::ButtonState::Pressed
                        } else {
                            crate::widget::ButtonState::Normal
                        };
                    }
                }
                return true;
            }
            // Panel-visibility toggles in the left rail. Flip the
            // hero-level visibility flag and the button's Pressed
            // state so the rail rendering reflects the new state
            // on the next frame.
            if id == ids::RAIL_SHOW_INSPECTOR {
                self.inspector_visible = !self.inspector_visible;
                if let Some(crate::interaction::InteractiveState::Button { state }) =
                    self.store.get_mut(ids::RAIL_SHOW_INSPECTOR)
                {
                    *state = if self.inspector_visible {
                        crate::widget::ButtonState::Pressed
                    } else {
                        crate::widget::ButtonState::Normal
                    };
                }
                return true;
            }
            if id == ids::RAIL_SHOW_HIERARCHY {
                self.hierarchy_visible = !self.hierarchy_visible;
                if let Some(crate::interaction::InteractiveState::Button { state }) =
                    self.store.get_mut(ids::RAIL_SHOW_HIERARCHY)
                {
                    *state = if self.hierarchy_visible {
                        crate::widget::ButtonState::Pressed
                    } else {
                        crate::widget::ButtonState::Normal
                    };
                }
                return true;
            }
            // M14.4c: Import… raises a host-polled flag so the
            // shell can open the native file picker. Other I/O
            // menu items remain placeholders.
            if id == ids::CTX_MENU_IMPORT {
                self.import_requested = true;
                self.store.close_context_menu();
                return true;
            }
            // M14.6 F: per-row Hierarchy actions. Each menu entry
            // pulls the target `row` NodeId from the most-recently
            // closed `HierarchyRow { row }` snapshot (dispatch moves
            // the request from `context_menu` to `last_context_menu`
            // on the menu-closing Down event), raises the matching
            // `pending_*` flag, and exits. The host drains the flag
            // next frame and runs the ECS mutation.
            if id == ids::CTX_MENU_HIER_DUPLICATE
                || id == ids::CTX_MENU_HIER_ADD_CHILD
                || id == ids::CTX_MENU_HIER_RESET_TRANSFORM
                || id == ids::CTX_MENU_HIER_DELETE
                || id == ids::CTX_MENU_HIER_RENAME
            {
                if let Some(req) = self.store.consume_last_context_menu()
                    && let crate::interaction::ContextMenuKind::HierarchyRow { row } = req.kind
                {
                    if id == ids::CTX_MENU_HIER_DUPLICATE {
                        self.pending_duplicate = Some(row);
                    } else if id == ids::CTX_MENU_HIER_ADD_CHILD {
                        self.pending_add_child = Some(row);
                    } else if id == ids::CTX_MENU_HIER_RESET_TRANSFORM {
                        self.pending_reset_transform = Some(row);
                    } else if id == ids::CTX_MENU_HIER_DELETE {
                        self.pending_delete = Some(row);
                    } else if id == ids::CTX_MENU_HIER_RENAME {
                        // M14.7 polish: enter inline-rename mode for
                        // this row. Painter swaps the name label for
                        // a TextInput; `pending_rename_seed` tells the
                        // host to fill the buffer with the entity's
                        // current Name on the next frame (one-shot —
                        // re-seeding every frame would clobber the
                        // user's Backspace edits).
                        open_rename(&mut self.store);
                        self.rename_target_row = Some(row);
                        self.pending_rename_seed = Some(row);
                    }
                }
                return true;
            }
            // M14.7 polish (6.3): top-level Settings cascade entry.
            // Clicking "Pixels per meter \u{25b6}" REPLACES the top-
            // level menu with the px/m presets submenu. Anchored to
            // the right edge of the clicked cascade row so the
            // submenu lands next to the chevron — Unity / Godot /
            // Blender convention. Falls back to the closed parent
            // menu's anchor when the hit rect isn't published yet
            // (defensive — shouldn't happen during normal flow).
            if id == ids::CTX_MENU_SETTINGS_PPM {
                let row_rect = self.hit_index.rect_for(id);
                let anchor = if let Some(r) = row_rect {
                    (r.x + r.w, r.y)
                } else {
                    self.store
                        .last_context_menu()
                        .map(|r| (r.x, r.y))
                        .unwrap_or((0.0, 0.0))
                };
                self.store
                    .open_context_menu(crate::interaction::ContextMenuRequest {
                        x: anchor.0,
                        y: anchor.1,
                        kind: crate::interaction::ContextMenuKind::SettingsPpmSubmenu,
                    });
                return true;
            }
            // Pixels-per-meter presets (Settings cluster). Writes
            // `project.pixels_per_meter` and closes the menu; the
            // shell will read the new value on the next import.
            let ppm_preset = if id == ids::CTX_MENU_PPM_16 {
                Some(16.0)
            } else if id == ids::CTX_MENU_PPM_32 {
                Some(32.0)
            } else if id == ids::CTX_MENU_PPM_100 {
                Some(100.0)
            } else if id == ids::CTX_MENU_PPM_256 {
                Some(256.0)
            } else if id == ids::CTX_MENU_PPM_1024 {
                Some(1024.0)
            } else {
                None
            };
            if let Some(v) = ppm_preset {
                self.project.set_pixels_per_meter(v);
                self.store.close_context_menu();
                return true;
            }
            // Save / Save As / Open Project — placeholders until the
            // pilot project wires real file I/O. Close the menu and
            // return consumed so the click doesn't propagate.
            if matches!(
                id,
                x if x == ids::CTX_MENU_SAVE
                    || x == ids::CTX_MENU_SAVE_AS
                    || x == ids::CTX_MENU_OPEN_PROJECT
            ) {
                self.store.close_context_menu();
                return true;
            }
            // Scene row click in the SceneList popover → set the
            // chip's name and close the menu. We re-filter the
            // scene list with the same query the painter used so
            // index→name maps correctly.
            if let Some(slot) = ids::CTX_SCENE_ROWS.iter().position(|x| *x == id) {
                let query = self
                    .store
                    .get(ids::CTX_SCENE_SEARCH)
                    .and_then(|s| {
                        if let crate::interaction::InteractiveState::TextInput { text, .. } = s {
                            Some(text.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");
                let lower_q = query.to_lowercase();
                let visible: Vec<&'static str> = fixture::scenes()
                    .iter()
                    .copied()
                    .filter(|s| lower_q.is_empty() || s.to_lowercase().contains(&lower_q))
                    .take(ids::CTX_SCENE_ROWS.len())
                    .collect();
                if let Some(name) = visible.get(slot) {
                    self.store.set_current_scene_name(*name);
                }
                self.store.close_context_menu();
                return true;
            }
        }
        // Image Tools mode toggle — intercepted at Hero level because
        // `image_tools_mode` lives on `HeroScreen`, not on the
        // WidgetStore. Same pattern as the theme menu / eye-toggle
        // branches above. Runs BEFORE the topbar's stub `apply_event`
        // so a click on the Image Tools pill flips the mode (and
        // doesn't fall through to the still-empty topbar handler).
        if let WidgetEvent::Click(id) = event
            && id == ids::TOPBAR_IMAGE_TOOLS
        {
            self.image_tools_mode = !self.image_tools_mode;
            return true;
        }
        // Widget Gallery toggle — palette pill in the TopBar opens /
        // closes the floating reference panel. State lives on
        // `HeroScreen::widget_gallery_visible`; geometry is materialized
        // lazily on the first show against the current viewport (see
        // `paint_hero_screen`). Same handler covers the panel's own
        // close (X) hit registered at `GAL_CLOSE` so dismissing from
        // either entry point goes through one code path.
        if let WidgetEvent::Click(id) = event
            && (id == ids::TOPBAR_WIDGET_GALLERY || id == ids::GAL_CLOSE)
        {
            self.widget_gallery_visible = !self.widget_gallery_visible;
            return true;
        }
        // Grid Settings panel toggle — TopBar pill toggles the
        // floating panel. Inner panel widgets (close X, kind dropdown,
        // toggles) are routed below via `grid_snap::apply_event`.
        if let WidgetEvent::Click(id) = event
            && id == ids::TOPBAR_GRID_SETTINGS
        {
            self.grid_snap_state.panel_visible = !self.grid_snap_state.panel_visible;
            return true;
        }
        // Route remaining events into the grid-snap panel's own handler
        // — covers GS_CLOSE, kind cycler, snap toggles, target cycler.
        // Pass `&self.store` so the handler can read the post-flip `on`
        // value the dispatcher already set on the Toggle widgets.
        if crate::grid_snap::apply_event(&mut self.grid_snap_state, event, &self.store) {
            return true;
        }
        // Trim Transparency action — raise the `pending_trim_transparency`
        // intent with whatever entity the gizmo currently has selected.
        // Host drains next frame. When nothing is selected we still
        // consume the click (so the dispatcher doesn't keep walking
        // regions) but raise nothing — the host can no-op silently or
        // surface a toast on its side.
        if let WidgetEvent::Click(id) = event
            && id == ids::IMAGE_ACTION_TRIM
        {
            self.pending_trim_transparency = self.gizmo_selection;
            return true;
        }
        // Make Square action — mirror of Trim Transparency. Click raises
        // `pending_make_square` with the current `gizmo_selection`; host
        // drains, runs `make_square`, replaces sprite pixels, reprojects
        // pivot, pushes an "Make square" undo entry. Empty selection
        // still consumes the click (silent no-op surface).
        if let WidgetEvent::Click(id) = event
            && id == ids::IMAGE_ACTION_MAKE_SQUARE
        {
            self.pending_make_square = self.gizmo_selection;
            return true;
        }
        if topbar::apply_event(&mut self.store, event) {
            return true;
        }
        if left_rail::apply_event(&mut self.store, event) {
            return true;
        }
        // M14.6 D: when a click lands on a live hierarchy row, raise
        // `pending_hierarchy_row_click` BEFORE the hierarchy itself
        // consumes the event. The host drains and resolves the row →
        // sim entity, then updates `gizmo_selection` so the canvas
        // gizmo follows the hierarchy click. This runs before
        // `hierarchy::apply_event` so the existing selection-label
        // update still happens too.
        if let WidgetEvent::Click(id) = event
            && let Some(live) = self.live_hierarchy_entries.as_ref()
            && live.contains_key(&id)
        {
            self.pending_hierarchy_row_click = Some(id);
        }
        // M14.7 polish: double-click on a hierarchy row → focus the
        // entity (same intent as F/Home, but explicit gesture on the
        // panel). We still raise `pending_hierarchy_row_click` so the
        // gizmo selection updates first; then the view-focus drain
        // pans the camera onto the freshly-selected entity.
        if let WidgetEvent::DoubleClick(id) = event
            && let Some(live) = self.live_hierarchy_entries.as_ref()
            && live.contains_key(&id)
        {
            self.pending_hierarchy_row_click = Some(id);
            self.pending_view_focus = Some(ViewFocusKind::Selected);
            return true;
        }
        // M14.7 polish: long-press on a hierarchy row → enter inline
        // rename mode. Same effect as right-click → "Rename..." but
        // modeless — works on touch / pen where a context menu isn't
        // the natural gesture.
        if let WidgetEvent::LongPress(id) = event
            && let Some(live) = self.live_hierarchy_entries.as_ref()
            && live.contains_key(&id)
        {
            open_rename(&mut self.store);
            self.rename_target_row = Some(id);
            self.pending_rename_seed = Some(id);
            return true;
        }
        // M14.7 polish: inline-rename commit / cancel for the
        // hierarchy row in rename mode. The dispatch's Enter / Esc
        // path emits these on `HIER_RENAME_INPUT`.
        if let WidgetEvent::Submit(id) = event
            && id == ids::HIER_RENAME_INPUT
            && let Some(row) = self.rename_target_row.take()
        {
            let buf = match self.store.get(ids::HIER_RENAME_INPUT) {
                Some(crate::interaction::InteractiveState::TextInput { text, .. }) => text.clone(),
                _ => String::new(),
            };
            let trimmed = buf.trim().to_owned();
            if !trimmed.is_empty() {
                self.pending_rename_commit = Some((row, trimmed));
            }
            return true;
        }
        if let WidgetEvent::Cancel(id) = event
            && id == ids::HIER_RENAME_INPUT
        {
            self.rename_target_row = None;
            return true;
        }
        // M14.7 polish: click outside the rename TextInput → commit
        // (Finder / macOS convention). Without this, focus left
        // HIER_RENAME_INPUT but `rename_target_row` stayed Some, so
        // the row remained in edit mode visually with no caret.
        // Treat the Blur as an implicit Submit: stage the current
        // buffer as a pending commit, drop rename mode.
        if let WidgetEvent::Blur(id) = event
            && id == ids::HIER_RENAME_INPUT
            && let Some(row) = self.rename_target_row.take()
        {
            let buf = match self.store.get(ids::HIER_RENAME_INPUT) {
                Some(crate::interaction::InteractiveState::TextInput { text, .. }) => text.clone(),
                _ => String::new(),
            };
            let trimmed = buf.trim().to_owned();
            if !trimmed.is_empty() {
                self.pending_rename_commit = Some((row, trimmed));
            }
            // Don't return true — Blur isn't "consumed" exclusively
            // by rename; other panels may want to observe it too.
        }
        if hierarchy::apply_event(
            &mut self.store,
            &mut self.selection,
            self.live_hierarchy_entries.as_ref(),
            event,
        ) {
            return true;
        }
        // M14.5 inspector phase (6.4): Reimport button → raise
        // `pending_reimport` so the host re-decodes the source asset
        // at the current `project.pixels_per_meter`. Captured BEFORE
        // delegating to `inspector::apply_event` because that helper
        // doesn't know about HeroScreen-level pending fields.
        if let WidgetEvent::Click(id) = event
            && id == ids::INSP_RENDER_SOURCE_REIMPORT
            && let Some(info) = self.inspector_sprite.as_ref()
            && info.can_reimport
        {
            self.pending_reimport = Some(info.entity_bits);
            return true;
        }
        // M14.A: Transform editor commits — ValueChanged on any of the
        // 5 NumberInputs (Enter / blur per `dispatch_key` semantics)
        // builds a fresh `InspectorTransformInfo` from the current
        // store values and publishes it via `pending_transform_edit`.
        // The shell drains this once per frame and pushes a
        // `EditorCommand::SetComponent` for `Transform` (first real
        // consumer of the editor command pipeline).
        if let WidgetEvent::ValueChanged(id) = event
            && matches!(
                id,
                ids::INSP_TRANSFORM_POS_X
                    | ids::INSP_TRANSFORM_POS_Y
                    | ids::INSP_TRANSFORM_ROT
                    | ids::INSP_TRANSFORM_SCALE_X
                    | ids::INSP_TRANSFORM_SCALE_Y,
            )
            && let Some(info) = self.inspector_transform
        {
            let x = self
                .store
                .number_value(ids::INSP_TRANSFORM_POS_X)
                .unwrap_or(info.translation[0] as f64) as f32;
            let y = self
                .store
                .number_value(ids::INSP_TRANSFORM_POS_Y)
                .unwrap_or(info.translation[1] as f64) as f32;
            let rot_deg =
                self.store
                    .number_value(ids::INSP_TRANSFORM_ROT)
                    .unwrap_or((info.rotation_rad as f64).to_degrees()) as f32;
            let sx = self
                .store
                .number_value(ids::INSP_TRANSFORM_SCALE_X)
                .unwrap_or(info.scale[0] as f64) as f32;
            let sy = self
                .store
                .number_value(ids::INSP_TRANSFORM_SCALE_Y)
                .unwrap_or(info.scale[1] as f64) as f32;
            self.pending_transform_edit = Some(InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [x, y],
                rotation_rad: rot_deg.to_radians(),
                scale: [sx, sy],
            });
            return true;
        }
        // Reset-to-Identity button — publishes the Identity transform
        // for the currently selected entity. Same path as a field
        // commit so the shell's queue-push code path stays uniform.
        if let WidgetEvent::Click(id) = event
            && id == ids::INSP_TRANSFORM_RESET
            && let Some(info) = self.inspector_transform
        {
            self.pending_transform_edit = Some(InspectorTransformInfo {
                entity_bits: info.entity_bits,
                translation: [0.0, 0.0],
                rotation_rad: 0.0,
                scale: [1.0, 1.0],
            });
            return true;
        }
        // M14.D: Visibility checkbox toggled. The dispatch already
        // flipped `CheckboxValue` in the store and emitted
        // `WidgetEvent::Toggled(INSP_VISIBILITY_CHECK)`; we read the
        // POST-toggle value and publish `pending_visibility_edit` for
        // the shell to push as `EditorCommand::SetComponent` for the
        // `ph2d_ecs::Visibility` component.
        if let WidgetEvent::Toggled(id) = event
            && id == ids::INSP_VISIBILITY_CHECK
            && let Some(info) = self.inspector_visibility
        {
            let visible = matches!(
                self.store.checkbox(id).map(|(_, v)| v),
                Some(crate::widget::CheckboxValue::Checked),
            );
            self.pending_visibility_edit = Some(InspectorVisibilityInfo {
                entity_bits: info.entity_bits,
                visible,
            });
            return true;
        }
        // M14.C: Render Source Strategy switcher. A click on a
        // non-pressed button raises `pending_sprite_source_change`
        // with the requested kind; the shell does the renderer-side
        // swap on drain. Clicks on the already-Pressed button are
        // consumed silently (no-op).
        if let WidgetEvent::Click(id) = event
            && let Some(requested) = match id {
                ids::INSP_RENDER_STRATEGY_ATLAS => Some(RequestedSpriteStrategy::Atlas),
                ids::INSP_RENDER_STRATEGY_INDIVIDUAL => Some(RequestedSpriteStrategy::Individual),
                ids::INSP_RENDER_STRATEGY_HANDPACKED => Some(RequestedSpriteStrategy::HandPacked),
                _ => None,
            }
            && let Some(info) = self.inspector_sprite.as_ref()
        {
            let current = match info.source_kind {
                InspectorSpriteSource::Atlas { .. } => RequestedSpriteStrategy::Atlas,
                InspectorSpriteSource::Individual { .. } => RequestedSpriteStrategy::Individual,
                InspectorSpriteSource::HandPacked => RequestedSpriteStrategy::HandPacked,
            };
            if requested != current {
                self.pending_sprite_source_change = Some((info.entity_bits, requested));
            }
            // Audit fix #7 (HIGH): reset the just-clicked button's
            // stored state back to Normal regardless of whether the
            // swap will succeed. The painter re-pins the matching
            // strategy to Pressed each frame from the snapshot, so
            // leaving the clicked button in the dispatch-set
            // Pressed/Hovered state would visually claim "active" on
            // a button that the host either rejected (toast path) or
            // is about to overwrite anyway. Clearing here keeps the
            // painter's snapshot-driven pin as the single source of
            // visual truth for which strategy is "current".
            if let Some(InteractiveState::Button { state }) = self.store.get_mut(id) {
                *state = crate::widget::ButtonState::Normal;
            }
            return true;
        }
        // M14.E: entity-name TextInput edits. Live commit on every
        // `TextChanged` — `Option` coalescing in `pending_name_edit`
        // means the shell drains at most once per frame regardless of
        // typing speed.
        if let WidgetEvent::TextChanged(id) = event
            && id == ids::INSP_ENTITY_NAME
            && let Some(info) = self.inspector_name.as_ref()
        {
            let text = self.store.text(id).unwrap_or("").to_string();
            self.pending_name_edit = Some(InspectorNameInfo {
                entity_bits: info.entity_bits,
                name: text,
            });
            return true;
        }
        if inspector::apply_event(&mut self.store, event) {
            return true;
        }
        false
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

    let mut layout = HeroLayout::for_viewport_mirrored(viewport, hero.ui_mirrored);
    // Apply user-driven panel drag offsets to the Inspector +
    // Hierarchy rects. The offsets live on the WidgetStore's
    // `blender_picker_offset` side-table (panel-agnostic — the
    // dispatch's BlenderHitKind::DragHandle path stores the
    // offset under the `parent` NodeId regardless of widget kind).
    //
    // Two clamps:
    //   1. Horizontal: keep ≥60px of the panel inside the viewport
    //      so the user can always grab the drag bar back.
    //   2. Vertical: the panel's top stays inside the viewport and
    //      its bottom never crosses `viewport.bottom - 8`. When the
    //      user drags DOWN past where `base.h` fits, the panel
    //      auto-shrinks (floor at MIN_H so the header + a row stay
    //      visible). Dragging back up restores the natural height.
    //
    // The clamped offset is also written back into the store so
    // subsequent drag-begins capture the visible offset rather than
    // an accumulated raw value — eliminates the "rubber band" the
    // user perceived as discrete jumps when reversing direction.
    const MIN_W: f32 = 220.0;
    const MIN_H: f32 = 120.0;
    // `resize` lets the user manually grow/shrink the panel via the
    // bottom-right gripper (state `panel_resize_delta`). Manual size
    // is computed FIRST so the auto-shrink-on-drag-down logic below
    // sees the user's chosen base height.
    let clamp_panel = |base: Rect,
                       off: (f32, f32),
                       resize: (f32, f32),
                       viewport: Rect|
     -> (Rect, (f32, f32), (f32, f32)) {
        let raw_w = (base.w + resize.0).max(MIN_W);
        let raw_h = (base.h + resize.1).max(MIN_H);
        let max_w = (viewport.w * 0.7).max(MIN_W);
        let new_w = raw_w.min(max_w);
        let new_h_user = raw_h.min(viewport.h.max(MIN_H));
        let clamped_dw = new_w - base.w;
        let clamped_dh = new_h_user - base.h;

        let max_x = (viewport.x + viewport.w - 60.0) - base.x;
        let min_x = (viewport.x + 60.0) - (base.x + new_w);
        let max_bottom = viewport.y + viewport.h - 8.0;
        let min_y = viewport.y - base.y;
        let max_y = (max_bottom - MIN_H) - base.y;
        let dx = off.0.clamp(min_x, max_x);
        let dy = off.1.clamp(min_y.min(max_y), max_y);
        let new_y = base.y + dy;
        let natural_bottom = new_y + new_h_user;
        let final_h = if natural_bottom > max_bottom {
            (max_bottom - new_y).max(MIN_H)
        } else {
            new_h_user
        };
        (
            Rect::new(base.x + dx, new_y, new_w, final_h),
            (dx, dy),
            (clamped_dw, clamped_dh),
        )
    };
    let insp_off = hero.store.blender_picker_offset(ids::INSP_PANEL);
    let hier_off = hero.store.blender_picker_offset(ids::HIER_PANEL);
    let insp_resize = hero.store.panel_resize_delta(ids::INSP_PANEL);
    let hier_resize = hero.store.panel_resize_delta(ids::HIER_PANEL);
    let (insp_rect, insp_clamped_off, insp_clamped_resize) =
        clamp_panel(layout.inspector, insp_off, insp_resize, viewport);
    let (hier_rect, hier_clamped_off, hier_clamped_resize) =
        clamp_panel(layout.hierarchy, hier_off, hier_resize, viewport);
    layout.inspector = insp_rect;
    layout.hierarchy = hier_rect;
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
    if hero.grid_view.is_none() {
        paint_canvas_bg(&layout, scene, hero.theme);
    }
    // M14.4b: world-space grid overlay. Painted between the canvas
    // background and the selection marquee so the marquee remains
    // legible over the grid. Skipped when toggle is off or host
    // hasn't published a camera view. We substitute the layout's
    // computed canvas rect into the view so the host doesn't have
    // to mirror layout math — it only owns camera + window dims.
    if hero.grid_visible
        && let Some(view) = hero.grid_view
    {
        let view = crate::grid::GridView {
            canvas: layout.canvas,
            ..view
        };
        crate::grid_snap::render::paint(scene, &view, &hero.grid_snap_state);
    }
    // M14.4c: the legacy mockup selection marquee draws a fixed-size
    // dashed rect at the CANVAS center in screen pixels — it has no
    // world-space coupling and so doesn't follow pan/zoom. Skip it
    // when a `grid_view` is published (live ECS mode) so we don't
    // mislead users into thinking the marquee tracks an entity.
    // Fixture mode keeps the placeholder marquee for the mockup
    // screenshots.
    if hero.grid_view.is_none()
        && let Some(sel) = hero.selection.as_ref()
    {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
    }
    // M14.7 B: live-mode sprite gizmo. The host publishes a
    // `gizmo_view` carrying the selected sprite's world-space bbox +
    // current camera; the painter projects to screen pixels with the
    // same math the grid uses (so the gizmo and grid stay aligned
    // across pan/zoom).
    if let Some(view) = hero.gizmo_view {
        crate::gizmo::paint_sprite_gizmo(scene, &view, hero.theme, &mut hero.hit_index);
    }
    paint_top_bar(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        hero.image_tools_mode,
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
    if hero.inspector_visible {
        hero.store.set_panel_rect(ids::INSP_PANEL, layout.inspector);
    } else {
        hero.store.clear_panel_rect(ids::INSP_PANEL);
    }
    if hero.hierarchy_visible {
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
    }
    hierarchy::set_selection_label(hero.selection.as_ref().map(|s| s.label.clone()));
    // Publish live entries (if any) to the hierarchy painter so it
    // overrides `fixture::hierarchy()`. Cleared at the end of paint
    // so the next frame's `sync_from_hierarchy` is the single source.
    hierarchy::set_live_entries(hero.live_hierarchy_entries.clone());
    hierarchy::set_rename_target(hero.rename_target_row);
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

    // Paint each panel in z-order — bottom-first, so the panel most
    // recently clicked / dragged / opened sits on top.  Panels that
    // haven't been touched yet inherit a default order at the bottom.
    let mut z_order: Vec<ph2d_a11y::NodeId> = hero.store.panel_z_order().to_vec();
    for &fallback in &[ids::HIER_PANEL, ids::INSP_PANEL, ids::INSP_BLENDER_PICKER] {
        if !z_order.contains(&fallback) {
            z_order.push(fallback);
        }
    }
    for panel_id in z_order {
        if panel_id == ids::INSP_PANEL && hero.inspector_visible {
            inspector_sync::sync_inspector_from_snapshots(hero);
            // Publish the host-supplied sprite snapshot for the
            // Render Source section. Cleared after paint so a stale
            // snapshot can't leak into the next frame.
            inspector::set_current_inspector_sprite(hero.inspector_sprite.clone());
            inspector::set_current_inspector_transform(hero.inspector_transform);
            inspector::set_current_inspector_visibility(hero.inspector_visibility);
            inspector::set_current_inspector_name(hero.inspector_name.clone());
            paint_inspector(
                &layout,
                hero.selection.as_ref(),
                scene,
                text_system,
                hero.theme,
                &mut hero.hit_index,
                &hero.store,
            );
            inspector::set_current_inspector_sprite(None);
            inspector::set_current_inspector_transform(None);
            inspector::set_current_inspector_visibility(None);
            inspector::set_current_inspector_name(None);
            // Publish content_h + clamp scroll right after paint so
            // `dispatch_wheel` sees the new bounds on the very next
            // event (avoids a one-frame overshoot when a section
            // collapses or notes are added).
            let content_h = inspector::last_inspector_content_h();
            let visible_h = inspector::last_inspector_visible_h();
            hero.store.set_panel_content_h(ids::INSP_PANEL, content_h);
            hero.store.set_panel_visible_h(ids::INSP_PANEL, visible_h);
            let max_scroll = (content_h - visible_h).max(0.0);
            let cur = hero.store.panel_scroll(ids::INSP_PANEL);
            if cur > max_scroll {
                hero.store.set_panel_scroll(ids::INSP_PANEL, max_scroll);
            }
        } else if panel_id == ids::HIER_PANEL && hero.hierarchy_visible {
            paint_hierarchy(
                &layout,
                scene,
                text_system,
                hero.theme,
                &mut hero.hit_index,
                &mut hero.store,
            );
            let content_h = hierarchy::last_hierarchy_content_h();
            hero.store.set_panel_content_h(ids::HIER_PANEL, content_h);
            let visible_h = (layout.hierarchy.h - 60.0).max(0.0);
            let max_scroll = (content_h - visible_h).max(0.0);
            let cur = hero.store.panel_scroll(ids::HIER_PANEL);
            if cur > max_scroll {
                hero.store.set_panel_scroll(ids::HIER_PANEL, max_scroll);
            }
        } else if panel_id == ids::INSP_BLENDER_PICKER && hero.store.picker_target().is_some() {
            // The picker paint is a no-op if `picker_target` isn't
            // set (early-out inside the demo painter); the visibility
            // guard mirrors that so we don't waste an iteration.
            color_picker_demo::paint_blender_picker_demo(
                &layout,
                scene,
                text_system,
                hero.theme,
                &mut hero.hit_index,
                &hero.store,
            );
        }
    }
    if hero.stats_visible {
        paint_bottom_hud(&layout, scene, text_system, hero.theme, hero.stats);
    }
    // Widget Gallery floating panel. Sits above every docked panel
    // and the bottom HUD but below tooltips + context menus so a
    // user inspecting the gallery can still summon menus / hover
    // helpers. Geometry materialized lazily on first show; once set,
    // the base rect persists and drag / resize deltas come from the
    // store (`blender_picker_offset` + `panel_resize_delta`) so the
    // gallery is movable + resizable like the Inspector.
    if hero.widget_gallery_visible {
        let base_rect = match hero.widget_gallery_rect {
            Some(r) => r,
            None => {
                let r = widget_gallery::default_rect(
                    layout.viewport.w,
                    layout.viewport.h,
                    layout.inspector.w,
                );
                hero.widget_gallery_rect = Some(r);
                r
            }
        };
        let gal_off = hero.store.blender_picker_offset(ids::GAL_PANEL);
        let gal_resize = hero.store.panel_resize_delta(ids::GAL_PANEL);
        let (gallery_rect, gal_clamped_off, gal_clamped_resize) =
            clamp_panel(base_rect, gal_off, gal_resize, viewport);
        if (gal_clamped_off.0 - gal_off.0).abs() > f32::EPSILON
            || (gal_clamped_off.1 - gal_off.1).abs() > f32::EPSILON
        {
            hero.store.set_blender_picker_offset(
                ids::GAL_PANEL,
                gal_clamped_off.0,
                gal_clamped_off.1,
            );
        }
        if (gal_clamped_resize.0 - gal_resize.0).abs() > f32::EPSILON
            || (gal_clamped_resize.1 - gal_resize.1).abs() > f32::EPSILON
        {
            hero.store.set_panel_resize_delta(
                ids::GAL_PANEL,
                gal_clamped_resize.0,
                gal_clamped_resize.1,
            );
        }
        // Publish the panel rect so wheel-event dispatch + chrome
        // routing recognize "inside the gallery". Independent panel
        // id (`GAL_PANEL`) keeps Inspector's `INSP_PANEL` state
        // untouched.
        hero.store.set_panel_rect(ids::GAL_PANEL, gallery_rect);
        widget_gallery::paint(
            gallery_rect,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
        );
        // Publish content + visible heights so `dispatch_wheel` knows
        // the scroll bound for `GAL_PANEL`. Clamp scroll here too so
        // a shrunken panel can't leave scroll past max.
        let content_h = widget_gallery::last_content_h();
        let visible_h = widget_gallery::last_visible_h();
        hero.store.set_panel_content_h(ids::GAL_PANEL, content_h);
        hero.store.set_panel_visible_h(ids::GAL_PANEL, visible_h);
        let max_scroll = (content_h - visible_h).max(0.0);
        let cur = hero.store.panel_scroll(ids::GAL_PANEL);
        if cur > max_scroll {
            hero.store.set_panel_scroll(ids::GAL_PANEL, max_scroll);
        }
    }
    // Grid Settings floating panel — mirrors the Widget Gallery
    // pattern (lazy default rect on first show, drag/resize deltas via
    // store, panel rect published for chrome routing).
    if hero.grid_snap_state.panel_visible {
        let base_rect = match hero.grid_snap_state.panel_rect {
            Some(r) => r,
            None => {
                let r = crate::grid_snap::default_rect(layout.viewport.w, layout.viewport.h);
                hero.grid_snap_state.panel_rect = Some(r);
                r
            }
        };
        let gs_off = hero
            .store
            .blender_picker_offset(crate::grid_snap::ids::GS_PANEL);
        let gs_resize = hero
            .store
            .panel_resize_delta(crate::grid_snap::ids::GS_PANEL);
        let (gs_rect, gs_clamped_off, gs_clamped_resize) =
            clamp_panel(base_rect, gs_off, gs_resize, viewport);
        if (gs_clamped_off.0 - gs_off.0).abs() > f32::EPSILON
            || (gs_clamped_off.1 - gs_off.1).abs() > f32::EPSILON
        {
            hero.store.set_blender_picker_offset(
                crate::grid_snap::ids::GS_PANEL,
                gs_clamped_off.0,
                gs_clamped_off.1,
            );
        }
        if (gs_clamped_resize.0 - gs_resize.0).abs() > f32::EPSILON
            || (gs_clamped_resize.1 - gs_resize.1).abs() > f32::EPSILON
        {
            hero.store.set_panel_resize_delta(
                crate::grid_snap::ids::GS_PANEL,
                gs_clamped_resize.0,
                gs_clamped_resize.1,
            );
        }
        hero.store
            .set_panel_rect(crate::grid_snap::ids::GS_PANEL, gs_rect);
        crate::grid_snap::paint(
            gs_rect,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            &hero.grid_snap_state,
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
    );
    // M14.4e: file-drop overlay sits above EVERY layer (chrome,
    // tooltips, context menus) so the user always sees the "Drop to
    // import" hint while the OS drag is active.
    if let Some((paths, cursor)) = hero.dragging_files.as_ref() {
        paint_drop_overlay(&layout, paths, *cursor, scene, text_system, hero.theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::ButtonState;

    fn ipad12_viewport() -> Rect {
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
    }

    #[test]
    fn layout_top_bar_inset_from_edge() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!((layout.top_bar.x - style::EDGE_PAD).abs() < f32::EPSILON);
        assert!((layout.top_bar.h - style::TOPBAR_H).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_left_rail_below_top_bar() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!(layout.left_rail.y > layout.top_bar.y + layout.top_bar.h);
        assert!((layout.left_rail.w - style::RAIL_W).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_hierarchy_after_rail_by_default() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!(layout.hierarchy.x > layout.left_rail.x + layout.left_rail.w);
        assert!((layout.hierarchy.w - style::HIERARCHY_W).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_inspector_pinned_right_by_default() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let right_edge = layout.inspector.x + layout.inspector.w;
        assert!((right_edge - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
    }

    #[test]
    fn layout_canvas_spans_full_viewport_default() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        // Canvas is the full-viewport backdrop; chrome floats over.
        assert!((layout.canvas.x - layout.viewport.x).abs() < f32::EPSILON);
        assert!((layout.canvas.w - layout.viewport.w).abs() < f32::EPSILON);
        // Side panels still sit at their canonical positions.
        assert!(layout.hierarchy.x > layout.left_rail.x + layout.left_rail.w);
        let insp_right = layout.inspector.x + layout.inspector.w;
        assert!((insp_right - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
    }

    #[test]
    fn layout_mirror_swaps_sides() {
        let layout = HeroLayout::for_viewport_mirrored(ipad12_viewport(), true);
        // Mirrored: inspector after rail (left), hierarchy pinned right.
        assert!(layout.inspector.x > layout.left_rail.x + layout.left_rail.w);
        let hier_right = layout.hierarchy.x + layout.hierarchy.w;
        assert!((hier_right - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
        // Canvas is full-viewport in either orientation.
        assert!((layout.canvas.w - layout.viewport.w).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_bottom_hud_centered_horizontally() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mid = layout.bottom_hud.x + layout.bottom_hud.w * 0.5;
        assert!((mid - HERO_VIEWPORT_W * 0.5).abs() < 0.5);
    }

    #[test]
    fn hero_default_carries_fixture_selection() {
        let h = HeroScreen::new(NodeId(1));
        assert!(h.selection.is_some());
    }

    #[test]
    fn hero_selection_clearable() {
        let h = HeroScreen::new(NodeId(1)).selection(None);
        assert!(h.selection.is_none());
    }

    #[test]
    fn a11y_root_is_window() {
        let h = HeroScreen::new(NodeId(1));
        let node = h.build_a11y(ipad12_viewport());
        assert_eq!(node.role(), Role::Window);
    }

    #[test]
    fn paint_hero_smoke_default() {
        let mut hero = HeroScreen::new(NodeId(1));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_hero_smoke_alternate_theme() {
        let mut hero = HeroScreen::new(NodeId(1)).theme(Theme::Sunstone);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_hero_smoke_no_selection() {
        let mut hero = HeroScreen::new(NodeId(1)).selection(None);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_hero_smoke_all_themes() {
        for theme in [
            Theme::Forge,
            Theme::Workshop,
            Theme::Sunstone,
            Theme::Blueprint,
        ] {
            let mut hero = HeroScreen::new(NodeId(1)).theme(theme);
            let mut scene = VectorScene::new();
            let mut text = TextSystem::new();
            paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        }
    }

    use bumpalo::Bump;
    use ph2d_host::{PointerEvent, PointerKind, PointerSource};

    fn down(x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: 0,
        }
    }

    #[allow(dead_code)]
    fn up(x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind: PointerKind::Up,
            source: PointerSource::Mouse,
            button: ph2d_host::PointerButton::Primary,
            timestamp_ns: 0,
        }
    }

    #[test]
    fn hero_pre_populates_store_with_topbar_and_tools() {
        let hero = HeroScreen::new(NodeId(1));
        for id in [
            ids::TOPBAR_SAVE,
            ids::TOPBAR_PROJECT,
            ids::TOPBAR_PLAY_BUTTON,
            ids::TOPBAR_RIGHT_LAYERS,
            ids::HIERARCHY_ADD,
            ids::TOOL_TRANSLATE,
            ids::TOOL_REDO,
        ] {
            assert!(
                hero.store.contains(id),
                "store missing pre-populated id {id:?}"
            );
        }
    }

    #[test]
    fn hero_translate_tool_starts_pressed() {
        let hero = HeroScreen::new(NodeId(1));
        assert_eq!(
            hero.store.button_state(ids::TOOL_TRANSLATE),
            Some(ButtonState::Pressed),
        );
    }

    #[test]
    fn hero_topbar_save_click_opens_save_menu() {
        // Save chip on the topbar now opens the SaveMenu context
        // menu (same pattern as the Theme chip → ThemeSelector). The
        // pointer Down → menu-open short-circuits the Up's
        // Click(TOPBAR_SAVE) emit, so we assert on the open menu's
        // kind instead.
        let mut hero = HeroScreen::new(NodeId(1));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        let arena = Bump::new();
        let mut save_x = 0.0;
        let mut save_y = 0.0;
        'outer: for y_int in (14..54).step_by(4) {
            for x_int in (14..1352).step_by(4) {
                if hero.hit_index.hit(x_int as f32, y_int as f32) == Some(ids::TOPBAR_SAVE) {
                    save_x = x_int as f32;
                    save_y = y_int as f32;
                    break 'outer;
                }
            }
        }
        assert!(save_x > 0.0, "TOPBAR_SAVE rect not found in hit_index");
        let _ = hero.handle_pointer(down(save_x, save_y), &arena);
        assert!(matches!(
            hero.store.context_menu().map(|r| r.kind),
            Some(crate::interaction::ContextMenuKind::SaveMenu)
        ));
    }

    #[test]
    fn hero_apply_event_hierarchy_click_changes_selection() {
        // Placeholder fixture only registers Scene Root; the reserved
        // HIER_* ids return None from `hierarchy_label_for_id` until
        // the pilot project wires real entities.
        let mut hero = HeroScreen::new(NodeId(1));
        let consumed = hero.apply_event(WidgetEvent::Click(ids::HIER_PLAYER));
        assert!(consumed);
        assert_eq!(
            hero.selection.as_ref().map(|s| s.label.as_str()),
            Some("Scene Root")
        );
    }

    #[test]
    fn hero_apply_event_unrelated_click_returns_false() {
        let mut hero = HeroScreen::new(NodeId(1));
        let consumed = hero.apply_event(WidgetEvent::Click(ids::TOPBAR_SAVE));
        assert!(!consumed);
    }

    /// Regression: the Widget Gallery must publish content_h /
    /// visible_h to the store after painting so the wheel dispatch
    /// can clamp the scroll bound on `GAL_PANEL`. Without this the
    /// user reports "scroll doesn't work" — wheel events would either
    /// be ignored (no panel match) or fail to advance (max_scroll = 0).
    #[test]
    fn gallery_publishes_scroll_bounds_after_paint() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.widget_gallery_visible = true;
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        let content_h = hero
            .store
            .panel_content_h(ids::GAL_PANEL)
            .expect("GAL_PANEL content_h must be published after paint");
        let visible_h = hero
            .store
            .panel_visible_h(ids::GAL_PANEL)
            .expect("GAL_PANEL visible_h must be published after paint");
        assert!(
            content_h > 0.0,
            "gallery content_h should be positive (sections painted), got {content_h}"
        );
        assert!(
            visible_h > 0.0,
            "gallery visible_h should be positive (body region), got {visible_h}"
        );
        assert!(
            content_h > visible_h,
            "gallery should overflow (content_h={content_h} > visible_h={visible_h}) \
             so scroll has effect — otherwise wheel is a no-op"
        );
        let panel_rect = hero
            .store
            .panel_rect(ids::GAL_PANEL)
            .expect("GAL_PANEL rect must be registered for panel_at");
        // The cursor at the center of the panel must select GAL_PANEL
        // when dispatch_wheel calls `panel_at`.
        let cx = panel_rect.x + panel_rect.w * 0.5;
        let cy = panel_rect.y + panel_rect.h * 0.5;
        assert_eq!(
            hero.store.panel_at(cx, cy),
            Some(ids::GAL_PANEL),
            "cursor over gallery center should resolve to GAL_PANEL"
        );
        // End-to-end wheel: dispatch a wheel event at the gallery
        // center with a negative delta (macOS "swipe up" / scroll
        // forward) and assert panel_scroll advanced.
        let arena = bumpalo::Bump::new();
        let before = hero.store.panel_scroll(ids::GAL_PANEL);
        let _ = crate::interaction::dispatch_wheel(
            &mut hero.store,
            ph2d_host::WheelEvent {
                x: cx,
                y: cy,
                delta_x: 0.0,
                delta_y: -40.0,
                modifiers: ph2d_host::Modifiers::default(),
                timestamp_ns: 0,
            },
            &arena,
        );
        let after = hero.store.panel_scroll(ids::GAL_PANEL);
        assert!(
            after > before,
            "wheel down on gallery should increase panel_scroll \
             (before={before}, after={after})"
        );
    }

    /// Regression: right-clicking inside the gallery body → choosing
    /// "Create note" must push a `NoteData` keyed on `GAL_PANEL` (NOT
    /// `INSP_PANEL`) so the gallery renders it on the next frame. The
    /// gallery is the canonical UI ground-truth for peripheral agents
    /// — features the showcase advertises (sticky notes, section
    /// outline) need to work in the in-app gallery, not just in the
    /// retired reference snapshot.
    #[test]
    fn gallery_create_note_targets_gal_panel() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.widget_gallery_visible = true;
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        // Paint once so `panel_rect(GAL_PANEL)` is published and
        // `LAST_BODY_TOP_SCREEN_Y` is set for the upcoming dispatch.
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        let gallery_rect = hero.store.panel_rect(ids::GAL_PANEL).unwrap();
        let cx = gallery_rect.x + gallery_rect.w * 0.5;
        let cy = gallery_rect.y + gallery_rect.h * 0.5;
        // Open the CreateNote context menu via the same path the
        // pointer dispatch uses for a secondary-button down at the
        // gallery center.
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: cx,
                y: cy,
                kind: crate::interaction::ContextMenuKind::CreateNote {
                    panel: ids::GAL_PANEL,
                    before_section: None,
                },
            });
        // The real pointer dispatch closes the menu on the Down that
        // hit the menu item, snapshotting the request into
        // `last_context_menu` before the Click reaches `apply_event`.
        // Skipping this step would leave the request in the still-open
        // `context_menu` slot where `consume_last_context_menu` can't
        // see it.
        hero.store.close_context_menu();
        // Click "Create note" — inspector::apply_event handles it.
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_CREATE_NOTE));
        assert!(consumed, "CTX_MENU_CREATE_NOTE click should be consumed");
        assert_eq!(
            hero.store.notes_for_panel(ids::GAL_PANEL).len(),
            1,
            "exactly one note should be pushed against GAL_PANEL"
        );
        assert_eq!(
            hero.store.notes_for_panel(ids::INSP_PANEL).len(),
            0,
            "INSP_PANEL should be untouched — the gallery's note must \
             not leak into the live Inspector"
        );
    }

    /// Regression: right-clicking on a gallery section header →
    /// choosing a color must write `section_outline_color` so the
    /// gallery's next paint draws the colored ring around that
    /// section's body. Mirror of the live Inspector's right-click
    /// outline path — same NodeIds (`INSP_SECTION_*`) because the
    /// gallery re-uses the section painters.
    #[test]
    fn gallery_section_outline_color_writes_through() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.widget_gallery_visible = true;
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        // Open the SectionOutline menu for the Inputs section header.
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::SectionOutline {
                    section: ids::INSP_SECTION_INPUTS,
                },
            });
        // Mirror the real Down-on-menu-item path that snapshots the
        // request into `last_context_menu` before the Click fires.
        hero.store.close_context_menu();
        // Pick "Yellow" (color_idx 0).
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_OUTLINE_0));
        assert!(consumed, "CTX_MENU_OUTLINE_0 click should be consumed");
        assert_eq!(
            hero.store.section_outline_color(ids::INSP_SECTION_INPUTS),
            Some(0),
            "Inputs section should have outline color 0 (Yellow) set"
        );
    }

    #[test]
    fn paint_top_bar_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(32);
        paint_top_bar(
            &layout,
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hits,
            &store,
            false,
        );
    }

    /// With `image_tools_mode = true`, the painter must register the
    /// `IMAGE_ACTION_TRIM` hit and must NOT register the right-side
    /// default clusters (Project/Play/Right/Settings).
    #[test]
    fn paint_top_bar_image_tools_mode_swaps_right_side() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(32);
        paint_top_bar(
            &layout,
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hits,
            &store,
            true,
        );
        assert!(
            hits.rect_for(ids::IMAGE_ACTION_TRIM).is_some(),
            "trim action pill must be hit-registered when image_tools_mode is on",
        );
        for default_right in [
            ids::TOPBAR_PROJECT,
            ids::TOPBAR_PLAY_BUTTON,
            ids::TOPBAR_RIGHT_LAYERS,
            ids::TOPBAR_SETTINGS,
        ] {
            assert!(
                hits.rect_for(default_right).is_none(),
                "right-side default cluster {default_right:?} must NOT be registered in image_tools mode",
            );
        }
        // Left half stays intact — Save/Open/ImageTools are still hit-able.
        assert!(hits.rect_for(ids::TOPBAR_SAVE).is_some());
        assert!(hits.rect_for(ids::TOPBAR_OPEN).is_some());
        assert!(hits.rect_for(ids::TOPBAR_IMAGE_TOOLS).is_some());
    }

    /// Clicking the Image Tools pill flips `image_tools_mode`; clicking
    /// again flips it back. Verified through `HeroScreen::apply_event`
    /// so the dispatcher hook is exercised end-to-end.
    #[test]
    fn click_on_image_tools_pill_toggles_mode() {
        let mut hero = HeroScreen::new(NodeId(1));
        assert!(!hero.image_tools_mode);
        assert!(hero.apply_event(WidgetEvent::Click(ids::TOPBAR_IMAGE_TOOLS)));
        assert!(hero.image_tools_mode);
        assert!(hero.apply_event(WidgetEvent::Click(ids::TOPBAR_IMAGE_TOOLS)));
        assert!(!hero.image_tools_mode);
    }

    /// M14.A: a `ValueChanged` event on any Transform NumberInput
    /// publishes a fresh `InspectorTransformInfo` via
    /// `pending_transform_edit`, taking the current store values for
    /// every axis (X/Y/Rot/Scale-X/Scale-Y) plus the selected entity
    /// id from `inspector_transform`. Rotation is converted from
    /// degrees (UI) back to radians (canonical) at commit.
    #[test]
    fn transform_field_commit_raises_pending_with_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        // No selection → no pending fired even on commit (avoids
        // silently editing a non-existent entity).
        hero.inspector_transform = None;
        assert!(!hero.apply_event(WidgetEvent::ValueChanged(ids::INSP_TRANSFORM_POS_X)));
        assert_eq!(hero.pending_transform_edit, None);

        // With selection + custom store values → pending mirrors the
        // store snapshot exactly. We seed the store with non-identity
        // numbers and verify the commit assembles them all.
        hero.inspector_transform = Some(InspectorTransformInfo {
            entity_bits: 0xCAFE_F00D,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
        });
        hero.store.set_number_value(ids::INSP_TRANSFORM_POS_X, 1.5);
        hero.store
            .set_number_value(ids::INSP_TRANSFORM_POS_Y, -2.25);
        hero.store.set_number_value(ids::INSP_TRANSFORM_ROT, 90.0); // degrees
        hero.store
            .set_number_value(ids::INSP_TRANSFORM_SCALE_X, 2.0);
        hero.store
            .set_number_value(ids::INSP_TRANSFORM_SCALE_Y, 0.5);
        assert!(hero.apply_event(WidgetEvent::ValueChanged(ids::INSP_TRANSFORM_POS_X)));
        let pending = hero.pending_transform_edit.expect("pending populated");
        assert_eq!(pending.entity_bits, 0xCAFE_F00D);
        assert_eq!(pending.translation, [1.5, -2.25]);
        // 90° → π/2 rad. `to_radians` is bit-deterministic (HR-5).
        assert!((pending.rotation_rad - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_eq!(pending.scale, [2.0, 0.5]);
    }

    /// M14.A: clicking the Reset-to-Identity button publishes an
    /// Identity transform via `pending_transform_edit`. Same commit
    /// path as a field ValueChanged so the shell's queue-push code
    /// stays uniform.
    #[test]
    fn transform_reset_button_publishes_identity() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_transform = Some(InspectorTransformInfo {
            entity_bits: 0xBABE_0042,
            translation: [10.0, 20.0],
            rotation_rad: 1.0,
            scale: [3.0, 3.0],
        });
        // Even if the store has garbage in it, Reset always publishes
        // pure identity — independent of buffer state.
        hero.store.set_number_value(ids::INSP_TRANSFORM_POS_X, 99.0);
        assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_TRANSFORM_RESET)));
        let pending = hero.pending_transform_edit.expect("pending populated");
        assert_eq!(pending.entity_bits, 0xBABE_0042);
        assert_eq!(pending.translation, [0.0, 0.0]);
        assert_eq!(pending.rotation_rad, 0.0);
        assert_eq!(pending.scale, [1.0, 1.0]);

        // Without a selection, Reset is a no-op (consumes the click
        // returning false → dispatcher walks; matches non-sprite
        // Reimport behavior).
        hero.inspector_transform = None;
        hero.pending_transform_edit = None;
        assert!(!hero.apply_event(WidgetEvent::Click(ids::INSP_TRANSFORM_RESET)));
        assert_eq!(hero.pending_transform_edit, None);
    }

    /// M14.D: Toggled on the Visibility checkbox publishes
    /// `pending_visibility_edit` with the POST-toggle store value.
    /// Sequence: snapshot says visible=true → dispatch flipped
    /// Checkbox to Unchecked → apply_event reads Unchecked → publish
    /// `visible: false`.
    #[test]
    fn visibility_toggle_publishes_pending_with_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        // Selection that has a Transform component (we don't paint
        // here, just exercise apply_event semantics).
        hero.inspector_visibility = Some(InspectorVisibilityInfo {
            entity_bits: 0xBABE_BEEF,
            visible: true,
        });
        // Simulate the dispatch having toggled Checked → Unchecked.
        if let Some(InteractiveState::Checkbox { value, .. }) =
            hero.store.get_mut(ids::INSP_VISIBILITY_CHECK)
        {
            *value = crate::widget::CheckboxValue::Unchecked;
        }
        assert!(hero.apply_event(WidgetEvent::Toggled(ids::INSP_VISIBILITY_CHECK)));
        let pending = hero.pending_visibility_edit.expect("pending populated");
        assert_eq!(pending.entity_bits, 0xBABE_BEEF);
        assert!(!pending.visible, "toggle should commit visible=false");
    }

    /// M14.C: Click on a Strategy button different from the current
    /// `source_kind` publishes `pending_sprite_source_change` with
    /// the requested kind. Same-kind click is consumed silently.
    #[test]
    fn strategy_click_raises_pending_when_kind_differs() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_sprite = Some(InspectorSpriteInfo {
            entity_bits: 0xC0FF_EE00,
            name: "Player".into(),
            world_size: [1.0, 1.0],
            source_kind: InspectorSpriteSource::Atlas { key: 7 },
            source_pixels: Some((256, 256)),
            can_reimport: true,
        });
        // Current = Atlas → click on Individual button publishes.
        assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_INDIVIDUAL)));
        assert_eq!(
            hero.pending_sprite_source_change,
            Some((0xC0FF_EE00, RequestedSpriteStrategy::Individual))
        );

        // Click on Atlas (already-current) is consumed but no pending.
        hero.pending_sprite_source_change = None;
        assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_ATLAS)));
        assert_eq!(hero.pending_sprite_source_change, None);

        // HandPacked → publishes too (shell decides to skip with toast).
        assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_HANDPACKED)));
        assert_eq!(
            hero.pending_sprite_source_change,
            Some((0xC0FF_EE00, RequestedSpriteStrategy::HandPacked))
        );
    }

    /// M14.C: Without `inspector_sprite` (nothing selected), Strategy
    /// clicks are no-ops — apply_event returns false so the dispatcher
    /// keeps walking and pending stays `None`.
    #[test]
    fn strategy_click_no_pending_without_sprite_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_sprite = None;
        assert!(!hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_INDIVIDUAL)));
        assert_eq!(hero.pending_sprite_source_change, None);
    }

    /// M14.E: `TextChanged` on the editable entity-name field
    /// publishes the current store text via `pending_name_edit`. The
    /// `Option` coalesces multi-keystroke spans — only the latest
    /// value survives until the shell drains.
    #[test]
    fn name_text_changed_publishes_pending_with_current_text() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_name = Some(InspectorNameInfo {
            entity_bits: 0xDEAD_BEEF,
            name: "Old".to_string(),
        });
        // Simulate the dispatch having mutated the TextInput buffer
        // to "Player" via a sequence of keystrokes.
        if let Some(InteractiveState::TextInput { text, caret, .. }) =
            hero.store.get_mut(ids::INSP_ENTITY_NAME)
        {
            text.clear();
            text.push_str("Player");
            *caret = text.len();
        }
        assert!(hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
        let pending = hero
            .pending_name_edit
            .as_ref()
            .expect("pending populated after TextChanged");
        assert_eq!(pending.entity_bits, 0xDEAD_BEEF);
        assert_eq!(pending.name, "Player");
    }

    /// M14.E: without an `inspector_name` snapshot (no selection),
    /// `TextChanged` is a no-op — apply_event returns false so the
    /// dispatcher keeps walking.
    #[test]
    fn name_text_changed_no_pending_without_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_name = None;
        assert!(!hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
        assert_eq!(hero.pending_name_edit, None);
    }

    /// M14.E: TextChanged on the entity-name field with a selection
    /// publishes `pending_name_edit` with the current store buffer.
    /// Without a selection, returns false and pending stays None.
    #[test]
    fn entity_name_text_changed_raises_pending_with_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        // Seed the TextInput buffer with what the user just typed.
        if let Some(InteractiveState::TextInput { text, caret, .. }) =
            hero.store.get_mut(ids::INSP_ENTITY_NAME)
        {
            *text = "Player Two".to_string();
            *caret = text.len();
        }
        hero.inspector_name = Some(InspectorNameInfo {
            entity_bits: 0xDEAD_F00D,
            name: "Player".into(),
        });
        assert!(hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
        let p = hero.pending_name_edit.as_ref().expect("pending populated");
        assert_eq!(p.entity_bits, 0xDEAD_F00D);
        assert_eq!(p.name, "Player Two");

        // No selection → no pending.
        hero.inspector_name = None;
        hero.pending_name_edit = None;
        assert!(!hero.apply_event(WidgetEvent::TextChanged(ids::INSP_ENTITY_NAME)));
        assert_eq!(hero.pending_name_edit, None);
    }

    /// Audit #2 fix (MEDIUM): `paint_hero_screen` selection-change
    /// block resets the entity-name TextInput state to `Normal` (not
    /// just `text`/`caret`/`selection_anchor`). Otherwise the
    /// painter keeps drawing the focused chrome (caret + focus ring)
    /// on a field the user hasn't authored yet — same canonical
    /// cleanup dispatch.rs:1189 does on Blur.
    #[test]
    fn selection_switch_resets_entity_name_input_state_to_normal() {
        let mut hero = HeroScreen::new(NodeId(1));
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        // 1) Frame 1: select entity A, mark its TextInput Focused
        //    (simulating user click on the field).
        hero.inspector_name = Some(InspectorNameInfo {
            entity_bits: 0xAAAA_0001,
            name: "Player A".into(),
        });
        hero.inspector_transform = Some(InspectorTransformInfo {
            entity_bits: 0xAAAA_0001,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
        });
        paint_hero_screen(&mut hero, layout.viewport, &mut scene, &mut text);
        if let Some(InteractiveState::TextInput { state, .. }) =
            hero.store.get_mut(ids::INSP_ENTITY_NAME)
        {
            *state = crate::widget::TextInputState::Focused;
        }
        // 2) Frame 2: switch to entity B. The selection-change block
        //    must flip state back to Normal regardless of the focus
        //    snapshot the user left on entity A.
        hero.inspector_name = Some(InspectorNameInfo {
            entity_bits: 0xBBBB_0002,
            name: "Player B".into(),
        });
        hero.inspector_transform = Some(InspectorTransformInfo {
            entity_bits: 0xBBBB_0002,
            translation: [0.0, 0.0],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
        });
        paint_hero_screen(&mut hero, layout.viewport, &mut scene, &mut text);
        match hero.store.get(ids::INSP_ENTITY_NAME) {
            Some(InteractiveState::TextInput { state, text, .. }) => {
                assert_eq!(
                    *state,
                    crate::widget::TextInputState::Normal,
                    "state must reset to Normal on selection switch"
                );
                assert_eq!(text, "Player B", "buffer must reset to new entity's name");
            }
            _ => panic!("INSP_ENTITY_NAME state missing"),
        }
    }

    /// Audit fix #7 (HIGH): clicking a strategy button resets the
    /// stored ButtonState to Normal so the painter's snapshot-driven
    /// `Pressed` pin is the single visual source of truth.
    #[test]
    fn strategy_click_resets_button_state_to_normal() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_sprite = Some(InspectorSpriteInfo {
            entity_bits: 0x00C0_FFEE,
            name: "S".into(),
            world_size: [1.0, 1.0],
            source_kind: InspectorSpriteSource::Individual { texture_id: 1 },
            source_pixels: Some((64, 64)),
            can_reimport: true,
        });
        // Simulate dispatch having set Pressed on the click target.
        if let Some(InteractiveState::Button { state }) =
            hero.store.get_mut(ids::INSP_RENDER_STRATEGY_ATLAS)
        {
            *state = crate::widget::ButtonState::Pressed;
        }
        assert!(hero.apply_event(WidgetEvent::Click(ids::INSP_RENDER_STRATEGY_ATLAS)));
        // After apply_event: pending raised AND button state forced
        // back to Normal so the painter's pin re-runs cleanly.
        assert!(matches!(
            hero.store.button_state(ids::INSP_RENDER_STRATEGY_ATLAS),
            Some(crate::widget::ButtonState::Normal),
        ));
    }

    /// M14.D: Toggled without an `inspector_visibility` snapshot
    /// (e.g. nothing selected) is a no-op — apply_event returns
    /// false so the dispatcher keeps walking and `pending` stays
    /// `None`.
    #[test]
    fn visibility_toggle_no_pending_without_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.inspector_visibility = None;
        assert!(!hero.apply_event(WidgetEvent::Toggled(ids::INSP_VISIBILITY_CHECK)));
        assert_eq!(hero.pending_visibility_edit, None);
    }

    /// Clicking the Trim Transparency action pill captures the
    /// current `gizmo_selection` into `pending_trim_transparency`
    /// so the host can drain it next frame. When nothing is
    /// selected, the pending stays `None` (click still consumed so
    /// the dispatcher doesn't keep walking).
    #[test]
    fn click_on_trim_pill_raises_pending_with_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        // No selection → nothing pending after click.
        hero.gizmo_selection = None;
        assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
        assert_eq!(hero.pending_trim_transparency, None);

        // With selection → pending mirrors gizmo_selection.
        hero.gizmo_selection = Some(0xDEAD_BEEF);
        assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_TRIM)));
        assert_eq!(hero.pending_trim_transparency, Some(0xDEAD_BEEF));
    }

    /// Make Square pill mirrors the Trim pending-slot semantics.
    #[test]
    fn click_on_make_square_pill_raises_pending_with_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.gizmo_selection = None;
        assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_MAKE_SQUARE)));
        assert_eq!(hero.pending_make_square, None);

        hero.gizmo_selection = Some(0xCAFE_BABE);
        assert!(hero.apply_event(WidgetEvent::Click(ids::IMAGE_ACTION_MAKE_SQUARE)));
        assert_eq!(hero.pending_make_square, Some(0xCAFE_BABE));
    }

    #[test]
    fn paint_left_rail_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(32);
        paint_left_rail(
            &layout,
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hits,
            &store,
        );
    }

    #[test]
    fn paint_inspector_smoke_with_selection() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let sel = fixture::default_selection();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(32);
        paint_inspector(
            &layout,
            Some(&sel),
            &mut scene,
            &mut text,
            Theme::Sunstone,
            &mut hits,
            &store,
        );
    }

    #[test]
    fn paint_inspector_smoke_no_selection() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let store = WidgetStore::with_capacity(32);
        paint_inspector(
            &layout,
            None,
            &mut scene,
            &mut text,
            Theme::Blueprint,
            &mut hits,
            &store,
        );
    }

    #[test]
    fn paint_hierarchy_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        let mut hits = HitIndex::new();
        let mut store = WidgetStore::with_capacity(32);
        paint_hierarchy(
            &layout,
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hits,
            &mut store,
        );
    }

    #[test]
    fn paint_bottom_hud_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_bottom_hud(
            &layout,
            &mut scene,
            &mut text,
            Theme::Workshop,
            BottomHudStats::default(),
        );
    }

    #[test]
    fn paint_selection_overlay_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let sel = fixture::default_selection();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_selection_overlay(&layout, &sel, &mut scene, &mut text, Theme::Forge);
    }

    // ─────────────── M14.6 F: per-row context-menu apply_event ────────────────

    /// Stage a closed HierarchyRow snapshot so `apply_event` can read
    /// it via `consume_last_context_menu`. Mirrors what dispatch does
    /// on the menu-closing Down → next-frame-Click sequence.
    fn stage_hierarchy_row_snapshot(hero: &mut HeroScreen, row: NodeId) {
        hero.store
            .open_context_menu(crate::interaction::ContextMenuRequest {
                x: 0.0,
                y: 0.0,
                kind: crate::interaction::ContextMenuKind::HierarchyRow { row },
            });
        // Closing copies the request into `last_context_menu`, which
        // is what `consume_last_context_menu` returns.
        hero.store.close_context_menu();
    }

    #[test]
    fn hier_menu_duplicate_sets_pending_duplicate() {
        let mut hero = HeroScreen::new(NodeId(1));
        let row = NodeId(100_500);
        stage_hierarchy_row_snapshot(&mut hero, row);
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
        assert!(consumed);
        assert_eq!(hero.pending_duplicate, Some(row));
        // Snapshot was consumed.
        assert!(hero.store.last_context_menu().is_none());
    }

    #[test]
    fn hier_menu_add_child_sets_pending_add_child() {
        let mut hero = HeroScreen::new(NodeId(1));
        let row = NodeId(100_501);
        stage_hierarchy_row_snapshot(&mut hero, row);
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_ADD_CHILD));
        assert!(consumed);
        assert_eq!(hero.pending_add_child, Some(row));
    }

    #[test]
    fn hier_menu_reset_transform_sets_pending() {
        let mut hero = HeroScreen::new(NodeId(1));
        let row = NodeId(100_502);
        stage_hierarchy_row_snapshot(&mut hero, row);
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_RESET_TRANSFORM));
        assert!(consumed);
        assert_eq!(hero.pending_reset_transform, Some(row));
    }

    #[test]
    fn hier_menu_delete_sets_pending_delete() {
        let mut hero = HeroScreen::new(NodeId(1));
        let row = NodeId(100_503);
        stage_hierarchy_row_snapshot(&mut hero, row);
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE));
        assert!(consumed);
        assert_eq!(hero.pending_delete, Some(row));
    }

    #[test]
    fn hier_menu_click_without_snapshot_consumes_but_no_pending() {
        // Defensive case: stray Click without any prior right-click
        // snapshot still consumes the event so the click doesn't
        // bubble to row selection, but no pending action is raised.
        let mut hero = HeroScreen::new(NodeId(1));
        let consumed = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
        assert!(consumed);
        assert!(hero.pending_duplicate.is_none());
    }

    #[test]
    fn hierarchy_row_click_raises_pending_for_live_entries() {
        // Build a live-mode hierarchy with one entry, then click the
        // matching NodeId. `pending_hierarchy_row_click` should fire
        // so the host can sync `gizmo_selection`.
        let mut hero = HeroScreen::new(NodeId(1));
        let row_id = NodeId(100_500);
        let mut entries = std::collections::BTreeMap::new();
        entries.insert(
            row_id,
            fixture::HierarchyEntity {
                name: "hero_001".into(),
                icon: crate::icons::IconId::Sprite,
                indent: 0,
                badge: None,
                swatch: None,
                visible: true,
                selected: false,
                muted: false,
            },
        );
        hero.sync_from_hierarchy(&[row_id], entries);
        let consumed = hero.apply_event(WidgetEvent::Click(row_id));
        assert!(consumed, "live-mode row click should consume");
        assert_eq!(hero.pending_hierarchy_row_click, Some(row_id));
    }

    #[test]
    fn hierarchy_row_click_silent_for_fixture_only_rows() {
        // Fixture-mode click (no `sync_from_hierarchy`) shouldn't
        // raise `pending_hierarchy_row_click` — the M14.6 D path is
        // live-only.
        let mut hero = HeroScreen::new(NodeId(1));
        let _ = hero.apply_event(WidgetEvent::Click(ids::HIER_PLAYER));
        assert_eq!(hero.pending_hierarchy_row_click, None);
    }

    #[test]
    fn hier_menu_one_action_per_drain() {
        // Two consecutive clicks (Duplicate then Delete) only fire
        // the first — the snapshot is consumed and the second click
        // sees an empty `last_context_menu`. This protects against
        // double-trigger if a synthetic event stream emits both.
        let mut hero = HeroScreen::new(NodeId(1));
        let row = NodeId(100_504);
        stage_hierarchy_row_snapshot(&mut hero, row);
        let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DUPLICATE));
        let _ = hero.apply_event(WidgetEvent::Click(ids::CTX_MENU_HIER_DELETE));
        assert_eq!(hero.pending_duplicate, Some(row));
        assert!(hero.pending_delete.is_none());
    }
}
