//! Bulk **time-transform** intent bodies — scale the markers a time-scale box
//! carries, and reverse a selection about one global pivot. Split from
//! `intent_apply.rs` under the workspace LOC cap; a CHILD module (the
//! `intent_loop_sync` idiom) so it reaches the parent's `edit` /
//! `for_selected_tracks` (private) instead of widening them.

use super::{edit, for_selected_tracks};
use crate::TimelineState;
use ph2d_anim::RationalTime;

/// Body of [`TimelineIntent::ScaleMarkers`](crate::TimelineIntent::ScaleMarkers):
/// scale the listed markers' times about `pivot_seconds` by `factor`, UNSNAPPED so
/// the incremental stream composes to the same total as the keys (`scale_keys` is
/// also unsnapped). Read the current times, compute the targets, then move; a bad
/// index (a marker deleted mid-drag) is skipped.
pub(crate) fn scale_markers(
    state: &mut TimelineState,
    indices: &[usize],
    pivot_seconds: f64,
    factor: f64,
) {
    edit(state, |doc, _| {
        let updates: Vec<(usize, RationalTime)> = indices
            .iter()
            .filter_map(|&i| {
                doc.markers().get(i).map(|m| {
                    let t = m.t.to_seconds();
                    let s = pivot_seconds + (t - pivot_seconds) * factor;
                    (i, RationalTime::from_seconds(s))
                })
            })
            .collect();
        for (i, t) in updates {
            doc.move_marker(i, t);
        }
    });
}

/// Body of [`TimelineIntent::ReverseSelectedKeys`](crate::TimelineIntent::ReverseSelectedKeys):
/// mirror the selection about ONE global pivot — the centre of the union of
/// selected key times — so a multi-track selection reverses coherently (AE). Read
/// the span before mutating; the immutable borrow ends before `for_selected_tracks`.
pub(crate) fn reverse_selected(state: &mut TimelineState) {
    edit(state, |doc, sel| {
        let span = {
            let clip = doc.active_clip();
            sel.keys()
                .iter()
                .filter_map(|sk| clip.track(sk.target)?.key(sk.key))
                .fold((f64::MAX, f64::MIN), |(lo, hi), k| {
                    let t = k.t.to_seconds();
                    (lo.min(t), hi.max(t))
                })
        };
        if span.0 <= span.1 {
            let pivot = 0.5 * (span.0 + span.1);
            for_selected_tracks(doc, sel, |track, ids| track.reverse_keys(ids, pivot));
        }
    });
}

/// Body of [`TimelineIntent::DistributeSelectedKeys`](crate::TimelineIntent::DistributeSelectedKeys):
/// respace each track's selected keys uniformly within their own span. Per-track
/// (`for_selected_tracks`) — a multi-track selection distributes each track
/// independently, and [`Track::distribute_keys`](ph2d_anim::Track::distribute_keys)
/// no-ops a track with fewer than three selected keys.
pub(crate) fn distribute_selected(state: &mut TimelineState) {
    edit(state, |doc, sel| {
        for_selected_tracks(doc, sel, |track, ids| track.distribute_keys(ids));
    });
}

/// Body of [`TimelineIntent::StaggerSelectedKeys`](crate::TimelineIntent::StaggerSelectedKeys):
/// shift each selected track rigidly by `rank · step_seconds`, where `rank` is
/// the track's position in the selection's stable order (`distinct_targets`, by
/// `AnimTarget`). Rank 0 stays put; each later track cascades further.
///
/// The delta is frame-snapped through the parent's `delta_time` (like
/// `MoveSelectedKeys`), so a frame-aligned step keeps every key frame-exact.
/// The step arrives incremental from the drag; a constant rank makes the
/// per-frame shifts compose to `rank · total_step`.
pub(crate) fn stagger_selected(state: &mut TimelineState, step_seconds: f64) {
    let fps = state.doc.fps_display;
    let frame_snap = state.flags.frame_snap;
    edit(state, |doc, sel| {
        for (rank, target) in super::distinct_targets(sel).into_iter().enumerate() {
            if rank == 0 {
                continue; // the first track anchors the cascade
            }
            let ids = sel.ids_for(target);
            let delta = super::delta_time(step_seconds * rank as f64, fps, frame_snap);
            if delta == ph2d_anim::RationalTime::ZERO {
                continue;
            }
            if let Some(track) = doc.active_clip_mut().track_mut(target) {
                track.move_keys(&ids, delta);
            }
        }
    });
}
