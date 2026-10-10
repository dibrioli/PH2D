use super::*;

/// ⭐⭐ **A matriz de modelo** (doc ObjectsAndComponents 01 §13): um ponto LOCAL cai no ecrã onde o
/// ponto `model·p` cairia na câmara sem modelo; ida e volta pelo cursor devolve o local; a afim do
/// Vello e a `view_proj` dizem o mesmo que as duas portas de ponto. (Mutação: `unmodel_point`
/// devolver `w` ⇒ RED.)
#[test]
fn the_model_matrix_moves_every_door_together() {
    let win = WindowSize {
        width: 800,
        height: 600,
    };
    let base = Camera2d::new([3.0, -2.0], 12.0);
    let m = [0.0, 2.0, -2.0, 0.0, 5.0, 1.0]; // 90° e escala 2, deslocado (5, 1)
    let cam = base.with_model(m);
    let p = [1.5, -0.5];
    let mp = [
        m[0] * p[0] + m[2] * p[1] + m[4],
        m[1] * p[0] + m[3] * p[1] + m[5],
    ];
    let (sx, sy) = cam.world_to_screen(p, win);
    let (bx, by) = base.world_to_screen(mp, win);
    assert!((sx - bx).abs() < 1e-3 && (sy - by).abs() < 1e-3);
    let back = cam.screen_to_world((sx, sy), win);
    assert!((back[0] - p[0]).abs() < 1e-4 && (back[1] - p[1]).abs() < 1e-4);
    let a = cam.world_to_screen_affine(win) * vello::kurbo::Point::new(1.5, -0.5);
    assert!((a.x as f32 - sx).abs() < 1e-3 && (a.y as f32 - sy).abs() < 1e-3);
    let ndc = cam
        .view_proj(win)
        .transform_point3(ph2d_core::Vec3::new(p[0], p[1], 0.0));
    let px = (ndc.x + 1.0) * 0.5 * 800.0;
    let py = (1.0 - ndc.y) * 0.5 * 600.0;
    assert!(
        (px - sx).abs() < 1e-2 && (py - sy).abs() < 1e-2,
        "{px},{py} ≠ {sx},{sy}"
    );
    assert_eq!(
        Camera2d::new([3.0, -2.0], 12.0).world_to_screen(p, win),
        base.with_model(Camera2d::MODEL_IDENTITY)
            .world_to_screen(p, win),
        "identidade = a câmara de sempre"
    );
}

#[test]
fn cull_mask_defaults_to_all_layers_visible() {
    // W3.T3.12: a fresh camera culls nothing (every VisibilityLayer
    // intersects u32::MAX), so the extract's cull is a no-op until a
    // non-default mask is set.
    assert_eq!(Camera2d::default().cull_mask, u32::MAX);
    assert_eq!(Camera2d::new([1.0, 2.0], 5.0).cull_mask, u32::MAX);
}

#[test]
fn subrect_uniform_uses_the_subrect_aspect() {
    let cam = Camera2d::new([0.0, 0.0], 10.0); // half_h = 5
    // A 2:1 sub-rect → half_w = half_h * 2 = 10. Orthographic scale on x is
    // 2/(right-left) = 1/half_w; on y it's 1/half_h.
    let m = cam.view_proj_for_subrect(400.0, 200.0).to_cols_array_2d();
    assert!((m[0][0] - 0.1).abs() < 1e-6, "x scale = 1/half_w = 0.1");
    assert!((m[1][1] - 0.2).abs() < 1e-6, "y scale = 1/half_h = 0.2");
}

#[test]
fn subrect_with_window_dims_equals_full_window_view_proj() {
    // Passing the whole window through the sub-rect path must be identical to
    // the full-window projection (no special-casing needed at the call site).
    let cam = Camera2d::new([3.0, -2.0], 8.0);
    let win = WindowSize::new(1280, 720);
    let full = cam.view_proj(win).to_cols_array_2d();
    let sub = cam
        .view_proj_for_subrect(win.width as f32, win.height as f32)
        .to_cols_array_2d();
    for c in 0..4 {
        for r in 0..4 {
            assert!(
                (full[c][r] - sub[c][r]).abs() < 1e-6,
                "mismatch at [{c}][{r}]"
            );
        }
    }
}

#[test]
fn view_proj_centered_origin_maps_to_clip_origin() {
    let cam = Camera2d::default();
    let win = WindowSize::new(800, 600);
    let m = cam.view_proj(win);
    // World (0,0) → clip ~(0,0).
    let p = m * ph2d_core::Vec4::new(0.0, 0.0, 0.0, 1.0);
    assert!(p.x.abs() < 1e-4);
    assert!(p.y.abs() < 1e-4);
}

#[test]
fn zoom_factor_one_is_identity() {
    let mut cam = Camera2d::default();
    let before = cam.height_world;
    cam.zoom(1.0);
    assert_eq!(cam.height_world, before);
}

#[test]
fn world_to_screen_round_trips_screen_to_world() {
    let cam = Camera2d::new([3.0, -2.0], 8.0);
    let win = WindowSize {
        width: 800,
        height: 600,
    };
    for px in [(0.0, 0.0), (400.0, 300.0), (799.0, 599.0), (123.0, 456.0)] {
        let world = cam.screen_to_world(px, win);
        let (sx, sy) = cam.world_to_screen(world, win);
        assert!((sx - px.0).abs() < 1e-3, "x: {sx} vs {}", px.0);
        assert!((sy - px.1).abs() < 1e-3, "y: {sy} vs {}", px.1);
    }
}

#[test]
fn world_to_screen_affine_matches_world_to_screen() {
    // H5/M3: the Affine MUST produce the same screen pixels as the
    // per-point `world_to_screen`, so the Vector shell bridge can drop
    // its hand-rolled copy without any visual drift. Pure scale +
    // translate (no trig) → no libm/determinism concern (HR-5).
    let cam = Camera2d::new([3.0, -2.0], 8.0);
    let win = WindowSize {
        width: 800,
        height: 600,
    };
    let affine = cam.world_to_screen_affine(win);
    for world in [[0.0, 0.0], [3.0, -2.0], [7.5, 4.25], [-12.0, 9.0]] {
        let (ex, ey) = cam.world_to_screen(world, win);
        let p = affine * vello::kurbo::Point::new(world[0] as f64, world[1] as f64);
        assert!((p.x - ex as f64).abs() < 1e-3, "x: {} vs {ex}", p.x);
        assert!((p.y - ey as f64).abs() < 1e-3, "y: {} vs {ey}", p.y);
    }
}

#[test]
fn zoom_in_decreases_height_world() {
    let mut cam = Camera2d::default();
    cam.zoom(0.5);
    assert!(cam.height_world < 10.0);
    assert!((cam.height_world - 5.0).abs() < 1e-5);
}

#[test]
fn zoom_clamps_to_min() {
    let mut cam = Camera2d::default();
    for _ in 0..200 {
        cam.zoom(0.5);
    }
    assert_eq!(cam.height_world, Camera2d::ZOOM_MIN_HEIGHT_WORLD);
}

#[test]
fn zoom_clamps_to_max() {
    let mut cam = Camera2d::default();
    for _ in 0..200 {
        cam.zoom(2.0);
    }
    assert_eq!(cam.height_world, Camera2d::ZOOM_MAX_HEIGHT_WORLD);
}

#[test]
fn zoom_rejects_non_finite_factor() {
    let mut cam = Camera2d::default();
    let before = cam.height_world;
    cam.zoom(f32::NAN);
    cam.zoom(f32::INFINITY);
    cam.zoom(-1.0);
    cam.zoom(0.0);
    assert_eq!(cam.height_world, before);
}

#[test]
fn pan_zero_delta_is_identity() {
    let mut cam = Camera2d::default();
    let before = cam.center;
    cam.pan_screen_delta(0.0, 0.0, 800.0, 600.0);
    assert_eq!(cam.center, before);
}

#[test]
fn pan_screen_right_moves_camera_left() {
    // Cursor drags right by 100 px on an 800×600 window with
    // height_world 10 → pixels_per_meter = 60 → world delta = 100/60 ≈ 1.667.
    // Camera center should move LEFT by that amount (hand-pan
    // semantics: world scrolls right under the cursor).
    let mut cam = Camera2d::default();
    cam.pan_screen_delta(100.0, 0.0, 800.0, 600.0);
    let expected = -(100.0 / 60.0);
    assert!((cam.center[0] - expected).abs() < 1e-3);
    assert_eq!(cam.center[1], 0.0);
}

#[test]
fn pan_screen_down_moves_camera_up_world() {
    // With the projection's Y-flip (view_proj swaps bottom/top so
    // world Y-up renders as screen Y-down), cursor moving DOWN on
    // screen corresponds to camera moving UP in world Y-up coords
    // — keeping the visible world point under the cursor (Photoshop
    // "hand" pan). cursor +60 px @ 60 px/m → center.y += 1.
    let mut cam = Camera2d::default();
    cam.pan_screen_delta(0.0, 60.0, 800.0, 600.0);
    assert!(
        (cam.center[1] - 1.0).abs() < 1e-3,
        "expected center.y = +1.0, got {}",
        cam.center[1]
    );
}

#[test]
fn pan_rejects_zero_window() {
    let mut cam = Camera2d::default();
    let before = cam.center;
    cam.pan_screen_delta(100.0, 100.0, 0.0, 0.0);
    assert_eq!(cam.center, before);
}

#[test]
fn pan_at_zoomed_in_camera_moves_less_world() {
    // Same screen-pixel delta should produce SMALLER world
    // motion when zoomed in. height_world=2 (zoomed in 5×)
    // makes 1 px = 1/300 m instead of 1/60 m.
    let mut cam_wide = Camera2d::default();
    let mut cam_zoomed = Camera2d::new([0.0, 0.0], 2.0);
    cam_wide.pan_screen_delta(60.0, 0.0, 800.0, 600.0);
    cam_zoomed.pan_screen_delta(60.0, 0.0, 800.0, 600.0);
    // Zoomed-in camera should move less in world coords (smaller
    // |center[0]|).
    assert!(cam_zoomed.center[0].abs() < cam_wide.center[0].abs());
}

#[test]
fn y_up_world_maps_to_positive_clip_y() {
    // Standard ortho (no Y-flip post-M14.4e v2): world Y-up
    // maps to clip Y-up (wgpu NDC has +1 at top per WebGPU spec).
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 800);
    let m = cam.view_proj(win);
    let p = m * ph2d_core::Vec4::new(0.0, 5.0, 0.0, 1.0);
    assert!(
        p.y > 0.0,
        "world +Y should map to positive clip Y, got {}",
        p.y
    );
}

#[test]
fn screen_to_world_center_pixel_is_camera_center() {
    let cam = Camera2d::new([3.5, -2.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [wx, wy] = cam.screen_to_world((400.0, 300.0), win);
    assert!((wx - 3.5).abs() < 1e-4, "cx mismatch: {wx}");
    assert!((wy - -2.0).abs() < 1e-4, "cy mismatch: {wy}");
}

#[test]
fn screen_to_world_top_pixel_maps_to_positive_world_y() {
    // World Y-up, standard projection (no Y-flip): cursor at
    // screen top maps to the TOP of the camera view in world
    // coords = cy + half_h.
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [_, wy] = cam.screen_to_world((400.0, 0.0), win);
    assert!(wy > 0.0, "screen-top maps to world +Y, got {wy}");
    assert!((wy - 5.0).abs() < 1e-4);
}

#[test]
fn screen_to_world_bottom_pixel_maps_to_negative_world_y() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [_, wy] = cam.screen_to_world((400.0, 600.0), win);
    assert!(wy < 0.0, "screen-bottom maps to world -Y, got {wy}");
    assert!((wy - -5.0).abs() < 1e-4);
}

#[test]
fn screen_to_world_right_pixel_maps_to_world_right() {
    // Aspect-aware: 800×600 → half_w = 5 * (800/600) ≈ 6.667.
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let win = WindowSize::new(800, 600);
    let [wx, _] = cam.screen_to_world((800.0, 300.0), win);
    let aspect = 800.0_f32 / 600.0_f32;
    let expected = 5.0 * aspect;
    assert!(
        (wx - expected).abs() < 1e-4,
        "expected {expected}, got {wx}"
    );
}

#[test]
fn screen_to_world_respects_camera_zoom() {
    // Half the height_world → cursor at top maps to half the world Y.
    let cam = Camera2d::new([0.0, 0.0], 5.0);
    let win = WindowSize::new(800, 600);
    let [_, wy_top] = cam.screen_to_world((400.0, 0.0), win);
    assert!((wy_top - 2.5).abs() < 1e-4);
}

/// ⭐ **F6: o zoom à volta de um ponto, dito como um pan** — depois de `zoomed` + o arrasto de
/// `anchor_pan_px`, o mundo sob o ponto não mexeu; e o CONTROLO: sem o arrasto, mexeu.
#[test]
fn the_anchor_pan_keeps_the_world_under_the_finger_still() {
    let win = WindowSize::new(800, 600);
    let at = (650.0, 120.0);
    let centre = [400.0, 300.0];
    for factor in [0.5, 2.0, 0.7] {
        let mut cam = Camera2d::new([1.0, -2.0], 10.0);
        let before = cam.screen_to_world(at, win);
        let h_from = cam.height_world;
        cam.zoom(factor);
        let h_to = cam.height_world;
        let c = cam;
        let [dx, dy] = Camera2d::anchor_pan_px([at.0, at.1], centre, h_from, h_to, h_to);
        cam.pan_screen_delta(dx, dy, 800.0, 600.0);
        let after = cam.screen_to_world(at, win);
        assert!(
            (after[0] - before[0]).abs() < 1e-3 && (after[1] - before[1]).abs() < 1e-3,
            "factor {factor}: o mundo sob o dedo andou de {before:?} para {after:?}"
        );
        let bare = c.screen_to_world(at, win);
        assert!(
            (bare[0] - before[0]).abs() > 1e-2,
            "o controlo: sem o arrasto o mundo sob o dedo não andou ({factor})"
        );
    }
    assert_eq!(
        Camera2d::anchor_pan_px([1.0, 1.0], [0.0, 0.0], 1.0, 2.0, 0.0),
        [0.0, 0.0]
    );
}

/// **A inversa da roda**: `wheel_px_for(wheel_factor(dy)) == dy`, e a pinça não inventa lei.
#[test]
fn the_pinch_enters_by_the_inverse_of_the_wheel_law() {
    for dy in [16.0, -16.0, 5.0, -37.0] {
        let back = Camera2d::wheel_px_for(Camera2d::wheel_factor(dy));
        assert!((back - dy).abs() < 1e-3, "{dy} voltou como {back}");
    }
    assert_eq!(Camera2d::wheel_px_for(0.0), 0.0);
    assert_eq!(Camera2d::wheel_px_for(f32::NAN), 0.0);
    assert!(
        Camera2d::wheel_factor(16.0) < 1.0,
        "a roda para cima não aproxima"
    );
}
