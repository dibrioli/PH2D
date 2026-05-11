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

pub use bottom_hud::paint_bottom_hud;
pub use canvas::paint_canvas_bg;
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

        let left_rail = Rect::new(viewport.x + EDGE_PAD, chrome_top, RAIL_W, chrome_h);
        // Default panel sides (mirrored=false):
        //   - Hierarchy LEFT (just past the rail)
        //   - Inspector RIGHT (pinned to viewport edge)
        // Mirrored flips both.
        let (hierarchy_x, inspector_x) = if mirrored {
            (
                viewport.x + viewport.w - EDGE_PAD - HIERARCHY_W,
                viewport.x + EDGE_PAD + RAIL_W + EDGE_PAD,
            )
        } else {
            (
                viewport.x + EDGE_PAD + RAIL_W + EDGE_PAD,
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
        let canvas_x = left_panel_right + EDGE_PAD;
        let canvas_w = (right_panel_left - canvas_x - EDGE_PAD).max(0.0);
        // Canvas extends UPWARD to the viewport top so the topbar's
        // bg reads as transparent — clicks in the gap between pills
        // resolve to canvas (no widget hit), and the canvas tint
        // (`Bg1`) is visible behind the floating chip clusters.
        let canvas_y = viewport.y;
        let canvas_h = chrome_bot - canvas_y;
        let canvas = Rect::new(canvas_x, canvas_y, canvas_w, canvas_h.max(0.0));

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
}

impl HeroScreen {
    pub fn new(id: NodeId) -> Self {
        let mut store = WidgetStore::with_capacity(64);
        Self::pre_populate_store(&mut store);
        Self {
            id,
            theme: Theme::ForgeSdf,
            selection: Some(fixture::default_selection()),
            store,
            hit_index: HitIndex::new(),
            ui_mirrored: false,
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
        // Theme + radius selector from the TopBar theme menu —
        // intercepted at the Hero level because `self.theme` lives
        // here, not on the WidgetStore.
        if let WidgetEvent::Click(id) = event {
            let new_theme = if id == ids::CTX_MENU_THEME_FORGE {
                Some(Theme::ForgeSdf)
            } else if id == ids::CTX_MENU_THEME_PAINT {
                Some(Theme::PaintStudio)
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
            // Save / Save As — placeholders until the pilot project
            // wires the real save pipeline. Close the menu and
            // return consumed so the click doesn't propagate.
            if id == ids::CTX_MENU_SAVE || id == ids::CTX_MENU_SAVE_AS {
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
        if hierarchy::apply_event(&mut self.store, &mut self.selection, event) {
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

    paint_canvas_bg(&layout, scene, hero.theme);
    if let Some(sel) = hero.selection.as_ref() {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
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
    hero.store.set_panel_rect(ids::INSP_PANEL, layout.inspector);
    hero.store.set_panel_rect(ids::HIER_PANEL, layout.hierarchy);
    paint_inspector(
        &layout,
        hero.selection.as_ref(),
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    // Publish the inspector's measured content height to the store
    // so `dispatch_wheel` can clamp the next scroll event at the
    // upper bound BEFORE the visible jump happens. Also clamp the
    // current scroll value in case the previous paint left it
    // overshooting (e.g. after collapsing a section).
    {
        let content_h = inspector::last_inspector_content_h();
        let visible_h = inspector::last_inspector_visible_h();
        hero.store.set_panel_content_h(ids::INSP_PANEL, content_h);
        // Publish visible_h too so dispatch_wheel can clamp on the
        // exact viewport instead of an approximate panel.h - 60.
        hero.store.set_panel_visible_h(ids::INSP_PANEL, visible_h);
        let max_scroll = (content_h - visible_h).max(0.0);
        let cur = hero.store.panel_scroll(ids::INSP_PANEL);
        if cur > max_scroll {
            hero.store.set_panel_scroll(ids::INSP_PANEL, max_scroll);
        }
    }
    // Mirror the global picker's current value into the target
    // widget's `widget_colors` slot each frame so the section's
    // color circle (and any other color-target painter) tracks
    // live edits.
    if let Some(target) = hero.store.picker_target()
        && let Some((value, _, _, _)) = hero.store.blender_picker(ids::INSP_BLENDER_PICKER)
    {
        hero.store.set_widget_color(target, value.rgba);
    }
    hierarchy::set_selection_label(hero.selection.as_ref().map(|s| s.label.clone()));
    paint_hierarchy(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    // Publish hierarchy content_h + clamp scroll (same pattern as
    // inspector). Headroom of 60 px covers the hierarchy header.
    {
        let content_h = hierarchy::last_hierarchy_content_h();
        hero.store.set_panel_content_h(ids::HIER_PANEL, content_h);
        let visible_h = (layout.hierarchy.h - 60.0).max(0.0);
        let max_scroll = (content_h - visible_h).max(0.0);
        let cur = hero.store.panel_scroll(ids::HIER_PANEL);
        if cur > max_scroll {
            hero.store.set_panel_scroll(ids::HIER_PANEL, max_scroll);
        }
    }
    paint_bottom_hud(&layout, scene, text_system, hero.theme);
    // Floating BlenderColorPicker on top of the canvas. Pure
    // function of `(layout, store)` — drag offset comes from the
    // store. The Inspector keeps the picker's state under
    // `INSP_BLENDER_PICKER` even though the picker is painted out
    // here, not inside the Inspector chrome.
    //
    // Publish the picker's outer rect to the store so the dispatch's
    // outside-click-closes logic can test against the FULL panel
    // (not just its sub-control hit zones). Without this, clicking
    // dead space INSIDE the picker (gaps between controls, padding
    // areas) resolved to no BlenderHit and the picker closed —
    // user's "se eu clicar dentro do painel mas fora de qualquer
    // controle, o picker fecha".
    if hero.store.picker_target().is_some()
        && let Some(picker_rect) = color_picker_demo::current_picker_rect(&layout, &hero.store)
    {
        hero.store
            .set_panel_rect(ids::INSP_BLENDER_PICKER, picker_rect);
    } else {
        hero.store.clear_panel_rect(ids::INSP_BLENDER_PICKER);
    }
    color_picker_demo::paint_blender_picker_demo(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
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
    fn layout_canvas_between_panels_default() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        // Default: hierarchy left, inspector right.
        assert!(layout.canvas.x > layout.hierarchy.x + layout.hierarchy.w);
        assert!(layout.canvas.x + layout.canvas.w < layout.inspector.x);
    }

    #[test]
    fn layout_mirror_swaps_sides() {
        let layout = HeroLayout::for_viewport_mirrored(ipad12_viewport(), true);
        // Mirrored: inspector after rail (left), hierarchy pinned right.
        assert!(layout.inspector.x > layout.left_rail.x + layout.left_rail.w);
        let hier_right = layout.hierarchy.x + layout.hierarchy.w;
        assert!((hier_right - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
        assert!(layout.canvas.x > layout.inspector.x + layout.inspector.w);
        assert!(layout.canvas.x + layout.canvas.w < layout.hierarchy.x);
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
            Theme::ForgeSdf,
            Theme::PaintStudio,
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
            Theme::ForgeSdf,
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
            Theme::ForgeSdf,
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
        let store = WidgetStore::with_capacity(32);
        paint_hierarchy(
            &layout,
            &mut scene,
            &mut text,
            Theme::ForgeSdf,
            &mut hits,
            &store,
        );
    }

    #[test]
    fn paint_bottom_hud_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_bottom_hud(&layout, &mut scene, &mut text, Theme::PaintStudio);
    }

    #[test]
    fn paint_selection_overlay_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let sel = fixture::default_selection();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_selection_overlay(&layout, &sel, &mut scene, &mut text, Theme::ForgeSdf);
    }
}
