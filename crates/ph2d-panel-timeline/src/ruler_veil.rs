//! The veil's DURATION HANDLE paint (Enio, 2026-07-23) — split from `ruler.rs`
//! under the panel LOC cap. A child module of `ruler`, so it reads the parent's
//! ruler geometry consts + helpers (`fill_triangle`, `RULER_H`, `BRACE_W`) through
//! `super::*`.

use super::*;

/// Horizontal offset of the ↔ glyph CENTRE from the veil edge — well to the RIGHT
/// (Enio, 2026-07-23: moved another 20 px out, *"pois ela conflita com a área do
/// loop quando o loop vai até o fim"*). The grab lives ON the glyph, so at this
/// distance neither the glyph nor its grab overlaps the loop brace sitting at the
/// edge when the loop runs to the end.
const DUR_ARROW_OFFSET: f32 = 40.0; // LITERAL-PX-OK: ↔ glyph offset from the duration edge
/// Half the shaft length of the ↔ glyph (2× — Enio, 2026-07-23: *"maior (2x)"*).
const DUR_ARROW_HALF_W: f32 = 8.0; // LITERAL-PX-OK: ↔ glyph shaft half-width
/// Half-thickness of the ↔ glyph shaft (2×).
const DUR_ARROW_SHAFT_HW: f32 = 2.0; // LITERAL-PX-OK: ↔ glyph shaft half-thickness
/// Arrowhead size of the ↔ glyph (2×).
const DUR_ARROW_HEAD: f32 = 6.0; // LITERAL-PX-OK: ↔ glyph arrowhead size
/// Padding around the ↔ glyph the grab still covers.
const DUR_ARROW_PAD: f32 = 4.0; // LITERAL-PX-OK: grab padding around the ↔ glyph

/// Paint the ↔ drag grip for the authored duration and register it as a
/// [`TimelineHitKind::DurationHandle`] surface. A no-op unless the view has an
/// authored duration whose end is on-screen — same gate as the veil
/// ([`beyond_end_shade`]).
///
/// **No bar on the edge, and the grip is offset RIGHT** (Enio, 2026-07-23): the
/// loop brace sits ON the edge when the loop runs to the end, so a bar there and a
/// grab reaching to the edge fought it for the pointer. The affordance is just the
/// ↔ glyph, well to the right over the dark veil, and the grab is tight around the
/// glyph — clear of the edge, so the loop brace stays grabbable. The drag is
/// grab-relative (`duration_drag`), so grabbing the offset glyph does not jump the
/// duration to the glyph's position.
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
    // The ↔ glyph, well to the right of the edge (over the dark veil), centred
    // vertically in the strip: a thin shaft with a triangle arrowhead at each end.
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
    // The grab target: tight around the ↔ glyph and NOTHING at the edge, so the loop
    // brace at the duration end keeps its own grab. Across the ruler strip.
    let hit = grab_rect(edge, region.y);
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

/// The duration handle's grab rect for a veil edge at `edge` — tight around the ↔
/// glyph, offset RIGHT so it never overlaps the loop brace's own grab at the edge.
/// Pure, so the "clear of the edge" rule has an oracle the paint cannot give it.
fn grab_rect(edge: f32, region_y: f32) -> Rect {
    let gx = edge + DUR_ARROW_OFFSET;
    let hit_left = gx - DUR_ARROW_HALF_W - DUR_ARROW_HEAD - DUR_ARROW_PAD;
    let hit_right = gx + DUR_ARROW_HALF_W + DUR_ARROW_HEAD + DUR_ARROW_PAD;
    Rect::new(hit_left, region_y, hit_right - hit_left, RULER_H)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The grab is clear of the edge, so the loop brace at the duration end stays
    /// grabbable** (Enio, 2026-07-23). The loop brace's own grab reaches `BRACE_HIT_HW`
    /// either side of the edge; the duration grab must start strictly to the RIGHT of
    /// that, or (registered last, so topmost) it would steal the brace's press when the
    /// loop runs to the end.
    #[test]
    fn the_grab_does_not_reach_the_edge_where_the_loop_brace_sits() {
        let edge = 200.0;
        let g = grab_rect(edge, 0.0);
        assert!(
            g.x > edge + BRACE_HIT_HW,
            "grab starts at {} but the loop brace reaches to {}",
            g.x,
            edge + BRACE_HIT_HW
        );
        // And it does cover the ↔ glyph (so the affordance is actually grabbable).
        let gx = edge + DUR_ARROW_OFFSET;
        assert!(
            g.x < gx && g.x + g.w > gx,
            "the grab must cover the arrow at {gx}"
        );
    }
}
