//! Vector Style panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] + stale-rect
//!   cleanup on hide.
//! - Right-dock rect from `ctx.layout.inspector` (shared Inspector slot; the
//!   shell hides the real Inspector while the `vector` tool is active).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dots, drag / resize dock
//!   handles, [`paint_panel_title`], an X close button, then the body:
//!   a Width slider+chip row, a Stroke colour-swatch row, and a Fill
//!   colour-swatch row with a "None" button. Every painter is the SHARED
//!   source-of-truth from `panel_chrome` / `widget`.
//!
//! The two colour swatches are **picker swatches** (`register_picker_swatch`):
//! a Down opens the shared OKLCH picker (generic dispatch); the shell's
//! `vector_bridge` reads the pick back into the tool + keeps `widget_color`
//! synced to the live colour, which this paint draws as the swatch fill.

use crate::paint_sections::BodyCtx;
use crate::state::{self, VectorPanelState, set_last_content_h, set_last_visible_h};
use crate::{VectorPanel, ids};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    NUMBER_INPUT_MIN_W_PX, SCROLLBAR_W, VECTOR_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

pub(crate) fn paint(_state: &mut VectorPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(VectorPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // VECTOR_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::VECTOR_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    let snap = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::VECTOR_PANEL, rect);

    // Dark-glass surface + corner accents — identical chrome to the Inspector /
    // Padding panels.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles. Reuse Inspector IDs because image-tool
    // panels share the right dock slot — the resize delta persists when the
    // user switches between Inspector / image tool.
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ph2d_editor_core::ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    // Canonical panel title — reserve room on the right for the X close button.
    let title_size = paint_panel_title(
        rect,
        "Vector",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Canonical X close button (painted on the chrome).
    paint_panel_close_button(
        rect,
        ids::VECTOR_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let inner_x = rect.x + PANEL_HEAD_PAD;
    // Reserve the scrollbar rail on the right so body content never paints under
    // it (stable layout whether or not the bar is currently shown).
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let chip_w = NUMBER_INPUT_MIN_W_PX;
    let font = TypeToken::Base.px();

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);

    // Body paint — `store_and_hit_index_mut()` hands out an IMMUTABLE store (for
    // reading widget values while painting) + a mutable hit_index. The
    // picker-swatch MARK + content_h publish need `&mut store`, so they run
    // AFTER this borrow ends (Phase B below).
    let content_h = {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        // Vertical scroll — the wheel already updates `panel_scroll` via the
        // generic dispatch (this panel publishes its rect + heights); thumb-drag
        // via `VECTOR_SCROLLBAR_ID`. Clip the body region and shift content up by
        // `scroll_y` — mirror of the Inspector.
        let scroll_y = store.panel_scroll(ids::VECTOR_PANEL).max(0.0);
        let clip = rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h));
        scene.push_clip(&clip);
        let body_top_y = body_top - scroll_y;
        // Section painters live in `paint_sections` (panel LOC cap): each takes
        // the running `y` and returns the advanced `y`.
        let mut b = BodyCtx {
            scene,
            text_system,
            store,
            hit_index,
            theme,
            inner_x,
            inner_w,
            row_h,
            row_gap,
            chip_w,
            font,
        };
        let mut y = body_top_y;
        y = b.stroke_style(&snap, y);
        y = b.fill_style(&snap, y);
        y = b.draw_modes(&snap, y);
        y = b.snap_section(y);
        y = b.transform_section(y);
        y = b.vertex_boolean_arrange(y);

        // Total painted height (independent of scroll — both ends shift with it).
        let content_h = (y - body_top_y + PANEL_HEAD_PAD).max(0.0);
        // Scrollbar (painted inside the clip, right rail) + its drag hit target.
        if scrollbar_is_needed(content_h, body_h) {
            let body = Rect::new(rect.x, body_top, rect.w, body_h);
            let thumb =
                scrollbar_thumb_rect(scrollbar_track_rect(body), scroll_y, content_h, body_h);
            let is_active =
                matches!(b.store.scrollbar_drag(), Some(d) if d.panel == ids::VECTOR_PANEL);
            paint_scrollbar(body, scroll_y, content_h, body_h, is_active, b.scene, theme);
            b.hit_index.register(VECTOR_SCROLLBAR_ID, thumb);
        }
        b.scene.pop_layer();
        content_h
    };

    // ── Phase B (mutable store) ─────────────────────────────────────────
    // Mark the two colour swatches so a Down opens the shared OKLCH picker
    // (generic `is_picker_swatch` dispatch). Idempotent — a set membership.
    {
        let store = ctx.host.store_mut();
        store.register_picker_swatch(ids::VECTOR_STROKE_SWATCH);
        store.register_picker_swatch(ids::VECTOR_FILL_SWATCH);
        // Seed the Transform fields from the published bbox (skip the one being
        // edited so a keystroke isn't clobbered). 1-frame post-commit lag, ok.
        if let Some([tx, ty, tw, th]) = state::current_transform() {
            let focus = store.focus_id();
            for (id, v) in [
                (ids::VECTOR_TRANSFORM_X, tx),
                (ids::VECTOR_TRANSFORM_Y, ty),
                (ids::VECTOR_TRANSFORM_W, tw),
                (ids::VECTOR_TRANSFORM_H, th),
            ] {
                if focus != Some(id) {
                    store.set_number_value(id, v);
                }
            }
            // The Angle field is a RELATIVE scrub: seed it to 0 (and reset the
            // gesture accumulator) whenever it isn't being edited, so each new
            // drag/type starts from 0 and the shell rotates by the reported delta.
            if focus != Some(ids::VECTOR_TRANSFORM_R) {
                store.set_number_value(ids::VECTOR_TRANSFORM_R, 0.0);
                state::set_rot_last(0.0);
            }
        }
        // Seed the variation-axis fields (value + range) from the published axes of
        // the current font — skip the field being edited so a keystroke/drag isn't
        // clobbered. Fixed slots adapt to whatever axes the font has (paint reads the
        // seeded value; a change flows back as a `SetValue` the shell applies).
        let focus = store.focus_id();
        let axes: Vec<(f64, f64, f64)> =
            state::with_text_axes(|a| a.iter().map(|s| (s.min, s.max, s.value)).collect());
        for (i, (min, max, value)) in axes.into_iter().enumerate() {
            let id = ids::vector_text_axis_id(i);
            let step = ((max - min) / 40.0).max(0.01); // LITERAL-PX-OK: ~40 scrub steps across the axis range (drag granularity, not a metric)
            store.set_number_range(id, min, max, step);
            if focus != Some(id) {
                store.set_number_value(id, value);
            }
        }
        store.set_panel_content_h(ids::VECTOR_PANEL, content_h);
        store.set_panel_visible_h(ids::VECTOR_PANEL, body_h);
        // Clamp any stale scroll if the content shrank (e.g. a mode switch hid a
        // section) so we never leave a blank gap below the last row.
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(ids::VECTOR_PANEL) > max_scroll {
            store.set_panel_scroll(ids::VECTOR_PANEL, max_scroll);
        }
    }
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    // Re-register close chrome after the body (last-registered-wins so the X
    // stays clickable over the body region).
    ctx.host
        .hit_index_mut()
        .register(ids::VECTOR_CLOSE, panel_close_button_rect(rect));

    // Deferred: the Font dropdown popover paints (and hit-registers its rows) ON
    // TOP of every section — the chip stashed its rect during the body paint when
    // open. Painted last so the floating list occludes the body + owns the clicks.
    if let Some(chip_rect) = state::take_pending_font_dd() {
        crate::font_dropdown::paint(ctx, chip_rect, theme);
    }
}
