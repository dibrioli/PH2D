//! **Time-scale of a key selection** (crown-jewels plan §4) — the loved retiming
//! box. A multi-key selection grows a bounding-box with a grip at each time edge;
//! dragging a grip scales the selected keys' TIME about the OPPOSITE edge (the
//! pivot), streaming one incremental `ScaleSelectedKeys` per frame inside a single
//! `BeginEdit`/`EndEdit` bracket — so it tracks the cursor and undoes in one step.
//!
//! ⚠️ This is a KEY edit and ONLY a key edit. The grip lives on the dope-sheet
//! selection ([`TimelineHitKind::SelectionTimeHandle`]); it NEVER touches a strip.
//! Strip retiming (`StretchStrip`) is a different verb on a different surface (the
//! stack lane) — the precious Clips/Strips/Fade system. The `interact` router
//! sends this hit here and here only; a gate pins that it emits `ScaleSelectedKeys`
//! and never `StretchStrip`, so a mis-route can never reach the fade.

use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TimelineGesture, TimelineHitKind,
};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TimelineViewSnapshot};
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};

use crate::loop_drag::time_at;
use crate::state::{self, ScaleDrag, TimelinePanelState};
use crate::{geom, graph, ids};

/// Visible width of a grip bar.
const HANDLE_W: f32 = 6.0; // LITERAL-PX-OK: time-scale grip bar width
/// Gap between the selection's edge diamond and the grip, so the grip's hit rect
/// never overlaps the edge key underneath (a grab there must scale, not move).
const GAP: f32 = 9.0; // LITERAL-PX-OK: grip clears the edge diamond (KEY_HIT_HW + 2)
/// Extra hit padding around a grip bar so a thin bar is still easy to grab.
const HIT_PAD: f32 = 3.0; // LITERAL-PX-OK: grip grab padding
/// The scale floor — a drag can shrink the selection close to a point but never
/// through it (a factor <= 0 would invert the key order).
const MIN_FACTOR: f64 = 0.01;

/// The `[min, max]` time (seconds) spanned by the SELECTED keys across every
/// dope-sheet track, or `None` when fewer than two distinct times are selected —
/// a single instant has no extent to scale, so no box is offered.
pub(crate) fn selection_extent(snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for t in &snap.tracks {
        for k in t.keys.iter().filter(|k| k.selected) {
            lo = lo.min(k.t_seconds);
            hi = hi.max(k.t_seconds);
        }
    }
    (hi > lo).then_some((lo, hi))
}

/// Paint the selection's time box + its two grips over the rows, and register a
/// [`TimelineHitKind::SelectionTimeHandle`] per grip. Call AFTER the rows/diamonds
/// so the grips register last (they win the hit where they sit — though the `GAP`
/// keeps them clear of the edge diamonds anyway). No box while a key-move or a
/// marquee is in flight (they own the pointer).
pub(crate) fn paint_selection_box(
    ctx: &mut PaintCtx,
    theme: Theme,
    g: &geom::Geom,
    view: graph::TimeView,
    state: &TimelinePanelState,
    snap: &TimelineViewSnapshot,
) {
    if state.key_drag.is_some() || state.box_drag.is_some() {
        return;
    }
    let Some((lo, hi)) = selection_extent(snap) else {
        return;
    };
    let rows = g.rows;
    let x_at = |t: f64| view.time_x + ((t - view.view_start) * view.px_per_s) as f32;
    let (x_lo, x_hi) = (x_at(lo), x_at(hi));
    // Cull entirely off-screen selections (both grips past an edge).
    if x_hi + GAP < view.time_x || x_lo - GAP > view.right {
        return;
    }
    let accent = resolve(ColorToken::Accent, theme);
    // A thin bracket line along the top of the rows tying the two grips together,
    // so the box reads as one selection rather than two loose bars.
    let top = Rect::new(x_lo, rows.y, (x_hi - x_lo).max(0.0), StrokeToken::Thin.px());
    fill_rounded_rect(ctx.scene, top, 0.0, accent);
    for right in [false, true] {
        let bar_x = if right {
            x_hi + GAP
        } else {
            x_lo - GAP - HANDLE_W
        };
        let bar = Rect::new(bar_x, rows.y, HANDLE_W, rows.h);
        fill_rounded_rect(ctx.scene, bar, Radius::Xs.px(), accent);
        let id = if right {
            ids::TIMELINE_SCALE_HANDLE_R
        } else {
            ids::TIMELINE_SCALE_HANDLE_L
        };
        let hit = Rect::new(bar.x - HIT_PAD, bar.y, bar.w + HIT_PAD * 2.0, bar.h);
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: ids::TIMELINE_PANEL,
                kind: TimelineHitKind::SelectionTimeHandle { right },
                canvas: hit,
            },
        );
        ctx.host.hit_index_mut().register(id, hit);
    }
}

/// Interpret one time-scale grip gesture. `right` picks the grabbed edge; the
/// pivot is the opposite one. `time_x`/`px_per_s` are the ruler's mapping.
pub(crate) fn apply(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    right: bool,
    g: TimelineGesture,
) {
    match g.phase {
        GesturePhase::Begin => {
            // The box is only drawn (thus grabbable) when an extent exists, but
            // guard anyway: no extent -> no bracket, no drag.
            let Some((lo, hi)) = selection_extent(snap) else {
                return;
            };
            let (pivot, edge) = if right { (lo, hi) } else { (hi, lo) };
            state::push_intent(TimelineIntent::BeginEdit);
            state.scale_drag = Some(ScaleDrag {
                pivot_seconds: pivot,
                edge_seconds: edge,
                right,
                applied: 1.0,
            });
        }
        GesturePhase::Update => emit_scale(state, time_x, px_per_s, snap, g.x),
        GesturePhase::End => {
            emit_scale(state, time_x, px_per_s, snap, g.x);
            state.scale_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
        GesturePhase::Click | GesturePhase::DoubleClick => {
            // A grip is a grab target, not a seek: a plain click does nothing but
            // close the (empty) bracket, which commits no undo step.
            state.scale_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Emit the incremental factor that has accrued since the last frame. The target
/// factor comes from the FIXED drag geometry (`edge`/`pivot` captured at Begin),
/// never the live extent — the scale is itself moving that extent, so reading it
/// back would feed on its own output.
fn emit_scale(
    state: &mut TimelinePanelState,
    time_x: f32,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
    x: f32,
) {
    let Some(d) = state.scale_drag else {
        return;
    };
    let span = d.edge_seconds - d.pivot_seconds;
    if span == 0.0 {
        return;
    }
    let t_cur = time_at(state.view_start_s, time_x, px_per_s, x, snap);
    let want = ((t_cur - d.pivot_seconds) / span).max(MIN_FACTOR);
    let inc = want / d.applied;
    if (inc - 1.0).abs() < 1e-9 {
        return;
    }
    if let Some(d) = state.scale_drag.as_mut() {
        d.applied = want;
    }
    state::push_intent(TimelineIntent::ScaleSelectedKeys {
        pivot_seconds: d.pivot_seconds,
        factor: inc,
    });
}

#[cfg(test)]
#[path = "scale_drag_tests.rs"]
mod tests;
