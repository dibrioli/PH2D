//! Drag-scrub numeric-field rows for the Grain / Shape param sections (Enio 2026-06-25) — the SAME
//! number box as the Inspector's Transform (the app standard): [`paint_number_input_with_buffer`] with
//! up/down steppers, click-drag scrub (vertical = precise, horizontal = fast, Shift = super-precise — the
//! foundational `number_input_drag` dispatch) and type-to-edit. X/Y pairs (Size, Offset) share one line
//! with red **X** / green **Y** axis tags like the reference; short-label per-pattern params pair
//! two-per-line, long labels go solo.
//!
//! Each box registers its `[min, max]` + `step` via `WidgetStore::set_number_range`, so the drag-scrub
//! is PROPORTIONAL to the range (a fixed drag spans `[min,max]`, no racing) + clamped, and the stepper
//! uses `step`. The live value is mirrored into the store each frame (when unfocused). The
//! committed/scrubbed REAL value forwards over `event::is_param_field`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{NumberInput, TextInputState, paint_number_input_with_buffer};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

const LABEL_W: f32 = 70.0; // LITERAL-PX-OK: row-label column (fits "Brightness"/"Randomness")
const PAIR_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: paired-param label column (compact)
const AXIS_W: f32 = 16.0; // LITERAL-PX-OK: the "X"/"Y" axis-tag column
/// Max label length (chars) for a per-pattern param to share its line with the next one.
const PAIR_MAX_LEN: usize = 7;
/// NumberInput steps registered via `set_number_range` — the stepper increment + the drag base.
/// `pub(crate)` so the Grain/Shape sections pass the right one per field.
pub(crate) const FINE_STEP: f64 = 0.01; // LITERAL-PX-OK: NumberInput step (0..1 / offset / depth / params)
pub(crate) const SIZE_STEP: f64 = 0.1; // LITERAL-PX-OK: NumberInput step (scale)
pub(crate) const ANGLE_STEP: f64 = 1.0; // LITERAL-PX-OK: NumberInput step (whole degrees)

/// Whether `id` is a Grain/Shape param NumberInput field (Angle / Offset / Size / Depth / per-pattern
/// param) — vs a main brush slider. Drives the panel's number-field `ValueChanged` route (`event.rs`),
/// which forwards the committed/scrubbed REAL value (the tool's real-value setters clamp it).
pub(crate) fn is_param_field(id: NodeId) -> bool {
    use ph2d_editor_core::ids as c;
    id == c::PAINTER_BRUSH_TEXTURE_ANGLE
        || id == c::PAINTER_BRUSH_TEXTURE_OFFSET_X
        || id == c::PAINTER_BRUSH_TEXTURE_OFFSET_Y
        || id == c::PAINTER_BRUSH_TEXTURE_SIZE_X
        || id == c::PAINTER_BRUSH_TEXTURE_SIZE_Y
        || c::PAINTER_BRUSH_TEXTURE_PARAMS.contains(&id)
        || c::PAINTER_BRUSH_STENCIL_FIELDS.contains(&id)
        || c::PAINTER_SHAPE_SLIDERS.contains(&id)
        || c::PAINTER_SHAPE_PARAMS.contains(&id)
        || c::PAINTER_WATERCOLOR_FIELDS.contains(&id)
}

/// Format a param value: whole number when `decimals == 0` (Angle degrees), else fixed decimals (so the
/// scrub dispatch infers the fine `0.01` step from the `.` in the buffer).
fn fmt_val(v: f32, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.decimals$}")
    }
}

/// Register (if absent) + mirror the live `value` into the store's `NumberInput` when unfocused, with a
/// `decimals`-formatted buffer (mirror of the Inspector's per-frame sync).
fn mirror_value(store: &mut WidgetStore, id: NodeId, value: f32, decimals: usize) {
    let text = fmt_val(value, decimals);
    let _ = store.register_if_absent(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(value),
            buffer: text.clone(),
            caret: text.len(),
            last_committed: f64::from(value),
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::NumberInput {
            value: v,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
    {
        *v = f64::from(value);
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = f64::from(value);
    }
}

/// One number box filling `rect` (the Inspector widget): mirror the live value, register its
/// `[min, max]` range + `step` (so the drag-scrub is range-proportional + clamped and the stepper uses
/// `step` — see `WidgetStore::set_number_range`), then paint with steppers + the edit buffer + caret.
/// `pub(crate)` so the per-layer-colour opacity box (factory-id) reuses the exact app-standard widget.
#[allow(clippy::too_many_arguments)]
pub(crate) fn chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    id: NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) {
    {
        let store = ctx.host.store_mut();
        mirror_value(store, id, value, decimals);
        store.set_number_range(id, f64::from(min), f64::from(max), step);
    }
    let (state, _v, buf, caret, anchor) = read_number_input(ctx.host.store(), id);
    let buf = buf.to_string();
    let input = NumberInput::new(id, "", f64::from(value))
        .step(step)
        .state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// A left-aligned, vertically-centred label clipped to width `w` (colour `tok`).
fn label_tok(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    x: f32,
    y: f32,
    w: f32,
    tok: ColorToken,
) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(tok, theme),
    );
}

/// Label + ONE number box (Angle / Depth / a solo param). Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_num_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id: NodeId,
    value: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) -> f32 {
    let gap = Spacing::Sm.px();
    label_tok(ctx, theme, label_txt, x, y, LABEL_W, ColorToken::Text2);
    let cx = x + LABEL_W + gap;
    let cw = (x + content_w - cx).max(0.0);
    chip(
        ctx,
        theme,
        Rect::new(cx, y, cw, ROW_H_PX),
        id,
        value,
        min,
        max,
        step,
        decimals,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Label + TWO number boxes on one line, with red **X** / green **Y** axis tags (Size / Offset), like the
/// Inspector Transform reference. Returns the next `y`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_num_xy(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    label_txt: &str,
    id_x: NodeId,
    vx: f32,
    id_y: NodeId,
    vy: f32,
    min: f32,
    max: f32,
    step: f64,
    decimals: usize,
) -> f32 {
    let gap = Spacing::Sm.px();
    let tag_gap = Spacing::Xxs.px();
    label_tok(ctx, theme, label_txt, x, y, LABEL_W, ColorToken::Text2);
    let fields_x = x + LABEL_W + gap;
    let avail = (x + content_w - fields_x).max(0.0);
    let box_w = ((avail - 2.0 * (AXIS_W + tag_gap) - gap) / 2.0).max(0.0);
    // X (red) tag + box.
    label_tok(ctx, theme, "X", fields_x, y, AXIS_W, ColorToken::Danger);
    let xb = fields_x + AXIS_W + tag_gap;
    chip(
        ctx,
        theme,
        Rect::new(xb, y, box_w, ROW_H_PX),
        id_x,
        vx,
        min,
        max,
        step,
        decimals,
    );
    // Y (green) tag + box.
    let y_tag_x = xb + box_w + gap;
    label_tok(ctx, theme, "Y", y_tag_x, y, AXIS_W, ColorToken::Success);
    let yb = y_tag_x + AXIS_W + tag_gap;
    chip(
        ctx,
        theme,
        Rect::new(yb, y, box_w, ROW_H_PX),
        id_y,
        vy,
        min,
        max,
        step,
        decimals,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Per-pattern params (all `0..1`, step `0.01`): pair two consecutive SHORT-label params on one line
/// (each ≤ [`PAIR_MAX_LEN`] chars, e.g. Voronoi's Metric / Edges), else one per line. Returns the next `y`.
pub(crate) fn paint_num_params(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    mut y: f32,
    params: &[(&str, NodeId, f32)],
) -> f32 {
    let mut i = 0;
    while i < params.len() {
        let (l0, id0, v0) = params[i];
        if i + 1 < params.len() && l0.len() <= PAIR_MAX_LEN && params[i + 1].0.len() <= PAIR_MAX_LEN
        {
            let (l1, id1, v1) = params[i + 1];
            let gap = Spacing::Sm.px();
            let half = ((content_w - gap) * 0.5).max(0.0);
            half_param(ctx, theme, x, half, y, l0, id0, v0);
            half_param(ctx, theme, x + half + gap, half, y, l1, id1, v1);
            y += ROW_H_PX + Spacing::Xs.px();
            i += 2;
        } else {
            y = paint_num_row(
                ctx, theme, x, content_w, y, l0, id0, v0, 0.0, 1.0, FINE_STEP, 2,
            );
            i += 1;
        }
    }
    y
}

/// One `[label | box]` half of a paired-param line, within `[x, x + w]`.
#[allow(clippy::too_many_arguments)]
fn half_param(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    label_txt: &str,
    id: NodeId,
    v: f32,
) {
    let gap = Spacing::Xs.px();
    label_tok(ctx, theme, label_txt, x, y, PAIR_LABEL_W, ColorToken::Text2);
    let cx = x + PAIR_LABEL_W + gap;
    let cw = (x + w - cx).max(0.0);
    chip(
        ctx,
        theme,
        Rect::new(cx, y, cw, ROW_H_PX),
        id,
        v,
        0.0,
        1.0,
        FINE_STEP,
        2,
    );
}
