//! **The ⚠ inert-warning badge** (ADR-0155) — the pip drawn on a node whose output the
//! diagnoser found semantically dead (a force writing `accel` no integrator consumes).
//! Split from `paint` for the panel LOC cap; `super` is `paint`. Drawn by `draw_card` at the
//! card's top-left corner; its hit lives in `crate::hits`, its geometry in `crate::geom`.

use crate::geom::{self, View};
use crate::snapshot::GraphNodeView;
use ph2d_editor_core::paint::{fill_circle, fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

const INERT_MARK_W: f32 = 0.16; // LITERAL-PX-OK: exclamation bar width as a fraction of the badge
const INERT_MARK_TOP: f32 = 0.52; // LITERAL-PX-OK: bar top above centre (fraction of radius)
const INERT_MARK_BAR_H: f32 = 0.6; // LITERAL-PX-OK: bar height (fraction of radius)
const INERT_MARK_DOT_Y: f32 = 0.5; // LITERAL-PX-OK: dot centre below centre (fraction of radius)
const INERT_MARK_DOT_R: f32 = 0.55; // LITERAL-PX-OK: dot radius as a fraction of the bar width

/// Draw the ⚠ inert-warning badge (ADR-0155) on a node the diagnoser found semantically
/// dead: a `Danger` disc with a `Text1` exclamation, on the card's top-left corner. `None`
/// (a healthy node) draws nothing. This panel has no icon atlas, so the mark is composed
/// from primitives — a disc + a bar + a dot read as a warning without a glyph — and `Text1`
/// (which flips with the theme) keeps the "!" legible on the red disc in light AND dark.
pub(super) fn draw_inert_badge(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    let Some(b) = geom::inert_badge_rect(n, view) else {
        return;
    };
    let cx = b.x + b.w * 0.5;
    let cy = b.y + b.h * 0.5;
    let r = b.w * 0.5;
    fill_circle(ctx.scene, cx, cy, r, resolve(ColorToken::Danger, theme));
    let mark = resolve(ColorToken::Text1, theme);
    let bw = (b.w * INERT_MARK_W).max(1.0);
    // The "!": a rounded bar above a dot, both centred on the disc.
    let bar = Rect::new(
        cx - bw * 0.5,
        cy - r * INERT_MARK_TOP,
        bw,
        r * INERT_MARK_BAR_H,
    );
    fill_rounded_rect(ctx.scene, bar, bw * 0.5, mark);
    fill_circle(
        ctx.scene,
        cx,
        cy + r * INERT_MARK_DOT_Y,
        bw * INERT_MARK_DOT_R,
        mark,
    );
}
