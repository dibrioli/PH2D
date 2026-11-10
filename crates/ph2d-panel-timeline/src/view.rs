//! View gestures of the dope sheet: what moves the *camera*, not the document.
//!
//! Sibling of `interact` (which owns the key/selection gestures), split out
//! under the HR-18 panel LOC cap. Three behaviours, all Blender-shaped:
//!
//! - **wheel** — plain = anchored zoom of the time axis (the time under the
//!   cursor stays put), Ctrl = horizontal pan, Shift = vertical row scroll.
//! - **middle-drag** — grab-and-slide both axes at once.
//! - **F / transport jump** — fit to what THIS TAB is showing (the keys, or the
//!   lanes' strips), or pan the playhead into view.
//!
//! Chrome resizes (panel edges, label column, graph height) live in `resize`.
//! None of these raise intents: the view is panel-local and never undoable.

use ph2d_editor_core::interaction::{GesturePhase, TimelineGesture, TimelineWheel};
use ph2d_timeline::TimelineViewSnapshot;

use crate::state::{DEFAULT_PX_PER_S, MAX_PX_PER_S, MIN_PX_PER_S, TimelinePanelState};

/// Wheel **pixels** per e-fold of zoom. The shell delivers line-deltas already
/// scaled to logical px (16 px per notch), so one notch is ~7% zoom here — the
/// same sensitivity the motion graph uses.
const ZOOM_WHEEL_DIV: f64 = 240.0; // LITERAL-PX-OK: wheel px → zoom-factor sensitivity divisor

/// Fraction of the fitted span left as breathing room on each side, so the first
/// and last diamonds never sit exactly on the panel border.
const FIT_PAD: f64 = 0.05; // LITERAL-PX-OK: fit margin as a fraction of the span
/// The span a fit falls back to when every key sits at the same instant (or the
/// clip has a single key): one second centred on it, so the zoom stays sane.
const FIT_DEGENERATE_SPAN_S: f64 = 1.0; // LITERAL-PX-OK: fallback span for a zero-width key extent
/// Breathing room kept between a revealed playhead and the edge it came in from,
/// as a fraction of the visible span.
const REVEAL_MARGIN: f64 = 0.1; // LITERAL-PX-OK: reveal margin as a fraction of the span

/// Apply one frame's accumulated wheel: `pan` slides `view_start_s`, `scroll`
/// slides the rows, `zoom` scales `px_per_s` about the cursor holding the time
/// under `anchor_x` fixed. `view_start_s` never goes negative (t=0 is the left
/// bound of the clip); `scroll_y` stays within the measured overflow.
pub(crate) fn apply_wheel(state: &mut TimelinePanelState, time_x: f32, w: TimelineWheel) {
    // Pan first, in the pre-zoom scale (what the user saw when they scrolled).
    // Deltas are already in logical px; a positive delta scrolls the content
    // right/down ⇒ the view moves EARLIER / up, the same sign convention as the
    // panel-scroll path (`panel_scroll - delta_y`).
    if w.pan_delta != 0.0 {
        state.view_start_s -= f64::from(w.pan_delta) / state.px_per_s;
    }
    if w.scroll_delta != 0.0 {
        state.scroll_y = (state.scroll_y - w.scroll_delta).clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max
    }
    if w.zoom_delta != 0.0 {
        let old = state.px_per_s;
        let new = (old * (f64::from(w.zoom_delta) / ZOOM_WHEEL_DIV).exp())
            .clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max, non-NaN
        // Hold the time under the cursor: t = start + (anchor - time_x)/px_per_s.
        let off_px = f64::from(w.anchor_x - time_x);
        let t_anchor = state.view_start_s + off_px / old;
        state.view_start_s = t_anchor - off_px / new;
        state.px_per_s = new;
    }
    state.view_start_s = state.view_start_s.max(0.0);
}

/// Fit the time axis to the extent of the content this tab shows (`F`, Blender's
/// per-area focus) — see [`content_extent`] — with a small margin. An empty document — or a zero-width
/// `time_area` — resets to the default zoom at `t = 0`; a single key (or several
/// stacked at one instant) gets a one-second window around it rather than an
/// infinite zoom.
pub(crate) fn apply_fit(
    state: &mut TimelinePanelState,
    time_area_w: f32,
    snap: &TimelineViewSnapshot,
) {
    fit_to(state, time_area_w, content_extent(state, snap));
}

/// ⭐ **Abrir a fita ou trocar de clipe** (cartões, doc 15 onda 2): o mesmo enquadrar do `F`, mas sobre
/// o CLIPE inteiro, `0 → fim`, unido às chaves que passem dele — o `F` enquadra só as chaves, e um
/// clipe com chaves em 1–2 s abria sem o começo.
pub(crate) fn apply_fit_clip(
    state: &mut TimelinePanelState,
    time_area_w: f32,
    snap: &TimelineViewSnapshot,
) {
    let clipe = (snap.view_length_seconds > 0.0).then_some((0.0_f64, snap.view_length_seconds));
    let extent = match (clipe, content_extent(state, snap)) {
        (Some((a, b)), Some((c, d))) => Some((a.min(c), b.max(d))),
        (x, y) => x.or(y),
    };
    fit_to(state, time_area_w, extent);
}

fn fit_to(state: &mut TimelinePanelState, time_area_w: f32, extent: Option<(f64, f64)>) {
    let w = f64::from(time_area_w);
    let Some((first, last)) = extent.filter(|_| w > 0.0) else {
        state.view_start_s = 0.0;
        state.px_per_s = DEFAULT_PX_PER_S;
        return;
    };
    let (first, last) = if last - first > 0.0 {
        let pad = (last - first) * FIT_PAD;
        (first - pad, last + pad)
    } else {
        let half = FIT_DEGENERATE_SPAN_S * 0.5;
        (first - half, first + half)
    };
    state.px_per_s = (w / (last - first)).clamp(MIN_PX_PER_S, MAX_PX_PER_S); // CLAMP-OK: const bounds, min<max, non-NaN
    // Re-derive the start from the CLAMPED zoom, so a clamp keeps the extent
    // centred instead of pinning it to the left edge.
    let visible = w / state.px_per_s;
    state.view_start_s = ((first + last) * 0.5 - visible * 0.5).max(0.0);
}

/// Pan the time axis until `t` is visible, keeping [`REVEAL_MARGIN`] of the span
/// between it and the edge it was revealed against. A no-op when `t` is already
/// comfortably in view, so a frame-step in the middle of the strip never jerks
/// the view; `view_start_s` never goes negative, so go-to-start lands flush on
/// `t = 0`.
pub(crate) fn reveal_time(state: &mut TimelinePanelState, time_area_w: f32, t: f64) {
    let w = f64::from(time_area_w);
    if w <= 0.0 || state.px_per_s <= 0.0 {
        return;
    }
    let span = w / state.px_per_s;
    let margin = span * REVEAL_MARGIN;
    let start = state.view_start_s;
    if t < start + margin {
        state.view_start_s = (t - margin).max(0.0);
    } else if t > start + span - margin {
        state.view_start_s = (t - span + margin).max(0.0);
    }
}

/// What a fit frames: **the content THIS TAB is showing**.
///
/// One ruler, two contents (see `tab.rs`): the Keys tab draws the active clip's
/// keys, the Arrange tab draws the lanes' strips. Framing the keys in Arrange sized
/// the view to the active CLIP's duration — and for a first strip that plays that
/// clip 1:1 from the top, "the clip's duration" and "the first strip" are the same
/// pixels, so the fit looked deliberate while every strip after it stayed off
/// screen (Enio, 2026-07-16). The tab already decides what the ruler MEANS; it has
/// to decide what the ruler FITS by the same token.
///
/// Arrange falls back to the keys when no lane holds a strip: `F` on a document
/// nobody has arranged yet should still frame something rather than snapping back
/// to the default zoom.
fn content_extent(state: &TimelinePanelState, snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    if state.tab.shows_lanes() {
        strip_extent(snap).or_else(|| key_extent(snap))
    } else {
        key_extent(snap)
    }
}

/// The `(earliest, latest)` timeline second any strip occupies, across every lane,
/// or `None` when the stack holds none.
///
/// **A strip's outward lead-in counts.** It is lane time the strip's entry reaches
/// back over — content, not margin — so a fit that cropped it would hide the travel
/// the artist authored, at the one moment they asked to see everything.
fn strip_extent(snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    snap.lanes
        .iter()
        .flat_map(|l| &l.strips)
        .fold(None, |acc: Option<(f64, f64)>, s| {
            let span = (s.t_start - s.lead_in.max(0.0), s.t_end);
            Some(acc.map_or(span, |(lo, hi)| (lo.min(span.0), hi.max(span.1))))
        })
}

/// The `(earliest, latest)` key time across every track, or `None` if the
/// document has no keys at all.
fn key_extent(snap: &TimelineViewSnapshot) -> Option<(f64, f64)> {
    let mut times = snap
        .tracks
        .iter()
        .flat_map(|t| &t.keys)
        .map(|k| k.t_seconds);
    let first = times.next()?;
    Some(times.fold((first, first), |(lo, hi), t| (lo.min(t), hi.max(t))))
}

/// Middle-drag: grab-and-slide both axes. Dragging right moves the content right,
/// so the view moves EARLIER; dragging down reveals earlier rows.
pub(crate) fn apply_pan_drag(state: &mut TimelinePanelState, px_per_s: f64, g: TimelineGesture) {
    match g.phase {
        GesturePhase::Begin => state.pan_drag = Some((g.x, g.y)),
        GesturePhase::Update => {
            if let Some((ax, ay)) = state.pan_drag {
                state.view_start_s = (state.view_start_s - f64::from(g.x - ax) / px_per_s).max(0.0);
                state.scroll_y = (state.scroll_y - (g.y - ay)).clamp(0.0, state.scroll_max); // CLAMP-OK: measured bounds, min<=max
                state.pan_drag = Some((g.x, g.y));
            }
        }
        _ => state.pan_drag = None,
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
