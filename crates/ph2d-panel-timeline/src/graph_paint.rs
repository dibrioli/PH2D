//! Graph-editor **paint** (W3) — the expanded row's band, curve, anchors and
//! bézier handles. Split from `graph` (the gesture + drag math) under the HR-18
//! panel LOC cap; that module owns every fact this one draws.
//!
//! Handles show only for segments touching a **selected** key, so a dense track
//! stays legible (the plan's anti-pollution rule). A `Hold`/`Eased` segment has
//! no two-handle form, so its handles paint at the LINEAR positions — dragging
//! one is what converts the segment to `Bezier`.

use ph2d_editor_core::interaction::{InteractiveState, TimelineHitKind};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, paint_text, rect_to_vello, resolve, stroke_polyline,
    stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::{KeyView, TrackView, drawn_extent, handle_point, sample_keys};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};

use crate::graph::{Band, TimeView, handle_pair, resolve_drag};
use crate::ids;
use crate::state::TimelinePanelState;

/// Horizontal spacing of the polyline samples. Two pixels is below the eye's
/// resolution for a smooth curve and keeps a wide band under ~500 samples.
const CURVE_STEP_PX: f32 = 2.0; // LITERAL-PX-OK: curve polyline sample spacing
const CURVE_W: f32 = 1.5; // LITERAL-PX-OK: curve stroke width
const ANCHOR_R: f32 = 3.0; // LITERAL-PX-OK: key anchor dot radius
const HANDLE_R: f32 = 3.5; // LITERAL-PX-OK: bezier handle dot radius
const HANDLE_HIT_R: f32 = 7.0; // LITERAL-PX-OK: bezier handle grab radius
const BAND_INSET: f32 = 2.0; // LITERAL-PX-OK: gap between the strip and its graph band
/// Height of the grab strip along the band bottom (the vertical resize grip).
const GRIP_H: f32 = 6.0; // LITERAL-PX-OK: graph-height grip strip height
/// Length of the visible grabber drawn in the middle of that strip.
const GRIP_BAR_W: f32 = 28.0; // LITERAL-PX-OK: graph-height grabber bar width
const GRIP_BAR_H: f32 = 2.0; // LITERAL-PX-OK: graph-height grabber bar thickness

/// Paint one expanded track's graph band and register its handle hits, then let
/// an in-flight handle drag resolve against the geometry it can finally see.
pub(crate) fn paint_track(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &mut TimelinePanelState,
    rect: Rect,
    label_w: f32,
    view: TimeView,
    track: &TrackView,
) {
    // Aligned with the lanes above it: the curve for `t = view_start` lands on
    // the band's left edge, clear of the splitter grip.
    let band_rect = Rect::new(
        view.time_x,
        rect.y + BAND_INSET,
        (rect.x + rect.w - view.time_x).max(0.0),
        (rect.h - BAND_INSET * 2.0).max(0.0),
    );
    let dragging_here = state
        .handle_drag
        .is_some_and(|d| d.target == track.target.get());
    let frozen = state
        .handle_drag
        .filter(|_| dragging_here)
        .and_then(|d| d.range);
    let band = match frozen {
        // Hold the mapping still for the length of a drag: a band that refit each
        // frame would change what the pointer's y MEANS, and the handle would
        // crawl away from the cursor.
        Some((lo, hi)) => Band::from_range(band_rect, lo, hi),
        // Otherwise fit to everything the row draws — the curve (overshoot and
        // all) and the visible handles, not just the key values. Fitting the keys
        // alone let a strong ease spill out of the band onto the next row.
        None => Band::fit(band_rect, drawn_extent_of(&band_rect, view, track)),
    };
    if dragging_here
        && let Some(d) = state.handle_drag.as_mut()
        && d.range.is_none()
    {
        d.range = Some(band.range());
    }
    paint_frame(ctx, theme, rect, band_rect, label_w, &band);
    if track.keys.is_empty() {
        return;
    }
    // Everything below is confined to the band: a drag can push a handle past the
    // frozen range, and it must not draw over the neighbouring rows.
    ctx.scene.push_clip(&rect_to_vello(band_rect));
    paint_curve(ctx, theme, &band, view, &track.keys);
    paint_anchors(ctx, theme, &band, view, &track.keys);
    paint_handles(ctx, theme, state, &band, view, track);
    ctx.scene.pop_layer();
    resolve_drag(state, &band, view, track);
    paint_height_grip(ctx, theme, state, rect, track.target.get());
}

/// The grip along the bottom of an expanded row: a short centred bar plus a
/// full-width invisible strip to grab it by. Every row's grip drags the SAME
/// height, so a graph opened later comes up at the size the author chose.
fn paint_height_grip(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    row: Rect,
    target: u64,
) {
    let strip = Rect::new(row.x, row.y + row.h - GRIP_H, row.w, GRIP_H);
    let bar = Rect::new(
        row.x + (row.w - GRIP_BAR_W) * 0.5,
        strip.y + (GRIP_H - GRIP_BAR_H) * 0.5,
        GRIP_BAR_W,
        GRIP_BAR_H,
    );
    let tok = if state.graph_resize.is_some() {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    fill_rounded_rect(ctx.scene, bar, Radius::Xs.px(), resolve(tok, theme));

    let id = ids::timeline_graph_resize_id(target);
    ctx.host.store_mut().register(
        id,
        InteractiveState::TimelineSurface {
            parent: ids::TIMELINE_PANEL,
            kind: TimelineHitKind::GraphResize,
            canvas: strip,
        },
    );
    ctx.host.hit_index_mut().register(id, strip);
}

/// The value extent of what this row draws across its visible time window, at
/// the same sample density the polyline uses.
fn drawn_extent_of(band_rect: &Rect, view: TimeView, track: &TrackView) -> Option<(f32, f32)> {
    let samples = (band_rect.w / CURVE_STEP_PX).ceil() as usize;
    drawn_extent(
        &track.keys,
        view.t(band_rect.x),
        view.t(band_rect.x + band_rect.w),
        samples,
    )
}

/// The inset background + border, and the fitted range's min/max labels in the
/// label column (P6: no ambiguous spec — the vertical scale is always readable).
fn paint_frame(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: Rect,
    band_rect: Rect,
    label_w: f32,
    band: &Band,
) {
    fill_rounded_rect(
        ctx.scene,
        band_rect,
        Radius::Xs.px(),
        resolve(ColorToken::Bg0, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        band_rect,
        Radius::Xs.px(),
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    let font = TypeToken::Xs.px();
    let x = row.x + Spacing::Xs.px();
    let w = (label_w - Spacing::Xs.px() * 2.0).max(0.0);
    let color = resolve(ColorToken::Text3, theme);
    for (v, y) in [
        (band.v_max, band_rect.y),
        (band.v_min, band_rect.y + band_rect.h - font),
    ] {
        paint_text(
            ctx.text_system,
            ctx.scene,
            &format!("{v:.2}"),
            x,
            y,
            font,
            w,
            color,
        );
    }
}

/// The curve itself: one sample every [`CURVE_STEP_PX`] across the visible span,
/// through the same sampler the runtime plays.
fn paint_curve(ctx: &mut PaintCtx, theme: Theme, band: &Band, view: TimeView, keys: &[KeyView]) {
    let (left, right) = (band.rect.x, band.rect.x + band.rect.w);
    if right <= left {
        return;
    }
    let n = ((right - left) / CURVE_STEP_PX).ceil() as usize + 1;
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let x = (left + i as f32 * CURVE_STEP_PX).min(right);
        if let Some(v) = sample_keys(keys, view.t(x)) {
            points.push((x, band.y(f64::from(v))));
        }
    }
    stroke_polyline(
        ctx.scene,
        &points,
        CURVE_W,
        resolve(ColorToken::Accent, theme),
    );
}

/// A dot per key, on the curve. Selected keys take the accent, matching their
/// diamond in the strip above.
fn paint_anchors(ctx: &mut PaintCtx, theme: Theme, band: &Band, view: TimeView, keys: &[KeyView]) {
    for k in keys {
        let (x, y) = (view.x(k.t_seconds), band.y(f64::from(k.value)));
        if x < band.rect.x || x > band.rect.x + band.rect.w {
            continue;
        }
        let tok = if k.selected {
            ColorToken::Accent
        } else {
            ColorToken::Text1
        };
        fill_circle(ctx.scene, x, y, ANCHOR_R, resolve(tok, theme));
    }
}

/// Two handles per segment that touches a selected key, each with its tangent
/// line back to the anchor it belongs to, plus their grab targets.
fn paint_handles(
    ctx: &mut PaintCtx,
    theme: Theme,
    state: &TimelinePanelState,
    band: &Band,
    view: TimeView,
    track: &TrackView,
) {
    let target = track.target.get();
    for w in track.keys.windows(2) {
        let (k0, k1) = (&w[0], &w[1]);
        if !(k0.selected || k1.selected) {
            continue;
        }
        for (which, (hx, hy)) in handle_pair(k0.interp).into_iter().enumerate() {
            let which = which as u8;
            let (t, v) = handle_point(
                k0.t_seconds,
                f64::from(k0.value),
                k1.t_seconds,
                f64::from(k1.value),
                (hx, hy),
            );
            let (px, py) = (view.x(t), band.y(v));
            let anchor = if which == 0 { k0 } else { k1 };
            let (ax, ay) = (view.x(anchor.t_seconds), band.y(f64::from(anchor.value)));
            if px < band.rect.x || px > band.rect.x + band.rect.w {
                continue;
            }
            stroke_polyline(
                ctx.scene,
                &[(ax, ay), (px, py)],
                StrokeToken::Thin.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
            let dragged = state
                .handle_drag
                .is_some_and(|d| d.target == target && d.key == k0.id.get() && d.which == which);
            let tok = if dragged {
                ColorToken::AccentPress
            } else {
                ColorToken::Accent
            };
            fill_circle(ctx.scene, px, py, HANDLE_R, resolve(tok, theme));

            // A handle a drag pushed outside the frozen band is drawn clipped;
            // do not leave a grab target for it floating over another row.
            if py < band.rect.y || py > band.rect.y + band.rect.h {
                continue;
            }
            let id = ids::timeline_handle_hit_id(target, k0.id.get(), which);
            let hit = Rect::new(
                px - HANDLE_HIT_R,
                py - HANDLE_HIT_R,
                HANDLE_HIT_R * 2.0,
                HANDLE_HIT_R * 2.0,
            );
            ctx.host.store_mut().register(
                id,
                InteractiveState::TimelineSurface {
                    parent: ids::TIMELINE_PANEL,
                    kind: TimelineHitKind::CurveHandle {
                        target,
                        key: k0.id.get(),
                        which,
                    },
                    canvas: band.rect,
                },
            );
            ctx.host.hit_index_mut().register(id, hit);
        }
    }
}
