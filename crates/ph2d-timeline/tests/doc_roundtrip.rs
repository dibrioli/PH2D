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
fn schema_version_is_six() {
    // v2 = per-key roving flags appended to the track payload.
    // v3 = each clip carries its OWN loop (`NamedClip.loop_range` +
    //      `loop_ping_pong`, appended) — a loop belongs to the animation it
    //      brackets, not to the document (Enio, 2026-07-12).
    // v4 = the clip STACK (`TimelineDoc.stack`) + each binding's captured `rest`
    //      (ADR-0115). Both appended; an empty stack behaves exactly as v3 did.
    // v5 = a SECOND loop per clip (`keys_loop_range` + `keys_loop_ping_pong`,
    //      appended) — the Keys view loops the clip's clock independently of the
    //      Arrange timeline loop (Enio, 2026-07-16).
    // v6 = a strip's fade-in can reach OUTWARD into the gap before it
    //      (`ClipStrip.lead_in`, appended) — the travel fade (Enio, 2026-07-16).
    // Postcard is positional: an older blob must be REJECTED by the version gate,
    // not misread field-for-field.
    assert_eq!(DOC_VERSION, 6);
    assert_eq!(TimelineDoc::new().version, 6);
}

/// **Both loops survive the trip, and independently.** A clip whose Arrange loop and
/// Keys loop differ must come back with each intact — the two are separate fields,
/// not one read twice, and postcard is positional so a layout slip would swap them.
#[test]
fn each_clips_two_loops_round_trip_independently() {
    let mut doc = TimelineDoc::new();
    doc.set_active_loop_for(false, Some((0.0, 2.0))); // Arrange (timeline) loop
    doc.set_active_ping_pong_for(false, false);
    doc.set_active_loop_for(true, Some((1.5, 4.0))); // Keys (clip-clock) loop
    doc.set_active_ping_pong_for(true, true); // and it ping-pongs; the other does not
    let back = roundtrip(&doc);
    assert_eq!(
        back.active_loop_for(false),
        Some((0.0, 2.0)),
        "Arrange loop"
    );
    assert!(!back.active_ping_pong_for(false), "Arrange wraps");
    assert_eq!(back.active_loop_for(true), Some((1.5, 4.0)), "Keys loop");
    assert!(back.active_ping_pong_for(true), "Keys ping-pongs");
}

/// **A strip's outward lead-in survives the trip.** Appended field (v6), positional
/// format — a layout slip would read it from the wrong bytes.
#[test]
fn a_strips_outward_lead_survives_the_trip() {
    let mut doc = TimelineDoc::new();
    let lane = doc.add_lane("L".into()).unwrap();
    let id = doc.add_strip(lane, 0, 3.0, 6.0).unwrap();
    doc.strip_mut(lane, id).unwrap().lead_in = 1.25;
    let back = roundtrip(&doc);
    assert!(
        (back.strip(lane, id).unwrap().lead_in - 1.25).abs() < 1e-12,
        "the outward lead-in survives serialization"
    );
}

#[test]
fn to_bytes_from_bytes_round_trips_the_document() {
    use ph2d_timeline::{PropKind, TimelineDoc};
    let mut doc = TimelineDoc::new();
    let target = doc.bind(1, PropKind::TranslationX);
    doc.bindings_mut()[0].wire_id = ph2d_timeline::WireId(42);
    doc.add_marker(ph2d_anim::RationalTime::from_seconds(1.0), "beat");
    let _ = target;
    let bytes = doc.to_bytes().expect("serialize");
    let back = TimelineDoc::from_bytes(&bytes).expect("deserialize");
    assert_eq!(back.bindings()[0].wire_id, ph2d_timeline::WireId(42));
    assert_eq!(back.markers()[0].label, "beat");
}

#[test]
fn from_bytes_rejects_a_future_schema_version() {
    // A postcard blob whose first field (version) is a value this build doesn't
    // know must be refused, not silently misread.
    use ph2d_timeline::TimelineDoc;
    let good = TimelineDoc::new().to_bytes().unwrap();
    // The version is the first varint; bump it well past DOC_VERSION.
    let mut bad = good.clone();
    bad[0] = 0x7f; // a large single-byte varint, != DOC_VERSION (1)
    assert!(
        TimelineDoc::from_bytes(&bad).is_err(),
        "wrong-version file rejected"
    );
}
