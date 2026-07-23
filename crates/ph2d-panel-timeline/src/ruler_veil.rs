//! The veil's DURATION HANDLE paint (Enio, 2026-07-23) — split from `ruler.rs`
//! under the panel LOC cap. A child module of `ruler`, so it reads the parent's
//! ruler geometry consts + helpers (`fill_triangle`, `RULER_H`, `BRACE_W`) through
//! `super::*`.

use super::*;

/// Half-width of the duration handle's grab target — generous, like a brace, so a
/// press near the thin edge line still grabs it.
const DUR_HANDLE_HIT_HW: f32 = 7.0; // LITERAL-PX-OK: duration-handle grab half-width
/// Horizontal offset of the ↔ glyph from the veil edge (to the right, over the veil).
const DUR_ARROW_OFFSET: f32 = 9.0; // LITERAL-PX-OK: ↔ glyph offset from the duration edge
/// Half the shaft length of the ↔ glyph.
const DUR_ARROW_HALF_W: f32 = 4.0; // LITERAL-PX-OK: ↔ glyph shaft half-width
/// Half-thickness of the ↔ glyph shaft.
const DUR_ARROW_SHAFT_HW: f32 = 1.0; // LITERAL-PX-OK: ↔ glyph shaft half-thickness
/// Arrowhead size of the ↔ glyph.
const DUR_ARROW_HEAD: f32 = 3.0; // LITERAL-PX-OK: ↔ glyph arrowhead size

/// Paint the drag grip at the veil's left edge (the authored duration end) and
/// register it as a [`TimelineHitKind::DurationHandle`] surface. A no-op unless
/// the view has an authored duration whose end is on-screen — same gate as the
/// veil ([`beyond_end_shade`]); the handle IS the veil's edge, so it cannot exist
/// where the veil does not.
///
/// A vertical grip bar sits ON the edge; a small ↔ glyph sits a little to its
/// RIGHT, over the dark veil (Enio, 2026-07-23: *"um pouco à direita do início do
/// véu"*), so the affordance reads without hiding the tick at the edge. The grab
/// rect is centred on the edge and reaches into both sides, so a press just left
/// or right of the line still grabs it.
pub(super) fn paint_duration_handle(
    ctx: &mut PaintCtx,
    theme: Theme,
    region: Rect,
    view_start: f64,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    if !snap.view_length_explicit {
        return;
    }
    let right = region.x + region.w;
    let edge = region.x + ((snap.view_length_seconds - view_start) * px_per_s) as f32;
    // Off-screen to the right (the veil never starts) → no handle. A handle at the
    // left clamp would drag from a time the edge is not at.
    if edge < region.x || edge > right {
        return;
    }
    let color = resolve(ColorToken::TimelinePlayhead, theme);
    // The grip bar on the edge.
    fill_rounded_rect(
        ctx.scene,
        Rect::new(edge - BRACE_W * 0.5, region.y, BRACE_W, RULER_H),
        Radius::Xs.px(),
        color,
    );
    // The ↔ glyph, a little to the right of the edge, centred vertically in the
    // strip: a thin shaft with a triangle arrowhead at each end.
    let cy = region.y + RULER_H * 0.5;
    let gx = edge + DUR_ARROW_OFFSET;
    let half = DUR_ARROW_HALF_W;
    fill_rounded_rect(
        ctx.scene,
        Rect::new(
            gx - half,
            cy - DUR_ARROW_SHAFT_HW,
            half * 2.0,
            DUR_ARROW_SHAFT_HW * 2.0,
        ),
        Radius::Xs.px(),
        color,
    );
    fill_triangle(
        ctx,
        [
            (gx - half - DUR_ARROW_HEAD, cy),
            (gx - half, cy - DUR_ARROW_HEAD),
            (gx - half, cy + DUR_ARROW_HEAD),
        ],
        color,
    );
    fill_triangle(
        ctx,
        [
            (gx + half + DUR_ARROW_HEAD, cy),
            (gx + half, cy - DUR_ARROW_HEAD),
            (gx + half, cy + DUR_ARROW_HEAD),
        ],
        color,
    );
    // The grab target: centred on the edge, reaching a generous half-width into
    // BOTH sides (so a press just left/right of the thin line still grabs), and
    // extending across the ruler strip.
    let hit = Rect::new(
        edge - DUR_HANDLE_HIT_HW,
        region.y,
        DUR_HANDLE_HIT_HW * 2.0,
        RULER_H,
    );
    ctx.host.store_mut().register(
        ids::TIMELINE_DUR_HANDLE,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::DurationHandle,
            canvas: hit,
        },
    );
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_DUR_HANDLE, hit);
}
