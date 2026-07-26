//! [`ClockIndex`] — every remapped entity's sampling clock, resolved **once per
//! frame** instead of once per binding.
//!
//! A Time Remap track is the entity's clock (the AE precomp model): every OTHER
//! track of that entity samples at `remapped_time(entity, playhead)` rather than
//! at the playhead. The obvious way to honour that — ask for the clock inside
//! the apply's per-binding loop — is what [`crate::apply_from_doc`] used to do,
//! and it is **quadratic**: each of the B bindings scanned all B bindings
//! looking for its entity's remap track, and re-ran the track lookup + curve
//! sample every time. An entity with six animated properties resolved the same
//! clock six times.
//!
//! This resolves it **per entity**: one pass over the bindings collects each
//! remapped entity's source time; the apply then looks the answer up. The
//! expensive part (track lookup + `sample`) now runs once per *remapped entity*,
//! and the lookup is a binary search over a list that is almost always empty.
//!
//! It is the prerequisite of the clip stack (ADR-0115 §4): a strip loop nested
//! inside a quadratic apply would be **cubic**, and the kill-criterion could not
//! even be measured against it.
//!
//! **Zero-alloc after warm-up** (HR-3): the entry buffer lives on the document
//! and is cleared, never freed. It is runtime scratch, not document identity —
//! hence it never serializes and always compares equal.

use ph2d_anim::{AnimTarget, AnimValue, AttributeEvaluator, Clip, Interp};

use crate::binding::TargetBinding;
use crate::doc::TimelineDoc;
use crate::prop::PropKind;

/// Resolved sampling clocks for this frame, keyed by entity bits.
///
/// Empty for every entity without a Time Remap track — which is the common case,
/// and why the lookup costs nothing when nobody uses the feature.
#[derive(Debug, Default, Clone)]
pub(crate) struct ClockIndex {
    /// `(entity bits, source time)`, sorted by entity for the binary search.
    entries: Vec<(u64, f64)>,
}

/// Scratch is **not identity**: two documents that differ only in a frame's
/// resolved clocks are the same document. Without this, `TimelineDoc`'s derived
/// equality would report a change every frame the playhead moves.
impl PartialEq for ClockIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ClockIndex {
    /// Resolve the clock of every entity that has a (live, keyed) Time Remap
    /// track in `clip`. Call once per frame, **after** the liveness pass — a
    /// dead entity's remap track must not drive anything.
    pub(crate) fn rebuild(&mut self, doc: &TimelineDoc, clip: &Clip, t: f64) {
        self.entries.clear();
        for b in doc.bindings() {
            if b.prop != PropKind::TimeRemap || b.missing {
                continue;
            }
            let Some(t_src) = remap_through(clip, b.target, t) else {
                continue; // no track, empty track, or a non-Float value: identity
            };
            // First binding wins, matching the pre-hoist linear scan. (A second
            // Time Remap binding on one entity is not reachable from the panel,
            // but the document allocator does not forbid it.)
            if self.entries.iter().any(|&(e, _)| e == b.entity) {
                continue;
            }
            self.entries.push((b.entity, t_src));
        }
        self.entries.sort_unstable_by_key(|&(e, _)| e);
    }

    /// The time `entity`'s tracks sample at — its resolved source time, or `t`
    /// (the identity) when it has no clock of its own.
    pub(crate) fn get(&self, entity: u64, t: f64) -> f64 {
        match self.entries.binary_search_by_key(&entity, |&(e, _)| e) {
            Ok(i) => self.entries[i].1,
            Err(_) => t,
        }
    }
}

/// One entity's source time through the Time Remap track at `target`, or `None`
/// when that track cannot drive a clock (absent, empty, or not a `Float`).
///
/// **Outside the keyed range the clock extrapolates at slope 1** (the identity,
/// offset through the boundary key) instead of the track's flat hold: a hold
/// would freeze the entity's whole timeline the moment the FIRST key lands, and
/// `K` seeds the identity — so one seeded key must change nothing anywhere,
/// exactly like zero keys. The one exception is a **`Hold` last key**, which
/// freezes from there on: the deliberate AE freeze-frame.
///
/// Clamped non-negative, like every playhead time.
pub(crate) fn remap_through(clip: &Clip, target: AnimTarget, t: f64) -> Option<f64> {
    let as_f64 = |v: AnimValue| match v {
        AnimValue::Float(f) => Some(f64::from(f)),
        _ => None,
    };
    let tr = clip.track(target)?;
    let (first, last) = (tr.keys().first()?, tr.keys().last()?);
    let (t0, t1) = (first.t.to_seconds(), last.t.to_seconds());
    let source = if t < t0 {
        as_f64(first.value).map(|v| v + (t - t0))
    } else if t > t1 {
        as_f64(last.value).map(|v| match last.interp {
            Interp::Hold => v,
            _ => v + (t - t1),
        })
    } else {
        as_f64(tr.sample(t))
    }?;
    Some(source.max(0.0))
}

/// `entity`'s sampling clock in `clip`, resolved by scanning `bindings`.
///
/// [tests] `remapped_time_in` and the [`ClockIndex`] path must agree — they are
/// the same clock read two ways, and a mismatch is precisely the seed/sample
/// divergence that broke Time Remap three times.
///
/// The single-entity path, for callers outside the frame loop (the shell's key
/// authoring: auto-key, the displaced-pose pin, `K`). The apply's whole-document
/// path is [`ClockIndex`] — do not call this in a loop over bindings, that is
/// the quadratic shape this module exists to remove.
pub(crate) fn remapped_time_in(
    clip: &Clip,
    bindings: &[TargetBinding],
    entity: u64,
    t: f64,
) -> f64 {
    for b in bindings {
        if b.entity != entity || b.prop != PropKind::TimeRemap || b.missing {
            continue;
        }
        if let Some(t_src) = remap_through(clip, b.target, t) {
            return t_src;
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_anim::{Interp, RationalTime};

    fn s(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    /// A doc where `entities` objects each animate all six scene props, and the
    /// first `remapped` of them also carry a Time Remap track (identity + a
    /// half-speed ramp, so the clock is not the playhead).
    fn doc_with(entities: u64, remapped: u64) -> TimelineDoc {
        let mut doc = TimelineDoc::new();
        for e in 0..entities {
            for prop in PropKind::ALL {
                doc.insert_key(e, prop, s(0.0), AnimValue::Float(0.0), Interp::Linear);
            }
            if e < remapped {
                // t_src = t/2 : a half-speed clock, distinguishable from identity.
                let tr = PropKind::TimeRemap;
                doc.insert_key(e, tr, s(0.0), AnimValue::Float(0.0), Interp::Linear);
                doc.insert_key(e, tr, s(4.0), AnimValue::Float(2.0), Interp::Linear);
            }
        }
        doc
    }

    /// THE hoist, asserted structurally: an entity's clock is resolved **once**,
    /// however many properties it animates. Before this module, six animated
    /// properties meant six identical resolutions of the same curve — the
    /// quadratic that would have gone cubic under a clip stack (ADR-0115 §4).
    #[test]
    fn one_clock_per_entity_not_one_per_binding() {
        let doc = doc_with(8, 8); // 8 entities x (6 props + 1 remap) = 56 bindings
        assert_eq!(doc.bindings().len(), 56, "the doc really is binding-dense");

        let mut index = ClockIndex::default();
        index.rebuild(&doc, doc.active_clip(), 1.0);

        assert_eq!(
            index.entries.len(),
            8,
            "one entry per remapped ENTITY, not per binding"
        );
    }

    /// An unremapped document costs nothing to look up: the index is empty and
    /// every entity reads back the playhead. This is the common case, and it is
    /// why the hoist is free for animators who never touch Time Remap.
    #[test]
    fn a_document_without_time_remap_has_an_empty_index_and_is_the_identity() {
        let doc = doc_with(4, 0);
        let mut index = ClockIndex::default();
        index.rebuild(&doc, doc.active_clip(), 1.25);

        assert!(index.entries.is_empty(), "nothing to resolve");
        for e in 0..4 {
            assert_eq!(index.get(e, 1.25), 1.25, "entity {e} rides the playhead");
        }
    }

    /// The two readings of the same clock — the whole-document index (what the
    /// apply samples with) and the single-entity scan (what key authoring seeds
    /// with) — must never disagree. A derived coordinate seeded by one transform
    /// and read by another is the bug that broke Time Remap three times
    /// (`feedback_derived_coordinate_seed_must_match_sample`).
    #[test]
    fn the_index_and_the_single_entity_scan_agree_everywhere() {
        let doc = doc_with(6, 3); // half remapped, half not
        let clip = doc.active_clip();

        for step in 0..=40 {
            let t = f64::from(step) * 0.15; // straddles the keys and both extrapolations
            let mut index = ClockIndex::default();
            index.rebuild(&doc, clip, t);
            for e in 0..6 {
                assert_eq!(
                    index.get(e, t),
                    remapped_time_in(clip, doc.bindings(), e, t),
                    "entity {e} at t={t}"
                );
            }
        }
    }

    /// Per-track EXTRAPOLATION is INERT on a Time Remap track (crown-jewels plan
    /// §6): the Time-Remap clock is `remap_through`, which contours `Track::sample`
    /// outside the range with its OWN rule (slope-1 / Hold-freeze), so it never
    /// reaches the extrapolation path. Setting `post = Loop` on the Time-Remap
    /// track must change nothing here — a Loop would fold `t=6` back to `t=2` (a
    /// half-speed value of 1.0), but the clock keeps its slope-1 extension (4.0).
    ///
    /// The mutation this pins: if `remap_through` ever routed out-of-range `t`
    /// through the track's extrapolation, this reads 1.0 and fails.
    #[test]
    fn extrapolation_is_inert_on_a_time_remap_track() {
        use ph2d_anim::Extrap;
        let mut doc = doc_with(1, 1);
        // The Time-Remap binding of entity 0, and its track.
        let target = doc
            .bindings()
            .iter()
            .find(|b| b.entity == 0 && b.prop == PropKind::TimeRemap)
            .expect("time-remap binding")
            .target;
        doc.active_clip_mut()
            .track_mut(target)
            .expect("time-remap track")
            .set_post(Extrap::Loop);

        let clip = doc.active_clip();
        // t = 6 is beyond the last Time-Remap key (t1 = 4). Slope-1: 2.0 + (6-4) = 4.0.
        let got = remap_through(clip, target, 6.0).expect("remap");
        assert!(
            (got - 4.0).abs() < 1e-9,
            "the Time-Remap clock keeps its slope-1 extension (4.0), not a looped 1.0; got {got}"
        );
    }

    /// A dead entity's clock must not survive: the liveness pass runs first, and
    /// a missing binding is skipped. (Otherwise a deleted object could keep
    /// retiming a namesake that reclaimed its bits.)
    #[test]
    fn a_missing_binding_contributes_no_clock() {
        let mut doc = doc_with(2, 2);
        for b in doc.bindings_mut() {
            if b.entity == 0 {
                b.missing = true;
            }
        }
        let mut index = ClockIndex::default();
        index.rebuild(&doc, doc.active_clip(), 2.0);

        assert_eq!(index.entries.len(), 1, "only the live entity has a clock");
        assert_eq!(
            index.get(0, 2.0),
            2.0,
            "the dead one falls back to identity"
        );
        assert_eq!(index.get(1, 2.0), 1.0, "the live one is half-speed");
    }

    /// Zero-alloc after warm-up (HR-3): the buffer is cleared, never freed, so a
    /// steady-state frame reuses it. Asserted as CAPACITY, not as a global
    /// allocation counter — that counter is process-wide and flaky
    /// (`feedback_zero_alloc_gate_capacity_not_global_counter`).
    #[test]
    fn the_clock_buffer_is_reused_across_frames() {
        let doc = doc_with(8, 8);
        let mut index = ClockIndex::default();
        index.rebuild(&doc, doc.active_clip(), 0.0);
        let warm = index.entries.capacity();
        assert!(warm >= 8);

        for step in 1..100 {
            index.rebuild(&doc, doc.active_clip(), f64::from(step) * 0.05);
            assert_eq!(
                index.entries.capacity(),
                warm,
                "frame {step} reallocated the clock buffer"
            );
        }
    }
}
