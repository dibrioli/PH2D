//! W1.T1 — the `TimelineDoc` persists through serde (JSON here; the project save
//! format is the shell's choice). Bindings' prop + wire_id survive; the runtime
//! entity bits reset (rebuilt on load, W1.T8); markers + version survive; the
//! target allocator reseats past loaded bindings.

use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, RationalTime};
use ph2d_timeline::{DOC_VERSION, PropKind, TimelineDoc, WireId};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn roundtrip(doc: &TimelineDoc) -> TimelineDoc {
    let json = serde_json::to_string(doc).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn authored() -> TimelineDoc {
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        10,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        10,
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(10.0),
        Interp::Hold,
    );
    doc.insert_key(
        20,
        PropKind::Opacity,
        s(0.0),
        AnimValue::Float(1.0),
        Interp::Linear,
    );
    // A resolved wire id on the first binding (what the save path fills in).
    doc.bindings_mut()[0].wire_id = WireId(42);
    doc.add_marker(s(1.0), "hit");
    doc
}

#[test]
fn doc_roundtrips_bindings_tracks_and_markers() {
    let doc = authored();
    let back = roundtrip(&doc);

    assert_eq!(back.version, DOC_VERSION);
    assert_eq!(back.clips().len(), doc.clips().len());
    assert_eq!(back.bindings().len(), doc.bindings().len());
    assert_eq!(back.markers().len(), 1);
    assert_eq!(back.markers()[0].label, "hit");

    // Bindings: prop + wire_id survive; runtime entity resets to 0 (serde skip).
    for (a, b) in doc.bindings().iter().zip(back.bindings()) {
        assert_eq!(a.prop, b.prop, "prop survives");
        assert_eq!(a.target, b.target, "opaque target survives");
        assert_eq!(a.wire_id, b.wire_id, "wire id survives");
        assert!(!b.missing);
    }
    assert_eq!(back.bindings()[0].entity, 0, "runtime entity not persisted");

    // The active clip samples identically after the trip.
    let tx = doc.bindings()[0].target;
    for &t in &[-1.0, 0.5, 1.0, 2.5] {
        assert_eq!(
            doc.active_clip().track(tx).unwrap().sample(t),
            back.active_clip().track(tx).unwrap().sample(t),
            "sample identical at {t}"
        );
    }
}

#[test]
fn allocator_reseats_past_loaded_bindings() {
    let doc = authored();
    let mut back = roundtrip(&doc);
    back.reseat_allocator();
    // A fresh bind after load must not collide with any loaded target.
    let existing: Vec<_> = back.bindings().iter().map(|b| b.target).collect();
    let fresh = back.bind(999, PropKind::ScaleX);
    assert!(!existing.contains(&fresh), "new target avoids loaded ids");
}

#[test]
fn schema_version_is_one() {
    assert_eq!(DOC_VERSION, 1);
    assert_eq!(TimelineDoc::new().version, 1);
}
