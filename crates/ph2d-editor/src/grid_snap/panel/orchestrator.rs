//! `paint()` — Grid Snap floating panel paint orchestrator.
//!
//! Extracted from `panel/mod.rs` in Wave 2.5 PR 11.7a to bring the
//! parent module under the HR-18 hygiene cap. Same private surface
//! as before; re-exported by `mod.rs` so call sites are unchanged.

use super::*;

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
    // Magnetism radius (world meters). 0 = always-snap (no gate).
    // Label carries the unit suffix so the user sees the active
    // DisplayUnit alongside the number.
    let mag_label = format!("Magnetism radius{}", unit_suffix_paren());
    y = paint_number_row_from_state(
        &mag_label,
        ids::GS_CFG_SNAP_MAGNETISM_RADIUS,
        meters_to_display(state.snap_magnetism_radius),
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
    y += ROW_H + ROW_GAP;
    // Layer-order segmented row (In front / Behind).
    let layer_idx = if state.grid_in_front { 0 } else { 1 };
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
        scene,
        text_system,
        theme,
        hit_index,
        store,
    );
    y += ROW_GAP;

    // ─── Inspect section ────────────────────────────────────────
    let inspect_h = super::super::inspect::height();
    super::super::inspect::paint(
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

    // Vertical scrollbar at the right edge of the body region —
    // mirrors the Inspector / Widget Gallery setup. Skips paint +
    // hit registration when the content fits the visible area.
    if crate::widget::scrollbar_is_needed(content_h, body_h) {
        let track = crate::widget::scrollbar_track_rect(body_rect);
        let thumb = crate::widget::scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(store.scrollbar_drag(), Some(d) if d.panel == ids::GS_PANEL);
        crate::widget::paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, scene, theme,
        );
        hit_index.register(crate::widget::GRID_SETTINGS_SCROLLBAR_ID, thumb);
    }

    paint_panel_corner_dot(rect, scene, theme);
    hit_index.register(ids::GS_RESIZE_HANDLE, panel_resize_handle_rect(rect));
}
