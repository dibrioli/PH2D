//! W0.T1 + W0.T5 + W0.T2 — authoring ops on `Track` (stable `KeyId`, single +
//! bulk edits, cursor safety) and the `Interp` bézier-handle helpers.

use ph2d_anim::{
    AnimValue, AttributeEvaluator, Easing, EasingFamily, EasingMode, FitChannel, Interp,
    RationalTime, Track,
};

fn as_f(v: AnimValue) -> f32 {
    match v {
        AnimValue::Float(x) => x,
        other => panic!("expected Float, got {other:?}"),
    }
}

fn secs(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

// ── W0.T1: insert / lookup / sorted invariant ────────────────────────────────
#[test]
fn insert_assigns_unique_ids_and_keeps_sorted() {
    let mut tr = Track::new(vec![]);
    // Insert out of order; the track must end up sorted by t.
    let b = tr.insert_key(secs(2.0), AnimValue::Float(20.0), Interp::Hold);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let m = tr.insert_key(secs(1.0), AnimValue::Float(10.0), Interp::Linear);
    assert_ne!(a, b);
    assert_ne!(b, m);
    assert_eq!(tr.len(), 3);
    // Sorted: keys[0].t < keys[1].t < keys[2].t
    let ts: Vec<f64> = tr.keys().iter().map(|k| k.t.to_seconds()).collect();
    assert!(ts[0] < ts[1] && ts[1] < ts[2]);
    // Lookups resolve each id to the right key regardless of index.
    assert!((tr.key(a).unwrap().t.to_seconds() - 0.0).abs() < 1e-9);
    assert!((tr.key(m).unwrap().t.to_seconds() - 1.0).abs() < 1e-9);
    assert!((tr.key(b).unwrap().t.to_seconds() - 2.0).abs() < 1e-9);
    // Sample the linear 0→10 segment at 0.5 s.
    assert!((as_f(tr.sample(0.5)) - 5.0).abs() < 1e-5);
}

// ── W0.T1: move keeps the id, re-sorts, and the cursor stays correct ─────────
#[test]
fn move_key_preserves_id_and_resamples() {
    let mut tr = Track::new(vec![]);
    let first = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let _last = tr.insert_key(secs(2.0), AnimValue::Float(10.0), Interp::Hold);
    // Warm the cursor by sampling mid-segment.
    let _ = tr.sample(1.0);
    // Move the first key past the last → it becomes the last, but its id stands.
    assert!(tr.move_key(first, secs(3.0)));
    assert!((tr.key(first).unwrap().t.to_seconds() - 3.0).abs() < 1e-9);
    // keys[0] is now the old "last" (at 2 s); sampling before it clamps to its value.
    assert!((as_f(tr.sample(1.0)) - 10.0).abs() < 1e-5);
    // A dangling id (from THIS track — ids are per-track) is a no-op.
    let ghost = tr.insert_key(secs(9.0), AnimValue::Float(0.0), Interp::Hold);
    assert!(tr.remove_key(ghost));
    assert!(!tr.move_key(ghost, secs(0.0)));
}

// ── W0.T1: set_value / set_interp / remove ───────────────────────────────────
#[test]
fn set_value_interp_and_remove() {
    let mut tr = Track::new(vec![]);
    let k0 = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let _k1 = tr.insert_key(secs(1.0), AnimValue::Float(10.0), Interp::Hold);
    assert!(tr.set_value(k0, AnimValue::Float(4.0)));
    assert!(tr.set_interp(k0, Interp::Hold));
    // Hold from k0 → holds 4.0 across the segment.
    assert!((as_f(tr.sample(0.5)) - 4.0).abs() < 1e-5);
    assert!(tr.remove_key(k0));
    assert_eq!(tr.len(), 1);
    assert!(!tr.remove_key(k0)); // already gone
}

// ── W0.T5: bulk shift / scale / remove / duplicate ───────────────────────────
#[test]
fn bulk_move_and_scale() {
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(1.0), Interp::Linear);
    let c = tr.insert_key(secs(2.0), AnimValue::Float(2.0), Interp::Hold);
    tr.move_keys(&[a, b, c], secs(0.5));
    for (id, want) in [(a, 0.5), (b, 1.5), (c, 2.5)] {
        assert!((tr.key(id).unwrap().t.to_seconds() - want).abs() < 1e-4);
    }
    tr.scale_keys(&[a, b, c], 0.5, 2.0); // pivot 0.5, factor 2: 0.5→0.5, 1.5→2.5, 2.5→4.5
    assert!((tr.key(a).unwrap().t.to_seconds() - 0.5).abs() < 1e-4);
    assert!((tr.key(b).unwrap().t.to_seconds() - 2.5).abs() < 1e-4);
    assert!((tr.key(c).unwrap().t.to_seconds() - 4.5).abs() < 1e-4);
}

#[test]
fn bulk_remove_and_duplicate() {
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(1.0), Interp::Linear);
    let dups = tr.duplicate_keys(&[a, b], secs(0.1));
    assert_eq!(dups.len(), 2);
    assert_eq!(tr.len(), 4);
    for d in &dups {
        assert_ne!(*d, a);
        assert_ne!(*d, b);
    }
    // Duplicates sit at 0.1 and 1.1.
    assert!((tr.key(dups[0]).unwrap().t.to_seconds() - 0.1).abs() < 1e-4);
    tr.remove_keys(&[a, b]);
    assert_eq!(tr.len(), 2); // only the duplicates remain
    assert!(tr.key(a).is_none());
}

#[test]
fn a_duplicate_landing_on_an_existing_key_overwrites_it() {
    // A track may not hold two keys at one instant: the copy replaces the key it
    // lands on and inherits its id, so `len` grows by one, not two.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(7.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(9.0), Interp::Hold);
    let dups = tr.duplicate_keys(&[a, b], secs(1.0));
    assert_eq!(tr.len(), 3, "0.0, 1.0 (overwritten), 2.0");
    assert_eq!(dups[0], b, "the copy took over the key it replaced");
    assert_eq!(tr.key(b).unwrap().value, AnimValue::Float(7.0));
    assert_eq!(tr.key(b).unwrap().interp, Interp::Linear);
    // The source is read before anything is written, so the second copy still
    // carries the ORIGINAL value of `b`, not the one that just overwrote it.
    assert_eq!(tr.key(dups[1]).unwrap().value, AnimValue::Float(9.0));
    assert_eq!(
        tr.key(a).unwrap().value,
        AnimValue::Float(7.0),
        "source kept"
    );
}

#[test]
fn a_frame_exact_bulk_move_stays_frame_exact() {
    // `to_seconds` round-trips would land 2/24 s at 83333 us; the rational path
    // must not, or a later equality test (upsert, duplicate-overwrite) misses.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(
        RationalTime::from_frame(0, 24),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    tr.move_keys(&[a], RationalTime::from_frame(2, 24));
    assert_eq!(tr.key(a).unwrap().t, RationalTime::from_frame(2, 24));
}

// ── W0.T2: Interp bézier handles ─────────────────────────────────────────────
#[test]
fn handles_of_each_interp() {
    assert_eq!(Interp::Linear.handles(), Some(Interp::LINEAR_HANDLES));
    assert_eq!(Interp::Hold.handles(), None);
    assert_eq!(
        Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)).handles(),
        None
    );
    let bez = Interp::bezier(0.42, 0.0, 0.58, 1.0);
    assert_eq!(bez.handles(), Some(((0.42, 0.0), (0.58, 1.0))));
}

#[test]
fn dragging_a_handle_upgrades_to_bezier() {
    // Dragging the out-handle of a Linear segment → Bezier, keeping the in-handle.
    let up = Interp::Linear.with_out_handle(0.1, 0.9);
    let ((x1, y1), (x2, y2)) = up.handles().unwrap();
    assert!((x1 - 0.1).abs() < 1e-9 && (y1 - 0.9).abs() < 1e-9);
    // In-handle preserved from the linear default (2/3, 1/3... i.e. (2/3, 2/3)).
    assert!((x2 - 2.0 / 3.0).abs() < 1e-9 && (y2 - 2.0 / 3.0).abs() < 1e-9);
    // x is clamped into [0,1] on the out-handle.
    let clamped = Interp::Linear.with_out_handle(1.5, 2.0);
    assert!((clamped.handles().unwrap().0.0 - 1.0).abs() < 1e-9);
    // Round-trip: handles → interp → handles is stable for a real bezier.
    let bez = Interp::bezier(0.3, 0.2, 0.7, 0.8);
    let ((a, b), (c, d)) = bez.handles().unwrap();
    assert_eq!(Interp::bezier(a, b, c, d), bez);
}

#[test]
fn upsert_value_keeps_the_easing_the_author_drew() {
    // The auto-key bug: nudging the object on canvas re-keyed the playhead's key
    // through `upsert_key`, which replaced BOTH value and interp — silently
    // reverting every bezier handle the author had dragged in the graph editor.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(1.0), Interp::Linear);
    let custom = Interp::bezier(0.9, 1.4, 0.1, -0.4);
    tr.set_interp(a, custom);

    let id = tr.upsert_value(secs(0.0), AnimValue::Float(7.0), Interp::Hold);
    assert_eq!(id, a, "the existing key was updated, not stacked");
    assert_eq!(tr.len(), 1);
    assert_eq!(tr.key(a).unwrap().value, AnimValue::Float(7.0), "new pose");
    assert_eq!(tr.key(a).unwrap().interp, custom, "the ease survived");

    // On a fresh instant the `interp` argument IS the new key's interpolation.
    let b = tr.upsert_value(secs(1.0), AnimValue::Float(2.0), Interp::Hold);
    assert_eq!(tr.key(b).unwrap().interp, Interp::Hold);
}

#[test]
fn upsert_key_still_replaces_everything() {
    // Paste + duplicate carry a whole key; they must overwrite the interp too.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(
        secs(0.0),
        AnimValue::Float(1.0),
        Interp::bezier(0.9, 0.1, 0.1, 0.9),
    );
    tr.upsert_key(secs(0.0), AnimValue::Float(7.0), Interp::Hold);
    assert_eq!(tr.key(a).unwrap().interp, Interp::Hold);
}

// ── W3: handles the graph editor DRAWS (tangents, not the chord) ─────────────

/// The `(x, y)` slope of the cubic timing curve `P0=(0,0) P1 P2 P3=(1,1)` at its
/// endpoints. Derived from the bezier derivative, independent of the fn under
/// test — this is what "the handle is tangent to the curve" means.
fn bezier_endpoint_slopes(((x1, y1), (x2, y2)): ((f64, f64), (f64, f64))) -> (f64, f64) {
    (y1 / x1, (1.0 - y2) / (1.0 - x2))
}

#[test]
fn linear_tangent_handles_are_the_linear_handles() {
    // The general slope path and the special case must not disagree.
    assert_eq!(Interp::Linear.tangent_handles(), Interp::LINEAR_HANDLES);
    let (m0, m1) = bezier_endpoint_slopes(Interp::LINEAR_HANDLES);
    assert!((m0 - 1.0).abs() < 1e-9 && (m1 - 1.0).abs() < 1e-9);
}

#[test]
fn an_eased_segment_shows_handles_tangent_to_its_own_curve() {
    // Cubic InOut leaves and arrives flat. The old code drew the LINEAR handles,
    // which sit on the straight chord — visibly off the curve.
    let e = Easing::new(EasingFamily::Cubic, EasingMode::InOut);
    let h = Interp::Eased(e).tangent_handles();
    let (m0, m1) = bezier_endpoint_slopes(h);
    assert!(m0.abs() < 1e-3, "flat start, got slope {m0}");
    assert!(m1.abs() < 1e-3, "flat end, got slope {m1}");
    assert_ne!(h, Interp::LINEAR_HANDLES, "the chord is not the curve");

    // Cubic Out arrives flat but leaves at slope 3.
    let h = Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::Out)).tangent_handles();
    let (m0, m1) = bezier_endpoint_slopes(h);
    assert!((m0 - 3.0).abs() < 1e-2, "got {m0}");
    assert!(m1.abs() < 1e-2, "got {m1}");
}

#[test]
fn hold_puts_both_handles_on_the_flat_part_it_actually_draws() {
    let ((x1, y1), (x2, y2)) = Interp::Hold.tangent_handles();
    assert_eq!(
        (y1, y2),
        (0.0, 0.0),
        "the curve is flat; so are the handles"
    );
    assert!(x1 > 0.0 && x2 > x1 && x2 < 1.0);
}

#[test]
fn a_violent_easing_cannot_fling_its_handle_to_infinity() {
    // Expo Out leaves its anchor almost vertically. The tangent is clamped, so
    // the handle stays in a range the graph band can fit.
    let h = Interp::Eased(Easing::new(EasingFamily::Expo, EasingMode::Out)).tangent_handles();
    assert!(h.0.1.is_finite() && h.0.1 > 1.0, "steep, {:?}", h.0);
    assert!(h.0.1 < 6.0, "but bounded, {:?}", h.0);
}

#[test]
fn converting_an_eased_segment_keeps_the_untouched_end_where_it_was() {
    let eased = Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut));
    let (_, in_before) = eased.tangent_handles();
    // Drag the OUT handle; the IN handle must survive as the tangent, not snap
    // back to the linear default (which is what made the curve jump on grab).
    let out_dragged = eased.with_out_handle(0.9, 0.2);
    let (_, in_after) = out_dragged.tangent_handles();
    assert_eq!(in_after, in_before);

    let (out_before, _) = eased.tangent_handles();
    let (out_after, _) = eased.with_in_handle(0.1, 0.8).tangent_handles();
    assert_eq!(out_after, out_before);
}

#[test]
fn a_bezier_reports_its_own_control_points_as_its_tangent_handles() {
    let b = Interp::bezier(0.17, 0.67, 0.83, 0.33);
    assert_eq!(b.tangent_handles(), ((0.17, 0.67), (0.83, 0.33)));
}

#[test]
fn to_bezier_freezes_exactly_the_handles_the_editor_draws() {
    // "Custom" must move nothing on screen: the bezier it produces is the one
    // whose control points ARE the tangent handles already painted. If these two
    // ever disagree, picking Custom would visibly kick the curve.
    for i in [
        Interp::Linear,
        Interp::Hold,
        Interp::Eased(Easing::new(EasingFamily::Cubic, EasingMode::InOut)),
        Interp::Eased(Easing::new(EasingFamily::Bounce, EasingMode::Out)),
        Interp::Eased(Easing::new(EasingFamily::Expo, EasingMode::In)),
    ] {
        let b = i.to_bezier();
        assert!(matches!(b, Interp::Bezier { .. }), "{i:?} stayed {b:?}");
        assert_eq!(
            b.handles(),
            Some(i.tangent_handles()),
            "{i:?} converted to a bezier the editor was not drawing"
        );
    }
}

#[test]
fn to_bezier_leaves_a_bezier_exactly_as_it_was() {
    // Idempotent, and bit-exact: re-picking Custom on an authored curve must not
    // round-trip it through a tangent estimate.
    let b = Interp::bezier(0.17, 0.67, 0.83, 0.33);
    assert_eq!(b.to_bezier(), b);
    assert_eq!(b.to_bezier().to_bezier(), b);
}

#[test]
fn to_bezier_keeps_the_endpoints_and_the_ends_slopes() {
    // The shape between the anchors may change (no cubic is a Bounce); the
    // endpoints, and the direction the curve leaves and arrives, must not.
    let sample = |i: Interp, t: f64| {
        let mut tr = Track::new(vec![]);
        tr.insert_key(secs(0.0), AnimValue::Float(0.0), i);
        tr.insert_key(secs(1.0), AnimValue::Float(1.0), Interp::Linear);
        f64::from(as_f(tr.sample(t)))
    };
    let e = Interp::Eased(Easing::new(EasingFamily::Quint, EasingMode::InOut));
    let b = e.to_bezier();
    for t in [0.0, 1.0] {
        assert!(
            (sample(e, t) - sample(b, t)).abs() < 1e-6,
            "endpoint {t} moved"
        );
    }
    // Quint InOut leaves flat and arrives flat; so must its frozen bezier.
    for t in [1e-3, 1.0 - 1e-3] {
        let (a, c) = (sample(e, t), sample(b, t));
        assert!((a - c).abs() < 1e-2, "slope diverged at {t}: {a} vs {c}");
    }
    // And the middle genuinely differs from a Bounce it cannot represent.
    let bounce = Interp::Eased(Easing::new(EasingFamily::Bounce, EasingMode::Out));
    assert!(
        (sample(bounce, 0.35) - sample(bounce.to_bezier(), 0.35)).abs() > 1e-3,
        "a cubic cannot be a bounce; the doc comment must stay honest"
    );
}

// ── move_keys merges overlapping keys (dope-sheet column drag) ────────────────

#[test]
fn moving_a_key_onto_another_merges_them_and_the_moved_one_wins() {
    // Two keys at t = 0 (value 0) and t = 1 (value 10). Drag the first onto the
    // second: they become ONE key at t = 1 carrying the MOVED key's value, not
    // two stacked at one instant.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Hold);
    let _b = tr.insert_key(secs(1.0), AnimValue::Float(10.0), Interp::Linear);
    tr.move_keys(&[a], secs(1.0)); // a: 0 -> 1, onto b
    assert_eq!(tr.len(), 1, "the two keys merged into one");
    let ts: Vec<f64> = tr.keys().iter().map(|k| k.t.to_seconds()).collect();
    assert_eq!(ts, vec![1.0]);
    assert_eq!(as_f(tr.sample(1.0)), 0.0, "the MOVED key's value survived");
    assert_eq!(
        tr.key(a).map(|k| k.interp),
        Some(Interp::Hold),
        "and its interp"
    );
}

#[test]
fn a_move_that_lands_clear_of_every_other_key_merges_nothing() {
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(10.0), Interp::Linear);
    tr.move_keys(&[a], secs(0.25)); // a -> 0.25, still short of b at 1.0
    assert_eq!(tr.len(), 2);
    assert!((tr.key(a).unwrap().t.to_seconds() - 0.25).abs() < 1e-9);
    assert!((tr.key(b).unwrap().t.to_seconds() - 1.0).abs() < 1e-9);
}

#[test]
fn a_rigid_group_move_never_merges_the_group_with_itself() {
    // Three keys moved by one delta keep their spacing — none collide with each
    // other, however far they travel.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(10.0), Interp::Linear);
    let c = tr.insert_key(secs(2.0), AnimValue::Float(20.0), Interp::Linear);
    tr.move_keys(&[a, b, c], secs(5.0));
    assert_eq!(tr.len(), 3, "the moved group stays three keys");
    let ts: Vec<f64> = tr.keys().iter().map(|k| k.t.to_seconds()).collect();
    assert_eq!(ts, vec![5.0, 6.0, 7.0]);
}

#[test]
fn a_group_dragged_onto_another_group_merges_pairwise() {
    // Columns {0,1} dragged right by 2 land on {2,3}: 0->2 and 1->3 each absorb
    // the stationary key there. Four keys become two, moved values winning.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(secs(0.0), AnimValue::Float(100.0), Interp::Linear);
    let b = tr.insert_key(secs(1.0), AnimValue::Float(200.0), Interp::Linear);
    tr.insert_key(secs(2.0), AnimValue::Float(0.0), Interp::Linear);
    tr.insert_key(secs(3.0), AnimValue::Float(0.0), Interp::Linear);
    tr.move_keys(&[a, b], secs(2.0));
    assert_eq!(tr.len(), 2);
    assert_eq!(
        as_f(tr.sample(2.0)),
        100.0,
        "moved a overwrote the key at 2"
    );
    assert_eq!(
        as_f(tr.sample(3.0)),
        200.0,
        "moved b overwrote the key at 3"
    );
}

#[test]
fn a_frame_exact_move_merges_where_to_seconds_would_have_missed() {
    // The merge uses exact RationalTime equality, so a frame-snapped drag lands
    // ON the target frame — the microsecond drift `to_seconds` leaves would have
    // left two keys a hair apart, unmerged.
    let mut tr = Track::new(vec![]);
    let a = tr.insert_key(
        RationalTime::from_frame(0, 24),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    let _b = tr.insert_key(
        RationalTime::from_frame(2, 24),
        AnimValue::Float(2.0),
        Interp::Linear,
    );
    tr.move_keys(&[a], RationalTime::from_frame(2, 24)); // 0 -> frame 2, exactly
    assert_eq!(tr.len(), 1, "exact frame equality merged them");
}

// ── W5: simplify_range (record cleanup — Schneider fit through Track) ─────────

/// Sample the track's value at time `t` (via the public evaluator).
fn tr_at(tr: &Track, t: f64) -> f32 {
    as_f(tr.sample(t))
}

#[test]
fn simplify_range_replaces_dense_keys_with_a_precise_minimal_fit() {
    // A dense recording: 200 keys, one per frame, tracing a smooth sine bump.
    let mut tr = Track::new(vec![]);
    let n = 200;
    for i in 0..n {
        let t = 4.0 * i as f64 / (n - 1) as f64;
        let v = 50.0 * (t * std::f64::consts::PI / 4.0).sin();
        tr.insert_key(secs(t), AnimValue::Float(v as f32), Interp::Linear);
    }
    assert_eq!(tr.len(), n);
    let changed = tr.simplify_range(0.0, 4.0, 0.25, FitChannel::LINEAR, 0);
    assert!(changed, "the dense run simplified");
    assert!(
        tr.len() < n / 10,
        "dramatic reduction: {} keys from {n}",
        tr.len()
    );
    // Fidelity: the simplified curve tracks the original within a few percent of
    // the range (approximate by design — a key per turn, not per frame).
    for i in 0..n {
        let t = 4.0 * i as f64 / (n - 1) as f64;
        let want = 50.0 * (t * std::f64::consts::PI / 4.0).sin();
        assert!(
            (f64::from(tr_at(&tr, t)) - want).abs() <= 1.5, // 1.5% of the 100 range
            "fidelity at t={t}"
        );
    }
}

#[test]
fn simplify_range_leaves_keys_outside_the_range_untouched() {
    let mut tr = Track::new(vec![]);
    // A key well before the range, then a dense run inside [1, 2].
    let before = tr.insert_key(secs(0.0), AnimValue::Float(9.0), Interp::Hold);
    for i in 0..60 {
        let t = 1.0 + i as f64 / 59.0;
        let v = (t * 6.0).sin();
        tr.insert_key(secs(t), AnimValue::Float(v as f32), Interp::Linear);
    }
    tr.simplify_range(1.0, 2.0, 0.05, FitChannel::LINEAR, 0);
    // The outside key survives with its value and its Hold interp.
    assert_eq!(tr.key(before).map(|k| k.interp), Some(Interp::Hold));
    assert!((tr_at(&tr, 0.0) - 9.0).abs() < 1e-6, "pre-range value held");
}

#[test]
fn simplify_range_is_a_noop_below_three_keys() {
    let mut tr = Track::new(vec![]);
    tr.insert_key(secs(0.0), AnimValue::Float(0.0), Interp::Linear);
    tr.insert_key(secs(1.0), AnimValue::Float(1.0), Interp::Linear);
    assert!(
        !tr.simplify_range(0.0, 1.0, 0.1, FitChannel::LINEAR, 0),
        "two keys: nothing to fit"
    );
    assert_eq!(tr.len(), 2);
}
