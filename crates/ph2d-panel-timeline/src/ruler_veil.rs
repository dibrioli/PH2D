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
/// **No bar on the edge; the ↔ glyph is the visual cue AND the grab** (Enio,
/// 2026-07-26 — REVERSES the earlier whole-veil grab of `d489f9d67`: *"o clique
/// para ajustar o véu é só na própria seta … qualquer região à direita dela … é
/// como se estivesse clicando nela"*). The whole-veil grab stole every press in
/// the dark dead-zone; now the grab is a tight box on the ↔ glyph only
/// ([`grab_rect`]), so clicking the veil to the right does nothing. The loop brace
/// sits ON the edge when the loop runs to the end, so the ↔ is drawn
/// `DUR_ARROW_OFFSET` to the right (clear of it). The drag is grab-relative
/// (`duration_drag`), so grabbing the glyph does not jump the duration to the pointer.
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
    // The grab target: ONLY the ↔ glyph (Enio, 2026-07-26 — reversed the whole-veil
    // grab). Clicking the dark veil to the right must NOT resize the duration; only
    // the arrow does.
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

/// The duration handle's grab rect: a tight box on the ↔ glyph ONLY (Enio,
/// 2026-07-26 — reverses the earlier whole-veil grab). The glyph is centred
/// `DUR_ARROW_OFFSET` right of the edge and reaches `DUR_ARROW_HALF_W +
/// DUR_ARROW_HEAD` either side; the grab adds `DUR_ARROW_PAD`. The 40 px offset
/// keeps it clear of the loop brace at the edge, and it never spills into the dark
/// veil to the right (the bug). Pure, so the geometry has an oracle the paint cannot give.
fn grab_rect(edge: f32, region_y: f32) -> Rect {
    let gx = edge + DUR_ARROW_OFFSET;
    let reach = DUR_ARROW_HALF_W + DUR_ARROW_HEAD + DUR_ARROW_PAD;
    Rect::new(gx - reach, region_y, reach * 2.0, RULER_H)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The grab is ONLY the ↔ glyph — it covers the arrow, stays clear of the loop
    /// brace at the edge, and does NOT spill into the veil to the right** (Enio,
    /// 2026-07-26: clicking the dark veil right of the arrow must not resize the
    /// duration; only the arrow does). Reverses the earlier whole-veil grab.
    #[test]
    fn the_grab_is_only_the_arrow_not_the_whole_veil() {
        let (edge, right) = (200.0, 800.0);
        let g = grab_rect(edge, 0.0);
        let gx = edge + DUR_ARROW_OFFSET;
        // Covers the arrow glyph...
        assert!(
            g.x <= gx && gx <= g.x + g.w,
            "the grab must cover the arrow at {gx} (got x={}..{})",
            g.x,
            g.x + g.w
        );
        // ...clear of the loop brace at the edge (registered last = topmost)...
        assert!(
            g.x > edge + BRACE_HIT_HW,
            "grab starts at {} but the loop brace reaches to {}",
            g.x,
            edge + BRACE_HIT_HW
        );
        // ...and does NOT reach the dark veil to the right (the bug): a click well
        // right of the arrow misses the grab.
        assert!(
            g.x + g.w < right,
            "the grab must NOT cover the veil to the right edge {right} (only the arrow), got {}",
            g.x + g.w
        );
        assert!(
            gx + 100.0 > g.x + g.w,
            "a click 100px right of the arrow must miss the grab"
        );
    }
}
