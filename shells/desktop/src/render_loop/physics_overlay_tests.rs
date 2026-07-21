//! **The outline must describe the COLLIDER, not the sprite.**
//!
//! Reported by Enio, 2026-07-18: *"os colliders parecem redondos mas os
//! desenhos são box"*. A sprite is a textured quad and a collider is
//! invisible, so a ball under a square sprite is indistinguishable from a
//! box under one — until it rolls. These pin that the drawn outline is the
//! collider's own geometry, at the body's own pose.
//!
//! Driving the pure geometry rather than the paint call is the point: "is
//! this circle actually round, in screen pixels, at this camera" is
//! answerable headless, and the answer is what the artist sees.

use super::collider_outline;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::ShapeDesc;
use ph2d_render::Camera2d;
use ph2d_vector::PathEl;

pub(super) fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

pub(super) fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

/// Every point the path visits, in screen pixels.
pub(super) fn points(path: &ph2d_vector::BezPath) -> Vec<(f64, f64)> {
    path.elements()
        .iter()
        .filter_map(|el| match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some((p.x, p.y)),
            _ => None,
        })
        .collect()
}

/// A ball is drawn ROUND: every rim point is the same distance from the
/// centre. Mutation-tested — an outline built from the sprite quad (or a
/// cuboid arm reused for the ball) makes the spread blow past the tolerance.
#[test]
fn a_ball_collider_is_drawn_as_a_circle_not_a_box() {
    let path = collider_outline(
        ShapeDesc::Ball { radius: 1.0 },
        0.0,
        0.0,
        0.0,
        &camera(),
        window(),
    );
    let pts = points(&path);
    // 32 rim points + the spoke's 2 (centre, rim).
    assert_eq!(pts.len(), 34, "expected a 32-segment rim plus a spoke");

    let (cx, cy) = (500.0f64, 500.0f64); // world origin at the screen centre
    let rim = &pts[..32];
    let radii: Vec<f64> = rim
        .iter()
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .collect();
    let min = radii.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = radii.iter().cloned().fold(0.0, f64::max);
    // 0.01 px. World coordinates are `f32` (that is what `Transform`
    // stores), so the rim carries ~1e-4 px of trig rounding — inherent,
    // not a defect. The bar still has to MEAN something: the shape error
    // this gate exists to catch is a box, whose corners sit 41 px further
    // out, so this tolerance is ~4000× tighter than the phenomenon.
    assert!(
        max - min < 0.01,
        "the rim is not round: radii span {min}..{max} px — the outline is describing \
         something other than the ball"
    );
    // …and it is a real circle, not a degenerate dot: r = 1 world unit on a
    // 10-unit-tall camera over 1000 px = 100 px.
    assert!(
        (max - 100.0).abs() < 0.01,
        "the circle is {max} px; expected 100 px for a 1-unit radius"
    );

    // A square of the same extent would put its corners at r·√2 ≈ 141 px.
    // Pinning that the max radius is NOT that is what makes this gate about
    // the reported bug rather than about arithmetic.
    assert!(
        max < 120.0,
        "the outline reaches {max} px — that is a box's corner, not a circle's rim"
    );
}

/// A cuboid is drawn as its four corners — and the box's corners DO sit at
/// r·√2, which is exactly what a circle must not do. The sibling that gives
/// the gate above its meaning.
#[test]
fn a_cuboid_collider_is_drawn_as_its_four_corners() {
    let path = collider_outline(
        ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 1.0,
        },
        0.0,
        0.0,
        0.0,
        &camera(),
        window(),
    );
    let pts = points(&path);
    assert_eq!(pts.len(), 4, "a box has four corners");
    for (x, y) in &pts {
        let r = ((x - 500.0f64).powi(2) + (y - 500.0f64).powi(2)).sqrt();
        assert!(
            (r - 100.0 * std::f64::consts::SQRT_2).abs() < 1e-4,
            "corner at radius {r}; a 1×1 half-extent box has corners at 141.4 px"
        );
    }
}

/// The outline turns with the body. Without this, a tumbling box would be
/// drawn axis-aligned while its sprite rotates — the same class of lie as
/// drawing a ball as a box, one level down.
///
/// Mutation-tested: dropping the rotation from `place` leaves the corners at
/// their unrotated positions and this goes red.
#[test]
fn the_outline_rotates_with_the_body() {
    let square = ShapeDesc::Cuboid {
        half_x: 1.0,
        half_y: 1.0,
    };
    let flat = points(&collider_outline(
        square,
        0.0,
        0.0,
        0.0,
        &camera(),
        window(),
    ));
    let tilted = points(&collider_outline(
        square,
        0.0,
        0.0,
        std::f32::consts::FRAC_PI_4,
        &camera(),
        window(),
    ));

    // At 45° a corner lands on an axis: (0, ±141) or (±141, 0) from centre.
    let moved = flat
        .iter()
        .zip(&tilted)
        .filter(|((ax, ay), (bx, by))| (ax - bx).abs() > 1.0 || (ay - by).abs() > 1.0)
        .count();
    assert_eq!(moved, 4, "a 45° rotation must move every corner");
    assert!(
        tilted
            .iter()
            .any(|(x, y)| (x - 500.0).abs() < 1e-3 && (y - 500.0).abs() > 140.0),
        "no corner landed on the vertical axis — the body's rotation is not being applied"
    );
}

/// The outline follows the body's live pose, so it sits on the sprite as the
/// sim moves it rather than staying at the origin.
#[test]
fn the_outline_follows_the_body_position() {
    let ball = ShapeDesc::Ball { radius: 0.5 };
    let here = points(&collider_outline(ball, 0.0, 0.0, 0.0, &camera(), window()));
    let there = points(&collider_outline(ball, 2.0, -1.0, 0.0, &camera(), window()));
    // +2 world x = +200 px; -1 world y = +100 px (screen y grows downward).
    for ((ax, ay), (bx, by)) in here.iter().zip(&there) {
        assert!(
            (bx - ax - 200.0).abs() < 1e-3 && (by - ay - 100.0).abs() < 1e-3,
            "the outline did not translate with the body"
        );
    }
}

/// **Screen space, and that is load-bearing.** Zooming in must grow the
/// PATH — because the geometry is already in pixels, the stroke width stays
/// the constant it was written as. Handing the world→screen affine to Vello's
/// `stroke` instead would multiply the width by pixels-per-world-unit, which
/// is the bug that turned the Flip selection halo into a screen-wide smear
/// (smoke, 2026-07-13).
#[test]
fn the_geometry_is_in_screen_pixels_so_the_stroke_width_is_not_scaled() {
    let ball = ShapeDesc::Ball { radius: 1.0 };
    let wide = Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    };
    let zoomed = Camera2d {
        center: [0.0, 0.0],
        height_world: 2.5, // 4× closer
        ..Camera2d::default()
    };
    let r = |cam: &Camera2d| {
        let p = points(&collider_outline(ball, 0.0, 0.0, 0.0, cam, window()));
        ((p[0].0 - 500.0f64).powi(2) + (p[0].1 - 500.0f64).powi(2)).sqrt()
    };
    let (a, b) = (r(&wide), r(&zoomed));
    assert!(
        (b / a - 4.0).abs() < 1e-4,
        "zooming 4× scaled the outline by {:.3}× — the points are not going through the camera",
        b / a
    );
}
