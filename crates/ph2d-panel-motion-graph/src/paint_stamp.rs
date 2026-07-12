//! **The postage stamp** (F3, doc 47) — the little scatter of what a node emits, drawn on its
//! card. Split from `paint` for the panel LOC cap; `super` is `paint`.

use super::{
    PREVIEW_DOT_MIN, PREVIEW_DOT_R, PREVIEW_INSET, PREVIEW_MIN_H, PREVIEW_RADIUS, domain_token,
};
use crate::geom::{self, View};
use crate::snapshot::GraphNodeView;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::paint_batch::fill_dots;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_tokens::{ColorToken, Theme};

/// **The postage stamp**: what this node passes on, as a little scatter of its own positions
/// (Nuke's thumbnails, for a node graph that carries points instead of pixels). It answers the
/// one question no wire can, however well it is drawn: *where does the spiral become a grid?*
///
/// The points are fitted to the strip with a UNIFORM scale (aspect preserved) and drawn y-up,
/// like the canvas — a stamp that stretched its contents to fill the box would show a circle as
/// an ellipse and lie about the very thing it exists to show.
pub(super) fn draw_preview(ctx: &mut PaintCtx, n: &GraphNodeView, view: &View, theme: Theme) {
    let (Some(pts), Some(rect)) = (n.preview.as_ref(), geom::preview_rect(n, view)) else {
        return;
    };
    // Zoomed out, the strip is a few pixels tall: the dots would be sub-pixel mush that reads
    // as noise. Draw the empty window instead — the card keeps its shape at every zoom.
    fill_rounded_rect(
        ctx.scene,
        rect,
        PREVIEW_RADIUS,
        resolve(ColorToken::GraphBg, theme),
    );
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
