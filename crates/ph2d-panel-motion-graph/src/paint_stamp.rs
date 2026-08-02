//! **The postage stamp** (F3, doc 47) — the little scatter of what a node emits, drawn on its
//! card. Split from `paint` for the panel LOC cap; `super` is `paint`.

use super::{
    PREVIEW_DOT_MIN, PREVIEW_DOT_R, PREVIEW_INSET, PREVIEW_MIN_H, PREVIEW_RADIUS, domain_token,
};
use crate::geom::{self, View};
use crate::snapshot::GraphNodeView;
use crate::state::PreviewPos;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use ph2d_editor_core::paint_batch::fill_dots;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

/// **The postage stamp**: what this node passes on, as a little scatter of its own positions
/// (Nuke's thumbnails, for a node graph that carries points instead of pixels). It answers the
/// one question no wire can, however well it is drawn: *where does the spiral become a grid?*
///
/// The points are fitted to the strip with a UNIFORM scale (aspect preserved) and drawn y-up,
/// like the canvas — a stamp that stretched its contents to fill the box would show a circle as
/// an ellipse and lie about the very thing it exists to show.
pub(super) fn draw_preview(
    ctx: &mut PaintCtx,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    pos: PreviewPos,
) {
    let (Some(pts), Some(rect)) = (n.preview.as_ref(), geom::preview_frame_rect(n, view, pos))
    else {
        return;
    };
    // The moldura is a box in its own right now (doc 86): a filled window PLUS a border, so it
    // reads as a detached frame floating over the canvas rather than a strip glued to the card.
    fill_rounded_rect(
        ctx.scene,
        rect,
        PREVIEW_RADIUS,
        resolve(ColorToken::GraphBg, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        PREVIEW_RADIUS,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    // Zoomed out, the box is a few pixels tall: the dots would be sub-pixel mush that reads as
    // noise. Keep the framed window and stop — it holds its shape at every zoom.
    if rect.h < PREVIEW_MIN_H || pts.is_empty() {
        return;
    }

    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in pts {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let (ex, ey) = (hi[0] - lo[0], hi[1] - lo[1]);
    // A degenerate extent (every point on top of the others, or a single point) has no scale to
    // speak of: centre it rather than dividing by zero.
    let pad = PREVIEW_INSET * view.zoom;
    let s = match (ex > 0.0 || ey > 0.0).then(|| {
        ((rect.w - 2.0 * pad) / ex.max(f32::EPSILON))
            .min((rect.h - 2.0 * pad) / ey.max(f32::EPSILON))
    }) {
        Some(s) if s.is_finite() => s,
        _ => 0.0,
    };
    let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    let (mx, my) = ((lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5);
    let color = resolve(
        n.outputs
            .first()
            .map(|p| domain_token(p.domain))
            .unwrap_or(ColorToken::Text2),
        theme,
    );
    let dot = (PREVIEW_DOT_R * view.zoom).max(PREVIEW_DOT_MIN);
    // ONE draw call for the whole stamp. Vello's cost is per draw OBJECT, and a stamp per card at
    // one fill per dot was ~4 000 objects a frame across the canvas — which is exactly where the
    // frame rate went (doc 53). y-up: the world's +y is the canvas's up, because the stamp is a
    // little window onto the canvas.
    let screen: Vec<(f32, f32)> = pts
        .iter()
        .map(|p| (cx + (p[0] - mx) * s, cy - (p[1] - my) * s))
        .collect();
    fill_dots(ctx.scene, &screen, dot, color);
}

/// **The header toggle** (doc 86): a small square at the card header's right that moves the
/// preview moldura above/below. It is not just a button — the accent BAR inside it sits in the
/// half the stamp will land (top for `Above`, bottom for `Below`), so it shows the direction
/// rather than only that it toggles. Drawn only when the node has a stamp (`preview_toggle_rect`
/// returns `None` otherwise), so a card with no preview never grows a dead control.
pub(super) fn draw_preview_toggle(
    ctx: &mut PaintCtx,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
    pos: PreviewPos,
) {
    let Some(btn) = geom::preview_toggle_rect(n, view) else {
        return;
    };
    let r = (btn.w * TOGGLE_RADIUS_FRAC).max(1.0);
    fill_rounded_rect(ctx.scene, btn, r, resolve(ColorToken::Bg3, theme));
    stroke_rounded_rect(ctx.scene, btn, r, 1.0, resolve(ColorToken::Border, theme));
    let inset = btn.w * TOGGLE_BAR_INSET;
    let bar_h = btn.h * TOGGLE_BAR_H;
    let bar_y = match pos {
        PreviewPos::Above => btn.y + inset,
        PreviewPos::Below => btn.y + btn.h - inset - bar_h,
    };
    let bar = Rect::new(btn.x + inset, bar_y, btn.w - 2.0 * inset, bar_h);
    fill_rounded_rect(ctx.scene, bar, r * 0.5, resolve(ColorToken::Accent, theme));
}

const TOGGLE_RADIUS_FRAC: f32 = 0.2; // LITERAL-PX-OK: toggle corner radius as a fraction of its side
const TOGGLE_BAR_INSET: f32 = 0.22; // LITERAL-PX-OK: direction-bar inset (fraction of the button)
const TOGGLE_BAR_H: f32 = 0.26; // LITERAL-PX-OK: direction-bar height (fraction of the button)
