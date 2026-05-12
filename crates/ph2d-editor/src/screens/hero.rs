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
pub mod left_rail;
pub mod selection;
pub mod style;
pub mod topbar;

pub use bottom_hud::{BottomHudStats, paint_bottom_hud};
pub use canvas::{paint_canvas_bg, paint_drop_overlay};
pub use color_picker_demo::paint_blender_picker_demo;
pub use hierarchy::paint_hierarchy;
pub use inspector::paint_inspector;
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
            live_hierarchy_entries: None,
            grid_visible: true,
            grid_view: None,
            grid_config: crate::grid::GridConfig::default(),
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
        } = event
        {
            self.pending_reparent = Some(HierReparentIntent {
                dragged,
                new_parent,
                before,
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
                // M14.4b.bis: 4-mode cycle (Selected → Camera → All
                // → Zero). When landing on Zero (mode 3), raise
                // `camera_reset_pending` so the host resets its
                // `Camera2d`. Other modes are placeholders for
                // future frame-selection / frame-all actions.
                let next = (self.store.tool_view_mode() + 1) % 4;
                self.store.set_tool_view_mode(next);
                if next == 3 {
                    self.camera_reset_pending = true;
                }
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
                    }
                }
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
        if hierarchy::apply_event(
            &mut self.store,
            &mut self.selection,
            self.live_hierarchy_entries.as_ref(),
            event,
        ) {
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
        crate::grid::paint_grid(scene, hero.theme, &view, &hero.grid_config);
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
            paint_inspector(
                &layout,
                hero.selection.as_ref(),
                scene,
                text_system,
                hero.theme,
                &mut hero.hit_index,
                &hero.store,
            );
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
        );
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
