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
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use crate::widget::{
    PILL_PADDING_PX, SectionHeader, SegmentTone, Slider, SliderState, StatusBar, StatusSegment,
    ToolRail, ToolRailEntry, paint_section_header, paint_slider, paint_status_bar, paint_tool_rail,
};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
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

#[derive(Clone, Debug)]
pub struct HeroScreen {
    pub id: NodeId,
    pub theme: Theme,
    pub selection: Option<HeroSelection>,
}

impl HeroScreen {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            theme: Theme::ForgeSdf,
            selection: Some(fixture::default_selection()),
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
/// and a centered `PH2D · EDITOR` wordmark.
pub fn paint_top_bar(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let clusters = fixture::topbar_clusters();
    let row_h = layout.top_bar.h;
    // Left cluster column.
    let mut x = layout.top_bar.x;
    let gap = Spacing::Md.px();
    // We split clusters into "left" (first 4) and "right" (last 1)
    // groups so the wordmark can sit in the middle.
    let split = 4.min(clusters.len());
    let mut id = 100u64;
    for cluster in &clusters[..split] {
        let (rect, next_id) = layout_top_bar_cluster(x, layout.top_bar.y, row_h, cluster, id);
        paint_top_bar_cluster(cluster, rect, scene, text_system, theme);
        x = rect.x + rect.w + gap;
        id = next_id;
    }
    // Centered wordmark fills the gap between left and right clusters.
    let right_clusters = &clusters[split..];
    let mut right_w = 0.0_f32;
    for c in right_clusters {
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
    for cluster in right_clusters {
        let (rect, next_id) = layout_top_bar_cluster(rx, layout.top_bar.y, row_h, cluster, id);
        paint_top_bar_cluster(cluster, rect, scene, text_system, theme);
        rx = rect.x + rect.w + gap;
        id = next_id;
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

fn layout_top_bar_cluster(
    x: f32,
    y: f32,
    h: f32,
    cluster: &fixture::TopBarCluster,
    id: u64,
) -> (Rect, u64) {
    let w = cluster_width(cluster);
    (Rect::new(x, y, w, h), id + 1)
}

fn paint_top_bar_cluster(
    cluster: &fixture::TopBarCluster,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
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
            let chip = Rect::new(
                rect.x + (rect.w - 32.0) * 0.5,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            paint_icon(scene, *icon, chip, resolve(ColorToken::Text2, theme), 1.5);
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
            // Theme-mode toggle on the left, accent play button on the right.
            let toggle_rect = Rect::new(rect.x + pad_x, rect.y + (rect.h - 22.0) * 0.5, 22.0, 22.0);
            paint_icon(
                scene,
                IconId::Light,
                toggle_rect,
                resolve(ColorToken::Text2, theme),
                1.5,
            );
            let play_rect = Rect::new(
                rect.x + rect.w - pad_x - 32.0,
                rect.y + (rect.h - 32.0) * 0.5,
                32.0,
                32.0,
            );
            fill_rounded_rect(
                scene,
                play_rect,
                Radius::Lg.px(),
                resolve(ColorToken::Danger, theme),
            );
            paint_icon(
                scene,
                IconId::Play,
                play_rect,
                resolve(ColorToken::AccentFg, theme),
                1.5,
            );
        }
        TopBarCluster::Right => {
            let icons = [IconId::Layers, IconId::Asset, IconId::Script];
            let chip = 32.0_f32;
            for (i, icon) in icons.iter().enumerate() {
                let cx = rect.x + pad_x + (chip + 4.0) * i as f32;
                let chip_rect = Rect::new(cx, rect.y + (rect.h - chip) * 0.5, chip, chip);
                paint_icon(
                    scene,
                    *icon,
                    chip_rect,
                    resolve(ColorToken::Text2, theme),
                    1.5,
                );
            }
        }
    }
}

/// Paint the LeftRail using the `ToolRail` widget with the
/// fixture's transform/space/history entries.
pub fn paint_left_rail(
    layout: &HeroLayout,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let rail = ToolRail::new(
        NodeId(200),
        "Editor tools",
        vec![
            ToolRailEntry::icon(NodeId(201), "Translate", IconId::Transform).active(),
            ToolRailEntry::icon(NodeId(202), "Rotate", IconId::Rotate),
            ToolRailEntry::icon(NodeId(203), "Scale", IconId::Scale),
            ToolRailEntry::icon(NodeId(204), "Pivot", IconId::Pivot),
            ToolRailEntry::Divider,
            ToolRailEntry::compound(NodeId(205), "Coordinate space", "Global", "SPACE"),
            ToolRailEntry::compound(NodeId(206), "Camera projection", "Persp", "PROJ"),
            ToolRailEntry::compound(NodeId(207), "Frame to home", "Home", "VIEW"),
            ToolRailEntry::Divider,
            ToolRailEntry::icon(NodeId(208), "Undo", IconId::Undo),
            ToolRailEntry::icon(NodeId(209), "Redo", IconId::Redo),
        ],
    );
    let rail_rect = Rect::new(
        layout.left_rail.x,
        layout.left_rail.y,
        layout.left_rail.w,
        rail.preferred_height(),
    );
    paint_tool_rail(&rail, rail_rect, scene, text_system, theme);
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
pub fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
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
            paint_inspector_field(
                field,
                rect.x + body_pad,
                rect.w - body_pad * 2.0,
                y,
                scene,
                text_system,
                theme,
            );
            y += FIELD_ROW_H * 2.0 + FIELD_GAP;
        }
        y += SECTION_GAP;
    }
}

fn paint_inspector_field(
    field: &fixture::InspectorField,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
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
            let mut s = Slider::new(NodeId(0), &field.label).accent(true);
            s.set_value(*value);
            paint_slider(&s, slider_rect, scene, theme);
            // Value chip.
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
                1.0,
                resolve(ColorToken::Border, theme),
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
            paint_icon(
                scene,
                IconId::ChevronDown,
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
    fill_rounded_rect(
        scene,
        add_rect,
        999.0,
        resolve(ColorToken::AccentSoft, theme),
    );
    stroke_rounded_rect(
        scene,
        add_rect,
        999.0,
        1.0,
        resolve(ColorToken::Accent, theme),
    );
    paint_icon(
        scene,
        IconId::Add,
        add_rect,
        resolve(ColorToken::Accent, theme),
        1.5,
    );

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

/// Phase 4 paint — full hero composition.
pub fn paint_hero_screen(
    hero: &HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    let layout = HeroLayout::for_viewport(viewport);
    paint_canvas_bg(&layout, scene, hero.theme);
    if let Some(sel) = hero.selection.as_ref() {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
    }
    paint_top_bar(&layout, scene, text_system, hero.theme);
    paint_left_rail(&layout, scene, text_system, hero.theme);
    paint_inspector(
        &layout,
        hero.selection.as_ref(),
        scene,
        text_system,
        hero.theme,
    );
    paint_hierarchy(&layout, scene, text_system, hero.theme);
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
        let hero = HeroScreen::new(NodeId(1));
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_hero_smoke_alternate_theme() {
        let hero = HeroScreen::new(NodeId(1)).theme(Theme::Sunstone);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_hero_smoke_no_selection() {
        let hero = HeroScreen::new(NodeId(1)).selection(None);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hero_screen(&hero, ipad12_viewport(), &mut scene, &mut text);
    }

    #[test]
    fn paint_top_bar_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_top_bar(&layout, &mut scene, &mut text, Theme::ForgeSdf);
    }

    #[test]
    fn paint_left_rail_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_left_rail(&layout, &mut scene, &mut text, Theme::ForgeSdf);
    }

    #[test]
    fn paint_inspector_smoke_with_selection() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let sel = fixture::default_selection();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_inspector(&layout, Some(&sel), &mut scene, &mut text, Theme::Sunstone);
    }

    #[test]
    fn paint_inspector_smoke_no_selection() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_inspector(&layout, None, &mut scene, &mut text, Theme::Blueprint);
    }

    #[test]
    fn paint_hierarchy_smoke() {
        let layout = HeroLayout::for_viewport(ipad12_viewport());
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_hierarchy(&layout, &mut scene, &mut text, Theme::ForgeSdf);
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
            let hero = HeroScreen::new(NodeId(1)).theme(theme);
            let mut scene = VectorScene::new();
            let mut text = TextSystem::new();
            paint_hero_screen(&hero, ipad12_viewport(), &mut scene, &mut text);
        }
    }
}
