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
pub mod fixture;
pub mod hierarchy;
pub mod ids;
pub mod inspector;
pub mod left_rail;
pub mod selection;
pub mod showcase;
pub mod style;
pub mod topbar;

pub use bottom_hud::paint_bottom_hud;
pub use canvas::paint_canvas_bg;
pub use hierarchy::paint_hierarchy;
pub use inspector::paint_inspector;
pub use left_rail::paint_left_rail;
pub use selection::paint_selection_overlay;
pub use showcase::paint_components_showcase;
pub use style::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
pub use topbar::paint_top_bar;

use crate::interaction::{HitIndex, WidgetEvent, WidgetStore, dispatch_pointer};
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
    pub fn for_viewport(viewport: Rect) -> Self {
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
        let inspector = Rect::new(
            viewport.x + EDGE_PAD + RAIL_W + EDGE_PAD,
            chrome_top,
            INSPECTOR_W,
            chrome_h.min(880.0),
        );
        let hierarchy = Rect::new(
            viewport.x + viewport.w - EDGE_PAD - HIERARCHY_W,
            chrome_top,
            HIERARCHY_W,
            chrome_h,
        );
        let canvas_x = inspector.x + inspector.w + EDGE_PAD;
        let canvas_w = (hierarchy.x - canvas_x - EDGE_PAD).max(0.0);
        let canvas = Rect::new(canvas_x, chrome_top, canvas_w, chrome_h);

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
        showcase::populate(store);
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

    /// Translate a [`WidgetEvent`] from the dispatcher into a
    /// hero-level state mutation. Walks each region's
    /// `apply_event` in z-order; first region that consumes the
    /// event wins. Returns true iff some region consumed it.
    pub fn apply_event(&mut self, event: WidgetEvent) -> bool {
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
        if showcase::apply_event(&mut self.store, event) {
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
    let layout = HeroLayout::for_viewport(viewport);
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
    paint_inspector(
        &layout,
        hero.selection.as_ref(),
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    hierarchy::set_selection_label(hero.selection.as_ref().map(|s| s.label.clone()));
    paint_hierarchy(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    paint_bottom_hud(&layout, scene, text_system, hero.theme);
    // Components Showcase disabled while debugging interaction pipeline.
    // BlenderColorPicker preserved as standalone demo (anchored bottom-
    // right of canvas). Re-enable showcase once everything else works.
    showcase::paint_blender_picker_demo(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    // Tooltip overlay on top of all chrome (Phase 3 polish).
    topbar::paint_hover_tooltip(scene, text_system, hero.theme, &hero.hit_index, &hero.store);
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
    fn layout_inspector_after_rail() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!(layout.inspector.x > layout.left_rail.x + layout.left_rail.w);
        assert!((layout.inspector.w - style::INSPECTOR_W).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_hierarchy_pinned_right() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let right_edge = layout.hierarchy.x + layout.hierarchy.w;
        assert!((right_edge - (HERO_VIEWPORT_W - style::EDGE_PAD)).abs() < 0.01);
    }

    #[test]
    fn layout_canvas_between_inspector_and_hierarchy() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
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
    fn hero_topbar_save_click_round_trip() {
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
        let evts = hero.handle_pointer(up(save_x, save_y), &arena);
        assert!(
            evts.iter()
                .any(|e| matches!(e, WidgetEvent::Click(id) if *id == ids::TOPBAR_SAVE)),
            "expected Click event for TOPBAR_SAVE, got {evts:?}"
        );
    }

    #[test]
    fn hero_apply_event_hierarchy_click_changes_selection() {
        let mut hero = HeroScreen::new(NodeId(1));
        let consumed = hero.apply_event(WidgetEvent::Click(ids::HIER_SLIME_01));
        assert!(consumed);
        assert_eq!(
            hero.selection.as_ref().map(|s| s.label.as_str()),
            Some("Slime_01")
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
