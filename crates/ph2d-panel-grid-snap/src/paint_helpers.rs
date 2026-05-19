//! Generic paint helpers (segmented buttons, labeled rows, swatches).
//!
//! Ported verbatim from
//! `ph2d_editor_core::grid_snap::panel::paint_helpers` during ADR-0029
//! Phase C.4. Internal-only — re-exported via `crate::paint`.

use crate::ids;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState};
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text, paint_text_centered, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::widget::{Button, ButtonKind, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_grid::snap::SnapTarget;
use ph2d_grid::square::SquareNeighborhood;
use ph2d_grid::tri::TriNeighborhood;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

use super::layout::ROW_GAP;

pub(crate) fn paint_section_label(
    label: &str,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    ph2d_editor_core::paint::paint_text_title(
        text_system,
        scene,
        label,
        x,
        y,
        ph2d_tokens::TypeToken::Md.px(),
        w,
        resolve(ColorToken::Text1, theme),
    );
    let after_title_y = y + ph2d_tokens::TypeToken::Md.px() + 8.0;
    // Accent-colored separator line — same visual as Inspector's
    // `paint_section_separator` (Border token reads as invisible on
    // dark themes; Accent reads as the canonical section divider).
    let sep_pad_x = 2.0_f32;
    let sep = Rect::new(
        x + sep_pad_x,
        after_title_y,
        (w - sep_pad_x * 2.0).max(0.0),
        1.0,
    );
    fill_rounded_rect(scene, sep, 0.5, resolve(ColorToken::Accent, theme));
    after_title_y + 1.0 + 8.0
}

/// Big individual Snap toggle at the top of the panel — primary
/// action chip that flips `state.snap_enabled`. Full-width;
/// `Accent` fill + `AccentFg` text when ON (canonical primary CTA
/// look — `AccentFg` is the contrast pair `Accent` was designed
/// against), `BgElev` + Border + `Text2` when OFF. Returns the Y
/// after the row + gap.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_snap_top_toggle(
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
    fill_rounded_rect(scene, rect, radius, resolve(fill_tok, theme));
    stroke_rounded_rect(scene, rect, radius, 1.5, resolve(border_tok, theme));
    let label = if on { "Snap: ON" } else { "Snap: OFF" };
    let font = ph2d_tokens::TypeToken::Md.px();
    paint_text_centered(
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
pub(crate) fn paint_segmented_button(
    rect: Rect,
    label: &str,
    pressed: bool,
    id: NodeId,
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
pub(crate) fn paint_kind_button_grid(
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
    let entries: [(GridKind, &str, NodeId); 9] = [
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
pub(crate) fn paint_target_button_stack(
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
    let entries: [(SnapTarget, &str, NodeId); 5] = [
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
pub(crate) fn paint_neighborhood_button_row(
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
    // Label above the row — without it the two buttons read as
    // anonymous (Von4 / Moore8 don't self-explain). Uses the same
    // small Text2 label style as Inspector property labels.
    let label_font = ph2d_tokens::TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        "Neighborhood",
        x,
        y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let y = y + label_font + 4.0;

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

/// Generic labeled segmented-button row — renders a small Text2 label
/// above a horizontal stack of equal-width buttons. The button at
/// `active_idx` is painted with `ButtonState::Pressed`. Used for Hex
/// Orientation (2 opts), Hex Offset (4 opts) and Stagger Parity (2 opts).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_labeled_segmented_row(
    label: &str,
    options: &[(&str, NodeId)],
    active_idx: usize,
    x: f32,
    w: f32,
    y: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> f32 {
    let label_font = ph2d_tokens::TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let y = y + label_font + 4.0;

    let h = 28.0_f32;
    let gap = 6.0_f32;
    let n = options.len() as f32;
    let cell_w = ((w - gap * (n - 1.0)) / n).max(40.0);
    for (i, (lbl, oid)) in options.iter().enumerate() {
        let rx = x + i as f32 * (cell_w + gap);
        let rect = Rect::new(rx, y, cell_w, h);
        paint_segmented_button(
            rect,
            lbl,
            i == active_idx,
            *oid,
            scene,
            text_system,
            theme,
            hit_index,
            store,
        );
    }
    y + h + ROW_GAP
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum NeighborhoodFamily {
    Square,
    Tri,
}

pub(crate) fn neighborhood_for_active_kind(state: &GridSnapState) -> SquareNeighborhood {
    match state.kind {
        GridKind::Square => state.square_cfg.neighborhood,
        GridKind::Iso => state.iso_cfg.neighborhood,
        GridKind::StaggeredSquare => state.staggered_square_cfg.neighborhood,
        GridKind::Chunks => state.chunks_cfg.neighborhood,
        _ => SquareNeighborhood::Von4,
    }
}

pub(crate) fn set_neighborhood_for_active_kind(
    state: &mut GridSnapState,
    value: SquareNeighborhood,
) {
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
pub(crate) fn paint_color_swatch_row(
    row: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    state: &GridSnapState,
) {
    let label_font = ph2d_tokens::TypeToken::Base.px();
    paint_text(
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
    fill_rounded_rect(
        scene,
        swatch_rect,
        ph2d_tokens::Radius::Sm.px(),
        ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    stroke_rounded_rect(
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
pub(crate) fn kind_option_ids_in_order() -> [(GridKind, NodeId); 9] {
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

pub(crate) fn button_state(store: &WidgetStore, id: NodeId) -> ButtonState {
    match store.get(id) {
        Some(InteractiveState::Button { state }) => *state,
        _ => ButtonState::Normal,
    }
}

pub(crate) fn toggle_state(
    store: &WidgetStore,
    id: NodeId,
) -> ph2d_editor_core::widget::ToggleState {
    match store.get(id) {
        Some(InteractiveState::Toggle { state, .. }) => *state,
        _ => ph2d_editor_core::widget::ToggleState::Normal,
    }
}
