//! Graph editor (W3) — an expanded track row's curve, anchors and bézier
//! handles, painted in the band under its dope-sheet strip (theatre.js's
//! expand-per-track model, not a hidden global mode: B0.P1).
//!
//! The curve itself comes from [`ph2d_timeline::sample_keys`], which is pinned
//! bit-for-bit to the real `Track::sample` by a golden test — the polyline you
//! drag IS the animation that plays (B0.P4).
//!
//! **Handles** are the two control points of the segment's
//! [`Interp::Bezier`](ph2d_timeline::Interp) in normalized `[0, 1]²` timing
//! space, drawn in the `(time, value)` plane. A `Hold`/`Eased` segment has no
//! two-handle form, so its handles paint at the *linear* positions: dragging one
//! converts the segment to `Bezier` (the sanctioned upgrade path). Handles show
//! only for segments touching a **selected** key, so a dense track stays legible.
//!
//! A drag streams one `SetInterp` per frame inside a `BeginEdit`/`EndEdit`
//! bracket, so the scene follows the cursor live and one Ctrl+Z undoes the whole
//! gesture.

use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{TimelineIntent, TrackView, weighted_with_handle, weighted_with_speed_handle};

use crate::state::{self, HandleDrag, TimelinePanelState};

/// Default height of the curve band added below an expanded row's dope-sheet
/// strip. The live height is panel state (draggable) — see `state.graph_h`.
pub(crate) const GRAPH_H_DEFAULT: f32 = 132.0; // LITERAL-PX-OK: default graph band height
/// Shortest a graph band may be dragged (a curve still readable).
const GRAPH_H_MIN: f32 = 64.0; // LITERAL-PX-OK: min graph band height
/// Tallest a graph band may be dragged.
const GRAPH_H_MAX: f32 = 640.0; // LITERAL-PX-OK: max graph band height

/// Hold a dragged graph height inside its bounds.
pub(crate) fn clamp_graph_h(h: f32) -> f32 {
    h.clamp(GRAPH_H_MIN, GRAPH_H_MAX) // CLAMP-OK: const bounds, min<max, non-NaN
}
/// Breathing room above and below the fitted range, as a fraction of it.
const V_PAD_FRAC: f64 = 0.1; // LITERAL-PX-OK: vertical fit margin, fraction of the drawn range
/// Half-height of the fitted range when everything drawn sits at one value (a
/// flat track has no range to take a fraction of).
const V_PAD_FLAT: f64 = 0.5; // LITERAL-PX-OK: fallback half-range for a flat track

/// The time axis, shared with the ruler and the dope-sheet lanes.
#[derive(Clone, Copy)]
pub(crate) struct TimeView {
    /// Screen x where `view_start` maps.
    pub time_x: f32,
    /// Right edge of the time area.
    pub right: f32,
    /// Seconds at `time_x`.
    pub view_start: f64,
    /// Zoom.
    pub px_per_s: f64,
}

impl TimeView {
    pub(crate) fn x(self, t: f64) -> f32 {
        self.time_x + ((t - self.view_start) * self.px_per_s) as f32
    }
    pub(crate) fn t(self, x: f32) -> f64 {
        self.view_start + f64::from(x - self.time_x) / self.px_per_s
    }
}

/// One expanded row's value↔pixel mapping: the fitted (and padded) value range
/// projected onto the band's rect, `v_max` at the top.
#[derive(Clone, Copy)]
pub(crate) struct Band {
    pub rect: Rect,
    pub v_min: f64,
    pub v_max: f64,
}

impl Band {
    /// Fit `extent` (what the row actually DRAWS — see
    /// [`ph2d_timeline::drawn_extent`]) into `rect`, with a margin. `None` (an
    /// empty track) gets a symmetric unit window so the mapping stays finite.
    pub(crate) fn fit(rect: Rect, extent: Option<(f32, f32)>) -> Self {
        let (lo, hi) = extent.map_or((0.0, 0.0), |(a, b)| (f64::from(a), f64::from(b)));
        let range = hi - lo;
        let pad = if range > 0.0 {
            range * V_PAD_FRAC
        } else {
            V_PAD_FLAT
        };
        Self::from_range(rect, lo - pad, hi + pad)
    }

    /// A band over an explicit value range — used to hold the mapping still for
    /// the length of a handle drag (see `graph_paint::paint_track`).
    pub(crate) fn from_range(rect: Rect, v_min: f64, v_max: f64) -> Self {
        Self { rect, v_min, v_max }
    }

    /// The value range this band maps, for freezing it across frames.
    pub(crate) fn range(&self) -> (f64, f64) {
        (self.v_min, self.v_max)
    }

    pub(crate) fn y(&self, v: f64) -> f32 {
        let span = self.v_max - self.v_min;
        let f = if span > 0.0 {
            (self.v_max - v) / span
        } else {
            0.5
        };
        self.rect.y + (f as f32) * self.rect.h
    }

    pub(crate) fn value(&self, y: f32) -> f64 {
        if self.rect.h <= 0.0 {
            return self.v_min;
        }
        let f = f64::from((y - self.rect.y) / self.rect.h);
        self.v_max - f * (self.v_max - self.v_min)
    }
}

/// The band this row is drawn against, and the one an in-flight drag resolves
/// its pointer through.
///
/// A band that refit every frame would feed back into the value the pointer maps
/// to: drag a handle (or an anchor) up, the fitted range grows, the same screen y
/// now means a smaller value, and the grabbed point crawls away from the cursor.
/// So the FIRST paint of a drag freezes the range, and every later frame reuses
/// it. An anchor drag needs this even more than a handle drag: it moves the very
/// key values `extent` is measured from.
///
/// `extent` is what the row actually draws (see [`ph2d_timeline::drawn_extent`]);
/// `None` (an empty track) fits to a symmetric unit window.
pub(crate) fn band_for(
    state: &mut TimelinePanelState,
    rect: Rect,
    target: u64,
    extent: Option<(f32, f32)>,
) -> Band {
    let handle_here = state.handle_drag.is_some_and(|d| d.target == target);
    let frozen = state
        .handle_drag
        .filter(|_| handle_here)
        .and_then(|d| d.range)
        .or_else(|| crate::anchor_drag::frozen_range(state, target));
    let band = match frozen {
        Some((lo, hi)) => Band::from_range(rect, lo, hi),
        None => Band::fit(rect, extent),
    };
    if handle_here
        && let Some(d) = state.handle_drag.as_mut()
        && d.range.is_none()
    {
        d.range = Some(band.range());
    }
    crate::anchor_drag::freeze_range(state, target, band.range());
    band
}

/// Interpret one gesture on a bézier handle. Geometry-free: the pointer position
/// is only *recorded* here (the band's value↔pixel mapping lives in `paint`), and
/// [`resolve_drag`] turns it into a `SetInterp` once the band is resolved.
pub(crate) fn apply_handle_gesture(
    state: &mut TimelinePanelState,
    target: u64,
    key: u64,
    which: u8,
    g: ph2d_editor_core::interaction::TimelineGesture,
) {
    use ph2d_editor_core::interaction::GesturePhase as P;
    match g.phase {
        P::Begin => {
            // Open the undo bracket now: every frame's SetInterp joins this step.
            state::push_intent(TimelineIntent::BeginEdit);
            state.handle_drag = Some(HandleDrag {
                target,
                key,
                which,
                x: g.x,
                y: g.y,
                range: None,
                ending: false,
            });
        }
        P::Update => {
            if let Some(d) = state.handle_drag.as_mut() {
                (d.x, d.y) = (g.x, g.y);
            }
        }
        P::End => {
            if let Some(d) = state.handle_drag.as_mut() {
                (d.x, d.y, d.ending) = (g.x, g.y, true);
            }
        }
        // A tap on a handle moved nothing: close the bracket (it commits no step
        // because the document did not change) and drop the drag.
        P::Click | P::DoubleClick => {
            state.handle_drag = None;
            state::push_intent(TimelineIntent::EndEdit);
        }
    }
}

/// Turn this frame of an in-flight handle drag into a `SetInterp`, now that the
/// band's value↔pixel mapping is known. Closes the undo bracket on the last one.
pub(crate) fn resolve_drag(
    state: &mut TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let Some(d) = state.handle_drag else {
        return;
    };
    if d.target != track.target.get() {
        return;
    }
    let Some(i) = track.keys.iter().position(|k| k.id.get() == d.key) else {
        return;
    };
    let Some(k1) = track.keys.get(i + 1) else {
        return; // the last key owns no segment
    };
    let k0 = &track.keys[i];
    // Every tangent drag lands on a WEIGHTED (value-space) pair — the strictly
    // more expressive form: the untouched handle keeps the exact position it is
    // drawn at (a lossless conversion for any interp), and a flat segment
    // finally takes the drag (its `dy` is absolute, the gap the normalized form
    // could not express).
    let interp = if state.speed_view {
        // Speed edit, the AE gesture: vertical = VELOCITY, horizontal =
        // INFLUENCE (the arm's length) — one tip drags both.
        weighted_with_speed_handle(k0, k1, d.which, view.t(d.x), band.value(d.y))
    } else {
        // Value edit: the dragged handle follows the pointer in (time, value).
        weighted_with_handle(k0, k1, d.which, view.t(d.x), band.value(d.y))
    };
    state::push_intent(TimelineIntent::SetInterp {
        target: track.target,
        key: k0.id,
        interp,
    });
    if d.ending {
        state::push_intent(TimelineIntent::EndEdit);
        state.handle_drag = None;
    }
}

#[cfg(test)]
#[path = "graph_tests.rs"]
mod tests;
