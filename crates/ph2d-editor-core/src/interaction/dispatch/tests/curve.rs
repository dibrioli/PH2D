use super::*;

/// Regression (W4 §3): a [`InteractiveState::CurvePoint`] must DRAG across a
/// Down→Move sequence, not just jump on Down. The Move-drag block in
/// `dispatch_pointer` is gated on `active_rect.is_some()`, so the CurvePoint
/// Down arm has to set `active_rect` as well as `active` — without it the Move
/// is skipped and `take_curve_point_drag` stays `None` after a real drag (the
/// "points won't drag, only jump a little on click" bug Enio hit). The
/// `dispatch/curve.rs` unit test exercises `apply_curve_point_drag` directly but
/// NOT this integrated path, which is why the gap slipped through.
#[test]
fn curve_point_down_then_move_drags() {
    let mut store = WidgetStore::with_capacity(2);
    let editor = NodeId(40);
    let handle = NodeId(41);
    let canvas = Rect::new(10.0, 20.0, 200.0, 100.0);
    store.register(
        handle,
        InteractiveState::CurvePoint {
            parent: editor,
            channel: 1,
            index: 0,
            canvas,
        },
    );
    let mut hits = HitIndex::new();
    // Small grab rect for the handle (the Down hit-test target).
    hits.register(handle, Rect::new(105.0, 65.0, 12.0, 12.0));
    let arena = Bump::new();

    // Down on the handle → active + active_rect + the initial click-to-move.
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 111.0, 71.0),
        &arena,
    );
    assert_eq!(
        store.active_id(),
        Some(handle),
        "Down makes the handle active"
    );
    assert!(
        store.active_rect().is_some(),
        "Down must set active_rect — the Move-drag gate depends on it"
    );
    let _ = store.take_curve_point_drag_if(|p| p == editor); // drain the Down's stash

    // Move to canvas mid-x / top quarter → the drag MUST fire (x=0.5, y=0.75).
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Move, 110.0, 45.0),
        &arena,
    );
    let drag = store.take_curve_point_drag_if(|p| p == editor);
    assert!(
        drag.is_some(),
        "Move must drag the active CurvePoint (regression: was None without active_rect)"
    );
    let (parent, channel, index, x, y) = drag.unwrap();
    assert_eq!((parent, channel, index), (editor, 1, 0));
    assert!(
        (x - 0.5).abs() < 1e-4 && (y - 0.75).abs() < 1e-4,
        "normalized (x, y) = ({x}, {y}), expected (0.5, 0.75)"
    );
}
