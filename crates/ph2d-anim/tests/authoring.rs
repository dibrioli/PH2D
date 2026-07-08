//! W0.T1 + W0.T5 + W0.T2 — authoring ops on `Track` (stable `KeyId`, single +
//! bulk edits, cursor safety) and the `Interp` bézier-handle helpers.

use ph2d_anim::{
    AnimValue, AttributeEvaluator, Easing, EasingFamily, EasingMode, Interp, RationalTime, Track,
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
    tr.move_keys(&[a, b, c], 0.5);
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
    let dups = tr.duplicate_keys(&[a, b], 0.1);
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
