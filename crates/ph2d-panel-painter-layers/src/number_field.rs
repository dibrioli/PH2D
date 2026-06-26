//! Drag-scrub numeric-field rows for the Grain / Shape param sections (Enio 2026-06-25): a label + a
//! NumberInput chip (the Inspector's drag-to-scrub box — click-drag vertical/horizontal + type-to-edit).
//! X/Y pairs (Size, Offset) share one line; short-label per-pattern params pack two per line, long
//! labels go solo. Reuses [`crate::paint_texture_ramp::paint_ramp_chip`] (registers a `NumberInput` +
//! mirrors the live value), so the foundational number-input dispatch drives the scrub + commit; the
//! scrub step is inferred from the buffer (whole number ⇒ step 1, decimals ⇒ step 0.01).

use crate::paint_texture_ramp::paint_ramp_chip;
use ph2d_a11y::NodeId;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, TypeToken};

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
        || c::PAINTER_SHAPE_SLIDERS.contains(&id)
        || c::PAINTER_SHAPE_PARAMS.contains(&id)
}

const SINGLE_LABEL_W: f32 = 56.0; // LITERAL-PX-OK: single-row label column
const PAIR_LABEL_W: f32 = 44.0; // LITERAL-PX-OK: paired-param / x-y label column (compact)
/// Max label length (chars) for a per-pattern param to share its line with the next one.
const PAIR_MAX_LEN: usize = 7;

/// Format a param value: whole number when `decimals == 0` (e.g. Angle degrees), else fixed decimals.
fn fmt_val(v: f32, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.decimals$}")
    }
}

/// A left-aligned, vertically-centred label clipped to width `w`.
fn label(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme, text: &str, x: f32, y: f32, w: f32) {
    let font = TypeToken::Sm.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        text,
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
}

/// One number chip filling `rect` (registers it + mirrors `value`, formatted to `decimals`).
fn chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    rect: Rect,
    id: NodeId,
    value: f32,
    decimals: usize,
) {
    paint_ramp_chip(
        ctx,
        theme,
        rect,
        id,
        f64::from(value),
        &fmt_val(value, decimals),
    );
}

/// Label + ONE number chip (Angle / Depth / a solo param). `decimals` formats + sets the scrub step
/// (`0` ⇒ whole-number / step 1, else 2-decimals / step 0.01). Returns the next `y`.
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
    decimals: usize,
) -> f32 {
    let gap = Spacing::Sm.px();
    label(ctx, theme, label_txt, x, y, SINGLE_LABEL_W);
    let cx = x + SINGLE_LABEL_W + gap;
    let cw = (x + content_w - cx).max(0.0);
    chip(
        ctx,
        theme,
        Rect::new(cx, y, cw, ROW_H_PX),
        id,
        value,
        decimals,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Label + TWO number chips on one line (an X/Y pair: Size / Offset). Returns the next `y`.
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
    decimals: usize,
) -> f32 {
    let gap = Spacing::Sm.px();
    let small = Spacing::Xs.px();
    label(ctx, theme, label_txt, x, y, PAIR_LABEL_W);
    let fx = x + PAIR_LABEL_W + gap;
    let total = (x + content_w - fx).max(0.0);
    let cw = ((total - small) * 0.5).max(0.0);
    chip(
        ctx,
        theme,
        Rect::new(fx, y, cw, ROW_H_PX),
        id_x,
        vx,
        decimals,
    );
    chip(
        ctx,
        theme,
        Rect::new(fx + cw + small, y, cw, ROW_H_PX),
        id_y,
        vy,
        decimals,
    );
    y + ROW_H_PX + Spacing::Xs.px()
}

/// Per-pattern params (all `0..1`, 2 decimals): pair two consecutive SHORT-label params on one line
/// (each ≤ [`PAIR_MAX_LEN`] chars, e.g. Voronoi's Metric / Edges), else one per line (Randomness,
/// Smoothness…). Returns the next `y`.
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
            y = paint_num_row(ctx, theme, x, content_w, y, l0, id0, v0, 2);
            i += 1;
        }
    }
    y
}

/// One `[label | chip]` half of a paired-param line, within `[x, x + w]`.
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
    label(ctx, theme, label_txt, x, y, PAIR_LABEL_W);
    let cx = x + PAIR_LABEL_W + gap;
    let cw = (x + w - cx).max(0.0);
    chip(ctx, theme, Rect::new(cx, y, cw, ROW_H_PX), id, v, 2);
}
