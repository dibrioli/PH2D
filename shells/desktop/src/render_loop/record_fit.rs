//! **Dense samples in, a curve out** — the cleanup that ends a recording
//! session, shared by everything that records one.
//!
//! Two things in this editor produce a value per frame and then have to become
//! keyframes an animator can edit: the timeline's **record** (a gizmo dragged
//! while the clock runs) and W4's **bake** (the physics simulation read out
//! tick by tick). They are the same problem — hundreds of dense keys that must
//! collapse to a handful without moving — and this module is the one answer, so
//! a baked curve and a recorded curve cannot come out looking like they were
//! made by different tools.
//!
//! It was extracted FROM the record rather than written for the bake: the
//! calibration below is the record's, arrived at through Enio's smokes (§17 of
//! the timeline handoff), and re-deriving it for the bake would have been a
//! second set of numbers to keep in agreement with the first.
//!
//! The pipeline, per entity: fit every track to find the times it wants → merge
//! near-coincident times into shared columns → re-fit every track AT those
//! columns. Column-aligned keys are what hand-keyed animation looks like, and
//! they are what lets the animator grab a column and re-time the whole object.

use std::collections::BTreeMap;

use ph2d_timeline::{PropKind, TimelineState};

/// Value tolerance the record-cleanup fit targets, as a fraction of each track's
/// recorded value range — 1% is visually lossless while cutting the dense
/// per-frame keys to a handful ([`ph2d_anim::fit_fcurve`]). Per-channel: the fit
/// normalises by the range, so this reads the same on a pixel track and a radian
/// track. (Paired with the low-pass below: together they take a noisy 120-sample
/// gesture to ~5-6 clean keys; measured in `curve_fit` calibration.)
pub(crate) const REC_SIMPLIFY_REL: f64 = 0.01;
/// Absolute value-tolerance floor, so a near-constant track (its range ~0) does
/// not get an impossibly tight tolerance that keeps every noise sample.
const REC_SIMPLIFY_FLOOR: f64 = 1e-4;
/// Low-pass passes applied to the recorded values before the fit — strips the
/// hand/mouse tremor that otherwise makes the fit over-subdivide (the "reduziu
/// um pouco" symptom). A binomial `[1,2,1]` kernel ×8 ≈ a ~9-sample window,
/// which at 60 fps is ~150 ms — removes jitter, keeps the gesture's shape.
pub(crate) const REC_SMOOTH_PASSES: usize = 8;
/// Two key times closer than this collapse into ONE dope-sheet column when the
/// session's tracks are aligned — so a channel that turns a frame or two after
/// another still shares its column instead of sitting beside it. ~2 frames at
/// 24 fps: tight enough that genuinely separate beats stay separate.
const COLUMN_MERGE_S: f64 = 0.08;

/// The recorded time+value span of one `(entity, prop)` track during a performing
/// session — enough to simplify exactly the recorded range at a proportional
/// tolerance when the drag ends.
#[derive(Clone, Copy)]
pub(crate) struct RecSpan {
    t_min: f64,
    t_max: f64,
    v_min: f64,
    v_max: f64,
}

impl RecSpan {
    pub(crate) fn seed(t: f64, v: f64) -> Self {
        Self {
            t_min: t,
            t_max: t,
            v_min: v,
            v_max: v,
        }
    }
    pub(crate) fn extend(&mut self, t: f64, v: f64) {
        self.t_min = self.t_min.min(t);
        self.t_max = self.t_max.max(t);
        self.v_min = self.v_min.min(v);
        self.v_max = self.v_max.max(v);
    }
}

/// Clean up everything one performing session recorded, **per entity**: fit each
/// of its tracks, then re-fit them all at ONE shared set of key times so the dope
/// sheet reads as aligned columns (Enio: "keys for x and y of translate and scale
/// created at the same point in time"). Column-aligned keys are what hand-keyed
/// animation looks like — the animator grabs a column and re-times every channel
/// of the object together.
///
/// The shared times are the union of each track's own extrema, with near-coincident
/// times merged ([`COLUMN_MERGE_S`]) so two channels that turn at *almost* the same
/// instant land on one column instead of two a frame apart. `simplify_range_at`
/// then pins every track's keys to those times (no splitting — a split would land
/// off the column grid).
pub(crate) fn simplify_recorded(
    timeline: &mut TimelineState,
    record: &BTreeMap<(u64, PropKind), RecSpan>,
    smooth_passes: usize,
) {
    // Group the session's tracks by entity — alignment is per OBJECT (two objects
    // recorded at once keep independent timing).
    let mut by_entity: BTreeMap<u64, Vec<(PropKind, RecSpan)>> = BTreeMap::new();
    for (&(entity, prop), &span) in record {
        by_entity.entry(entity).or_default().push((prop, span));
    }
    for (entity, props) in by_entity {
        // Pass 1 — each track's own fit proposes the times it wants (its extrema).
        let mut times: Vec<f64> = Vec::new();
        for &(prop, span) in &props {
            let Some(target) = timeline.doc.binding_for(entity, prop).map(|b| b.target) else {
                continue;
            };
            let Some(track) = timeline.doc.active_clip().track(target) else {
                continue;
            };
            let channel = prop.fit_channel();
            let Some(rs) = track.range_samples(span.t_min, span.t_max, channel, smooth_passes)
            else {
                continue;
            };
            // Tolerance off the PREPARED samples, not the raw `RecSpan`: an angle
            // channel's raw extent is one wrapped turn (~2π) however many times it
            // actually spun, so a raw-derived tolerance would be far too tight for
            // the unwrapped curve the fit now sees.
            let tol = value_tol(&rs.samples);
            times.extend(
                ph2d_anim::fit_fcurve(&rs.samples, tol, channel.bounds)
                    .iter()
                    .map(|k| k.t),
            );
        }
        if times.is_empty() {
            continue;
        }
        // Pass 2 — merge near-coincident times into single columns.
        times.sort_by(f64::total_cmp);
        let mut columns: Vec<f64> = Vec::with_capacity(times.len());
        for t in times {
            if columns.last().is_none_or(|&c| t - c > COLUMN_MERGE_S) {
                columns.push(t);
            }
        }
        // Pass 3 — re-fit every track of this entity AT those columns.
        for &(prop, span) in &props {
            let Some(target) = timeline.doc.binding_for(entity, prop).map(|b| b.target) else {
                continue;
            };
            if let Some(track) = timeline.doc.active_clip_mut().track_mut(target) {
                track.simplify_range_at(
                    span.t_min,
                    span.t_max,
                    &columns,
                    prop.fit_channel(),
                    smooth_passes,
                );
            }
        }
    }
}

/// The fit tolerance for one recorded track: a fraction of ITS value range, with
/// an absolute floor so a near-constant channel is not held to an impossible bar.
/// Measured on the PREPARED samples (see the caller) — the range the fit will
/// actually see, which for an unwrapped spin is every turn, not one.
fn value_tol(samples: &[(f64, f64)]) -> f64 {
    let (mut v_min, mut v_max) = (f64::MAX, f64::MIN);
    for &(_, v) in samples {
        v_min = v_min.min(v);
        v_max = v_max.max(v);
    }
    (REC_SIMPLIFY_REL * (v_max - v_min)).max(REC_SIMPLIFY_FLOOR)
}
