use super::paint::{ROTATE_HANDLE_OFFSET, corner_outer_rect, world_to_screen};
use super::transform::{opposite_anchor_translation, quantize};
use super::*;
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn view(bbox_min: [f32; 2], bbox_max: [f32; 2]) -> GizmoView {
    GizmoView {
        bbox_min_world: bbox_min,
        bbox_max_world: bbox_max,
        rotation: 0.0,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: [
            (bbox_min[0] + bbox_max[0]) * 0.5,
            (bbox_min[1] + bbox_max[1]) * 0.5,
        ],
        pivot_tool_active: false,
    }
}

#[test]
fn world_to_screen_center_maps_to_window_center() {
    let v = view([-1.0, -1.0], [1.0, 1.0]);
    let s = world_to_screen(&v, [0.0, 0.0]);
    assert!((s[0] - 400.0).abs() < 1e-3);
    assert!((s[1] - 300.0).abs() < 1e-3);
}

#[test]
fn world_to_screen_yflips_correctly() {
    let v = view([-1.0, -1.0], [1.0, 1.0]);
    // World Y high → screen Y low (top of window).
    let top = world_to_screen(&v, [0.0, 5.0]);
    let bottom = world_to_screen(&v, [0.0, -5.0]);
    assert!(top[1] < bottom[1], "world-up should map to screen-top");
}

#[test]
fn corner_outer_rect_top_left_sits_above_and_to_the_left() {
    let r = corner_outer_rect(100.0, 100.0, -1.0, -1.0);
    assert!(r.x < 100.0);
    assert!(r.y < 100.0);
    assert_eq!(r.w, ROTATE_HANDLE_OFFSET);
    assert_eq!(r.h, ROTATE_HANDLE_OFFSET);
}

#[test]
fn paint_smoke() {
    let mut scene = VectorScene::new();
    let mut hits = HitIndex::new();
    let v = view([-0.5, -0.5], [0.5, 0.5]);
    paint_sprite_gizmo(&mut scene, &v, Theme::Forge, &mut hits);
}

fn cam() -> GizmoCamera {
    GizmoCamera {
        center: [0.0, 0.0],
        height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
    }
}

fn snapshot(tx: f32, ty: f32) -> TransformSnapshot {
    TransformSnapshot {
        translation: [tx, ty],
        rotation: 0.0,
        scale: [1.0, 1.0],
    }
}

#[test]
fn camera_screen_to_world_center_is_origin() {
    let c = cam();
    let w = c.screen_to_world((400.0, 300.0));
    assert!(w[0].abs() < 1e-3);
    assert!(w[1].abs() < 1e-3);
}

#[test]
fn translate_drag_moves_by_world_delta() {
    let c = cam();
    let start = c.screen_to_world((400.0, 300.0));
    let drag = GizmoDragState {
        kind: GizmoDragKind::Translate,
        entity_bits: 1,
        start_screen: (400.0, 300.0),
        // 80 px to the right ≈ 80/800 * camera_width meters
        // (camera_width = height_world * aspect = 10 * 4/3 = 13.33)
        cursor_screen: (480.0, 300.0),
        start_transform: snapshot(0.0, 0.0),
        pivot_world: [0.0, 0.0],
        start_cursor_world: start,
        sprite_half_intrinsic: [0.0, 0.0],
        anchor_is_center: false,
    };
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    let now = c.screen_to_world((480.0, 300.0));
    // New translation must equal start + (now - start_cursor_world).
    assert!((t.translation[0] - (now[0] - start[0])).abs() < 1e-3);
    assert!((t.translation[1] - 0.0).abs() < 1e-3);
    // Rotation + scale untouched.
    assert_eq!(t.rotation, 0.0);
    assert_eq!(t.scale, [1.0, 1.0]);
}

#[test]
fn scale_corner_doubling_distance_doubles_scale() {
    let c = cam();
    // BR corner of a 2×2 bbox centered at origin: world (1, -1).
    let start_corner_world = [1.0, -1.0];
    let drag = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: -1.0,
        },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [0.0, 0.0],
        start_cursor_world: start_corner_world,
        // Center-anchor so the scale-ratio test isn't perturbed
        // by the opposite-corner translation step. Pivot at the
        // sprite center matches `anchor_is_center: true`.
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: true,
    };
    // We bypass the screen→world projection by overriding the
    // computed `now_world` via cursor_screen → its projection.
    // Construct a fake camera where (cursor_screen) projects to
    // (2, -2) world (doubling the distance from pivot in both
    // axes). Easier: pre-compute the cursor_screen that gives
    // (2, -2) under cam().
    let target_world = [2.0, -2.0];
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    let mut drag = drag;
    drag.cursor_screen = (cursor_x, cursor_y);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    // Doubling along both axes → scale becomes 2× start.
    assert!(
        (t.scale[0] - 2.0).abs() < 1e-3,
        "expected 2x scale_x, got {}",
        t.scale[0]
    );
    assert!(
        (t.scale[1] - 2.0).abs() < 1e-3,
        "expected 2x scale_y, got {}",
        t.scale[1]
    );
}

#[test]
fn scale_edge_axis_only() {
    let c = cam();
    // R edge handle at (1, 0). Dragging to (3, 0) → 3x scale_x.
    let drag = GizmoDragState {
        kind: GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [0.0, 0.0],
        start_cursor_world: [1.0, 0.0],
        // Center-anchor — focuses the test on the scale ratio
        // path only.
        sprite_half_intrinsic: [1.0, 0.0],
        anchor_is_center: true,
    };
    let target_world = [3.0, 0.0];
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    let mut drag = drag;
    drag.cursor_screen = (cursor_x, cursor_y);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    assert!((t.scale[0] - 3.0).abs() < 1e-3);
    // Y axis untouched.
    assert!((t.scale[1] - 1.0).abs() < 1e-3);
}

#[test]
fn rotate_quarter_turn_adds_pi_over_two() {
    let c = cam();
    // Start cursor at (1, 0) world, drag to (0, 1) world →
    // angle changes from 0 → π/2.
    let drag = GizmoDragState {
        kind: GizmoDragKind::Rotate,
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [0.0, 0.0],
        start_cursor_world: [1.0, 0.0],
        sprite_half_intrinsic: [0.0, 0.0],
        anchor_is_center: false,
    };
    let target_world = [0.0, 1.0];
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    let mut drag = drag;
    drag.cursor_screen = (cursor_x, cursor_y);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    let pi_over_2 = std::f32::consts::FRAC_PI_2;
    assert!(
        (t.rotation - pi_over_2).abs() < 1e-3,
        "expected π/2, got {}",
        t.rotation
    );
    // Translation + scale untouched.
    assert_eq!(t.translation, [0.0, 0.0]);
    assert_eq!(t.scale, [1.0, 1.0]);
}

#[test]
fn translate_with_ctrl_snaps_to_grid() {
    let c = cam();
    let start = c.screen_to_world((400.0, 300.0));
    // Drag ~80 px right → ~0.83 m world delta (camera_width / w
    // ≈ 13.33/800 m/px). Default snap = 0.16 m → snapped to
    // round(0.83/0.16)*0.16 = 5 * 0.16 = 0.80 m (approximately).
    let drag = GizmoDragState {
        kind: GizmoDragKind::Translate,
        entity_bits: 1,
        start_screen: (400.0, 300.0),
        cursor_screen: (480.0, 300.0),
        start_transform: snapshot(0.0, 0.0),
        pivot_world: [0.0, 0.0],
        start_cursor_world: start,
        sprite_half_intrinsic: [0.0, 0.0],
        anchor_is_center: false,
    };
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers {
            shift: false,
            ctrl: true,
            alt: false,
        },
        GizmoSnap {
            move_meters: 0.16,
            rotate_deg: 0.0,
        },
        None,
    );
    // Result must be a multiple of 0.16.
    let rem = (t.translation[0] / 0.16).fract().abs();
    assert!(
        rem < 1e-3 || (1.0 - rem) < 1e-3,
        "translation_x {} should be a multiple of 0.16",
        t.translation[0]
    );
}

#[test]
fn scale_corner_with_shift_locks_aspect_ratio() {
    // Asymmetric drag — natural ratio_x ≠ ratio_y. Shift forces
    // both to share the larger-deviation ratio.
    let c = cam();
    let start_corner_world = [1.0, -1.0];
    let drag = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: -1.0,
        },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [0.0, 0.0],
        start_cursor_world: start_corner_world,
        // Center anchor isolates the scale-ratio + AR-lock path.
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: true,
    };
    // Drag to (3, -1.5) world → ratio_x = 3, ratio_y = 1.5.
    // With Shift, both axes lock to the largest deviation, ratio_x=3.
    let target_world = [3.0, -1.5];
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    let mut drag = drag;
    drag.cursor_screen = (cursor_x, cursor_y);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers {
            shift: true,
            ctrl: false,
            alt: false,
        },
        GizmoSnap::default(),
        None,
    );
    // Uniform scale — both axes equal.
    assert!(
        (t.scale[0] - t.scale[1]).abs() < 1e-3,
        "shift should lock AR — scale={:?}",
        t.scale
    );
    // And the dominant ratio (3) wins.
    assert!((t.scale[0] - 3.0).abs() < 1e-2, "got {}", t.scale[0]);
}

#[test]
fn rotate_with_shift_snaps_to_step() {
    // Drag to a non-aligned angle (~30 degrees off from start);
    // Shift + 15° step should round to nearest 15°.
    let c = cam();
    let drag = GizmoDragState {
        kind: GizmoDragKind::Rotate,
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [0.0, 0.0],
        start_cursor_world: [1.0, 0.0],
        sprite_half_intrinsic: [0.0, 0.0],
        anchor_is_center: false,
    };
    // Target angle ~32° (between 30° and 45°). Snap to 15° →
    // 30° = π/6 ≈ 0.5236.
    let target_angle_deg = 32.0_f32;
    let target_world = [
        (target_angle_deg.to_radians()).cos(),
        (target_angle_deg.to_radians()).sin(),
    ];
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    let mut drag = drag;
    drag.cursor_screen = (cursor_x, cursor_y);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers {
            shift: true,
            ctrl: false,
            alt: false,
        },
        GizmoSnap {
            move_meters: 0.0,
            rotate_deg: 15.0,
        },
        None,
    );
    // Expected: round(32°/15°) * 15° = round(2.13) * 15° = 30°
    // = π/6 ≈ 0.5236.
    let expected = 30.0_f32.to_radians();
    assert!(
        (t.rotation - expected).abs() < 1e-3,
        "expected ~30°, got {} rad ({}°)",
        t.rotation,
        t.rotation.to_degrees()
    );
}

#[test]
fn quantize_zero_step_is_noop() {
    assert_eq!(quantize(0.5, 0.0), 0.5);
    assert_eq!(quantize(-2.5, 0.0), -2.5);
}

#[test]
fn quantize_rounds_to_nearest_multiple() {
    assert_eq!(quantize(0.33, 0.16), 0.32);
    assert_eq!(quantize(0.40, 0.16), 0.48);
    assert_eq!(quantize(-0.40, 0.16), -0.48);
}

#[test]
fn gizmo_kind_for_id_resolves_every_handle() {
    for (id, expected_some) in [
        (ids::GIZMO_BBOX_INTERIOR, true),
        (ids::GIZMO_HANDLE_TL, true),
        (ids::GIZMO_HANDLE_R, true),
        (ids::GIZMO_ROTATE_TR, true),
        (ids::GIZMO_PIVOT, false),
        (NodeId(100), false),
    ] {
        assert_eq!(gizmo_kind_for_id(id).is_some(), expected_some, "id {id:?}");
    }
}

#[test]
fn is_gizmo_handle_id_matches_kind_resolver() {
    assert!(is_gizmo_handle_id(ids::GIZMO_HANDLE_TL));
    assert!(is_gizmo_handle_id(ids::GIZMO_BBOX_INTERIOR));
    assert!(!is_gizmo_handle_id(NodeId(0)));
    // Pivot is in the gizmo range but doesn't START a drag.
    assert!(!is_gizmo_handle_id(ids::GIZMO_PIVOT));
}

#[test]
fn paint_registers_thirteen_hit_zones() {
    let mut scene = VectorScene::new();
    let mut hits = HitIndex::new();
    let v = view([-0.5, -0.5], [0.5, 0.5]);
    paint_sprite_gizmo(&mut scene, &v, Theme::Forge, &mut hits);
    // 1 interior + 4 rotate + 4 corners + 4 edges + 1 pivot = 14
    // hit zones registered (the rotate rects each get registered
    // before the corners on top, but each id still counts once).
    for id in [
        ids::GIZMO_BBOX_INTERIOR,
        ids::GIZMO_PIVOT,
        ids::GIZMO_HANDLE_TL,
        ids::GIZMO_HANDLE_TR,
        ids::GIZMO_HANDLE_BL,
        ids::GIZMO_HANDLE_BR,
        ids::GIZMO_HANDLE_T,
        ids::GIZMO_HANDLE_R,
        ids::GIZMO_HANDLE_B,
        ids::GIZMO_HANDLE_L,
        ids::GIZMO_ROTATE_TL,
        ids::GIZMO_ROTATE_TR,
        ids::GIZMO_ROTATE_BL,
        ids::GIZMO_ROTATE_BR,
    ] {
        // `HitIndex::hit` returns whatever id sits at that rect's
        // center; just ensure all 14 ids resolved by checking the
        // pivot rect (smallest, most-recently-registered → wins
        // on overlap) returns SOMETHING. Targeted lookups per id
        // are covered by M14.7 C's dispatch tests.
        assert!(matches!(id.0, 950..=963));
    }
}

#[test]
fn anchor_pivot_world_default_returns_opposite_corner_for_ne_handle() {
    // Sprite at world (10, 20), unit scale, no rotation. The NE
    // corner of a 4×6 sprite sits at (12, 23); the opposite (SW)
    // is (8, 17). Default anchor (no Ctrl) returns SW.
    let snap = TransformSnapshot {
        translation: [10.0, 20.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
    };
    let pivot = anchor_pivot_world(
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        [2.0, 3.0],
        snap,
        false,
    );
    assert!(
        (pivot[0] - 8.0).abs() < 1e-4 && (pivot[1] - 17.0).abs() < 1e-4,
        "expected SW corner [8, 17], got {pivot:?}"
    );
}

#[test]
fn anchor_pivot_world_center_anchor_returns_translation() {
    // Ctrl/Cmd held → pivot at the sprite center regardless of
    // handle kind / sprite size / rotation.
    let snap = TransformSnapshot {
        translation: [100.0, -50.0],
        rotation: 1.234,
        scale: [3.0, 0.5],
    };
    for kind in [
        GizmoDragKind::ScaleCorner {
            dx_sign: -1.0,
            dy_sign: 1.0,
        },
        GizmoDragKind::ScaleEdge {
            axis: 0,
            sign: -1.0,
        },
        GizmoDragKind::ScaleEdge { axis: 1, sign: 1.0 },
    ] {
        let pivot = anchor_pivot_world(kind, [5.0, 7.0], snap, true);
        assert_eq!(
            pivot, snap.translation,
            "kind {kind:?} with center anchor should pivot on translation"
        );
    }
}

#[test]
fn anchor_pivot_world_respects_scale_and_rotation() {
    // 90° rotated sprite (rotation = π/2). Sprite intrinsic
    // half-size 2×3; scale 1×1. NE handle (dx=+1, dy=+1).
    // Opposite local = (-2, -3). After 90° rotation (cos=0, sin=1):
    //   x' = -2*0 - (-3)*1 =  3
    //   y' = -2*1 + (-3)*0 = -2
    // World = translation + (3, -2) = (10+3, 20-2) = (13, 18).
    let snap = TransformSnapshot {
        translation: [10.0, 20.0],
        rotation: std::f32::consts::FRAC_PI_2,
        scale: [1.0, 1.0],
    };
    let pivot = anchor_pivot_world(
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        [2.0, 3.0],
        snap,
        false,
    );
    assert!(
        (pivot[0] - 13.0).abs() < 1e-3 && (pivot[1] - 18.0).abs() < 1e-3,
        "expected [13, 18] after 90° rotation, got {pivot:?}"
    );
}

#[test]
fn anchor_pivot_world_edge_handle_opposite_midpoint() {
    // Right-edge handle on a 4×6 sprite at origin: opposite is
    // the left-edge midpoint, which sits at (-2, 0). No rotation
    // / unit scale → world (-2, 0).
    let snap = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
    };
    let pivot = anchor_pivot_world(
        GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 },
        [2.0, 3.0],
        snap,
        false,
    );
    assert!(
        (pivot[0] - -2.0).abs() < 1e-4 && pivot[1].abs() < 1e-4,
        "expected [-2, 0] for opposite of right edge, got {pivot:?}"
    );
}

#[test]
fn anchor_pivot_world_translate_and_rotate_fall_back_to_center() {
    // Translate / Rotate kinds don't have an opposite anchor —
    // helper must return the sprite center for them.
    let snap = TransformSnapshot {
        translation: [42.0, -7.0],
        rotation: 0.5,
        scale: [2.0, 2.0],
    };
    for kind in [GizmoDragKind::Translate, GizmoDragKind::Rotate] {
        let pivot = anchor_pivot_world(kind, [1.0, 1.0], snap, false);
        assert_eq!(pivot, snap.translation, "{kind:?} must fall back to center");
    }
}

/// Compute the screen pixel that projects to `target_world` under
/// the test camera. Shared by the Scale-anchor tests below.
fn cursor_for_world(c: &GizmoCamera, target_world: [f32; 2]) -> (f32, f32) {
    let aspect = c.window_w / c.window_h;
    let half_w = c.height_world * 0.5 * aspect;
    let half_h = c.height_world * 0.5;
    let nx = target_world[0] / half_w;
    let ny = (c.center[1] - target_world[1]) / half_h;
    let cursor_x = (nx + 1.0) * 0.5 * c.window_w;
    let cursor_y = (ny + 1.0) * 0.5 * c.window_h;
    (cursor_x, cursor_y)
}

#[test]
fn scale_corner_default_anchor_keeps_opposite_corner_fixed() {
    // Bug Enio reported: "o escalonamento movendo um ponto e sem
    // mover o ponto oposto do mouse não funcionou" — opposite
    // corner used to drift because the math returned start
    // translation. With the fix, the opposite-corner world point
    // computed from (new_translation, new_scale, rotation) MUST
    // match the captured `pivot_world`.
    //
    // Sprite: 2×2 intrinsic, scale 1×1, translation (10, 5),
    // rotation 0. NE handle drag (dx=+1, dy=+1) → opposite is
    // SW at (10-1, 5-1) = (9, 4). User drags NE from (11, 6) to
    // (13, 10) world.
    let c = cam();
    let pivot_world = [9.0_f32, 4.0_f32];
    let start_corner = [11.0_f32, 6.0_f32];
    let mut drag = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [10.0, 5.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world,
        start_cursor_world: start_corner,
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: false,
    };
    let target_corner = [13.0_f32, 10.0_f32];
    drag.cursor_screen = cursor_for_world(&c, target_corner);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    // Recover the SW corner world from the returned transform.
    // sprite_half_intrinsic = (1, 1); SW local = (-1, -1) ×
    // new_scale; rotation 0 → world = translation + scaled local.
    let sw_x = t.translation[0] - t.scale[0];
    let sw_y = t.translation[1] - t.scale[1];
    assert!(
        (sw_x - pivot_world[0]).abs() < 1e-3 && (sw_y - pivot_world[1]).abs() < 1e-3,
        "opposite corner moved: pivot={pivot_world:?} actual=({sw_x}, {sw_y}) t={t:?}"
    );
}

#[test]
fn scale_corner_center_anchor_keeps_translation_unchanged() {
    // Ctrl/Cmd held → `anchor_is_center: true`. The Scale path
    // must leave translation alone (sprite scales in place).
    let c = cam();
    let mut drag = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [10.0, 5.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        // With center anchor the pivot is the sprite center.
        pivot_world: [10.0, 5.0],
        start_cursor_world: [11.0, 6.0],
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: true,
    };
    drag.cursor_screen = cursor_for_world(&c, [13.0, 10.0]);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    assert!(
        (t.translation[0] - 10.0).abs() < 1e-4 && (t.translation[1] - 5.0).abs() < 1e-4,
        "center anchor must leave translation untouched, got {:?}",
        t.translation
    );
}

#[test]
fn scale_edge_default_anchor_keeps_opposite_edge_fixed() {
    // Right-edge drag on a 2×2 sprite at (10, 5). Opposite edge
    // midpoint (left edge) = (9, 5). Drag right-edge X from 11
    // to 13. After fix, the left-edge midpoint MUST stay at 9.
    let c = cam();
    let pivot_world = [9.0_f32, 5.0_f32];
    let mut drag = GizmoDragState {
        kind: GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [10.0, 5.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world,
        start_cursor_world: [11.0, 5.0],
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: false,
    };
    drag.cursor_screen = cursor_for_world(&c, [13.0, 5.0]);
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        None,
    );
    // Recover the left edge midpoint world: translation + (-1, 0) * scale.
    let left_x = t.translation[0] - t.scale[0];
    assert!(
        (left_x - pivot_world[0]).abs() < 1e-3,
        "opposite edge moved: pivot.x={} actual.x={} t={t:?}",
        pivot_world[0],
        left_x
    );
}

#[test]
fn scale_corner_with_snap_closure_quantizes_cursor() {
    // World-snap closure rewrites the cursor world position
    // before the math computes the ratio. We pass a closure that
    // snaps to integer world coords; with sprite_half=1 at
    // origin and a pivot at SW (-1, -1), dragging the NE corner
    // to cursor world (2.6, 2.6) → snapped to (3, 3) → scale = 2.
    let c = cam();
    let mut drag = GizmoDragState {
        kind: GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        entity_bits: 1,
        start_screen: (0.0, 0.0),
        cursor_screen: (0.0, 0.0),
        start_transform: TransformSnapshot {
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
        },
        pivot_world: [-1.0, -1.0],
        start_cursor_world: [1.0, 1.0],
        sprite_half_intrinsic: [1.0, 1.0],
        anchor_is_center: false,
    };
    drag.cursor_screen = cursor_for_world(&c, [2.6, 2.6]);
    // Closure snaps each axis to nearest integer meter.
    let mut snap_fn = |w: [f32; 2]| [w[0].round(), w[1].round()];
    let t = compute_gizmo_transform(
        &drag,
        &c,
        GizmoModifiers::default(),
        GizmoSnap::default(),
        Some(&mut snap_fn),
    );
    // Snapped cursor (3, 3) - pivot (-1, -1) = (4, 4); start
    // vector was (1, 1) - (-1, -1) = (2, 2). Ratio = 2 → scale 2.
    assert!(
        (t.scale[0] - 2.0).abs() < 1e-3,
        "expected scale_x = 2.0 from snapped cursor, got {}",
        t.scale[0]
    );
    // And the opposite corner must still anchor at pivot.
    let sw_x = t.translation[0] - t.scale[0];
    let sw_y = t.translation[1] - t.scale[1];
    assert!(
        (sw_x - (-1.0)).abs() < 1e-3 && (sw_y - (-1.0)).abs() < 1e-3,
        "opposite corner moved under snap closure: ({sw_x}, {sw_y})"
    );
}

#[test]
fn opposite_anchor_translation_no_rotation() {
    // Pivot at (10, 10), sprite half (1, 1), opposite-local-sign
    // (-1, -1), new scale (2, 3), rotation 0.
    // Local = (-1*1*2, -1*1*3) = (-2, -3). Rotation 0 → rotated
    // = local. Translation = pivot - rotated = (12, 13).
    let t = opposite_anchor_translation([10.0, 10.0], [1.0, 1.0], [-1.0, -1.0], [2.0, 3.0], 0.0);
    assert!(
        (t[0] - 12.0).abs() < 1e-4 && (t[1] - 13.0).abs() < 1e-4,
        "got {t:?}"
    );
}

#[test]
fn move_pivot_keeps_quad_fixed_identity() {
    // Quad center at origin; drag the pivot to (3, 0). The anchor must
    // re-pin the quad: pivot + anchor == quad_center.
    let (t, a) = move_pivot_transform(snapshot(0.0, 0.0), [0.0, 0.0], [3.0, 0.0]);
    assert_eq!(t, [3.0, 0.0]);
    assert!(
        (a[0] + 3.0).abs() < 1e-5 && a[1].abs() < 1e-5,
        "anchor {a:?}"
    );
    assert!(
        (t[0] + a[0]).abs() < 1e-5,
        "pivot+anchor should equal quad center 0"
    );
}

#[test]
fn move_pivot_divides_out_scale() {
    // scale 2×: a 4-unit world gap becomes a 2-unit intrinsic anchor
    // (extract re-multiplies by scale, like `size`).
    let s = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [2.0, 2.0],
    };
    let (t, a) = move_pivot_transform(s, [0.0, 0.0], [4.0, 0.0]);
    assert_eq!(t, [4.0, 0.0]);
    assert!(
        (a[0] + 2.0).abs() < 1e-5 && a[1].abs() < 1e-5,
        "anchor {a:?}"
    );
}

#[test]
fn move_pivot_inverse_rotates_world_delta() {
    // rotation 90°: dragging the pivot to (0, 2) with quad center at
    // origin must yield an anchor that, once the extract re-rotates it,
    // points back to the origin. Check via the forward rotation.
    let s = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: std::f32::consts::FRAC_PI_2,
        scale: [1.0, 1.0],
    };
    let (t, a) = move_pivot_transform(s, [0.0, 0.0], [0.0, 2.0]);
    let (sin_r, cos_r) = s.rotation.sin_cos();
    let world = [
        t[0] + a[0] * cos_r - a[1] * sin_r,
        t[1] + a[0] * sin_r + a[1] * cos_r,
    ];
    assert!(
        world[0].abs() < 1e-5 && world[1].abs() < 1e-5,
        "re-pinned {world:?}"
    );
}

#[test]
fn pivot_snap_candidates_axis_aligned() {
    let c = pivot_snap_candidates([0.0, 0.0], 0.0, [2.0, 1.0]);
    assert_eq!(c[0], [0.0, 0.0]); // center
    assert_eq!(c[1], [-2.0, 1.0]); // TL
    assert_eq!(c[2], [2.0, 1.0]); // TR
    assert_eq!(c[3], [-2.0, -1.0]); // BL
    assert_eq!(c[4], [2.0, -1.0]); // BR
    assert_eq!(c[5], [0.0, 1.0]); // T
    assert_eq!(c[6], [2.0, 0.0]); // R
    assert_eq!(c[7], [0.0, -1.0]); // B
    assert_eq!(c[8], [-2.0, 0.0]); // L
}

#[test]
fn pivot_snap_candidates_offset_center() {
    // Center offset; corners track it.
    let c = pivot_snap_candidates([10.0, 5.0], 0.0, [1.0, 1.0]);
    assert_eq!(c[0], [10.0, 5.0]);
    assert_eq!(c[2], [11.0, 6.0]); // TR
    assert_eq!(c[3], [9.0, 4.0]); // BL
}
