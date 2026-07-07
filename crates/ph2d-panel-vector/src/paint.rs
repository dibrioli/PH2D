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

use crate::state::{self, VectorPanelState, set_last_content_h, set_last_visible_h};
use crate::{VectorPanel, ids};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, paint_segmented_button, panel_close_button_rect,
    panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, ColorSwatch, NUMBER_INPUT_MIN_W_PX, SwatchSize, paint_button,
    paint_color_swatch, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_vector::params::{DrawMode, sides_to_slider};
use ph2d_tool_vector::{VertexType, px_to_slider};

/// Label column width for the Width slider row + the Stroke / Fill labels.
const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

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
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
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
        let mut y = body_top;

        // ── Width slider + px chip ──────────────────────────────────────
        // Slider track = live stored value (smooth drag) ?? normalized
        // snapshot; chip shows the live stored px ?? snapshot px.
        let track = store
            .slider(ids::VECTOR_WIDTH)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap.stroke_width_px));
        let px = store
            .number_value(ids::VECTOR_WIDTH_NUM)
            .unwrap_or(snap.stroke_width_px);
        let px_display = format!("{}", px.round() as i64);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            "Width",
            track,
            px,
            Some(&px_display),
            ids::VECTOR_WIDTH,
            ids::VECTOR_WIDTH_NUM,
            LABEL_COL_W,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + row_gap;

        let swatch_w = SwatchSize::Md.px();

        // ── Stroke colour swatch ────────────────────────────────────────
        paint_text(
            text_system,
            scene,
            "Stroke",
            inner_x,
            y + (row_h - font) * 0.5,
            font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, theme),
        );
        let stroke_swatch_rect = Rect::new(inner_x + inner_w - swatch_w, y, swatch_w, row_h);
        let stroke_swatch =
            ColorSwatch::new(ids::VECTOR_STROKE_SWATCH, "Stroke color", snap.stroke)
                .size(SwatchSize::Md);
        paint_color_swatch(&stroke_swatch, stroke_swatch_rect, scene, theme);
        hit_index.register(ids::VECTOR_STROKE_SWATCH, stroke_swatch_rect);
        y += row_h + row_gap;

        // ── Fill colour swatch + "None" button ──────────────────────────
        paint_text(
            text_system,
            scene,
            "Fill",
            inner_x,
            y + (row_h - font) * 0.5,
            font,
            LABEL_COL_W,
            resolve(ColorToken::Text1, theme),
        );
        let fill_swatch_rect = Rect::new(inner_x + inner_w - swatch_w, y, swatch_w, row_h);
        // Alpha 0 ⇒ "None" — the swatch renders transparent, the accent None
        // button reads as active.
        let fill_swatch =
            ColorSwatch::new(ids::VECTOR_FILL_SWATCH, "Fill color", snap.fill).size(SwatchSize::Md);
        paint_color_swatch(&fill_swatch, fill_swatch_rect, scene, theme);
        hit_index.register(ids::VECTOR_FILL_SWATCH, fill_swatch_rect);

        // "None" button, pinned just left of the swatch.
        let none_w = NUMBER_INPUT_MIN_W_PX;
        let none_rect = Rect::new(
            fill_swatch_rect.x - Spacing::Sm.px() - none_w,
            y,
            none_w,
            row_h,
        );
        let none_kind = if snap.fill[3] == 0 {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let none_state = store
            .button_state(ids::VECTOR_FILL_NONE)
            .unwrap_or(ButtonState::Normal);
        let none_btn = Button::new(ids::VECTOR_FILL_NONE, "None")
            .kind(none_kind)
            .state(none_state);
        paint_button(&none_btn, none_rect, scene, text_system, theme);
        hit_index.register(ids::VECTOR_FILL_NONE, none_rect);
        y += row_h + row_gap;

        let label_font = TypeToken::Sm.px();

        // ── Draw mode: Pen (draw/edit) vs a drag-to-size shape ──────────
        paint_text(
            text_system,
            scene,
            "Draw",
            inner_x,
            y,
            label_font,
            inner_w,
            resolve(ColorToken::Text2, theme),
        );
        y += label_font + Spacing::Xs.px();
        let modes = [
            (ids::VECTOR_MODE_PEN, "Pen", DrawMode::Pen),
            (ids::VECTOR_MODE_RECT, "Rect", DrawMode::Rectangle),
            (ids::VECTOR_MODE_ELLIPSE, "Oval", DrawMode::Ellipse),
            (ids::VECTOR_MODE_POLYGON, "Poly", DrawMode::Polygon),
        ];
        let seg_gap = Spacing::Sm.px();
        let seg_w = ((inner_w - seg_gap * (modes.len() as f32 - 1.0)) / modes.len() as f32).max(1.0);
        for (i, (id, label, m)) in modes.iter().enumerate() {
            let rx = inner_x + i as f32 * (seg_w + seg_gap);
            let rect = Rect::new(rx, y, seg_w, row_h);
            let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
            paint_segmented_button(rect, label, snap.mode == *m, state, scene, text_system, theme);
            hit_index.register(*id, rect);
        }
        y += row_h + row_gap;

        // ── Polygon Sides slider (only meaningful in Polygon mode) ──────
        if snap.mode == DrawMode::Polygon {
            let track = store
                .slider(ids::VECTOR_SIDES)
                .map(|(_, v)| v)
                .unwrap_or_else(|| sides_to_slider(snap.polygon_sides));
            let sides_val = store
                .number_value(ids::VECTOR_SIDES_NUM)
                .unwrap_or(f64::from(snap.polygon_sides));
            let sides_display = format!("{}", sides_val.round() as i64);
            let used = paint_slider_with_chip_layout_adaptive(
                Rect::new(inner_x, y, inner_w, row_h),
                "Sides",
                track,
                sides_val,
                Some(&sides_display),
                ids::VECTOR_SIDES,
                ids::VECTOR_SIDES_NUM,
                LABEL_COL_W,
                chip_w,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
            y += used + row_gap;
        }

        // ── Vertex type (rich handle editing) — only with a vertex selected ──
        if let Some(vtype) = state::current_vertex_type() {
            paint_text(
                text_system,
                scene,
                "Vertex",
                inner_x,
                y,
                label_font,
                inner_w,
                resolve(ColorToken::Text2, theme),
            );
            y += label_font + Spacing::Xs.px();
            let verts = [
                (ids::VECTOR_VERT_CORNER, "Corner", VertexType::Corner),
                (ids::VECTOR_VERT_SMOOTH, "Smooth", VertexType::Smooth),
                (ids::VECTOR_VERT_SYMMETRIC, "Symm", VertexType::Symmetric),
            ];
            let vseg_gap = Spacing::Sm.px();
            let vseg_w =
                ((inner_w - vseg_gap * (verts.len() as f32 - 1.0)) / verts.len() as f32).max(1.0);
            for (i, (id, label, t)) in verts.iter().enumerate() {
                let rx = inner_x + i as f32 * (vseg_w + vseg_gap);
                let rect = Rect::new(rx, y, vseg_w, row_h);
                let bstate = store.button_state(*id).unwrap_or(ButtonState::Normal);
                paint_segmented_button(rect, label, vtype == *t, bstate, scene, text_system, theme);
                hit_index.register(*id, rect);
            }
            y += row_h + row_gap;
        }

        // ── Boolean ops — act on the two last closed regions ────────────
        paint_text(
            text_system,
            scene,
            "Boolean",
            inner_x,
            y,
            label_font,
            inner_w,
            resolve(ColorToken::Text2, theme),
        );
        y += label_font + Spacing::Xs.px();
        let bool_ops = [
            (ids::VECTOR_BOOL_UNION, "Union"),
            (ids::VECTOR_BOOL_SUBTRACT, "Subtract"),
            (ids::VECTOR_BOOL_INTERSECT, "Intersect"),
        ];
        for (id, label) in bool_ops {
            let rect = Rect::new(inner_x, y, inner_w, row_h);
            let state = store.button_state(id).unwrap_or(ButtonState::Normal);
            let btn = Button::new(id, label).kind(ButtonKind::Default).state(state);
            paint_button(&btn, rect, scene, text_system, theme);
            hit_index.register(id, rect);
            y += row_h + Spacing::Xs.px();
        }
        y += row_gap;

        (y - body_top + PANEL_HEAD_PAD).max(0.0)
    };

    // ── Phase B (mutable store) ─────────────────────────────────────────
    // Mark the two colour swatches so a Down opens the shared OKLCH picker
    // (generic `is_picker_swatch` dispatch). Idempotent — a set membership.
    {
        let store = ctx.host.store_mut();
        store.register_picker_swatch(ids::VECTOR_STROKE_SWATCH);
        store.register_picker_swatch(ids::VECTOR_FILL_SWATCH);
        store.set_panel_content_h(ids::VECTOR_PANEL, content_h);
        store.set_panel_visible_h(ids::VECTOR_PANEL, body_h);
    }
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    // Re-register close chrome after the body (last-registered-wins so the X
    // stays clickable over the body region).
    ctx.host
        .hit_index_mut()
        .register(ids::VECTOR_CLOSE, panel_close_button_rect(rect));
}
