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

use super::{
    DYNAMIC_RGBA, SENSOR_ACTIVE_RGBA, SENSOR_IDLE_RGBA, STATIC_RGBA, collider_outline,
    outline_rgba, outlines,
};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{BodyKind, ColliderShape, ShapeDesc};
use ph2d_render::Camera2d;
use ph2d_vector::PathEl;

fn window() -> WindowSize {
    WindowSize {
        width: 1000,
        height: 1000,
    }
}

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 10.0,
        ..Camera2d::default()
    }
}

/// Every point the path visits, in screen pixels.
fn points(path: &ph2d_vector::BezPath) -> Vec<(f64, f64)> {
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

fn physics_scene() -> ph2d_ecs::SimWorld {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{BodyKind, Collider, RigidBody};
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -1.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 2.0)),
    ));
    sim
}

/// The toggle is honoured where the decision is MADE, not by dimming a
/// paint call. Mutation-tested: dropping the `!show` early return draws
/// the outlines with the overlay switched off.
#[test]
fn switching_the_overlay_off_produces_nothing_to_draw() {
    let mut sim = physics_scene();
    assert!(
        outlines(false, false, &mut sim, &[], &camera(), window()).is_empty(),
        "the overlay drew while switched off"
    );
    assert_eq!(
        outlines(true, false, &mut sim, &[], &camera(), window()).len(),
        2,
        "the overlay drew nothing while switched on"
    );
}

/// **A parented body's outline sits on its SPRITE, not on its local pose.**
///
/// The outline exists to annotate the art, so it has to be where the art is
/// — and the art is drawn from the composed chain. Reading the raw
/// `Transform` puts every child's outline a full parent-offset away; under
/// a rig at `x = -3` they all drew at `x = 0`, so the whole scene's
/// colliders piled up in the middle, far from the sprites they described.
///
/// ⚠️ Every other test in this module uses a ROOT body, where local and
/// world are the same thing — so all twelve of them stayed green while this
/// shipped. The parent is what gives the fixture teeth.
#[test]
fn a_parented_bodys_outline_sits_on_its_sprite_not_its_local_pose() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{ChildOf, Transform};
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
    const RIG_X: f32 = -3.0;
    const LOCAL_Y: f32 = 2.0;

    let mut sim = ph2d_ecs::SimWorld::new();
    let rig = sim
        .world_mut()
        .spawn((Transform::from_translation(Vec2::new(RIG_X, 0.0)),))
        .id();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.5 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, LOCAL_Y)),
        ChildOf(rig),
    ));

    let drawn = outlines(true, false, &mut sim, &[], &camera(), window());
    assert_eq!(drawn.len(), 1, "expected exactly one outline");
    let pts = points(&drawn[0].0);
    let cx = pts.iter().map(|(x, _)| *x).sum::<f64>() / pts.len() as f64;

    // The camera maps +1 world x to +100 px, with world x = 0 at screen centre.
    let want = points(&collider_outline(
        ShapeDesc::Ball { radius: 0.5 },
        RIG_X,
        LOCAL_Y,
        0.0,
        &camera(),
        window(),
    ));
    let want_cx = want.iter().map(|(x, _)| *x).sum::<f64>() / want.len() as f64;
    assert!(
        (cx - want_cx).abs() < 1e-3,
        "the outline is centred at x = {cx:.1} px but its sprite is drawn at \
         {want_cx:.1} px — it was placed at the body's LOCAL pose, a full \
         parent-offset away from the art it annotates"
    );
}

/// **A sensor is magenta, and brightens when triggered.** The colour is the
/// whole visible reaction of a trigger: idle vs active is how you see it
/// fire, and a sensor overrides its body-kind colour so "is this a trigger?"
/// reads first. Mutation-tested: collapsing idle and active to one colour, or
/// letting a sensor keep its kind colour, fails an assert here.
#[test]
fn a_sensor_is_magenta_and_brightens_when_triggered() {
    assert_eq!(
        outline_rgba(true, false, BodyKind::Static),
        SENSOR_IDLE_RGBA
    );
    assert_eq!(
        outline_rgba(true, true, BodyKind::Dynamic),
        SENSOR_ACTIVE_RGBA
    );
    // A solid collider keeps its kind colour and is never magenta.
    assert_eq!(outline_rgba(false, false, BodyKind::Static), STATIC_RGBA);
    assert_ne!(
        SENSOR_IDLE_RGBA, SENSOR_ACTIVE_RGBA,
        "idle and active sensors must differ — the colour change IS the trigger firing"
    );
}

/// The scene-level path: a sensor entity passed in `triggered` draws the
/// active colour, and drawing it out of `triggered` (nothing inside) draws
/// the idle one — proof the overlay reads the bridge's trigger state.
#[test]
fn a_triggered_sensor_outline_uses_the_active_colour() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{Collider, RigidBody};

    let mut sim = ph2d_ecs::SimWorld::new();
    let sensor = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                is_sensor: true,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();

    let idle = outlines(true, false, &mut sim, &[], &camera(), window());
    assert_eq!(idle.len(), 1);
    assert_eq!(idle[0].1, SENSOR_IDLE_RGBA, "an empty sensor is drawn idle");

    let active = outlines(true, false, &mut sim, &[sensor], &camera(), window());
    assert_eq!(
        active[0].1, SENSOR_ACTIVE_RGBA,
        "a triggered sensor is drawn active"
    );
}

/// A scene with no physics costs nothing and shows nothing — a painter or
/// vector user must never see physics chrome appear over their artwork.
#[test]
fn a_scene_without_bodies_draws_no_physics_chrome() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    let mut sim = ph2d_ecs::SimWorld::new();
    sim.world_mut()
        .spawn((Transform::from_translation(Vec2::new(1.0, 1.0)),));
    assert!(
        outlines(true, false, &mut sim, &[], &camera(), window()).is_empty(),
        "physics chrome leaked into a scene with no bodies"
    );
}

/// Scenery and movers are told apart at a glance. Without this the floor
/// and the falling body draw identically, and "which of these can move?"
/// — the first question you ask a physics scene — has no answer on screen.
#[test]
fn static_and_dynamic_bodies_are_drawn_in_different_colours() {
    let mut sim = physics_scene();
    let out = outlines(true, false, &mut sim, &[], &camera(), window());
    let colours: Vec<[f32; 4]> = out.iter().map(|(_, c)| *c).collect();
    assert!(colours.contains(&STATIC_RGBA), "no static body was drawn");
    assert!(colours.contains(&DYNAMIC_RGBA), "no dynamic body was drawn");
    assert_ne!(
        STATIC_RGBA, DYNAMIC_RGBA,
        "static and dynamic share a colour — the distinction is invisible"
    );
}

/// **A non-uniformly scaled ball is drawn as the ELLIPSE it simulates.**
///
/// The bridge turns `Ball` under non-uniform scale into a `ShapeDesc::Ellipse`
/// (`scaled_shape`), so the outline — resolving through the same function —
/// has to draw an ellipse, not a circle. Mutation-tested: the ellipse arm
/// drawing a circle (any single radius) collapses the two extents together
/// and this goes red.
#[test]
fn a_nonuniform_scaled_ball_is_drawn_as_an_ellipse() {
    // rx = 1 world unit → 100 px, ry = 2 → 200 px on this camera.
    let path = collider_outline(
        ShapeDesc::Ellipse { rx: 1.0, ry: 2.0 },
        0.0,
        0.0,
        0.0,
        &camera(),
        window(),
    );
    let pts = points(&path);
    // ELLIPSE_SEGS rim points + the spoke's 2.
    let rim = &pts[..super::CIRCLE_SEGS as usize];
    let (cx, cy) = (500.0f64, 500.0f64);
    let radii: Vec<f64> = rim
        .iter()
        .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
        .collect();
    let min = radii.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = radii.iter().cloned().fold(0.0, f64::max);
    // A circle would have min == max. The ellipse's short axis is ~100 px
    // (rx) and its long axis ~200 px (ry) — the extents MUST differ, and by
    // ~2×, or the outline is describing a circle where the collider is an
    // ellipse (the exact wireframe-lies bug this module exists to prevent).
    assert!(
        (min - 100.0).abs() < 0.5,
        "the ellipse's short axis is {min} px; expected ~100 (rx = 1 unit)"
    );
    assert!(
        (max - 200.0).abs() < 0.5,
        "the ellipse's long axis is {max} px; expected ~200 (ry = 2 units)"
    );
}

/// **A parented body's outline grows with its WORLD scale.**
///
/// The collider inherits the composed parent scale (Unity/Godot do the
/// same), so a ball under a 2× parent draws — and simulates — at twice the
/// authored radius. Reading the raw local scale (which is unit here) would
/// leave it authored-size, so the fixture's teeth are the *parent*: every
/// unscaled-root test in this module stays green while this catches the
/// class ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
///
/// Mutation-tested: dropping `t.scale` from `outlines`' `scaled_shape` call
/// draws the 50 px authored radius and this goes red.
#[test]
fn a_parented_bodys_outline_grows_with_its_world_scale() {
    use ph2d_core::Vec2;
    use ph2d_ecs::{ChildOf, Transform};
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

    let mut sim = ph2d_ecs::SimWorld::new();
    // A rig scaled 2× uniformly (no translation, so the outline stays
    // centred and only its SIZE can change).
    let rig = sim
        .world_mut()
        .spawn((Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(2.0, 2.0),
            skew_x: 0.0,
            skew_y: 0.0,
        },))
        .id();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.5 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        ChildOf(rig),
    ));

    let drawn = outlines(true, false, &mut sim, &[], &camera(), window());
    assert_eq!(drawn.len(), 1, "expected exactly one outline");
    let pts = points(&drawn[0].0);
    let rim = &pts[..super::CIRCLE_SEGS as usize];
    let max = rim
        .iter()
        .map(|(x, y)| ((x - 500.0f64).powi(2) + (y - 500.0f64).powi(2)).sqrt())
        .fold(0.0f64, f64::max);
    // radius 0.5 × parent scale 2 = 1.0 world unit = 100 px. The authored
    // (un-scaled) radius would be 50 px — half of what a correct read gives.
    assert!(
        (max - 100.0).abs() < 0.5,
        "the outline's radius is {max} px; a radius-0.5 ball under a 2× parent \
         must draw at 100 px — it was drawn at its authored size, so the world \
         scale never reached the collider"
    );
}
