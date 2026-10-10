//! A caixa de mundo de um gizmo rodado (F8 — o que o *Enquadrar seleção* encaixa).

use super::*;
use crate::zones::Rect;

fn rodada(rotation: f32) -> GizmoView {
    GizmoView {
        bbox_min_world: [1.0, 2.0],
        bbox_max_world: [5.0, 4.0],
        rotation,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: [3.0, 3.0],
        pivot_tool_active: false,
        model: crate::gizmo::MODEL_IDENTITY,
    }
}

#[test]
fn the_world_box_of_a_turned_gizmo_covers_its_turned_corners() {
    assert_eq!(rodada(0.0).world_aabb(), ([1.0, 2.0], [5.0, 4.0]));
    // A 90° a caixa 4×2 vira 2×4 à volta do centro (3, 3).
    let (lo, hi) = rodada(std::f32::consts::FRAC_PI_2).world_aabb();
    for (a, b) in [(lo, [2.0, 1.0]), (hi, [4.0, 5.0])] {
        assert!((a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5, "{a:?} ≠ {b:?}");
    }
    // A 45° cada meia-extensão é (2 + 1)·√½.
    let (lo, hi) = rodada(std::f32::consts::FRAC_PI_4).world_aabb();
    let e = 3.0 * std::f32::consts::FRAC_1_SQRT_2;
    assert!((hi[0] - lo[0] - 2.0 * e).abs() < 1e-5 && (hi[1] - lo[1] - 2.0 * e).abs() < 1e-5);
}
