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
//! Phase 1 ships the scaffold + canvas BG only. Phases 2-4 layer
//! TopBar, LeftRail, Inspector, Hierarchy, BottomHUD, and selection
//! overlay on top. Fixture content (Player / Slime_01 / etc.) is
//! hardcoded — wired-to-real-ECS lands when a pilot project picks
//! the entity model.

use crate::icons::IconId;
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore, dispatch_pointer};
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use crate::widget::{
    ButtonState, PILL_PADDING_PX, SectionHeader, SegmentTone, Slider, SliderOrientation,
    SliderState, StatusBar, StatusSegment, ToolRail, ToolRailEntry, paint_section_header,
    paint_slider, paint_status_bar, paint_tool_rail,
};
use crate::zones::Rect;
use bumpalo::Bump;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_host::{KeyEvent, PointerEvent};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::Stroke;
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

pub mod fixture;

/// Default mockup viewport (iPad 12.9 landscape).
pub const HERO_VIEWPORT_W: f32 = 1366.0;
pub const HERO_VIEWPORT_H: f32 = 1024.0;

/// Padding from the screen edge to chrome.
const EDGE_PAD: f32 = 14.0;
const TOPBAR_H: f32 = 40.0;
const TOPBAR_GAP: f32 = 16.0; // pixels between TopBar bottom and chrome below
const RAIL_W: f32 = 56.0;
const INSPECTOR_W: f32 = 304.0;
const HIERARCHY_W: f32 = 308.0;
const HUD_H: f32 = 34.0;
const HUD_BOTTOM_PAD: f32 = 18.0;

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

/// Stable `NodeId`s for the hero's interactive widgets. Pre-populated
/// in [`WidgetStore`] at construction time so the dispatcher always
/// finds an entry on hit-test.
pub mod ids {
    use ph2d_a11y::NodeId;

    pub const TOPBAR_THEME: NodeId = NodeId(101);
    pub const TOPBAR_SAVE: NodeId = NodeId(102);
    pub const TOPBAR_PROJECT: NodeId = NodeId(103);
    pub const TOPBAR_PLAY_TOGGLE: NodeId = NodeId(104);
    pub const TOPBAR_PLAY_BUTTON: NodeId = NodeId(105);
    pub const TOPBAR_RIGHT_LAYERS: NodeId = NodeId(106);
    pub const TOPBAR_RIGHT_ASSETS: NodeId = NodeId(107);
    pub const TOPBAR_RIGHT_SCRIPT: NodeId = NodeId(108);

    pub const HIERARCHY_ADD: NodeId = NodeId(150);

    // ToolRail entries already use 200-209 (assigned in
    // `paint_left_rail` via fixture).
    pub const TOOL_TRANSLATE: NodeId = NodeId(201);
    pub const TOOL_ROTATE: NodeId = NodeId(202);
    pub const TOOL_SCALE: NodeId = NodeId(203);
    pub const TOOL_PIVOT: NodeId = NodeId(204);
    pub const TOOL_SPACE: NodeId = NodeId(205);
    pub const TOOL_PROJECTION: NodeId = NodeId(206);
    pub const TOOL_HOME: NodeId = NodeId(207);
    pub const TOOL_UNDO: NodeId = NodeId(208);
    pub const TOOL_REDO: NodeId = NodeId(209);

    // Inspector field ids (300-360 reserved).
    pub const INSP_MOVE_SPEED: NodeId = NodeId(300);
    pub const INSP_JUMP_HEIGHT: NodeId = NodeId(301);
    pub const INSP_FRICTION: NodeId = NodeId(302);
    pub const INSP_DAMPING: NodeId = NodeId(303);
    pub const INSP_DEBUG_SELECT: NodeId = NodeId(310);
    pub const INSP_LINK_DISTANCE: NodeId = NodeId(320);
    pub const INSP_LINK_MATERIAL: NodeId = NodeId(321);
    pub const INSP_CAM_YAW: NodeId = NodeId(330);
    pub const INSP_CAM_PITCH: NodeId = NodeId(331);
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

    fn pre_populate_store(store: &mut WidgetStore) {
        // TopBar single-icon buttons.
        for id in [
            ids::TOPBAR_THEME,
            ids::TOPBAR_SAVE,
            ids::TOPBAR_PROJECT,
            ids::TOPBAR_PLAY_TOGGLE,
            ids::TOPBAR_PLAY_BUTTON,
            ids::TOPBAR_RIGHT_LAYERS,
            ids::TOPBAR_RIGHT_ASSETS,
            ids::TOPBAR_RIGHT_SCRIPT,
            ids::HIERARCHY_ADD,
            ids::TOOL_TRANSLATE,
            ids::TOOL_ROTATE,
            ids::TOOL_SCALE,
            ids::TOOL_PIVOT,
            ids::TOOL_SPACE,
            ids::TOOL_PROJECTION,
            ids::TOOL_HOME,
            ids::TOOL_UNDO,
            ids::TOOL_REDO,
        ] {
            store.register(
                id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
        // The active tool starts highlighted (matches the mockup).
        if let Some(InteractiveState::Button { state }) = store.get_mut(ids::TOOL_TRANSLATE) {
            *state = ButtonState::Pressed;
        }

        // Inspector sliders — initial values mirror the mockup's
        // `display` strings (160 / 200 / 0.0010 / 0.70 / 0.57).
        for (id, value) in [
            (ids::INSP_MOVE_SPEED, 0.62),
            (ids::INSP_JUMP_HEIGHT, 0.30),
            (ids::INSP_FRICTION, 0.08),
            (ids::INSP_DAMPING, 0.48),
            (ids::INSP_CAM_YAW, 0.57),
            (ids::INSP_CAM_PITCH, 0.0),
        ] {
            store.register(
                id,
                InteractiveState::Slider {
                    state: SliderState::Normal,
                    value,
                    orientation: SliderOrientation::Horizontal,
                },
            );
        }

        // Inspector "Debug" select — Dropdown that toggles open/closed
        // on click (Phase C wiring).
        store.register(
            ids::INSP_DEBUG_SELECT,
            InteractiveState::Dropdown {
                state: crate::widget::DropdownState::Normal,
                open: false,
                selected_index: Some(0),
            },
        );
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn selection(mut self, sel: Option<HeroSelection>) -> Self {
        self.selection = sel;
        self
    }

    /// Forward a pointer event into the interaction store. Returns
    /// the events emitted in the caller's frame-local arena. Caller
    /// drains synchronously and resets the arena at end-of-frame.
    pub fn handle_pointer<'frame>(
        &mut self,
        event: PointerEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        dispatch_pointer(&mut self.store, &self.hit_index, event, arena)
    }

    /// Forward a key event into the interaction store. Same arena
    /// contract as [`Self::handle_pointer`].
    pub fn handle_key<'frame>(
        &mut self,
        event: KeyEvent,
        arena: &'frame Bump,
    ) -> &'frame [WidgetEvent] {
        crate::interaction::dispatch_key(&mut self.store, event, arena)
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

/// Paint a solid Bg0 fill across the canvas region. We deliberately
/// skip the radial gradient + perspective grid from the HTML mockup
/// in v1: real visual fidelity needs a screenshot harness to
/// validate (M14+). A solid fill keeps the chrome readable in any
/// theme without preempting the design.
pub fn paint_canvas_bg(layout: &HeroLayout, scene: &mut VectorScene, theme: Theme) {
    // Full viewport background (so areas outside any chrome group
    // stay token-correct).
    scene.fill_rect(
        rect_to_vello(layout.viewport),
        resolve(ColorToken::Bg0, theme),
    );
    // Canvas region tinted slightly different so the chrome reads as
    // floating above content, not flush with it.
    scene.fill_rect(
        rect_to_vello(layout.canvas),
        resolve(ColorToken::Bg1, theme),
    );
}

/// Paint the TopBar: 5 pill clusters from `fixture::topbar_clusters`
/// and a centered `PH2D · EDITOR` wordmark. Registers each cluster's
/// hit rect into [`HitIndex`] for pointer dispatch.
pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    let split = 4.min(clusters.len());
    for (id, cluster) in &clusters[..split] {
        let rect = Rect::new(x, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
        x = rect.x + rect.w + gap;
    }
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for (_, c) in right_clusters {
        right_w += cluster_width(c) + gap;
    }
    let right_x = layout.top_bar.x + layout.top_bar.w - right_w + gap.max(0.0);
    let wordmark_rect = Rect::new(x, layout.top_bar.y, (right_x - x).max(0.0), row_h);
    paint_text_centered(
        text_system,
        scene,
        "PH2D \u{00b7} EDITOR",
        wordmark_rect,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text3, theme),
    );
    let mut rx = right_x;
    for (id, cluster) in right_clusters {
        let rect = Rect::new(rx, layout.top_bar.y, cluster_width(cluster), row_h);
        paint_top_bar_cluster(
            *id,
            cluster,
            rect,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
        rx = rect.x + rect.w + gap;
    }
}

fn cluster_width(cluster: &fixture::TopBarCluster) -> f32 {
    use fixture::TopBarCluster;
    match cluster {
        TopBarCluster::Theme { .. } => 132.0,
        TopBarCluster::Single { .. } => 40.0 + PILL_PADDING_PX * 2.0,
        TopBarCluster::Project { .. } => 156.0,
        TopBarCluster::Play => 92.0,
        TopBarCluster::Right => 132.0,
    }
}

/// Pick a chrome icon's foreground tint based on its interactive
/// state. Used by TopBar single-icon clusters and the Right cluster.
fn icon_button_fg(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Disabled => ColorToken::TextDisabled,
        ButtonState::Normal | ButtonState::Loading => ColorToken::Text2,
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_top_bar_cluster(
    id: NodeId,
    cluster: &fixture::TopBarCluster,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    use fixture::TopBarCluster;
    let radius = Radius::Xl.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    let pad_x = Spacing::Md.px();
    let icon_w = 18.0;
    let font = TypeToken::Sm.px();
    match cluster {
        TopBarCluster::Theme { label } => {
            // Two color dots + label + chevron.
            let mut cx = rect.x + pad_x + 4.0;
            let cy = rect.y + rect.h * 0.5;
            for (i, token) in [ColorToken::Accent, ColorToken::AccentSoft]
                .iter()
                .enumerate()
            {
                let dot = Circle::new(Point::new(cx as f64, cy as f64), 4.0);
                scene.inner_mut().fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(resolve(*token, theme)),
                    None,
                    &dot,
                );
                cx += if i == 0 { 10.0 } else { 0.0 };
            }
            let label_x = rect.x + pad_x + 28.0;
            let label_y = rect.y + (rect.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                label,
                label_x,
                label_y,
                font,
                rect.w - (label_x - rect.x) - 24.0,
                resolve(ColorToken::Text1, theme),
            );
            let chev_rect = Rect::new(
                rect.x + rect.w - pad_x - icon_w,
                rect.y + (rect.h - icon_w) * 0.5,
                icon_w,
                icon_w,
            );
            paint_icon(
                scene,
                IconId::ChevronDown,
                chev_rect,
                resolve(ColorToken::Text3, theme),
                1.5,
            );
        }
        TopBarCluster::Single { icon, .. } => {
            hit_index.register(id, rect);
            let state = store.button_state(id).unwrap_or(ButtonState::Normal);
            let chip = Rect::new(
                rect.x + (rect.w - 32.0) * 0.5,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            paint_icon(
                scene,
                *icon,
                chip,
                resolve(icon_button_fg(state), theme),
                1.5,
            );
        }
        TopBarCluster::Project { name } => {
            let icon_rect = Rect::new(
                rect.x + pad_x,
                rect.y + (rect.h - icon_w) * 0.5,
                icon_w,
                icon_w,
            );
            paint_icon(
                scene,
                IconId::Folder,
                icon_rect,
                resolve(ColorToken::Text2, theme),
                1.5,
            );
            let name_x = icon_rect.x + icon_w + Spacing::Md.px();
            let name_y = rect.y + (rect.h - font) * 0.5;
            paint_text(
                text_system,
                scene,
                name,
                name_x,
                name_y,
                font,
                rect.w - (name_x - rect.x) - pad_x,
                resolve(ColorToken::Text1, theme),
            );
        }
        TopBarCluster::Play => {
            // Theme-mode toggle on the left.
            let toggle_rect = Rect::new(rect.x + pad_x, rect.y + (rect.h - 22.0) * 0.5, 22.0, 22.0);
            hit_index.register(ids::TOPBAR_PLAY_TOGGLE, toggle_rect);
            let toggle_state = store
                .button_state(ids::TOPBAR_PLAY_TOGGLE)
                .unwrap_or(ButtonState::Normal);
            paint_icon(
                scene,
                IconId::Light,
                toggle_rect,
                resolve(icon_button_fg(toggle_state), theme),
                1.5,
            );
            // Accent play button on the right — hit-test for the
            // canonical PLAY_BUTTON id (the cluster's own id parameter
            // doubles up to PLAY_BUTTON to keep the API uniform).
            let play_rect = Rect::new(
                rect.x + rect.w - pad_x - 32.0,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            hit_index.register(id, play_rect);
            let play_state = store.button_state(id).unwrap_or(ButtonState::Normal);
            let bg = match play_state {
                ButtonState::Pressed => ColorToken::AccentPress,
                ButtonState::Hovered => ColorToken::AccentSoft,
                _ => ColorToken::Danger,
            };
            fill_rounded_rect(scene, play_rect, Radius::Lg.px(), resolve(bg, theme));
            paint_icon(
                scene,
                IconId::Play,
                play_rect,
                resolve(ColorToken::AccentFg, theme),
                1.5,
            );
        }
        TopBarCluster::Right => {
            let icons = [
                (ids::TOPBAR_RIGHT_LAYERS, IconId::Layers),
                (ids::TOPBAR_RIGHT_ASSETS, IconId::Asset),
                (ids::TOPBAR_RIGHT_SCRIPT, IconId::Script),
            ];
            let chip = 32.0_f32;
            for (i, (icon_id, icon)) in icons.iter().enumerate() {
                let cx = rect.x + pad_x + (chip + 4.0) * i as f32;
                let chip_rect = Rect::new(cx, rect.y + (rect.h - chip) * 0.5, chip, chip);
                hit_index.register(*icon_id, chip_rect);
                let state = store.button_state(*icon_id).unwrap_or(ButtonState::Normal);
                paint_icon(
                    scene,
                    *icon,
                    chip_rect,
                    resolve(icon_button_fg(state), theme),
                    1.5,
                );
            }
        }
    }
}

/// Paint the LeftRail using the `ToolRail` widget with the
/// fixture's transform/space/history entries. The "active" tool is
/// whichever tool's [`InteractiveState::Button::state`] in the store
/// is `Pressed` (the others render as Normal/Hovered per their
/// per-widget state).
pub fn paint_left_rail(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let entries = [
        (ids::TOOL_TRANSLATE, "Translate", IconId::Transform),
        (ids::TOOL_ROTATE, "Rotate", IconId::Rotate),
        (ids::TOOL_SCALE, "Scale", IconId::Scale),
        (ids::TOOL_PIVOT, "Pivot", IconId::Pivot),
    ];
    let mut rail_entries: Vec<ToolRailEntry> = entries
        .iter()
        .map(|(id, label, icon)| {
            let mut e = ToolRailEntry::icon(*id, *label, *icon);
            if matches!(store.button_state(*id), Some(ButtonState::Pressed)) {
                e = e.active();
            }
            e
        })
        .collect();
    rail_entries.push(ToolRailEntry::Divider);
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_SPACE,
        "Coordinate space",
        "Global",
        "SPACE",
    ));
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_PROJECTION,
        "Camera projection",
        "Persp",
        "PROJ",
    ));
    rail_entries.push(ToolRailEntry::compound(
        ids::TOOL_HOME,
        "Frame to home",
        "Home",
        "VIEW",
    ));
    rail_entries.push(ToolRailEntry::Divider);
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_UNDO, "Undo", IconId::Undo));
    rail_entries.push(ToolRailEntry::icon(ids::TOOL_REDO, "Redo", IconId::Redo));

    let rail = ToolRail::new(NodeId(200), "Editor tools", rail_entries);
    let rail_rect = Rect::new(
        layout.left_rail.x,
        layout.left_rail.y,
        layout.left_rail.w,
        rail.preferred_height(),
    );
    paint_tool_rail(&rail, rail_rect, scene, text_system, theme);

    // Register per-entry hit rects. ToolRail lays out vertically;
    // we recompute slot rects here so dispatch can see them.
    let mut y = rail_rect.y;
    let gap = Spacing::Xs.px();
    let chip_x = rail_rect.x + (rail_rect.w - crate::widget::TOOL_CHIP_PX) * 0.5;
    for (i, entry) in rail.entries.iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        match entry {
            ToolRailEntry::Icon { id, .. } => {
                let chip = Rect::new(
                    chip_x,
                    y,
                    crate::widget::TOOL_CHIP_PX,
                    crate::widget::TOOL_CHIP_PX,
                );
                hit_index.register(*id, chip);
                y += crate::widget::TOOL_CHIP_PX;
            }
            ToolRailEntry::Compound { id, .. } => {
                let chip = Rect::new(
                    chip_x,
                    y,
                    crate::widget::TOOL_CHIP_PX,
                    crate::widget::TOOL_CHIP_PX,
                );
                hit_index.register(*id, chip);
                y += crate::widget::COMPOUND_TOTAL_H_PX;
            }
            ToolRailEntry::Divider => {
                y += crate::widget::DIVIDER_GAP_PX * 2.0 + 1.0;
            }
        }
    }
}

const PANEL_RADIUS: f32 = 16.0;
const PANEL_HEAD_PAD: f32 = 18.0;
const FIELD_ROW_H: f32 = 26.0;
const FIELD_GAP: f32 = 10.0;
const SECTION_HEAD_H: f32 = 26.0;
const SECTION_GAP: f32 = 6.0;

fn paint_panel_surface(rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = PANEL_RADIUS;
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
    // Drag handle: 36 x 4 pill at top-center.
    let handle = Rect::new(rect.x + (rect.w - 36.0) * 0.5, rect.y + 6.0, 36.0, 4.0);
    fill_rounded_rect(scene, handle, 999.0, resolve(ColorToken::BorderEmph, theme));
}

/// Paint the Inspector panel: drag handle + header (title + sub) +
/// description placeholder + sections from the fixture.
#[allow(clippy::too_many_arguments)]
pub fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);

    let title = selection
        .map(|s| s.label.as_str())
        .unwrap_or("(no selection)");
    let sub = "prefab.player.idle";

    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        title,
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        sub,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    // Divider under header.
    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 16.0;
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    let mut y = div_y + Spacing::Md.px();
    let body_pad = 10.0_f32;
    for section in fixture::inspector_sections() {
        let header_rect = Rect::new(
            rect.x + body_pad,
            y,
            rect.w - body_pad * 2.0,
            SECTION_HEAD_H,
        );
        let mut header = SectionHeader::new(NodeId(0), section.label.clone()).count(section.count);
        if let Some(open) = section.collapsible {
            header = header.collapsible(open);
        }
        paint_section_header(&header, header_rect, scene, text_system, theme);
        y += SECTION_HEAD_H;
        if matches!(section.collapsible, Some(false)) {
            y += SECTION_GAP;
            continue;
        }
        for field in &section.fields {
            if y + FIELD_ROW_H * 2.0 > rect.y + rect.h {
                return;
            }
            let field_id = inspector_field_id(&field.label);
            paint_inspector_field(
                field,
                field_id,
                rect.x + body_pad,
                rect.w - body_pad * 2.0,
                y,
                scene,
                text_system,
                theme,
                hit_index,
                store,
            );
            y += FIELD_ROW_H * 2.0 + FIELD_GAP;
        }
        y += SECTION_GAP;
    }
}

/// Map a fixture-label to the canonical interactive id for that
/// inspector field. `None` when the field is non-interactive in
/// Phase B (text-only or Phase C+ wiring).
fn inspector_field_id(label: &str) -> Option<NodeId> {
    Some(match label {
        "Move Speed" => ids::INSP_MOVE_SPEED,
        "Jump Height" => ids::INSP_JUMP_HEIGHT,
        "Friction" => ids::INSP_FRICTION,
        "Damping" => ids::INSP_DAMPING,
        "Cam Yaw" => ids::INSP_CAM_YAW,
        "Cam Pitch" => ids::INSP_CAM_PITCH,
        "Debug" => ids::INSP_DEBUG_SELECT,
        "Distance" => ids::INSP_LINK_DISTANCE,
        "Material" => ids::INSP_LINK_MATERIAL,
        _ => return None,
    })
}

#[allow(clippy::too_many_arguments)]
fn paint_inspector_field(
    field: &fixture::InspectorField,
    field_id: Option<NodeId>,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    use fixture::InspectorFieldKind;
    // Field head: marker dot + uppercase label.
    let head_rect = Rect::new(x, y, w, FIELD_ROW_H);
    let dot_r = 3.5;
    let dot_cx = x + dot_r + 4.0;
    let dot_cy = head_rect.y + head_rect.h * 0.5;
    let dot = Circle::new(Point::new(dot_cx as f64, dot_cy as f64), dot_r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &dot,
    );
    let label_x = dot_cx + dot_r + Spacing::Md.px();
    let label_y = head_rect.y + (head_rect.h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        &field.label,
        label_x,
        label_y,
        TypeToken::Xs.px(),
        head_rect.x + head_rect.w - label_x,
        resolve(ColorToken::Text2, theme),
    );

    let body_y = y + FIELD_ROW_H;
    let body_rect = Rect::new(x, body_y, w, FIELD_ROW_H);
    match &field.kind {
        InspectorFieldKind::Slider { value, display } => {
            let val_w = 56.0_f32;
            let val_rect = Rect::new(
                body_rect.x + body_rect.w - val_w,
                body_rect.y,
                val_w,
                body_rect.h,
            );
            let slider_rect = Rect::new(
                body_rect.x,
                body_rect.y,
                body_rect.w - val_w - Spacing::Md.px(),
                body_rect.h,
            );
            // If this field is wired into the store, register its
            // rect for hit-test and read the live value/state.
            // Otherwise fall back to the fixture's static value.
            let id = field_id.unwrap_or(NodeId(0));
            let (live_state, live_value) = field_id
                .and_then(|i| store.slider(i))
                .unwrap_or((SliderState::Normal, *value));
            if let Some(i) = field_id {
                hit_index.register(i, slider_rect);
            }
            let mut s = Slider::new(id, &field.label).accent(true);
            s.set_value(live_value);
            s.state = live_state;
            paint_slider(&s, slider_rect, scene, theme);
            // Value chip — show the original display string when no
            // store-backed value (Phase B+ fields keep the fixture
            // formatting; updates after drag are reflected by the
            // thumb position, not the numeric chip).
            fill_rounded_rect(
                scene,
                val_rect,
                Radius::Xs.px(),
                resolve(ColorToken::Bg3, theme),
            );
            paint_text_centered(
                text_system,
                scene,
                display,
                val_rect,
                TypeToken::Xs.px(),
                resolve(ColorToken::Text1, theme),
            );
        }
        InspectorFieldKind::Select { current } => {
            // Register the chip rect for hit-test when this Select
            // has a canonical id wired in the store. Read `open`
            // state from the Dropdown entry to flip the chevron.
            let is_open = field_id
                .and_then(|i| match store.get(i) {
                    Some(InteractiveState::Dropdown { open, .. }) => Some(*open),
                    _ => None,
                })
                .unwrap_or(false);
            if let Some(i) = field_id {
                hit_index.register(i, body_rect);
            }
            let border = if is_open {
                ColorToken::Accent
            } else {
                ColorToken::Border
            };
            fill_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                resolve(ColorToken::Bg3, theme),
            );
            stroke_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                if is_open { 2.0 } else { 1.0 },
                resolve(border, theme),
            );
            paint_text(
                text_system,
                scene,
                current,
                body_rect.x + Spacing::Lg.px(),
                body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
                TypeToken::Xs.px(),
                body_rect.w - Spacing::Lg.px() * 2.0 - 24.0,
                resolve(ColorToken::Text1, theme),
            );
            let chev_rect = Rect::new(
                body_rect.x + body_rect.w - Spacing::Lg.px() - 16.0,
                body_rect.y + (body_rect.h - 16.0) * 0.5,
                16.0,
                16.0,
            );
            let chev = if is_open {
                IconId::ChevronUp
            } else {
                IconId::ChevronDown
            };
            paint_icon(
                scene,
                chev,
                chev_rect,
                resolve(ColorToken::Text3, theme),
                1.5,
            );
        }
        InspectorFieldKind::Linked { source } => {
            fill_rounded_rect(
                scene,
                body_rect,
                Radius::Sm.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
            paint_text(
                text_system,
                scene,
                source,
                body_rect.x + Spacing::Lg.px(),
                body_rect.y + (body_rect.h - TypeToken::Xs.px()) * 0.5,
                TypeToken::Xs.px(),
                body_rect.w - Spacing::Lg.px() * 2.0,
                resolve(ColorToken::Accent, theme),
            );
        }
        InspectorFieldKind::LinkedSlider { value, display } => {
            let mut s = Slider::new(NodeId(0), &field.label)
                .accent(true)
                .state(SliderState::Normal);
            s.set_value(*value);
            let val_w = 56.0_f32;
            let slider_rect = Rect::new(
                body_rect.x,
                body_rect.y,
                body_rect.w - val_w - Spacing::Md.px(),
                body_rect.h,
            );
            paint_slider(&s, slider_rect, scene, theme);
            let val_rect = Rect::new(
                body_rect.x + body_rect.w - val_w,
                body_rect.y,
                val_w,
                body_rect.h,
            );
            fill_rounded_rect(
                scene,
                val_rect,
                Radius::Xs.px(),
                resolve(ColorToken::Bg3, theme),
            );
            if !display.is_empty() {
                paint_text_centered(
                    text_system,
                    scene,
                    display,
                    val_rect,
                    TypeToken::Xs.px(),
                    resolve(ColorToken::Text1, theme),
                );
            }
        }
    }
}

const HIER_ROW_H: f32 = 32.0;

/// Paint the Hierarchy panel: header (title + counts + add button) +
/// list of hardcoded entities from the fixture.
pub fn paint_hierarchy(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.hierarchy;
    paint_panel_surface(rect, scene, theme);

    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        "Hierarchy",
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0 - 40.0,
        resolve(ColorToken::Text1, theme),
    );
    let (entities, components) = fixture::hierarchy_counts();
    let counts = format!("{entities} entities \u{00b7} {components} components");
    paint_text(
        text_system,
        scene,
        &counts,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    // Add button at top-right.
    let add_size = 30.0_f32;
    let add_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - add_size,
        title_y - 2.0,
        add_size,
        add_size,
    );
    hit_index.register(ids::HIERARCHY_ADD, add_rect);
    let add_state = store
        .button_state(ids::HIERARCHY_ADD)
        .unwrap_or(ButtonState::Normal);
    let add_bg = match add_state {
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Hovered => ColorToken::AccentSoft,
        _ => ColorToken::AccentSoft,
    };
    fill_rounded_rect(scene, add_rect, 999.0, resolve(add_bg, theme));
    stroke_rounded_rect(
        scene,
        add_rect,
        999.0,
        1.0,
        resolve(ColorToken::Accent, theme),
    );
    let add_fg = if add_state == ButtonState::Pressed {
        ColorToken::AccentFg
    } else {
        ColorToken::Accent
    };
    paint_icon(scene, IconId::Add, add_rect, resolve(add_fg, theme), 1.5);

    let body_top = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 18.0;
    let body_pad = 8.0_f32;
    let mut y = body_top;
    for entity in fixture::hierarchy() {
        if y + HIER_ROW_H > rect.y + rect.h {
            break;
        }
        let row_rect = Rect::new(rect.x + body_pad, y, rect.w - body_pad * 2.0, HIER_ROW_H);
        paint_hierarchy_row(&entity, row_rect, scene, text_system, theme);
        y += HIER_ROW_H + 2.0;
    }
}

fn paint_hierarchy_row(
    entity: &fixture::HierarchyEntity,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    if entity.selected {
        fill_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            resolve(ColorToken::AccentSoft, theme),
        );
        stroke_rounded_rect(
            scene,
            rect,
            Radius::Sm.px(),
            1.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    let indent_w = 16.0 * entity.indent as f32;
    let pad = 10.0_f32;
    let icon_w = 16.0_f32;
    let icon_x = rect.x + pad + indent_w;
    let icon_rect = Rect::new(icon_x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    let icon_color = if entity.selected {
        ColorToken::Accent
    } else if entity.muted {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text3
    };
    paint_icon(
        scene,
        entity.icon,
        icon_rect,
        resolve(icon_color, theme),
        1.5,
    );

    // Trailing accessories occupy the right side; layout from right→left.
    let mut right_x = rect.x + rect.w - pad;
    let visibility_color = if entity.visible {
        ColorToken::Success
    } else {
        ColorToken::Border
    };
    let vis_r = 5.0_f32;
    let vis_cx = right_x - vis_r;
    let vis_cy = rect.y + rect.h * 0.5;
    let vis_dot = Circle::new(Point::new(vis_cx as f64, vis_cy as f64), vis_r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(visibility_color, theme)),
        None,
        &vis_dot,
    );
    right_x -= vis_r * 2.0 + 8.0;
    if let Some(swatch) = entity.swatch {
        let sw = 14.0_f32;
        let sw_rect = Rect::new(right_x - sw, rect.y + (rect.h - sw) * 0.5, sw, sw);
        let [r, g, b, a] = swatch;
        fill_rounded_rect(
            scene,
            sw_rect,
            4.0,
            ph2d_vector::Color::from_rgba8(r, g, b, a),
        );
        stroke_rounded_rect(scene, sw_rect, 4.0, 1.0, resolve(ColorToken::Border, theme));
        right_x -= sw + 6.0;
    }
    if let Some(badge) = &entity.badge {
        let badge_w = 32.0_f32;
        let badge_h = 18.0_f32;
        let badge_rect = Rect::new(
            right_x - badge_w,
            rect.y + (rect.h - badge_h) * 0.5,
            badge_w,
            badge_h,
        );
        let bg = if entity.selected {
            ColorToken::Accent
        } else {
            ColorToken::Bg3
        };
        let fg = if entity.selected {
            ColorToken::AccentFg
        } else {
            ColorToken::Text3
        };
        fill_rounded_rect(scene, badge_rect, Radius::Xs.px(), resolve(bg, theme));
        paint_text_centered(
            text_system,
            scene,
            badge,
            badge_rect,
            TypeToken::Xs.px() - 2.0,
            resolve(fg, theme),
        );
        right_x -= badge_w + 6.0;
    }

    let name_x = icon_rect.x + icon_w + 8.0;
    let name_color = if entity.muted {
        ColorToken::TextDisabled
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        &entity.name,
        name_x,
        rect.y + (rect.h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (right_x - name_x).max(0.0),
        resolve(name_color, theme),
    );
}

/// Paint the bottom HUD pill — `EDIT • 60 fps • 13101/16660 • 21n
/// • 100% • default-scene` — using the StatusBar widget.
pub fn paint_bottom_hud(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let bar = StatusBar::new(
        NodeId(300),
        "Editor HUD",
        vec![
            StatusSegment::new("EDIT")
                .dot(true)
                .tone(SegmentTone::Neutral),
            StatusSegment::new("60 fps"),
            StatusSegment::new("13101 / 16660").tone(SegmentTone::Accent),
            StatusSegment::new("21n"),
            StatusSegment::new("100%"),
            StatusSegment::new("default-scene").tone(SegmentTone::Muted),
        ],
    );
    let pref_w = bar.preferred_width().min(layout.viewport.w - 40.0);
    let rect = Rect::new(
        layout.viewport.x + (layout.viewport.w - pref_w) * 0.5,
        layout.bottom_hud.y,
        pref_w,
        layout.bottom_hud.h,
    );
    paint_status_bar(&bar, rect, scene, text_system, theme);
}

/// Paint the selection marquee + 4 corner handles + floating tag
/// "Player · PRF · 124, −48" above the canvas. Centered in the
/// canvas region.
pub fn paint_selection_overlay(
    layout: &HeroLayout,
    selection: &HeroSelection,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // Marquee dimensions are mockup-fixed (496 x 416 in the HTML).
    let marquee_w = (layout.canvas.w * 0.55).clamp(280.0, 520.0);
    let marquee_h = (layout.canvas.h * 0.5).clamp(220.0, 440.0);
    let cx = layout.canvas.x + layout.canvas.w * 0.5;
    let cy = layout.canvas.y + layout.canvas.h * 0.5;
    let marquee = Rect::new(
        cx - marquee_w * 0.5,
        cy - marquee_h * 0.5,
        marquee_w,
        marquee_h,
    );
    // Dashed stroke around the marquee.
    let stroke = Stroke::new(1.0).with_dashes(0.0, [6.0, 4.0]);
    let r = vello_dashed_rect(marquee);
    scene.inner_mut().stroke(
        &stroke,
        Affine::IDENTITY,
        &Brush::Solid(resolve(ColorToken::Accent, theme)),
        None,
        &r,
    );
    // 4 corner handles — 8x8 squares.
    let handle = 8.0_f32;
    for (hx, hy) in [
        (marquee.x - handle * 0.5, marquee.y - handle * 0.5),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y - handle * 0.5,
        ),
        (
            marquee.x - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
        (
            marquee.x + marquee.w - handle * 0.5,
            marquee.y + marquee.h - handle * 0.5,
        ),
    ] {
        let h_rect = Rect::new(hx, hy, handle, handle);
        fill_rounded_rect(scene, h_rect, 1.0, resolve(ColorToken::Bg0, theme));
        stroke_rounded_rect(scene, h_rect, 1.0, 2.0, resolve(ColorToken::Accent, theme));
    }
    // Floating selection tag above the marquee.
    let tag_w = 220.0_f32;
    let tag_h = 22.0_f32;
    let tag_rect = Rect::new(marquee.x, marquee.y - tag_h - 6.0, tag_w, tag_h);
    fill_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        resolve(ColorToken::BgElev, theme),
    );
    stroke_rounded_rect(
        scene,
        tag_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let pad = Spacing::Md.px();
    let label_y = tag_rect.y + (tag_rect.h - TypeToken::Xs.px()) * 0.5;
    paint_text(
        text_system,
        scene,
        &selection.label,
        tag_rect.x + pad,
        label_y,
        TypeToken::Xs.px(),
        80.0,
        resolve(ColorToken::Text1, theme),
    );
    // Mini badge after label.
    let badge_x = tag_rect.x + pad + 60.0;
    let badge_w = 32.0;
    let badge_rect = Rect::new(badge_x, tag_rect.y + 4.0, badge_w, tag_rect.h - 8.0);
    fill_rounded_rect(
        scene,
        badge_rect,
        Radius::Xs.px(),
        resolve(ColorToken::AccentSoft, theme),
    );
    paint_text_centered(
        text_system,
        scene,
        &selection.kind,
        badge_rect,
        TypeToken::Xs.px() - 2.0,
        resolve(ColorToken::Accent, theme),
    );
    let pos_text = format!(
        "\u{00b7} {:.0}, {:.0}",
        selection.world_pos.0, selection.world_pos.1
    );
    paint_text(
        text_system,
        scene,
        &pos_text,
        badge_x + badge_w + 8.0,
        label_y,
        TypeToken::Xs.px(),
        100.0,
        resolve(ColorToken::Text3, theme),
    );
}

fn vello_dashed_rect(rect: Rect) -> ph2d_vector::Rect {
    rect_to_vello(rect)
}

/// Phase A paint — full hero composition + hit-index population.
///
/// Mutates [`HeroScreen::hit_index`]: clears it at the start of the
/// frame and re-registers every interactive widget's rect as the
/// painters emit geometry. The [`WidgetStore`] supplies hover/press
/// state for visual feedback (Button/Toggle wired in Phase A;
/// Slider/Checkbox/etc. in Phases B-D).
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
    paint_hierarchy(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
    );
    paint_bottom_hud(&layout, scene, text_system, hero.theme);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipad12_viewport() -> Rect {
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
    }

    #[test]
    fn layout_top_bar_inset_from_edge() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!((layout.top_bar.x - EDGE_PAD).abs() < f32::EPSILON);
        assert!((layout.top_bar.h - TOPBAR_H).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_left_rail_below_top_bar() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!(layout.left_rail.y > layout.top_bar.y + layout.top_bar.h);
        assert!((layout.left_rail.w - RAIL_W).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_inspector_after_rail() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        assert!(layout.inspector.x > layout.left_rail.x + layout.left_rail.w);
        assert!((layout.inspector.w - INSPECTOR_W).abs() < f32::EPSILON);
    }

    #[test]
    fn layout_hierarchy_pinned_right() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let right_edge = layout.hierarchy.x + layout.hierarchy.w;
        assert!((right_edge - (HERO_VIEWPORT_W - EDGE_PAD)).abs() < 0.01);
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

    // -----------------------------------------------------------------
    // Phase A — interactive integration smokes
    // -----------------------------------------------------------------

    use bumpalo::Bump;
    use ph2d_host::{PointerEvent, PointerKind, PointerSource};

    fn down(x: f32, y: f32) -> PointerEvent {
        PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind: PointerKind::Down,
            source: PointerSource::Mouse,
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
        // Paint once to populate hit_index with TopBar Save rect.
        paint_hero_screen(&mut hero, ipad12_viewport(), &mut scene, &mut text);
        // The Save chip lives inside the Single cluster; find its rect
        // by querying hit_index for the canonical NodeId.
        // We synthesize the click at the first paint-registered rect
        // for that id by sweeping a few candidate Y rows in the topbar.
        // Topbar Y is around 14..54 — sweep with a known-good x.
        // Simpler: find center by iterating through the registered
        // rects in the index (test-only helper would be cleaner).
        // For Phase A we trust paint geometry: TOPBAR_SAVE chip
        // sits in the second pill cluster on the left.
        // Use a known-good coordinate based on layout knowledge:
        // theme cluster ~132 wide + 8 gap = chip cluster starts at ~154.
        // Safer: brute-force scan through registered rects.
        let arena = Bump::new();
        // hover into the Save chip first to exercise the hit pipeline
        let mut save_x = 0.0;
        let mut save_y = 0.0;
        // Sweep; find any (x, y) that hits TOPBAR_SAVE.
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
}
