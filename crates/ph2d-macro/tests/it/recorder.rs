use ph2d_macro::{
    Door, MacroRecorder, MacroTime, RecordError, Recordability, RefRect, TrackId, orphans,
    undecided,
};

#[test]
fn time_is_relative_to_the_arming_instant() {
    let mut r = MacroRecorder::new(RefRect { w: 8.0, h: 8.0 }, 5_000_000);
    let p = r.open_track("painter", 42, vec![1]);
    r.push(p, 5_000_000, vec![]).unwrap();
    r.push(p, 5_250_000, vec![7]).unwrap();
    let doc = r.finish();
    let ts: Vec<MacroTime> = doc.tracks[0].events.iter().map(|e| e.t).collect();
    assert_eq!(ts, [MacroTime(0), MacroTime(250_000)]);
    assert_eq!(doc.tracks[0].target, 42);
}

#[test]
fn time_going_backwards_is_refused_loudly() {
    let mut r = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 1_000);
    let t = r.open_track("motion", 0, vec![]);
    assert_eq!(
        r.push(t, 999, vec![]),
        Err(RecordError::TimeWentBackwards { track: 0 })
    );
    r.push(t, 2_000, vec![]).unwrap();
    assert_eq!(
        r.push(t, 1_500, vec![]),
        Err(RecordError::TimeWentBackwards { track: 0 })
    );
    assert_eq!(r.doc().tracks[0].events.len(), 1);
}

#[test]
fn two_tracks_keep_their_own_order() {
    let mut r = MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0);
    let a = r.open_track("painter", 1, vec![]);
    let b = r.open_track("motion", 2, vec![]);
    r.push(a, 10, vec![]).unwrap();
    r.push(b, 5, vec![]).unwrap();
    assert!(r.doc().is_time_ordered());
    assert_eq!(
        r.push(TrackId(9), 20, vec![]),
        Err(RecordError::UnknownTrack { track: 9 })
    );
}

#[test]
fn the_census_names_undecided_doors_and_orphan_declarations() {
    const DOORS: &[Door] = &[
        Door {
            name: "on_canvas_pointer",
            recordability: Recordability::Recorded,
        },
        Door {
            name: "foi_apagada",
            recordability: Recordability::Refused { why: "teste" },
        },
    ];
    let present = ["on_canvas_pointer", "porta_nova"];
    assert_eq!(undecided(DOORS, &present), ["porta_nova"]);
    assert_eq!(orphans(DOORS, &present), ["foi_apagada"]);
}
