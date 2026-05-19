//! `paint_kind_config` dispatch + 8 per-kind config painters.
//!
//! Ported verbatim from `ph2d_editor_core::grid_snap::panel::paint_kinds`
//! during ADR-0029 Phase C.4.

use crate::ids;
use crate::layout::{ROW_GAP, ROW_H};
use crate::paint_helpers::{
    NeighborhoodFamily, paint_labeled_segmented_row, paint_neighborhood_button_row,
};
use crate::paint_rows::{
    button_state, paint_aabb_rows, paint_number_row, paint_number_row_from_state, paint_origin_rows,
};
use crate::state::unit_suffix_paren;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_grid::hex::{HexOffset, HexOrientation};
use ph2d_grid::staggered::StaggerParity;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_kind_config(
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
pub(crate) fn paint_square_cfg(
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
        &format!("Cell size{}", unit_suffix_paren()),
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
        &format!("Major every{}", unit_suffix_paren()),
        ids::GS_CFG_SPACING_MAJOR,
        crate::state::meters_to_display(state.square_cfg.spacing_major),
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
pub(crate) fn paint_hex_cfg(
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
        &format!("Cell size{}", unit_suffix_paren()),
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
    let hex = if state.kind == GridKind::StaggeredHex {
        &state.staggered_hex_cfg.hex
    } else {
        &state.hex_cfg
    };
    let orient_idx = match hex.orientation {
        HexOrientation::Pointy => 0,
        HexOrientation::Flat => 1,
    };
    y = paint_labeled_segmented_row(
        "Orientation",
        &[
            ("Pointy", ids::GS_CFG_HEX_POINTY),
            ("Flat", ids::GS_CFG_HEX_FLAT),
        ],
        orient_idx,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    let offset_idx = match hex.offset_variant {
        HexOffset::OddR => 0,
        HexOffset::EvenR => 1,
        HexOffset::OddQ => 2,
        HexOffset::EvenQ => 3,
    };
    paint_labeled_segmented_row(
        "Offset",
        &[
            ("OddR", ids::GS_CFG_HEX_OFFSET_ODDR),
            ("EvenR", ids::GS_CFG_HEX_OFFSET_EVENR),
            ("OddQ", ids::GS_CFG_HEX_OFFSET_ODDQ),
            ("EvenQ", ids::GS_CFG_HEX_OFFSET_EVENQ),
        ],
        offset_idx,
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
pub(crate) fn paint_iso_cfg(
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
    let suffix = unit_suffix_paren();
    y = paint_number_row(
        &format!("Tile width{suffix}"),
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
        &format!("Tile height{suffix}"),
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
pub(crate) fn paint_staggered_sq_cfg(
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
        &format!("Cell size{}", unit_suffix_paren()),
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
    let parity_idx = match state.staggered_square_cfg.parity {
        StaggerParity::OddRows => 0,
        StaggerParity::EvenRows => 1,
    };
    y = paint_labeled_segmented_row(
        "Parity",
        &[
            ("Odd rows", ids::GS_CFG_STAGGER_PARITY_ODD),
            ("Even rows", ids::GS_CFG_STAGGER_PARITY_EVEN),
        ],
        parity_idx,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
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
pub(crate) fn paint_tri_cfg(
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
        &format!("Edge length{}", unit_suffix_paren()),
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
pub(crate) fn paint_quadtree_cfg(
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
pub(crate) fn paint_voronoi_cfg(
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
pub(crate) fn paint_chunks_cfg(
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
        &format!("Cell size{}", unit_suffix_paren()),
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
