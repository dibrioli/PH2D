//! Unit tests for the whole-curve transform gizmo — hit-test priority + the affine transforms.

use super::*;

/// A unit square's 4 anchors with collapsed handles, centred on (50,50).
fn square() -> (Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>) {
    let pts = vec![[40.0, 40.0], [60.0, 40.0], [60.0, 60.0], [40.0, 60.0]];
    let handles = pts.iter().map(|&p| [p, p]).collect();
    (pts, handles)
}

#[test]
fn centre_handle_grabs_a_move_and_translates_the_whole_curve() {
    let (pts, h) = square();
    let g = grab(&pts, &h, [50.0, 50.0], 6.0).expect("centre handle hit");
    assert!(matches!(g.drag, GizmoDrag::Move));
    let (op, _) = apply(&g, [70.0, 55.0]); // dragged +20,+5
    assert_eq!(op[0], [60.0, 45.0]);
    assert_eq!(op[2], [80.0, 65.0]);
}

#[test]
fn corner_handle_scales_about_the_centre() {
    let (pts, h) = square();
    let tol = 6.0;
    // BR corner of the INFLATED box (margin = tol*GIZMO_MARGIN).
    let m = tol * GIZMO_MARGIN;
    let corner = [60.0 + m, 60.0 + m];
    let g = grab(&pts, &h, corner, tol).expect("corner hit");
    assert!(matches!(
        g.drag,
        GizmoDrag::Scale {
            axes: [true, true],
            ..
        }
    ));
    // Drag the corner to twice its distance from the centre → 2× scale about (50,50).
    let pivot = [50.0, 50.0];
    let target = [
        pivot[0] + (corner[0] - pivot[0]) * 2.0,
        pivot[1] + (corner[1] - pivot[1]) * 2.0,
    ];
    let (op, _) = apply(&g, target);
    // Every anchor doubled its offset from the centre: (40,40) → (30,30), (60,60) → (70,70).
    assert!(
        (op[0][0] - 30.0).abs() < 1e-3 && (op[0][1] - 30.0).abs() < 1e-3,
        "{:?}",
        op[0]
    );
    assert!(
        (op[2][0] - 70.0).abs() < 1e-3 && (op[2][1] - 70.0).abs() < 1e-3,
        "{:?}",
        op[2]
    );
}

#[test]
fn edge_mid_handle_scales_one_axis_only() {
    let (pts, h) = square();
    let tol = 6.0;
    let m = tol * GIZMO_MARGIN;
    let right_mid = [60.0 + m, 50.0]; // right edge mid of the inflated box
    let g = grab(&pts, &h, right_mid, tol).expect("edge mid hit");
    assert!(matches!(
        g.drag,
        GizmoDrag::Scale {
            axes: [true, false],
            ..
        }
    ));
    let pivot = [50.0, 50.0];
    let target = [pivot[0] + (right_mid[0] - pivot[0]) * 2.0, 50.0]; // 2× on x only
    let (op, _) = apply(&g, target);
    assert!((op[0][0] - 30.0).abs() < 1e-3, "x scaled: {:?}", op[0]);
    assert!((op[0][1] - 40.0).abs() < 1e-3, "y untouched: {:?}", op[0]);
}

#[test]
fn rotate_ring_just_outside_a_corner_spins_about_the_centre() {
    let (pts, h) = square();
    let tol = 6.0;
    let m = tol * GIZMO_MARGIN;
    let corner = [60.0 + m, 60.0 + m];
    let pivot = [50.0, 50.0];
    // A point on the ring: along the corner direction, clear of the corner's scale tol but within the band.
    let dir = [corner[0] - pivot[0], corner[1] - pivot[1]];
    let ring = [corner[0] + dir[0] * 0.35, corner[1] + dir[1] * 0.35];
    let g = grab(&pts, &h, ring, tol).expect("rotate ring hit");
    assert!(is_rotate(&g));
    // Rotate the grab vector by +90°: from (dx,dy) to (-dy,dx) about the pivot.
    let rot90 = [
        pivot[0] - (ring[1] - pivot[1]),
        pivot[1] + (ring[0] - pivot[0]),
    ];
    let (op, _) = apply(&g, rot90);
    // (40,40) is at (-10,-10) off the centre; +90° (x' = -y, y' = x) → (10,-10) → (60,40).
    assert!(
        (op[0][0] - 60.0).abs() < 1e-2 && (op[0][1] - 40.0).abs() < 1e-2,
        "{:?}",
        op[0]
    );
}

#[test]
fn a_miss_grabs_nothing() {
    let (pts, h) = square();
    assert!(grab(&pts, &h, [200.0, 200.0], 6.0).is_none());
}

#[test]
fn move_preserves_relative_handles() {
    // A non-collapsed handle rides the translation rigidly.
    let pts = vec![[40.0, 40.0], [60.0, 40.0], [60.0, 60.0], [40.0, 60.0]];
    let mut handles: Vec<[[f32; 2]; 2]> = pts.iter().map(|&p| [p, p]).collect();
    handles[0] = [[35.0, 40.0], [45.0, 40.0]];
    let g = grab(&pts, &handles, [50.0, 50.0], 6.0).unwrap();
    let (_, oh) = apply(&g, [60.0, 50.0]); // +10 x
    assert_eq!(
        oh[0],
        [[45.0, 40.0], [55.0, 40.0]],
        "handles translated with the anchor"
    );
}
