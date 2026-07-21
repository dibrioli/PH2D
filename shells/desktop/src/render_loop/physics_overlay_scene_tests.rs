//! **The other half of the overlay gates: the SCENE walk.**
//!
//! `physics_overlay_tests` drives the pure geometry — "is this circle round, in
//! screen pixels, at this camera" — which is answerable without a world. These drive
//! [`super::outlines`] over a real `SimWorld`: which colour a body gets, whether a
//! sensor brightens, where a parented body's outline lands, and which annotations
//! (launch, push) are drawn on top.
//!
//! Split from its sibling for the shell's 600-LOC cap (W-Area). The helpers stay in
//! the geometry file — one `camera()`, one `window()`, one `points()`, so the two
//! halves cannot start disagreeing about what a pixel is.

use super::tests::{camera, points, window};
use super::{
    DYNAMIC_RGBA, EFFECTOR_RGBA, SENSOR_ACTIVE_RGBA, SENSOR_IDLE_RGBA, STATIC_RGBA,
    collider_outline, outline_rgba, outlines,
};
use ph2d_physics_ecs::{BodyKind, ColliderShape, ShapeDesc};

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

/// **A collider offset moves the outline off the sprite centre — and rotates
/// with the body** (W-Offset).
///
/// The offset is the collider's position relative to the sprite (a character's
/// feet below its art), and the outline is the only way the artist SEES it. It
/// is applied in `outlines` (not `collider_outline`), folding the body's signed
/// scale and rotation exactly as the bridge does for the solver — so the
/// wireframe sits where the collider actually is. Mutation-tested: dropping the
/// `+ wox`/`+ woy` from the centre draws the outline on the sprite, and the
/// shifted-centre assertion goes red.
#[test]
fn an_offset_collider_outline_sits_where_the_collider_is() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};

    // The outline's centre, in screen px, for a ball at the origin with the given
    // collider offset and body rotation.
    let centre = |offset: [f32; 2], rotation: f32| {
        let mut sim = ph2d_ecs::SimWorld::new();
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.5 },
                offset,
                ..Collider::default()
            },
            Transform {
                translation: Vec2::new(0.0, 0.0),
                rotation,
                scale: Vec2::new(1.0, 1.0),
                skew_x: 0.0,
                skew_y: 0.0,
            },
        ));
        let drawn = outlines(true, false, &mut sim, &[], &camera(), window());
        assert_eq!(drawn.len(), 1, "expected exactly one outline");
        let pts = points(&drawn[0].0);
        // Bounding-box centre, not the point-MEAN: the ball's outline carries a
        // spoke (centre → +x rim) whose two extra points would skew a mean toward
        // +x. The spoke lives inside the circle's bbox, so the bbox centre is the
        // true collider centre.
        let xs: Vec<f64> = pts.iter().map(|(x, _)| *x).collect();
        let ys: Vec<f64> = pts.iter().map(|(_, y)| *y).collect();
        let bbox_mid = |v: &[f64]| {
            let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (lo + hi) * 0.5
        };
        (bbox_mid(&xs), bbox_mid(&ys))
    };

    // Centred: the outline is at the screen centre (world origin).
    let (cx0, cy0) = centre([0.0, 0.0], 0.0);
    assert!(
        (cx0 - 500.0).abs() < 1e-3 && (cy0 - 500.0).abs() < 1e-3,
        "a centred collider's outline is not at the screen centre: ({cx0}, {cy0})"
    );

    // Offset +1 world unit in x, no rotation: the outline shifts +100 px right.
    let (cx1, cy1) = centre([1.0, 0.0], 0.0);
    assert!(
        (cx1 - 600.0).abs() < 1e-3 && (cy1 - 500.0).abs() < 1e-3,
        "a +1 x collider offset should draw the outline 100 px right of centre, \
         but it is at ({cx1}, {cy1}) — the offset did not reach the overlay"
    );

    // The SAME offset under a 90° body rotation rotates to world +y (up), which is
    // screen y = 400. This is the property that makes a rotated character's
    // foot-box turn with it — the offset is in the body's local frame.
    let (cx2, cy2) = centre([1.0, 0.0], std::f32::consts::FRAC_PI_2);
    assert!(
        (cx2 - 500.0).abs() < 1e-2 && (cy2 - 400.0).abs() < 1e-2,
        "a +1 x offset under a 90° rotation should draw the outline at screen \
         (500, 400) — up — but it is at ({cx2}, {cy2}); the offset is not \
         rotating with the body"
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

/// **A force zone draws an arrow showing which way it blows — and keeps drawing it
/// while the clock runs** (W-Area).
///
/// The zone's outline is already magenta because it is a sensor, and a sensor that
/// merely notices things looks exactly the same as one that pushes them. The arrow is
/// the whole difference, so it is the thing the gate asserts.
///
/// ⚠️ `show_velocity` is passed **false** throughout — that is the "the clock is
/// running" state, where the launch arrow is deliberately hidden because a body's
/// authored velocity stops being true once it moves. A force is a property of the
/// AREA and never stops being true, so it must survive that flag.
#[test]
fn a_force_zone_draws_its_push_even_while_the_clock_runs() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_physics_ecs::{AreaEffector, Collider, RigidBody};

    let zone = |force: [f32; 2]| {
        let mut sim = ph2d_ecs::SimWorld::new();
        sim.world_mut().spawn((
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
            AreaEffector { force },
        ));
        sim
    };

    // A pushing zone: outline + arrow, and the arrow is its own colour.
    let mut pushing = zone([20.0, 0.0]);
    let drawn = outlines(true, false, &mut pushing, &[], &camera(), window());
    assert_eq!(
        drawn.len(),
        2,
        "a force zone should draw its outline AND an arrow, got {} paths",
        drawn.len()
    );
    assert_eq!(
        drawn[1].1, EFFECTOR_RGBA,
        "the push arrow must not wear a colour that already means something else"
    );

    // It points the way it blows: the shaft's first two points run +X on screen.
    let pts = points(&drawn[1].0);
    assert!(
        pts[1].0 > pts[0].0 && (pts[1].1 - pts[0].1).abs() < 1e-6,
        "an arrow for a +X force should point +X on screen, got {pts:?}"
    );

    // A zone that pushes nothing draws no arrow — the outline alone.
    let mut idle = zone([0.0, 0.0]);
    assert_eq!(
        outlines(true, false, &mut idle, &[], &camera(), window()).len(),
        1,
        "a zero force must draw no arrow — an arrow of no length is a dot nobody can read"
    );

    // And the whole thing obeys the toggle, like every other piece of this chrome.
    assert!(outlines(false, false, &mut pushing, &[], &camera(), window()).is_empty());
}
