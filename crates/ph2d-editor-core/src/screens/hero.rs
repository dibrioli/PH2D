//! Editor hero — composes the `02-editor-main` mockup
//! ([`docs/design/screens/02-editor-main.html`]) into a single
//! `paint_hero_screen` call. Layout regions (viewport-relative px):
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
//! in [`style`]; stable `NodeId`s in [`ids`]. Hardcoded mockup content
//! stays in [`fixture`] until a pilot project picks the entity model.

pub mod bottom_hud;
pub mod canvas;
pub mod chrome;
pub mod color_picker_demo;
mod context_menu_dialogs;
pub mod context_menu_overlay;
pub mod fixture;
// Wave 6+7 Phase 2: hero ids promoted to ph2d-editor-core so dispatch
// and panel crates can reach them without depending back on hero. The
// `screens::hero::ids` path continues to resolve via this re-export.
pub use crate::ids;
pub mod left_rail;
pub mod pre_populate;
pub mod pre_populate_blender;
pub mod selection;
pub mod state;
pub mod style;
pub mod topbar;

mod inspector_model;
mod paint;

pub use inspector_model::*;
pub use paint::*;

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
    /// Pending Painter Falloff handle from the right-click menu (`HandleType` wire u8 `0`=Auto/`1`=Vector);
    /// `.take()`n by the shell onto the selected point.
    pub pending_falloff_point_handle: Option<u8>,
    /// Pending on-canvas Curve / Free Hand point handle kind from the right-click menu (wire u8 `0`=Free /
    /// `1`=Aligned / `2`=Vector / `3`=Auto); `.take()`n by the shell onto `set_curve_handle_kind`.
    pub pending_curve_point_handle: Option<u8>,
    // Wave 2.5 PR 11.8c: 6 hierarchy fields (visibility/reparent/duplicate/delete/reset-transform/add-child)
    // migrated to the bus as `EditorAction::Hier*`.
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
    // `InspectorVisibilityInfo` / `InspectorNameInfo` are defined in the
    // sibling `inspector_model` submodule and re-exported by `screens::hero`
    // (`pub use inspector_model::*`) for the crate-wide import surface, so
    // `state.rs` keeps re-importing them from `screens::hero` unchanged.
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
            text_rendering: ph2d_tokens::TextRendering::CrispHeavyPlus, // app default (Enio 2026-06-24)
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
            pending_falloff_point_handle: None,
            pending_curve_point_handle: None,
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
