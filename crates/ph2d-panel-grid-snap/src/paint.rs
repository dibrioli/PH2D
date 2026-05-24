//! Grid Snap floating panel paint orchestrator — ADR-0029 Phase C.4
//! port of the legacy `paint_thunk` + `grid_snap::panel::paint`.
//!
//! Full per-frame logic:
//! - Visibility gate via [`PanelHostInternal::panel_visible`].
//! - Stale-rect cleanup on hide (mirrors GAL_PANEL).
//! - Lazy default rect on first paint (writes back to
//!   `host.store_mut().grid_snap_panel_rect()` — see below).
//! - Drag / resize clamp via `widget::panel_chrome::clamp_panel_rect`.
//! - Chrome publish (`set_panel_rect`).
//! - Meter↔DisplayUnit reseed (`sync_meter_inputs_to_display_unit`) +
//!   thread-local pair via `state::set_current_display_unit`.
//! - Full chrome body paint (Snap top toggle / Kind grid / per-kind
//!   config / Target stack / Subdivisions / Magnetism / Display
//!   section / Inspect section) inside a vertical scroll clip.
//! - `content_h` / `visible_h` publish + scroll clamp.

use crate::GridSnapPanel;
use crate::ids;
use crate::layout::{PAD, ROW_GAP, ROW_H};
use crate::paint_helpers::{
    paint_color_swatch_row, paint_kind_button_grid, paint_labeled_segmented_row,
    paint_section_label, paint_snap_top_toggle, paint_target_button_stack,
};
use crate::paint_kinds::paint_kind_config;
use crate::paint_rows::{
    paint_number_row_from_state, paint_opacity_slider_row, paint_show_overlay_row,
};
use crate::state::{
    GridSnapPanelState, meters_to_display, set_current_display_unit, set_last_content_h,
    set_last_visible_h, unit_suffix_paren,
};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{paint_icon, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_TITLE_BASELINE, clamp_panel_rect, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
};
use ph2d_editor_core::widget::{
    GRID_SETTINGS_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Spacing, StrokeToken};

pub(crate) fn paint(_state: &mut GridSnapPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(GridSnapPanel::ID) {
        // Symmetric stale-rect cleanup (mirrors GAL_PANEL): without
        // this, `panel_at` keeps returning GS_PANEL after the panel
        // was hidden.
        ctx.host.store_mut().clear_panel_rect(ids::GS_PANEL);
        return;
    }
    // Resolve the persisted panel rect — Phase C.4 keeps it on the
    // editor-core canvas-renderer `GridSnapState` (not duplicated
    // here). We read + write via the host's typed accessor below.
    let viewport = ctx.viewport;
    let base_rect = current_or_default_rect(ctx, viewport);

    let gs_off = ctx.host.store().blender_picker_offset(ids::GS_PANEL);
    let gs_resize = ctx.host.store().panel_resize_delta(ids::GS_PANEL);
    let (gs_rect, gs_clamped_off, gs_clamped_resize) =
        clamp_panel_rect(base_rect, gs_off, gs_resize, viewport);
    {
        let store = ctx.host.store_mut();
        if (gs_clamped_off.0 - gs_off.0).abs() > f32::EPSILON
            || (gs_clamped_off.1 - gs_off.1).abs() > f32::EPSILON
        {
            store.set_blender_picker_offset(ids::GS_PANEL, gs_clamped_off.0, gs_clamped_off.1);
        }
        if (gs_clamped_resize.0 - gs_resize.0).abs() > f32::EPSILON
            || (gs_clamped_resize.1 - gs_resize.1).abs() > f32::EPSILON
        {
            store.set_panel_resize_delta(ids::GS_PANEL, gs_clamped_resize.0, gs_clamped_resize.1);
        }
        store.set_panel_rect(ids::GS_PANEL, gs_rect);
    }

    // Publish the active DisplayUnit + pixels_per_meter into the
    // panel's thread-local pair (read by meter↔display conversion
    // helpers inside `paint` / `apply_event`), then reseed every
    // meter-domain NumberInput's store value from the live state so
    // the rows that read from the store display the right magnitude
    // for the active unit. Focused fields are skipped by
    // `set_number_value` so in-progress edits don't get clobbered.
    let display_unit = ctx.host.project().display_unit;
    let ppm = ctx.host.project().pixels_per_meter;
    set_current_display_unit(display_unit, ppm);
    let snap_state = ctx.host.grid_snap_state_clone();
    {
        let store = ctx.host.store_mut();
        sync_meter_inputs_to_display_unit_impl(&snap_state, store);
    }

    paint_body(ctx, gs_rect, &snap_state, display_unit, ppm);
}

/// Compute the panel rect to use this frame. If editor-core's
/// `GridSnapState::panel_rect` is None, lazily initialize it from
/// [`default_rect`] and persist back. Returns the rect to clamp.
fn current_or_default_rect(ctx: &mut PaintCtx<'_>, viewport: Rect) -> Rect {
    let layout_w = ctx.layout.viewport.w;
    let layout_h = ctx.layout.viewport.h;
    let cur = ctx.host.grid_snap_panel_rect();
    let _ = viewport; // viewport kept for parity with clamp call later
    match cur {
        Some(r) => r,
        None => {
            let r = default_rect(layout_w, layout_h);
            ctx.host.set_grid_snap_panel_rect(Some(r));
            r
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_body(
    ctx: &mut PaintCtx<'_>,
    rect: Rect,
    state: &GridSnapState,
    display_unit: ph2d_editor_core::project::DisplayUnit,
    ppm: f32,
) {
    let theme = ctx.host.theme();
    paint_panel_surface(rect, ctx.scene, theme);
    {
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ids::GS_DRAG_HANDLE, panel_drag_handle_rect(rect));
    }

    let inner_x = rect.x + PAD;
    let inner_w = rect.w - PAD * 2.0;
    // Canonical panel title (single source of truth — `panel_chrome`).
    // Reserve ~close-button width on the right.
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    paint_panel_title(
        rect,
        "Grid Settings",
        Spacing::Xl3.px(),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let close_size = Density::Compact.row_h_px();
    let close_rect = Rect::new(
        rect.x + rect.w - close_size - PAD,
        title_y - 2.0,
        close_size,
        close_size,
    );
    paint_icon(
        ctx.scene,
        ph2d_editor_core::icons::IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    {
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ids::GS_CLOSE, close_rect);
    }

    // Body rect — sits below the title row, above the corner dot.
    let body_top = title_y + close_size + ROW_GAP * 2.0;
    let body_h = (rect.y + rect.h - body_top - PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::GS_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let mut y = body_top - scroll;

    // ─── Snap (BIG individual toggle) ───────────────────────────
    {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        y = paint_snap_top_toggle(
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

    // ─── Grid Kind section ──────────────────────────────────────
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

    // Per-kind config rows.
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

    // ─── Target section ─────────────────────────────────────────
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

    // ─── Display section ────────────────────────────────────────
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
    // Layer-order segmented row (In front / Behind).
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

    // ─── Inspect section ────────────────────────────────────────
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

    ctx.scene.pop_layer();

    // Content / visible-height publication for `dispatch_wheel`
    // scroll bound: stash into thread-locals; the host caller reads
    // via `state::last_content_h()` / `last_visible_h()` and publishes
    // through `&mut store` further below.
    let content_h = (y + scroll) - body_top;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    // Vertical scrollbar at the right edge of the body region —
    // mirrors the Inspector / Widget Gallery setup. Skips paint +
    // hit registration when the content fits the visible area.
    let needs_scrollbar = scrollbar_is_needed(content_h, body_h);
    if needs_scrollbar {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active =
            matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == ids::GS_PANEL);
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        {
            let hit_index = ctx.host.hit_index_mut();
            hit_index.register(GRID_SETTINGS_SCROLLBAR_ID, thumb);
        }
    }

    paint_panel_corner_dot(rect, ctx.scene, theme);
    {
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ids::GS_RESIZE_HANDLE, panel_resize_handle_rect(rect));
    }

    // Publish content_h / visible_h to the store + clamp scroll.
    {
        let store = ctx.host.store_mut();
        store.set_panel_content_h(ids::GS_PANEL, content_h);
        store.set_panel_visible_h(ids::GS_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        let cur = store.panel_scroll(ids::GS_PANEL);
        if cur > max_scroll {
            store.set_panel_scroll(ids::GS_PANEL, max_scroll);
        }
    }
}

/// Default panel rect when first opened — sized for title + 4
/// sections (Kind + per-kind config + Snap + Display + Inspect).
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    // Width matches Inspector (`style::INSPECTOR_W = 304`) per Enio's
    // 2026-05-15 redesign so the two floating panels read as siblings.
    let w = 304.0_f32.min(viewport_w - 16.0); // LITERAL-PX-OK: Inspector-matched floating-panel width (304) + 16 viewport edge inset
    let h = 640.0_f32.min(viewport_h - 16.0).max(440.0); // LITERAL-PX-OK: default panel height (640) clamped to viewport with 440 floor + 16 inset
    let x = ((viewport_w - w) * 0.5).max(8.0); // LITERAL-PX-OK: 8 = minimum panel edge inset
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: 8 = minimum panel edge inset
    Rect::new(x, y, w, h)
}

/// Reseed the store's NumberInput values for every meter-domain
/// field from the live state, converted through the active
/// DisplayUnit. Called by the host before `paint` so the rows that
/// read from the store (via `read_number_input`) display the right
/// magnitude for the current unit.
fn sync_meter_inputs_to_display_unit_impl(state: &GridSnapState, store: &mut WidgetStore) {
    use ph2d_editor_core::grid_snap::GridKind;
    let cell_size_m: Option<f32> = match state.kind {
        GridKind::Square => Some(state.square_cfg.cell_size),
        GridKind::Hex => Some(state.hex_cfg.cell_size),
        GridKind::StaggeredSquare => Some(state.staggered_square_cfg.cell_w),
        GridKind::StaggeredHex => Some(state.staggered_hex_cfg.hex.cell_size),
        GridKind::Tri => Some(state.tri_cfg.edge_length),
        GridKind::Chunks => Some(state.chunks_cfg.cell_size),
        GridKind::Iso | GridKind::Quadtree | GridKind::Voronoi => None,
    };
    if let Some(m) = cell_size_m {
        store.set_number_value(ids::GS_CFG_CELL_SIZE, meters_to_display(m));
    }
    store.set_number_value(
        ids::GS_CFG_ISO_TILE_W,
        meters_to_display(state.iso_cfg.tile_w),
    );
    store.set_number_value(
        ids::GS_CFG_ISO_TILE_H,
        meters_to_display(state.iso_cfg.tile_h),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MIN_X,
        meters_to_display(state.quadtree_cfg.bounds.min[0]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MIN_Y,
        meters_to_display(state.quadtree_cfg.bounds.min[1]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MAX_X,
        meters_to_display(state.quadtree_cfg.bounds.max[0]),
    );
    store.set_number_value(
        ids::GS_CFG_QT_BOUNDS_MAX_Y,
        meters_to_display(state.quadtree_cfg.bounds.max[1]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MIN_X,
        meters_to_display(state.voronoi_cfg.bounds.min[0]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MIN_Y,
        meters_to_display(state.voronoi_cfg.bounds.min[1]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MAX_X,
        meters_to_display(state.voronoi_cfg.bounds.max[0]),
    );
    store.set_number_value(
        ids::GS_CFG_VORONOI_BOUNDS_MAX_Y,
        meters_to_display(state.voronoi_cfg.bounds.max[1]),
    );
    store.set_number_value(ids::GS_PROBE_A_X, meters_to_display(state.probe_a[0]));
    store.set_number_value(ids::GS_PROBE_A_Y, meters_to_display(state.probe_a[1]));
    store.set_number_value(ids::GS_PROBE_B_X, meters_to_display(state.probe_b[0]));
    store.set_number_value(ids::GS_PROBE_B_Y, meters_to_display(state.probe_b[1]));
    store.set_number_value(
        ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
        meters_to_display(state.snap_magnetism_radius),
    );
}

// Public alias so external sites (host reseed before paint) can still
// call into the unit-aware reseed. Mirrors the legacy
// `crate::grid_snap::sync_meter_inputs_to_display_unit`.
pub fn sync_meter_inputs_to_display_unit_pub(state: &GridSnapState, store: &mut WidgetStore) {
    sync_meter_inputs_to_display_unit_impl(state, store)
}
