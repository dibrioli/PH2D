use super::{Area, ZOOM_RANGE};
use crate::Camera;

const AREA: Area = [100.0, 50.0, 800.0, 600.0];

fn close(a: [f64; 2], b: [f64; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9
}

#[test]
fn screen_and_world_are_inverse() {
    let cam = Camera {
        center_x: 37.5,
        center_y: -12.0,
        zoom: 2.75,
    };
    for p in [[0.0, 0.0], [123.4, -56.7], [1e6, -1e6]] {
        assert!(close(cam.to_world(AREA, cam.to_screen(AREA, p)), p));
    }
    assert!(
        close(cam.to_screen(AREA, [37.5, -12.0]), [500.0, 350.0]),
        "o centro vai ao meio da área"
    );
}

#[test]
fn zoom_keeps_the_point_under_the_cursor_fixed() {
    let mut cam = Camera::default();
    let cursor = [700.0, 120.0];
    let before = cam.to_world(AREA, cursor);
    for f in [1.1, 0.9, 3.0, 0.25] {
        cam.zoom_about(AREA, cursor, f);
        assert!(close(cam.to_world(AREA, cursor), before));
    }
}

#[test]
fn zoom_is_clamped_and_ignores_nonsense_factors() {
    let mut cam = Camera::default();
    for _ in 0..200 {
        cam.zoom_about(AREA, [0.0, 0.0], 10.0);
    }
    assert_eq!(cam.zoom, ZOOM_RANGE.1);
    let snapshot = cam;
    for f in [0.0, -2.0, f64::NAN, f64::INFINITY] {
        cam.zoom_about(AREA, [0.0, 0.0], f);
    }
    assert_eq!(cam, snapshot);
}

#[test]
fn panning_moves_the_world_with_the_cursor() {
    let mut cam = Camera {
        center_x: 0.0,
        center_y: 0.0,
        zoom: 2.0,
    };
    let p = [10.0, 10.0];
    let s0 = cam.to_screen(AREA, p);
    cam.pan_by_screen(30.0, -8.0);
    let s1 = cam.to_screen(AREA, p);
    assert!(close([s1[0] - s0[0], s1[1] - s0[1]], [30.0, -8.0]));
}

#[test]
fn fit_in_frames_the_box_inside_the_free_part_of_the_area() {
    // A área inteira à esquerda de um cartão de 240 px à direita e por cima de uma faixa de 90 px.
    let free: Area = [140.0, 60.0, 520.0, 500.0];
    let bbox = [-300.0, -50.0, 900.0, 650.0];
    let mut cam = Camera::default();
    cam.fit_in(AREA, free, bbox, 16.0);
    let a = cam.to_screen(AREA, [bbox[0], bbox[1]]);
    let b = cam.to_screen(AREA, [bbox[2], bbox[3]]);
    let eps = 1e-9;
    assert!(
        a[0] >= free[0] + 16.0 - eps && a[1] >= free[1] + 16.0 - eps,
        "{a:?}"
    );
    assert!(
        b[0] <= free[0] + free[2] - 16.0 + eps && b[1] <= free[1] + free[3] - 16.0 + eps,
        "{b:?}"
    );
    let mid = cam.to_screen(AREA, [300.0, 300.0]);
    assert!(
        close(mid, [free[0] + free[2] / 2.0, free[1] + free[3] / 2.0]),
        "{mid:?}"
    );
    // Um `free` sem lugar enquadra na área inteira, como o `fit`.
    let (mut x, mut y) = (Camera::default(), Camera::default());
    x.fit_in(AREA, [0.0, 0.0, 10.0, f64::NAN], bbox, 16.0);
    y.fit(AREA, bbox, 16.0);
    assert_eq!(x, y);
}
