//! `paint_kind_config` dispatch + 8 per-kind config painters.
//!
//! Ported verbatim from `ph2d_editor_core::grid_snap::panel::paint_kinds`
//! during ADR-0029 Phase C.4.

use crate::paint_helpers::{
    NeighborhoodFamily, paint_labeled_segmented_row, paint_neighborhood_button_row,
};
use crate::paint_rows::{paint_number_row, paint_number_row_from_state, paint_origin_rows};
use crate::state::unit_suffix_paren;
use ph2d_editor_core::grid_snap::{GridKind, GridSnapState};
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
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
        GridKind::Quadtree => crate::paint_kinds_bounded::paint_quadtree_cfg(
            x,
            w,
            y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            state,
        ),
        GridKind::Voronoi => crate::paint_kinds_bounded::paint_voronoi_cfg(
            x,
            w,
            y,
            scene,
            text_system,
            theme,
            hit_index,
            store,
            state,
        ),
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let cell = format!("Cell size{}", unit_suffix_paren());
    let major = format!("Major every{}", unit_suffix_paren());
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(text_system, &[&cell, &major, &origem[0], &origem[1]]);
    y = paint_number_row(
        &cell,
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_number_row_from_state(
        &major,
        ph2d_editor_core::grid_snap::ids::GS_CFG_SPACING_MAJOR,
        crate::state::meters_to_display(state.square_cfg.spacing_major),
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let cell = format!("Cell size{}", unit_suffix_paren());
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(text_system, &[&cell, &origem[0], &origem[1]]);
    y = paint_number_row(
        &cell,
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
    );
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
            (
                "Pointy",
                ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_POINTY,
            ),
            ("Flat", ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_FLAT),
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
            (
                "OddR",
                ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_ODDR,
            ),
            (
                "EvenR",
                ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_EVENR,
            ),
            (
                "OddQ",
                ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_ODDQ,
            ),
            (
                "EvenQ",
                ph2d_editor_core::grid_snap::ids::GS_CFG_HEX_OFFSET_EVENQ,
            ),
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let suffix = unit_suffix_paren();
    let tw = format!("Tile width{suffix}");
    let th = format!("Tile height{suffix}");
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(text_system, &[&tw, &th, &origem[0], &origem[1]]);
    y = paint_number_row(
        &tw,
        ph2d_editor_core::grid_snap::ids::GS_CFG_ISO_TILE_W,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_number_row(
        &th,
        ph2d_editor_core::grid_snap::ids::GS_CFG_ISO_TILE_H,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let cell = format!("Cell size{}", unit_suffix_paren());
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(text_system, &[&cell, &origem[0], &origem[1]]);
    y = paint_number_row(
        &cell,
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
    );
    let parity_idx = match state.staggered_square_cfg.parity {
        StaggerParity::OddRows => 0,
        StaggerParity::EvenRows => 1,
    };
    y = paint_labeled_segmented_row(
        "Parity",
        &[
            (
                "Odd rows",
                ph2d_editor_core::grid_snap::ids::GS_CFG_STAGGER_PARITY_ODD,
            ),
            (
                "Even rows",
                ph2d_editor_core::grid_snap::ids::GS_CFG_STAGGER_PARITY_EVEN,
            ),
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let edge = format!("Edge length{}", unit_suffix_paren());
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(text_system, &[&edge, &origem[0], &origem[1]]);
    y = paint_number_row(
        &edge,
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
    );
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
    // ⭐⭐ **A coluna é da SECÇÃO** — ver `paint_rows::seccao` e o report do dono de 2026-09-15.
    let cell = format!("Cell size{}", unit_suffix_paren());
    let origem = crate::paint_rows::origin_labels();
    let sec = crate::paint_rows::seccao(
        text_system,
        &[&cell, "Chunk size (cells)", &origem[0], &origem[1]],
    );
    y = paint_number_row(
        &cell,
        ph2d_editor_core::grid_snap::ids::GS_CFG_CELL_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_number_row(
        "Chunk size (cells)",
        ph2d_editor_core::grid_snap::ids::GS_CFG_CHUNKS_SIZE,
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        sec,
    );
    y = paint_origin_rows(
        x,
        w,
        y,
        scene,
        text_system,
        theme,
        hit_index,
        store,
        state,
        sec,
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
