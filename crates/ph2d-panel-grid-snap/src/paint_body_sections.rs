//! Per-section paint helpers for the Grid Snap docked panel.
//!
//! Wave 11 §2.2 (ADR-0042 §6 #3) — split of the 300-LOC
//! `paint_body()` orchestrator into 4 section helpers (Grid Kind /
//! Target / Display / Inspect). Same recipe as the CEQ + BgRemoval
//! splits: each helper takes `&mut PaintCtx`, the live `GridSnapState`,
//! geometry params, and a `y_in: f32` cursor; returns `y_out: f32`.

use crate::ids;
use crate::layout::{ROW_GAP, ROW_H};
use crate::paint_helpers::{
    paint_color_swatch_row, paint_kind_button_grid, paint_labeled_segmented_row,
    paint_section_label, paint_target_button_stack,
};
use crate::paint_kinds::paint_kind_config;
use crate::paint_rows::{
    paint_number_row_from_state, paint_opacity_slider_row, paint_show_overlay_row,
};
use crate::state::{meters_to_display, unit_suffix_paren};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;

/// Grid Kind label + 3×3 button grid + per-kind config rows.
pub(crate) fn paint_grid_kind_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    y = paint_section_label(
        "Grid Kind",
        inner_x,
        inner_w,
        y,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_kind_button_grid(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ROW_GAP;
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_kind_config(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ROW_GAP * 2.0;
    y
}

/// Target section: label + button stack (Center / Intersection / …) +
/// Subdivisions + Magnetism Radius number rows.
pub(crate) fn paint_target_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    y = paint_section_label(
        "Target",
        inner_x,
        inner_w,
        y,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_target_button_stack(
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
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
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        );
        let mag_label = format!("Magnetism radius{}", unit_suffix_paren());
        y = paint_number_row_from_state(
            &mag_label,
            ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
            meters_to_display(state.snap_magnetism_radius),
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        );
    }
    y += ROW_GAP * 2.0;
    y
}

/// Display section: label + Show Overlay toggle + Opacity slider +
/// Color swatch + Layer segmented (In front / Behind).
pub(crate) fn paint_display_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    y = paint_section_label(
        "Display",
        inner_x,
        inner_w,
        y,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    let overlay_row = Rect::new(inner_x, y, inner_w, ROW_H);
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_show_overlay_row(
            overlay_row,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ROW_H + ROW_GAP;

    let opacity_row = Rect::new(inner_x, y, inner_w, ROW_H);
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_opacity_slider_row(
            opacity_row,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ROW_H + ROW_GAP;

    let color_row = Rect::new(inner_x, y, inner_w, ROW_H);
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_color_swatch_row(
            color_row,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
        );
    }
    y += ROW_H + ROW_GAP;

    let layer_idx = if state.grid_in_front { 0 } else { 1 };
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_labeled_segmented_row(
            "Layer",
            &[
                ("In front", ids::GS_LAYER_IN_FRONT),
                ("Behind", ids::GS_LAYER_BEHIND),
            ],
            layer_idx,
            inner_x,
            inner_w,
            y,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
        );
    }
    y += ROW_GAP;
    y
}

/// Inspect section: delegates to `ph2d_editor_core::grid_snap::inspect`
/// (the shared inspector painter — height fixed by the editor-core).
pub(crate) fn paint_inspect_section(
    ctx: &mut PaintCtx<'_>,
    state: &GridSnapState,
    display_unit: ph2d_editor_core::project::DisplayUnit,
    ppm: f32,
    inner_x: f32,
    inner_w: f32,
    mut y: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let inspect_h = ph2d_editor_core::grid_snap::inspect::height();
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        ph2d_editor_core::grid_snap::inspect::paint(
            Rect::new(inner_x, y, inner_w, inspect_h),
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            store,
            state,
            display_unit,
            ppm,
        );
    }
    y += inspect_h;
    y
}
