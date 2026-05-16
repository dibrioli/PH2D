//! Floating-panel paint + event handler for the grid-snap subsystem.
//!
//! Layout: vertical stack of title bar / Grid Kind section (kind
//! cycler + per-kind parameter rows) / Snap section / Display
//! section (Show overlay + Opacity slider) / Inspect section.
//! Drag / resize handles match [`crate::screens::hero::widget_gallery`]
//! so the existing `BlenderHitKind::DragHandle` / `ResizeHandle`
//! dispatch moves the panel.
//!
//! Per-kind config rows are populated for **all** kinds at startup
//! (every NumberInput / cycle Button is registered) but painted +
//! hit-tested only for the active `state.kind`. Switching kind
//! preserves every other kind's params untouched.

use super::ids;
use super::state::{GridKind, GridSnapState};
use crate::interaction::{BlenderHitKind, HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{paint_icon, paint_text, paint_text_title, resolve};
use crate::screens::hero::style::{
    paint_panel_corner_dot, paint_panel_surface, panel_drag_handle_rect, panel_resize_handle_rect,
};
use crate::widget::{
    Button, ButtonKind, ButtonState, NumberInput, Slider, SliderOrientation, SliderState,
    TextInputState, Toggle, ToggleState, paint_button, paint_number_input_with_buffer,
    paint_slider, paint_toggle,
};
use crate::zones::Rect;
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::staggered::StaggerParity;
use ph2d_grid::tri::TriNeighborhood;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::VectorScene;

thread_local! {
    /// Last computed scrollable content height of the panel (set by
    /// `paint`, read by the host caller in `paint_hero_screen`).
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
    /// Last visible body height (panel rect minus title + paddings).
    static LAST_VISIBLE_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// Last computed content height — call from the host after `paint`
/// to publish via `store.set_panel_content_h(GS_PANEL, …)`.
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(|c| c.get())
}

/// Last computed visible height — call from the host after `paint`
/// to publish via `store.set_panel_visible_h(GS_PANEL, …)`.
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(|c| c.get())
}

const ROW_H: f32 = 28.0;
/// Top inset that frees space above the title for the drag pill +
/// matches the spacing other panels (Inspector/Hierarchy/Gallery) use
/// between the drag handle and the first label row.
const HEAD_PAD: f32 = 18.0;
const PAD: f32 = 12.0;
const ROW_GAP: f32 = 6.0;
const LABEL_FONT_SIZE: f32 = 13.0;
const TITLE_FONT_SIZE: f32 = 15.0;
/// Column where the widget (right side of a "Label: [widget]" row)
/// starts, measured from the inner-x of the row.
const LABEL_COL_W: f32 = 110.0;

/// Register the panel's interactive nodes in `store`. Called once
/// from `HeroScreen::pre_populate_store` (Coordenador wiring).
pub fn populate(store: &mut WidgetStore) {
    // Panel chrome — Blender drag/resize handles same as Widget Gallery.
    store.register(
        ids::GS_DRAG_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::DragHandle,
        },
    );
    store.register(
        ids::GS_RESIZE_HANDLE,
        InteractiveState::BlenderHit {
            parent: ids::GS_PANEL,
            kind: BlenderHitKind::ResizeHandle,
        },
    );
    // All selectors are segmented Button groups (style parity with
    // Inspector's Strategy switcher per Enio's 2026-05-15 redesign):
    // each option has its own NodeId registered as a Button; the
    // active option is painted with `ButtonState::Pressed` driven
    // from `state` at paint time. The remaining 2-option cycle
    // buttons (Hex orientation, Hex offset, Stagger parity) keep
    // the cycle pattern for now.
    for id in [
        ids::GS_CLOSE,
        // Kind group (9 options).
        ids::GS_KIND_OPT_SQUARE,
        ids::GS_KIND_OPT_HEX,
        ids::GS_KIND_OPT_ISO,
        ids::GS_KIND_OPT_STAGGERED_SQ,
        ids::GS_KIND_OPT_STAGGERED_HEX,
        ids::GS_KIND_OPT_TRI,
        ids::GS_KIND_OPT_QUADTREE,
        ids::GS_KIND_OPT_VORONOI,
        ids::GS_KIND_OPT_CHUNKS,
        // Target group (5 SnapTarget modes).
        ids::GS_SNAP_CENTER,
        ids::GS_SNAP_INTERSECTION,
        ids::GS_SNAP_TARGET_OPT_CORNER,
        ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION,
        ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
        // Neighborhood groups (Square family Von4/Moore8; Tri Edge3/Vertex12).
        ids::GS_CFG_NEIGHBORHOOD_4,
        ids::GS_CFG_NEIGHBORHOOD_8,
        ids::GS_CFG_TRI_EDGE3,
        ids::GS_CFG_TRI_VERTEX12,
        // Cycle buttons (untouched in this pass).
        ids::GS_CFG_HEX_POINTY,
        ids::GS_CFG_HEX_OFFSET_DROPDOWN,
        ids::GS_CFG_STAGGER_PARITY_ODD,
        ids::GS_CFG_VORONOI_RESEED,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Toggles — start values match GridSnapState::default().
    store.register(
        ids::GS_SNAP_ENABLED,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: false,
        },
    );
    store.register(
        ids::GS_SHOW_OVERLAY,
        InteractiveState::Toggle {
            state: ToggleState::Normal,
            on: true,
        },
    );
    // Opacity slider — value matches GridSnapState::default().opacity.
    store.register(
        ids::GS_OPACITY_SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.75,
            orientation: SliderOrientation::Horizontal,
        },
    );

    // NumberInputs — register one entry per reserved id, default
    // values pulled from GridSnapState::default() so paint stays
    // in sync on first frame.
    let defaults = GridSnapState::default();
    let number_specs: &[(crate::NodeId, f64)] = &[
        (ids::GS_CFG_CELL_SIZE, defaults.square_cfg.cell_size as f64),
        (ids::GS_CFG_ISO_TILE_W, defaults.iso_cfg.tile_w as f64),
        (ids::GS_CFG_ISO_TILE_H, defaults.iso_cfg.tile_h as f64),
        (
            ids::GS_CFG_QT_MAX_PER_LEAF,
            defaults.quadtree_cfg.max_points_per_leaf as f64,
        ),
        (
            ids::GS_CFG_QT_MAX_DEPTH,
            defaults.quadtree_cfg.max_depth as f64,
        ),
        (
            ids::GS_CFG_VORONOI_SEED_COUNT,
            defaults.voronoi_cfg.seed_count as f64,
        ),
        (
            ids::GS_CFG_VORONOI_RNG_SEED,
            defaults.voronoi_cfg.rng_seed as f64,
        ),
        (
            ids::GS_CFG_VORONOI_LLOYD_ITERS,
            defaults.voronoi_cfg.lloyd_iterations as f64,
        ),
        (
            ids::GS_CFG_CHUNKS_SIZE,
            defaults.chunks_cfg.chunk_size_cells as f64,
        ),
        // Universal extras — values mirror the active kind's *Cfg
        // on each frame (apply_event keeps store in sync).
        (ids::GS_CFG_ORIGIN_X, 0.0),
        (ids::GS_CFG_ORIGIN_Y, 0.0),
        (
            ids::GS_CFG_SPACING_MAJOR,
            defaults.square_cfg.spacing_major as f64,
        ),
        // Grid color RGB — alpha stays implicit (controlled by the
        // opacity slider).
        (ids::GS_CFG_COLOR_R, defaults.color_rgba[0] as f64),
        (ids::GS_CFG_COLOR_G, defaults.color_rgba[1] as f64),
        (ids::GS_CFG_COLOR_B, defaults.color_rgba[2] as f64),
        // Snap subdivisions (sub-grid factor; rendering unaffected).
        (
            ids::GS_CFG_SNAP_SUBDIVISIONS,
            defaults.snap_subdivisions as f64,
        ),
        // Inspect probes A / B (X, Y) — user-editable from the panel.
        (ids::GS_PROBE_A_X, defaults.probe_a[0] as f64),
        (ids::GS_PROBE_A_Y, defaults.probe_a[1] as f64),
        (ids::GS_PROBE_B_X, defaults.probe_b[0] as f64),
        (ids::GS_PROBE_B_Y, defaults.probe_b[1] as f64),
        // Quadtree bounds + demo seeds.
        (
            ids::GS_CFG_QT_BOUNDS_MIN_X,
            defaults.quadtree_cfg.bounds.min[0] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MIN_Y,
            defaults.quadtree_cfg.bounds.min[1] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MAX_X,
            defaults.quadtree_cfg.bounds.max[0] as f64,
        ),
        (
            ids::GS_CFG_QT_BOUNDS_MAX_Y,
            defaults.quadtree_cfg.bounds.max[1] as f64,
        ),
        (
            ids::GS_CFG_QT_DEMO_POINTS,
            defaults.quadtree_cfg.demo_point_count as f64,
        ),
        (
            ids::GS_CFG_QT_DEMO_SEED,
            defaults.quadtree_cfg.demo_rng_seed as f64,
        ),
        // Voronoi bounds.
        (
            ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
            defaults.voronoi_cfg.bounds.min[0] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
            defaults.voronoi_cfg.bounds.min[1] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
            defaults.voronoi_cfg.bounds.max[0] as f64,
        ),
        (
            ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
            defaults.voronoi_cfg.bounds.max[1] as f64,
        ),
    ];
    for (id, value) in number_specs {
        store.register(
            *id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: *value,
                buffer: format_value(*value),
                caret: 0,
                last_committed: *value,
                selection_anchor: None,
            },
        );
    }
}

fn format_value(v: f64) -> String {
    // Integers render without a decimal point so step=1 fields read
    // clean ("4" vs "4.0"). Non-integer values keep 2 decimals.
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

/// Default panel rect when first opened — sized for title + 4
/// sections (Kind + per-kind config + Snap + Display + Inspect).
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    // Width matches Inspector (`style::INSPECTOR_W = 304`) per Enio's
    // 2026-05-15 redesign so the two floating panels read as siblings.
    let w = 304.0_f32.min(viewport_w - 16.0);
    let h = 640.0_f32.min(viewport_h - 16.0).max(440.0);
    let x = ((viewport_w - w) * 0.5).max(8.0);
    let y = ((viewport_h - h) * 0.5).max(8.0);
    Rect::new(x, y, w, h)
}

/// Paint the panel into `rect`. Reads `state` for current values;
/// mutations flow through [`apply_event`] from the dispatcher.
///
/// Layout (top → bottom): title row → big Snap toggle → Kind 3×3
/// button grid → per-kind config → Target vertical stack →
/// Subdivisions → Display (overlay + opacity + color swatch) →
/// Inspect. Body is clipped to the panel rect and scroll-aware:
/// content height is published via `store.set_panel_content_h` so
/// `dispatch_wheel` knows the scroll bound, and the running Y
/// position is offset by `store.panel_scroll(GS_PANEL)`.
pub fn paint(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_panel_surface(rect, scene, theme);
    hit_index.register(ids::GS_DRAG_HANDLE, panel_drag_handle_rect(rect));

    let inner_x = rect.x + PAD;
    let inner_w = rect.w - PAD * 2.0;
    let title_y = rect.y + HEAD_PAD;

    // ─── Title row + close (above scroll clip) ────────────────
    paint_text_title(
        text_system,
        scene,
        "Grid Settings",
        inner_x,
        title_y,
        TITLE_FONT_SIZE,
        inner_w - 32.0,
        resolve(ColorToken::Text1, theme),
    );
    let close_size = 22.0_f32;
    let close_rect = Rect::new(
        rect.x + rect.w - close_size - PAD,
        title_y - 2.0,
        close_size,
        close_size,
    );
    paint_icon(
        scene,
        crate::icons::IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        1.5,
    );
    hit_index.register(ids::GS_CLOSE, close_rect);

    // Body rect — sits below the title row, above the corner dot.
    // Everything below is clipped to this rect so resize-up doesn't
    // spill rows past the panel edge.
    let body_top = title_y + close_size + ROW_GAP * 2.0;
    let body_h = (rect.y + rect.h - body_top - PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = store.panel_scroll(ids::GS_PANEL);

    scene.push_clip(&crate::paint::rect_to_vello(body_rect));
    let mut y = body_top - scroll;

    // ─── Snap (BIG individual toggle, above Kind) ───────────────
    y = paint_snap_top_toggle(
        inner_x,
        inner_w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_GAP * 2.0;

    // ─── Grid Kind section (3×3 button grid) ────────────────────
    y = paint_section_label("Grid Kind", inner_x, inner_w, y, scene, text_system, theme);
    y = paint_kind_button_grid(
        inner_x,
        inner_w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_GAP;

    // Per-kind config rows.
    y = paint_kind_config(
        inner_x,
        inner_w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_GAP * 2.0;

    // ─── Target section (vertical stack of 5 full-width buttons) ─
    y = paint_section_label("Target", inner_x, inner_w, y, scene, text_system, theme);
    y = paint_target_button_stack(
        inner_x,
        inner_w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y = paint_number_row_from_state(
        "Subdivisions",
        ids::GS_CFG_SNAP_SUBDIVISIONS,
        state.snap_subdivisions as f64,
        inner_x,
        inner_w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y += ROW_GAP * 2.0;

    // ─── Display section ────────────────────────────────────────
    y = paint_section_label("Display", inner_x, inner_w, y, scene, text_system, theme);
    let overlay_row = Rect::new(inner_x, y, inner_w, ROW_H);
    paint_show_overlay_row(
        overlay_row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_H + ROW_GAP;
    let opacity_row = Rect::new(inner_x, y, inner_w, ROW_H);
    paint_opacity_slider_row(
        opacity_row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_H + ROW_GAP;
    let color_row = Rect::new(inner_x, y, inner_w, ROW_H);
    paint_color_swatch_row(
        color_row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += ROW_H + ROW_GAP * 2.0;

    // ─── Inspect section ────────────────────────────────────────
    let inspect_h = super::inspect::height();
    super::inspect::paint(
        Rect::new(inner_x, y, inner_w, inspect_h),
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    );
    y += inspect_h;

    scene.pop_layer();

    // Content / visible-height publication for `dispatch_wheel`
    // scroll bound: stash into thread-local cells; the host caller
    // (hero.rs) reads via `last_content_h()` / `last_visible_h()` and
    // publishes through `&mut store`. Same pattern as widget_gallery /
    // inspector.
    let content_h = (y + scroll) - body_top;
    LAST_CONTENT_H.with(|c| c.set(content_h));
    LAST_VISIBLE_H.with(|c| c.set(body_h));

    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::GS_RESIZE_HANDLE, panel_resize_handle_rect(rect));
}

/// Inspector-style section header: `paint_text_title` in
/// `TypeToken::Md` + 1px Border separator below. Same visual as the
/// "Transform" / "Render Source" headers in the Inspector.
fn paint_section_label(
    label: &str,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    paint_text_title(
        text_system,
        scene,
        label,
        x,
        y,
        ph2d_tokens::TypeToken::Md.px(),
        w,
        resolve(ColorToken::Text1, theme),
    );
    let after_title_y = y + ph2d_tokens::TypeToken::Md.px() + 6.0;
    // Subtle 1px Border separator below the title (Inspector parity).
    let sep_rect = Rect::new(x, after_title_y, w, 1.0);
    crate::paint::fill_rounded_rect(scene, sep_rect, 0.0, resolve(ColorToken::Border, theme));
    after_title_y + 1.0 + ROW_GAP
}

/// Big individual Snap toggle at the top of the panel — primary
/// action chip that flips `state.snap_enabled`. Full-width;
/// `Accent` fill + `AccentFg` text when ON (canonical primary CTA
/// look — `AccentFg` is the contrast pair `Accent` was designed
/// against), `BgElev` + Border + `Text2` when OFF. Returns the Y
/// after the row + gap.
#[allow(clippy::too_many_arguments)]
fn paint_snap_top_toggle(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let h = 44.0_f32;
    let rect = Rect::new(x, y, w, h);
    let radius = ph2d_tokens::Radius::Md.px();
    let on = state.snap_enabled;
    let (fill_tok, border_tok, text_tok) = if on {
        // Bold Accent fill + AccentFg text — high contrast primary
        // CTA. AccentSoft was too dark / blended with AccentFg.
        (ColorToken::Accent, ColorToken::Accent, ColorToken::AccentFg)
    } else {
        (ColorToken::BgElev, ColorToken::Border, ColorToken::Text2)
    };
    crate::paint::fill_rounded_rect(scene, rect, radius, resolve(fill_tok, theme));
    crate::paint::stroke_rounded_rect(scene, rect, radius, 1.5, resolve(border_tok, theme));
    let label = if on { "Snap: ON" } else { "Snap: OFF" };
    let font = ph2d_tokens::TypeToken::Md.px();
    crate::paint::paint_text_centered(
        text_system,
        scene,
        label,
        rect,
        font,
        resolve(text_tok, theme),
    );
    hit_index.register(ids::GS_SNAP_ENABLED, rect);
    // Suppress unused-store warning while the snap state lives on the
    // Toggle in store too (canonical InteractiveState).
    let _ = store;
    y + h + ROW_GAP
}

/// Generic segmented button row helper — paints one Button at `rect`
/// with `Pressed` state when `pressed=true`, otherwise `Normal`.
/// Hit-registers `id`. Used by every segmented group below.
#[allow(clippy::too_many_arguments)]
fn paint_segmented_button(
    rect: Rect,
    label: &str,
    pressed: bool,
    id: crate::NodeId,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let state = if pressed {
        ButtonState::Pressed
    } else {
        store.button_state(id).unwrap_or(ButtonState::Normal)
    };
    let btn = Button::new(id, label)
        .kind(ButtonKind::Default)
        .state(state);
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(id, rect);
}

/// 3-column × 3-row grid of Kind buttons (abbreviated labels so they
/// fit a ~95 px-wide column). Returns Y after the grid.
#[allow(clippy::too_many_arguments)]
fn paint_kind_button_grid(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let entries: [(GridKind, &str, crate::NodeId); 9] = [
        (GridKind::Square, "Square", ids::GS_KIND_OPT_SQUARE),
        (GridKind::Hex, "Hex", ids::GS_KIND_OPT_HEX),
        (GridKind::Iso, "Iso", ids::GS_KIND_OPT_ISO),
        (
            GridKind::StaggeredSquare,
            "Stag Sq",
            ids::GS_KIND_OPT_STAGGERED_SQ,
        ),
        (
            GridKind::StaggeredHex,
            "Stag Hex",
            ids::GS_KIND_OPT_STAGGERED_HEX,
        ),
        (GridKind::Tri, "Tri", ids::GS_KIND_OPT_TRI),
        (GridKind::Quadtree, "Quadtree", ids::GS_KIND_OPT_QUADTREE),
        (GridKind::Voronoi, "Voronoi", ids::GS_KIND_OPT_VORONOI),
        (GridKind::Chunks, "Chunks", ids::GS_KIND_OPT_CHUNKS),
    ];
    let cols = 3.0_f32;
    let gap = 6.0_f32;
    let col_w = ((w - gap * (cols - 1.0)) / cols).max(40.0);
    let h = 28.0_f32;
    let mut cy = y;
    for (row_i, chunk) in entries.chunks(3).enumerate() {
        let row_y = y + row_i as f32 * (h + gap);
        for (col_i, (kind, label, oid)) in chunk.iter().enumerate() {
            let rx = x + col_i as f32 * (col_w + gap);
            let rect = Rect::new(rx, row_y, col_w, h);
            paint_segmented_button(
                rect,
                label,
                state.kind == *kind,
                *oid,
                scene,
                text_system,
                theme,
                hit_index,
                store,
            );
        }
        cy = row_y + h;
    }
    cy + ROW_GAP
}

/// Vertical stack of 5 full-width Target buttons (Center,
/// Intersection, Corner, Center+Intersection, Center+Int+Corners).
/// Active option painted with `ButtonState::Pressed`.
#[allow(clippy::too_many_arguments)]
fn paint_target_button_stack(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let h = 28.0_f32;
    let gap = 6.0_f32;
    let entries: [(SnapTarget, &str, crate::NodeId); 5] = [
        (SnapTarget::Center, "Center", ids::GS_SNAP_CENTER),
        (
            SnapTarget::Intersection,
            "Intersection",
            ids::GS_SNAP_INTERSECTION,
        ),
        (SnapTarget::Corner, "Corner", ids::GS_SNAP_TARGET_OPT_CORNER),
        (
            SnapTarget::CenterAndIntersection,
            "Center + Intersection",
            ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION,
        ),
        (
            SnapTarget::CenterIntersectionAndCorners,
            "Center + Intersection + Corners",
            ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS,
        ),
    ];
    let mut cy = y;
    for (i, (target, label, oid)) in entries.iter().enumerate() {
        let row_y = y + i as f32 * (h + gap);
        let rect = Rect::new(x, row_y, w, h);
        paint_segmented_button(
            rect,
            label,
            state.snap_target == *target,
            *oid,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
        cy = row_y + h;
    }
    cy + ROW_GAP
}

/// 2-button row for the Square-family Neighborhood (Von4 / Moore8)
/// or the Tri Neighborhood (Edge3 / Vertex12). Active option painted
/// with `ButtonState::Pressed`. The active kind decides which family
/// the caller invokes via.
#[allow(clippy::too_many_arguments)]
fn paint_neighborhood_button_row(
    x: f32,
    w: f32,
    y: f32,
    family: NeighborhoodFamily,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let h = 28.0_f32;
    let gap = 6.0_f32;
    let half_w = (w - gap) * 0.5;
    let (label_l, id_l, label_r, id_r, active_idx) = match family {
        NeighborhoodFamily::Square => {
            let n = neighborhood_for_active_kind(state);
            (
                "Von4",
                ids::GS_CFG_NEIGHBORHOOD_4,
                "Moore8",
                ids::GS_CFG_NEIGHBORHOOD_8,
                if matches!(n, SquareNeighborhood::Moore8) {
                    1
                } else {
                    0
                },
            )
        }
        NeighborhoodFamily::Tri => (
            "Edge3",
            ids::GS_CFG_TRI_EDGE3,
            "Vertex12",
            ids::GS_CFG_TRI_VERTEX12,
            if matches!(state.tri_cfg.neighborhood, TriNeighborhood::Vertex12) {
                1
            } else {
                0
            },
        ),
    };
    let rect_l = Rect::new(x, y, half_w, h);
    let rect_r = Rect::new(x + half_w + gap, y, half_w, h);
    paint_segmented_button(
        rect_l,
        label_l,
        active_idx == 0,
        id_l,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_segmented_button(
        rect_r,
        label_r,
        active_idx == 1,
        id_r,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y + h + ROW_GAP
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NeighborhoodFamily {
    Square,
    Tri,
}

fn neighborhood_for_active_kind(state: &GridSnapState) -> SquareNeighborhood {
    match state.kind {
        GridKind::Square => state.square_cfg.neighborhood,
        GridKind::Iso => state.iso_cfg.neighborhood,
        GridKind::StaggeredSquare => state.staggered_square_cfg.neighborhood,
        GridKind::Chunks => state.chunks_cfg.neighborhood,
        _ => SquareNeighborhood::Von4,
    }
}

fn set_neighborhood_for_active_kind(state: &mut GridSnapState, value: SquareNeighborhood) {
    match state.kind {
        GridKind::Square => state.square_cfg.neighborhood = value,
        GridKind::Iso => state.iso_cfg.neighborhood = value,
        GridKind::StaggeredSquare => state.staggered_square_cfg.neighborhood = value,
        GridKind::Chunks => state.chunks_cfg.neighborhood = value,
        _ => {}
    }
}

/// Color row — single clickable swatch tile that opens the
/// BlenderColorPicker bound to `GS_COLOR_PICKER` when clicked.
/// Replaces the prior 3× NumberInput (R / G / B) layout per Enio's
/// 2026-05-15 redesign.
#[allow(clippy::too_many_arguments)]
fn paint_color_swatch_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    let label_font = ph2d_tokens::TypeToken::Base.px();
    crate::paint::paint_text(
        text_system,
        scene,
        "Color",
        row.x,
        row.y + (row.h - label_font) * 0.5,
        label_font,
        row.w - 56.0,
        resolve(ColorToken::Text2, theme),
    );
    // Swatch tile on the right — size mirrors Inspector swatches.
    let swatch_w = 48.0_f32;
    let swatch_h = (row.h - 4.0).max(16.0);
    let swatch_rect = Rect::new(
        row.x + row.w - swatch_w,
        row.y + (row.h - swatch_h) * 0.5,
        swatch_w,
        swatch_h,
    );
    let rgba = store
        .widget_color(ids::GS_COLOR_PICKER)
        .unwrap_or(state.color_rgba);
    crate::paint::fill_rounded_rect(
        scene,
        swatch_rect,
        ph2d_tokens::Radius::Sm.px(),
        ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    crate::paint::stroke_rounded_rect(
        scene,
        swatch_rect,
        ph2d_tokens::Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    hit_index.register(ids::GS_COLOR_PICKER, swatch_rect);
}

/// Stable mapping: GridKind → reserved option NodeId, in
/// `GridKind::all()` order. Used by both populate (hit-test
/// registration) and the chip → option resolve in apply_event.
fn kind_option_ids_in_order() -> [(GridKind, crate::NodeId); 9] {
    [
        (GridKind::Square, ids::GS_KIND_OPT_SQUARE),
        (GridKind::Hex, ids::GS_KIND_OPT_HEX),
        (GridKind::Iso, ids::GS_KIND_OPT_ISO),
        (GridKind::StaggeredSquare, ids::GS_KIND_OPT_STAGGERED_SQ),
        (GridKind::StaggeredHex, ids::GS_KIND_OPT_STAGGERED_HEX),
        (GridKind::Tri, ids::GS_KIND_OPT_TRI),
        (GridKind::Quadtree, ids::GS_KIND_OPT_QUADTREE),
        (GridKind::Voronoi, ids::GS_KIND_OPT_VORONOI),
        (GridKind::Chunks, ids::GS_KIND_OPT_CHUNKS),
    ]
}

// =============================================================================
// Per-kind config rows
// =============================================================================

/// Paint the active kind's config rows starting at `y`. Returns the
/// Y after the last row (caller advances).
#[allow(clippy::too_many_arguments)]
fn paint_kind_config(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    match state.kind {
        GridKind::Square => {
            paint_square_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
        GridKind::Hex => paint_hex_cfg(x, w, y, scene, text_system, theme, hit_index, store, state),
        GridKind::Iso => paint_iso_cfg(x, w, y, scene, text_system, theme, hit_index, store, state),
        GridKind::StaggeredSquare => {
            paint_staggered_sq_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
        GridKind::StaggeredHex => {
            paint_hex_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
        GridKind::Tri => paint_tri_cfg(x, w, y, scene, text_system, theme, hit_index, store, state),
        GridKind::Quadtree => {
            paint_quadtree_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
        GridKind::Voronoi => {
            paint_voronoi_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
        GridKind::Chunks => {
            paint_chunks_cfg(x, w, y, scene, text_system, theme, hit_index, store, state)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_square_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Cell size",
        ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row_from_state(
        "Major every",
        ids::GS_CFG_SPACING_MAJOR,
        state.square_cfg.spacing_major as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    paint_neighborhood_button_row(
        x,
        w,
        y,
        NeighborhoodFamily::Square,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_hex_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Cell size",
        ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    let orient = match state.hex_cfg.orientation {
        HexOrientation::Pointy => "Orientation: Pointy \u{25B6}",
        HexOrientation::Flat => "Orientation: Flat \u{25B6}",
    };
    paint_cycle_row(
        ids::GS_CFG_HEX_POINTY,
        orient,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y += ROW_H + ROW_GAP;
    let offset = match state.hex_cfg.offset_variant {
        HexOffset::OddR => "Offset: OddR \u{25B6}",
        HexOffset::EvenR => "Offset: EvenR \u{25B6}",
        HexOffset::OddQ => "Offset: OddQ \u{25B6}",
        HexOffset::EvenQ => "Offset: EvenQ \u{25B6}",
    };
    paint_cycle_row(
        ids::GS_CFG_HEX_OFFSET_DROPDOWN,
        offset,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y + ROW_H + ROW_GAP
}

#[allow(clippy::too_many_arguments)]
fn paint_iso_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Tile width",
        ids::GS_CFG_ISO_TILE_W,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row(
        "Tile height",
        ids::GS_CFG_ISO_TILE_H,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    paint_neighborhood_button_row(
        x,
        w,
        y,
        NeighborhoodFamily::Square,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_staggered_sq_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Cell size",
        ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    let parity = match state.staggered_square_cfg.parity {
        StaggerParity::OddRows => "Parity: Odd rows \u{25B6}",
        StaggerParity::EvenRows => "Parity: Even rows \u{25B6}",
    };
    paint_cycle_row(
        ids::GS_CFG_STAGGER_PARITY_ODD,
        parity,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y += ROW_H + ROW_GAP;
    paint_neighborhood_button_row(
        x,
        w,
        y,
        NeighborhoodFamily::Square,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_tri_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Edge length",
        ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    paint_neighborhood_button_row(
        x,
        w,
        y,
        NeighborhoodFamily::Tri,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_quadtree_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Max / leaf",
        ids::GS_CFG_QT_MAX_PER_LEAF,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row(
        "Max depth",
        ids::GS_CFG_QT_MAX_DEPTH,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_aabb_rows(
        "QT bounds",
        ids::GS_CFG_QT_BOUNDS_MIN_X,
        ids::GS_CFG_QT_BOUNDS_MIN_Y,
        ids::GS_CFG_QT_BOUNDS_MAX_X,
        ids::GS_CFG_QT_BOUNDS_MAX_Y,
        state.quadtree_cfg.bounds.min,
        state.quadtree_cfg.bounds.max,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row_from_state(
        "Demo points",
        ids::GS_CFG_QT_DEMO_POINTS,
        state.quadtree_cfg.demo_point_count as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_number_row_from_state(
        "Demo seed",
        ids::GS_CFG_QT_DEMO_SEED,
        state.quadtree_cfg.demo_rng_seed as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_voronoi_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Seed count",
        ids::GS_CFG_VORONOI_SEED_COUNT,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row(
        "RNG seed",
        ids::GS_CFG_VORONOI_RNG_SEED,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row(
        "Lloyd iters",
        ids::GS_CFG_VORONOI_LLOYD_ITERS,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_aabb_rows(
        "Voronoi bounds",
        ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
        ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
        ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
        ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
        state.voronoi_cfg.bounds.min,
        state.voronoi_cfg.bounds.max,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    // Reseed button.
    let reseed_rect = Rect::new(x, y, w, ROW_H);
    let btn = Button {
        id: ids::GS_CFG_VORONOI_RESEED,
        label: "Reseed (next RNG)".to_string(),
        state: button_state(store, ids::GS_CFG_VORONOI_RESEED),
        kind: ButtonKind::Default,
    };
    paint_button(&btn, reseed_rect, scene, text_system, theme);
    hit_index.register(ids::GS_CFG_VORONOI_RESEED, reseed_rect);
    y + ROW_H + ROW_GAP
}

#[allow(clippy::too_many_arguments)]
fn paint_chunks_cfg(
    x: f32,
    w: f32,
    mut y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    y = paint_number_row(
        "Cell size",
        ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_number_row(
        "Chunk size (cells)",
        ids::GS_CFG_CHUNKS_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y = paint_origin_rows(x, w, y, scene, text_system, theme, hit_index, store, state);
    paint_neighborhood_button_row(
        x,
        w,
        y,
        NeighborhoodFamily::Square,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
    )
}

// =============================================================================
// Row helpers
// =============================================================================

/// Paint a "Label: [NumberInput]" row. Returns Y after the row.
#[allow(clippy::too_many_arguments)]
fn paint_number_row(
    label: &str,
    id: crate::NodeId,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let (state, value, buffer, caret, anchor) = read_number_input(store, id);
    paint_number_row_value(
        label,
        id,
        value,
        Some(buffer),
        caret,
        anchor,
        state,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
    )
}

/// Like [`paint_number_row`] but takes an explicit displayed value
/// (caller-supplied — typically read from state, not the store).
/// Used for "universal" origin / spacing rows that mirror the
/// ACTIVE kind's cfg field even though one NodeId is shared.
#[allow(clippy::too_many_arguments)]
fn paint_number_row_from_state(
    label: &str,
    id: crate::NodeId,
    value: f64,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let (state, _, buffer, caret, anchor) = read_number_input(store, id);
    // Keep the live buffer for in-progress edits; otherwise display
    // the state-supplied value (so switching kinds repaints with the
    // new kind's origin).
    let buffer_arg = if state == TextInputState::Focused {
        Some(buffer)
    } else {
        None
    };
    paint_number_row_value(
        label,
        id,
        value,
        buffer_arg,
        caret,
        anchor,
        state,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
    )
}

#[allow(clippy::too_many_arguments)]
fn paint_number_row_value(
    label: &str,
    id: crate::NodeId,
    value: f64,
    buffer: Option<&str>,
    caret: usize,
    anchor: Option<usize>,
    state: TextInputState,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    paint_text(
        text_system,
        scene,
        label,
        x,
        y + (ROW_H - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        LABEL_COL_W - Spacing::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    let input_rect = Rect::new(x + LABEL_COL_W, y, w - LABEL_COL_W, ROW_H);
    let input = NumberInput::new(id, "", value).state(state);
    paint_number_input_with_buffer(
        &input,
        buffer,
        caret,
        anchor,
        input_rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, input_rect);
    y + ROW_H + ROW_GAP
}

/// Paint Origin X + Origin Y rows reading current values from
/// `state.active_origin()`. Returns Y after the second row.
#[allow(clippy::too_many_arguments)]
fn paint_origin_rows(
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) -> f32 {
    let origin = state.active_origin();
    let y = paint_number_row_from_state(
        "Origin X",
        ids::GS_CFG_ORIGIN_X,
        origin[0] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_number_row_from_state(
        "Origin Y",
        ids::GS_CFG_ORIGIN_Y,
        origin[1] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    )
}

/// Paint a labeled AABB (min X / min Y / max X / max Y) as 4
/// stacked NumberInput rows reading current values from state.
/// Used by Quadtree and Voronoi for their `bounds` field.
#[allow(clippy::too_many_arguments)]
fn paint_aabb_rows(
    label_prefix: &str,
    min_x_id: crate::NodeId,
    min_y_id: crate::NodeId,
    max_x_id: crate::NodeId,
    max_y_id: crate::NodeId,
    min: ph2d_grid::Vec2,
    max: ph2d_grid::Vec2,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min X"),
        min_x_id,
        min[0] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} min Y"),
        min_y_id,
        min[1] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    let y = paint_number_row_from_state(
        &format!("{label_prefix} max X"),
        max_x_id,
        max[0] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    paint_number_row_from_state(
        &format!("{label_prefix} max Y"),
        max_y_id,
        max[1] as f64,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    )
}

/// Paint a full-row cycling Button.
#[allow(clippy::too_many_arguments)]
fn paint_cycle_row(
    id: crate::NodeId,
    label: &str,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let row = Rect::new(x, y, w, ROW_H);
    let btn = Button {
        id,
        label: label.to_string(),
        state: button_state(store, id),
        kind: ButtonKind::Default,
    };
    paint_button(&btn, row, scene, text_system, theme);
    hit_index.register(id, row);
}

fn paint_show_overlay_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_labeled_toggle(
        "Show grid",
        ids::GS_SHOW_OVERLAY,
        state.show_overlay,
        row,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_opacity_slider_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    paint_text(
        text_system,
        scene,
        "Opacity",
        row.x,
        row.y + (row.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        LABEL_COL_W - Spacing::Sm.px(),
        resolve(ColorToken::Text1, theme),
    );
    let slider_rect = Rect::new(
        row.x + LABEL_COL_W,
        row.y + 4.0,
        row.w - LABEL_COL_W,
        row.h - 8.0,
    );
    let (s_state, value) = store
        .slider(ids::GS_OPACITY_SLIDER)
        .unwrap_or((SliderState::Normal, state.opacity));
    let slider = Slider {
        id: ids::GS_OPACITY_SLIDER,
        label: String::new(),
        value,
        state: s_state,
        orientation: SliderOrientation::Horizontal,
        accent: true,
        ticks: Vec::new(),
    };
    paint_slider(&slider, slider_rect, scene, theme);
    hit_index.register(ids::GS_OPACITY_SLIDER, slider_rect);
}

/// "Color" row: 3 small R/G/B NumberInputs + a swatch preview.
/// Alpha is owned by the opacity slider; we never expose it as a
/// separate channel here so the two controls stay orthogonal.
#[allow(clippy::too_many_arguments)]
fn paint_labeled_toggle(
    label: &str,
    id: crate::NodeId,
    on: bool,
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    paint_text(
        text_system,
        scene,
        label,
        row.x,
        row.y + (row.h - LABEL_FONT_SIZE) * 0.5,
        LABEL_FONT_SIZE,
        row.w - 60.0,
        resolve(ColorToken::Text1, theme),
    );

    let toggle_w = 40.0;
    let toggle_h = 20.0;
    let toggle_rect = Rect::new(
        row.x + row.w - toggle_w - 4.0,
        row.y + (row.h - toggle_h) * 0.5,
        toggle_w,
        toggle_h,
    );
    let toggle = Toggle {
        id,
        label: String::new(),
        on,
        state: toggle_state(store, id),
    };
    paint_toggle(&toggle, toggle_rect, scene, theme);
    hit_index.register(id, toggle_rect);
}

// =============================================================================
// Store readers
// =============================================================================

fn button_state(store: &WidgetStore, id: crate::NodeId) -> ButtonState {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    }
}

fn toggle_state(store: &WidgetStore, id: crate::NodeId) -> ToggleState {
    match store.get(id) {
        Some(InteractiveState::Toggle { state, .. }) => *state,
        _ => ToggleState::Normal,
    }
}

fn read_number_input(
    store: &WidgetStore,
    id: crate::NodeId,
) -> (TextInputState, f64, &str, usize, Option<usize>) {
    store
        .number_input(id)
        .unwrap_or((TextInputState::Normal, 0.0, "", 0, None))
}

// =============================================================================
// Event dispatch
// =============================================================================

/// Handle a widget event for the grid-snap panel. Returns `true`
/// when the event mutates `state`. The dispatcher pre-flips Toggle
/// `on` and updates Slider `value` / NumberInput `value` in the
/// store BEFORE calling us; we just mirror those back into state.
pub fn apply_event(state: &mut GridSnapState, event: WidgetEvent, store: &WidgetStore) -> bool {
    match event {
        WidgetEvent::Toggled(id) => apply_toggle(state, id, store),
        WidgetEvent::ValueChanged(id) => apply_value_changed(state, id, store),
        WidgetEvent::Click(id) => apply_click(state, id),
        _ => false,
    }
}

fn apply_toggle(state: &mut GridSnapState, id: crate::NodeId, store: &WidgetStore) -> bool {
    if id == ids::GS_SNAP_ENABLED {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get(id) {
            state.snap_enabled = *on;
        }
        return true;
    }
    if id == ids::GS_SHOW_OVERLAY {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get(id) {
            state.show_overlay = *on;
        }
        return true;
    }
    false
}

fn apply_value_changed(state: &mut GridSnapState, id: crate::NodeId, store: &WidgetStore) -> bool {
    if id == ids::GS_OPACITY_SLIDER {
        if let Some((_, v)) = store.slider(id) {
            state.opacity = v.clamp(0.0, 1.0);
        }
        return true;
    }
    // NumberInput updates — pull value and write to the matching cfg
    // field of the ACTIVE kind. Cross-kind shared ids (CELL_SIZE,
    // NEIGHBORHOOD_4) write to the active kind's field; switching
    // kind re-reads from that kind's cfg on the next paint.
    let Some(v) = store.number_value(id) else {
        return false;
    };
    let v_f32 = v as f32;
    if id == ids::GS_CFG_CELL_SIZE {
        match state.kind {
            GridKind::Square => state.square_cfg.cell_size = v_f32.max(0.01),
            GridKind::Hex => state.hex_cfg.cell_size = v_f32.max(0.01),
            GridKind::StaggeredSquare => {
                state.staggered_square_cfg.cell_w = v_f32.max(0.01);
                state.staggered_square_cfg.cell_h = v_f32.max(0.01);
            }
            GridKind::StaggeredHex => state.staggered_hex_cfg.hex.cell_size = v_f32.max(0.01),
            GridKind::Tri => state.tri_cfg.edge_length = v_f32.max(0.01),
            GridKind::Chunks => state.chunks_cfg.cell_size = v_f32.max(0.01),
            _ => {}
        }
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_W {
        state.iso_cfg.tile_w = v_f32.max(0.01);
        return true;
    }
    if id == ids::GS_CFG_ISO_TILE_H {
        state.iso_cfg.tile_h = v_f32.max(0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_MAX_PER_LEAF {
        state.quadtree_cfg.max_points_per_leaf = (v as usize).max(1);
        return true;
    }
    if id == ids::GS_CFG_QT_MAX_DEPTH {
        state.quadtree_cfg.max_depth = (v as u32).max(1);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_SEED_COUNT {
        state.voronoi_cfg.seed_count = (v as usize).max(3);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_RNG_SEED {
        state.voronoi_cfg.rng_seed = v as u64;
        return true;
    }
    if id == ids::GS_CFG_VORONOI_LLOYD_ITERS {
        state.voronoi_cfg.lloyd_iterations = (v as u32).min(8);
        return true;
    }
    if id == ids::GS_CFG_CHUNKS_SIZE {
        state.chunks_cfg.chunk_size_cells = (v as u32).max(1);
        return true;
    }
    // Universal origin offset — applies to the active kind's cfg.
    if id == ids::GS_CFG_ORIGIN_X {
        write_active_origin_x(state, v_f32);
        return true;
    }
    if id == ids::GS_CFG_ORIGIN_Y {
        write_active_origin_y(state, v_f32);
        return true;
    }
    // Major-line spacing — Square only (other kinds ignore the id).
    if id == ids::GS_CFG_SPACING_MAJOR && state.kind == GridKind::Square {
        state.square_cfg.spacing_major = v_f32.max(state.square_cfg.cell_size);
        return true;
    }
    // Grid color RGB — clamp each channel to 0..=255 and write
    // into state.color_rgba. Alpha (index 3) is owned by the
    // opacity slider; leave it untouched.
    let to_u8 = |v: f64| v.clamp(0.0, 255.0) as u8;
    if id == ids::GS_CFG_COLOR_R {
        state.color_rgba[0] = to_u8(v);
        return true;
    }
    if id == ids::GS_CFG_COLOR_G {
        state.color_rgba[1] = to_u8(v);
        return true;
    }
    if id == ids::GS_CFG_COLOR_B {
        state.color_rgba[2] = to_u8(v);
        return true;
    }
    // Snap subdivisions — clamp to [1, 64] so sub-grid stays useful
    // (1 = no subdivision; 64 covers reasonable half/quarter use).
    if id == ids::GS_CFG_SNAP_SUBDIVISIONS {
        state.snap_subdivisions = (v as u32).clamp(1, 64);
        return true;
    }
    // Probe coords — direct write into state.probe_a/b.
    if id == ids::GS_PROBE_A_X {
        state.probe_a[0] = v_f32;
        return true;
    }
    if id == ids::GS_PROBE_A_Y {
        state.probe_a[1] = v_f32;
        return true;
    }
    if id == ids::GS_PROBE_B_X {
        state.probe_b[0] = v_f32;
        return true;
    }
    if id == ids::GS_PROBE_B_Y {
        state.probe_b[1] = v_f32;
        return true;
    }
    // Quadtree bounds (4 floats) + demo controls. AABB::new debug-
    // asserts only ordering; we soft-clamp max ≥ min to avoid
    // degenerate boxes after edits.
    if id == ids::GS_CFG_QT_BOUNDS_MIN_X {
        state.quadtree_cfg.bounds.min[0] = v_f32;
        state.quadtree_cfg.bounds.max[0] = state.quadtree_cfg.bounds.max[0].max(v_f32 + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MIN_Y {
        state.quadtree_cfg.bounds.min[1] = v_f32;
        state.quadtree_cfg.bounds.max[1] = state.quadtree_cfg.bounds.max[1].max(v_f32 + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_X {
        state.quadtree_cfg.bounds.max[0] = v_f32.max(state.quadtree_cfg.bounds.min[0] + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_BOUNDS_MAX_Y {
        state.quadtree_cfg.bounds.max[1] = v_f32.max(state.quadtree_cfg.bounds.min[1] + 0.01);
        return true;
    }
    if id == ids::GS_CFG_QT_DEMO_POINTS {
        state.quadtree_cfg.demo_point_count = (v as usize).min(4096);
        return true;
    }
    if id == ids::GS_CFG_QT_DEMO_SEED {
        state.quadtree_cfg.demo_rng_seed = v as u64;
        return true;
    }
    // Voronoi bounds (same soft-clamp pattern as Quadtree).
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_X {
        state.voronoi_cfg.bounds.min[0] = v_f32;
        state.voronoi_cfg.bounds.max[0] = state.voronoi_cfg.bounds.max[0].max(v_f32 + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MIN_Y {
        state.voronoi_cfg.bounds.min[1] = v_f32;
        state.voronoi_cfg.bounds.max[1] = state.voronoi_cfg.bounds.max[1].max(v_f32 + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_X {
        state.voronoi_cfg.bounds.max[0] = v_f32.max(state.voronoi_cfg.bounds.min[0] + 0.01);
        return true;
    }
    if id == ids::GS_CFG_VORONOI_BOUNDS_MAX_Y {
        state.voronoi_cfg.bounds.max[1] = v_f32.max(state.voronoi_cfg.bounds.min[1] + 0.01);
        return true;
    }
    false
}

fn write_active_origin_x(state: &mut GridSnapState, v: f32) {
    match state.kind {
        GridKind::Square => state.square_cfg.origin[0] = v,
        GridKind::Hex => state.hex_cfg.origin[0] = v,
        GridKind::Iso => state.iso_cfg.origin[0] = v,
        GridKind::StaggeredSquare => state.staggered_square_cfg.origin[0] = v,
        GridKind::StaggeredHex => state.staggered_hex_cfg.hex.origin[0] = v,
        GridKind::Tri => state.tri_cfg.origin[0] = v,
        GridKind::Chunks => state.chunks_cfg.origin[0] = v,
        // Quadtree/Voronoi use `bounds: AABB` instead; origin is a
        // no-op for them.
        GridKind::Quadtree | GridKind::Voronoi => {}
    }
}

fn write_active_origin_y(state: &mut GridSnapState, v: f32) {
    match state.kind {
        GridKind::Square => state.square_cfg.origin[1] = v,
        GridKind::Hex => state.hex_cfg.origin[1] = v,
        GridKind::Iso => state.iso_cfg.origin[1] = v,
        GridKind::StaggeredSquare => state.staggered_square_cfg.origin[1] = v,
        GridKind::StaggeredHex => state.staggered_hex_cfg.hex.origin[1] = v,
        GridKind::Tri => state.tri_cfg.origin[1] = v,
        GridKind::Chunks => state.chunks_cfg.origin[1] = v,
        GridKind::Quadtree | GridKind::Voronoi => {}
    }
}

fn apply_click(state: &mut GridSnapState, id: crate::NodeId) -> bool {
    if id == ids::GS_CLOSE {
        state.panel_visible = false;
        return true;
    }
    // Kind dropdown option clicks — map option id → GridKind.
    // (Click on the chip itself opens the dropdown; the dispatcher
    // handles open/closed state on Dropdown widgets, so we don't
    // handle GS_KIND_DROPDOWN here.)
    for (kind, opt_id) in kind_option_ids_in_order() {
        if id == opt_id {
            state.kind = kind;
            return true;
        }
    }
    // Target group option clicks (5-button vertical stack).
    if id == ids::GS_SNAP_CENTER {
        state.snap_target = SnapTarget::Center;
        return true;
    }
    if id == ids::GS_SNAP_INTERSECTION {
        state.snap_target = SnapTarget::Intersection;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CORNER {
        state.snap_target = SnapTarget::Corner;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CENTER_AND_INTERSECTION {
        state.snap_target = SnapTarget::CenterAndIntersection;
        return true;
    }
    if id == ids::GS_SNAP_TARGET_OPT_CENTER_INTERSECTION_AND_CORNERS {
        state.snap_target = SnapTarget::CenterIntersectionAndCorners;
        return true;
    }
    // Neighborhood group option clicks (Square family: Von4 / Moore8).
    if id == ids::GS_CFG_NEIGHBORHOOD_4 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Von4);
        return true;
    }
    if id == ids::GS_CFG_NEIGHBORHOOD_8 {
        set_neighborhood_for_active_kind(state, SquareNeighborhood::Moore8);
        return true;
    }
    if id == ids::GS_CFG_HEX_POINTY {
        let hex = if state.kind == GridKind::StaggeredHex {
            &mut state.staggered_hex_cfg.hex
        } else {
            &mut state.hex_cfg
        };
        hex.orientation = match hex.orientation {
            HexOrientation::Pointy => HexOrientation::Flat,
            HexOrientation::Flat => HexOrientation::Pointy,
        };
        return true;
    }
    if id == ids::GS_CFG_HEX_OFFSET_DROPDOWN {
        let hex = if state.kind == GridKind::StaggeredHex {
            &mut state.staggered_hex_cfg.hex
        } else {
            &mut state.hex_cfg
        };
        hex.offset_variant = cycle_hex_offset(hex.offset_variant);
        return true;
    }
    if id == ids::GS_CFG_STAGGER_PARITY_ODD {
        state.staggered_square_cfg.parity = match state.staggered_square_cfg.parity {
            StaggerParity::OddRows => StaggerParity::EvenRows,
            StaggerParity::EvenRows => StaggerParity::OddRows,
        };
        return true;
    }
    // Tri neighborhood group option clicks.
    if id == ids::GS_CFG_TRI_EDGE3 {
        state.tri_cfg.neighborhood = TriNeighborhood::Edge3;
        return true;
    }
    if id == ids::GS_CFG_TRI_VERTEX12 {
        state.tri_cfg.neighborhood = TriNeighborhood::Vertex12;
        return true;
    }
    // Color swatch click — open the BlenderColorPicker bound to
    // GS_COLOR_PICKER. Same pattern as Inspector samples.
    if id == ids::GS_COLOR_PICKER {
        // No state mutation here; the dispatcher sets picker_target.
        return true;
    }
    if id == ids::GS_CFG_VORONOI_RESEED {
        // Bump rng_seed by an odd prime to land on a new pattern
        // without collapsing back to the SplitMix64 starting state
        // for any seen seed.
        state.voronoi_cfg.rng_seed = state.voronoi_cfg.rng_seed.wrapping_add(2_654_435_761);
        return true;
    }
    false
}

fn cycle_hex_offset(o: HexOffset) -> HexOffset {
    match o {
        HexOffset::OddR => HexOffset::EvenR,
        HexOffset::EvenR => HexOffset::OddQ,
        HexOffset::OddQ => HexOffset::EvenQ,
        HexOffset::EvenQ => HexOffset::OddR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn populated_store() -> WidgetStore {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        store
    }

    fn flip_toggle(store: &mut WidgetStore, id: crate::NodeId) {
        if let Some(InteractiveState::Toggle { on, .. }) = store.get_mut(id) {
            *on = !*on;
        }
    }

    #[test]
    fn apply_event_close_clears_visible() {
        let mut s = GridSnapState {
            panel_visible: true,
            ..Default::default()
        };
        let store = populated_store();
        assert!(apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CLOSE),
            &store
        ));
        assert!(!s.panel_visible);
    }

    #[test]
    fn apply_event_kind_option_click_sets_active_kind() {
        let mut s = GridSnapState::default();
        let store = populated_store();
        assert_eq!(s.kind, GridKind::Square);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_HEX), &store);
        assert_eq!(s.kind, GridKind::Hex);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_OPT_VORONOI), &store);
        assert_eq!(s.kind, GridKind::Voronoi);
    }

    #[test]
    fn apply_event_chip_click_is_passthrough_for_dispatcher() {
        // The dispatcher handles Dropdown.open toggling on chip
        // clicks — apply_event must NOT mutate kind on a chip click.
        let mut s = GridSnapState::default();
        let store = populated_store();
        let before = s.kind;
        apply_event(&mut s, WidgetEvent::Click(ids::GS_KIND_DROPDOWN), &store);
        assert_eq!(s.kind, before);
    }

    #[test]
    fn apply_event_snap_toggle_via_toggled() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        flip_toggle(&mut store, ids::GS_SNAP_ENABLED);
        apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SNAP_ENABLED), &store);
        assert!(s.snap_enabled);
    }

    #[test]
    fn apply_event_overlay_toggle_via_toggled() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        let initial = s.show_overlay;
        flip_toggle(&mut store, ids::GS_SHOW_OVERLAY);
        apply_event(&mut s, WidgetEvent::Toggled(ids::GS_SHOW_OVERLAY), &store);
        assert_eq!(s.show_overlay, !initial);
    }

    #[test]
    fn apply_event_unrelated_id_returns_false() {
        let mut s = GridSnapState::default();
        let store = populated_store();
        let unrelated = crate::NodeId(42);
        assert!(!apply_event(&mut s, WidgetEvent::Click(unrelated), &store));
    }

    #[test]
    fn apply_event_number_changed_writes_to_square_cell_size() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        // Set the store's value as the dispatcher would, then fire
        // ValueChanged.
        if let Some(InteractiveState::NumberInput {
            value,
            last_committed,
            buffer,
            ..
        }) = store.get_mut(ids::GS_CFG_CELL_SIZE)
        {
            *value = 4.5;
            *last_committed = 4.5;
            *buffer = "4.5".to_string();
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_CELL_SIZE),
            &store,
        );
        assert!((s.square_cfg.cell_size - 4.5).abs() < 1e-5);
    }

    #[test]
    fn apply_event_neighborhood_options_set_value() {
        // Post-segmented-button semantics: each option id explicitly
        // SETS its value (Von4 / Moore8), no cycling. Idempotent.
        let mut s = GridSnapState::default();
        let store = populated_store();
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_8),
            &store,
        );
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Moore8);
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_NEIGHBORHOOD_4),
            &store,
        );
        assert_eq!(s.square_cfg.neighborhood, SquareNeighborhood::Von4);
    }

    #[test]
    fn apply_event_cycle_hex_orientation() {
        let mut s = GridSnapState {
            kind: GridKind::Hex,
            ..Default::default()
        };
        let store = populated_store();
        assert_eq!(s.hex_cfg.orientation, HexOrientation::Pointy);
        apply_event(&mut s, WidgetEvent::Click(ids::GS_CFG_HEX_POINTY), &store);
        assert_eq!(s.hex_cfg.orientation, HexOrientation::Flat);
    }

    #[test]
    fn apply_event_cycle_hex_offset_walks_four_states() {
        let mut s = GridSnapState {
            kind: GridKind::Hex,
            ..Default::default()
        };
        let store = populated_store();
        let order = [
            HexOffset::OddR,
            HexOffset::EvenR,
            HexOffset::OddQ,
            HexOffset::EvenQ,
            HexOffset::OddR,
        ];
        for expected in &order[1..] {
            apply_event(
                &mut s,
                WidgetEvent::Click(ids::GS_CFG_HEX_OFFSET_DROPDOWN),
                &store,
            );
            assert_eq!(s.hex_cfg.offset_variant, *expected);
        }
    }

    #[test]
    fn apply_event_color_r_writes_state() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_COLOR_R)
        {
            *value = 200.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_R),
            &store,
        );
        assert_eq!(s.color_rgba[0], 200);
        // Other channels untouched.
        let defaults = GridSnapState::default();
        assert_eq!(s.color_rgba[1], defaults.color_rgba[1]);
        assert_eq!(s.color_rgba[2], defaults.color_rgba[2]);
        assert_eq!(s.color_rgba[3], defaults.color_rgba[3]);
    }

    #[test]
    fn apply_event_subdivisions_clamps_to_unit_floor() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_SNAP_SUBDIVISIONS)
        {
            *value = 0.0; // floor enforced to 1
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_SNAP_SUBDIVISIONS),
            &store,
        );
        assert_eq!(s.snap_subdivisions, 1);
    }

    #[test]
    fn apply_event_subdivisions_actually_subdivides_snap() {
        // With cell_size=1 and subdivisions=2, snap_to_center should
        // pull to half-cell centers. World [0.3, 0.3] is in cell
        // (0, 0) of the sub-grid (cells span 0..0.5) → center
        // (0.25, 0.25), not (0.5, 0.5).
        let mut s = GridSnapState {
            snap_enabled: true,
            snap_target: SnapTarget::Center,
            snap_subdivisions: 2,
            ..Default::default()
        };
        let p = s.snap_world([0.3, 0.3], [0.0, 0.0]);
        assert!(
            (p[0] - 0.25).abs() < 1e-5 && (p[1] - 0.25).abs() < 1e-5,
            "expected [0.25, 0.25] with subdivisions=2; got {p:?}"
        );
    }

    #[test]
    fn apply_event_probe_a_x_writes_state() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(ids::GS_PROBE_A_X)
        {
            *value = 7.5;
        }
        apply_event(&mut s, WidgetEvent::ValueChanged(ids::GS_PROBE_A_X), &store);
        assert!((s.probe_a[0] - 7.5).abs() < 1e-5);
    }

    #[test]
    fn apply_event_qt_bounds_max_soft_clamps_to_min_plus_eps() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        // Try to set MAX below MIN — should snap up to min+eps.
        let min_x = s.quadtree_cfg.bounds.min[0];
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_QT_BOUNDS_MAX_X)
        {
            *value = min_x as f64 - 5.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_QT_BOUNDS_MAX_X),
            &store,
        );
        assert!(
            s.quadtree_cfg.bounds.max[0] > s.quadtree_cfg.bounds.min[0],
            "MAX must stay > MIN after clamp; got min={} max={}",
            s.quadtree_cfg.bounds.min[0],
            s.quadtree_cfg.bounds.max[0]
        );
    }

    #[test]
    fn apply_event_color_clamps_above_255() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::NumberInput { value, .. }) =
            store.get_mut(ids::GS_CFG_COLOR_B)
        {
            *value = 999.0;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_CFG_COLOR_B),
            &store,
        );
        assert_eq!(s.color_rgba[2], 255);
    }

    #[test]
    fn apply_event_voronoi_reseed_changes_rng_seed() {
        let mut s = GridSnapState {
            kind: GridKind::Voronoi,
            ..Default::default()
        };
        let store = populated_store();
        let before = s.voronoi_cfg.rng_seed;
        apply_event(
            &mut s,
            WidgetEvent::Click(ids::GS_CFG_VORONOI_RESEED),
            &store,
        );
        assert_ne!(s.voronoi_cfg.rng_seed, before);
    }

    #[test]
    fn opacity_slider_value_changed_clamps_to_unit() {
        let mut s = GridSnapState::default();
        let mut store = populated_store();
        if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(ids::GS_OPACITY_SLIDER)
        {
            *value = 1.5;
        }
        apply_event(
            &mut s,
            WidgetEvent::ValueChanged(ids::GS_OPACITY_SLIDER),
            &store,
        );
        assert!((s.opacity - 1.0).abs() < 1e-5);
    }
}
