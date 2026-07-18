//! **The collider outline** — what shape is this thing, *physically*?
//!
//! A sprite is a textured QUAD. A collider is a circle, or a box, or (later)
//! a capsule, and it is **invisible**. So a ball collider under a square
//! sprite looks exactly like a box collider under a square sprite, right up
//! until it rolls — which is precisely the report that produced this module
//! (*"os colliders parecem redondos mas os desenhos são box"*, Enio
//! 2026-07-18).
//!
//! That mismatch is not a bug in the demo scene: it is the **normal case**.
//! In a real project the art is whatever the artist drew and the collider is
//! a shape they chose, and the two are only related by intent. Every physics
//! editor answers this the same way — Unity, Godot and Box2D's own debug draw
//! all paint the collider as a wireframe on top of the art — so that is what
//! this does. Making the *sprite* round would only fix the demo.
//!
//! ## Screen space, deliberately
//!
//! The geometry is built in world units and every POINT is pushed through the
//! camera, but the resulting path is in screen pixels and is stroked under
//! `Affine::IDENTITY`. In Vello the stroke transform **multiplies the
//! width**, so handing the world→screen affine to `stroke` turns a 1.5 px
//! outline into `1.5 × pixels-per-world-unit` — hundreds of pixels of paint.
//! That is a scar, not a hypothesis: it is what happened to the Flip
//! selection halo (smoke, 2026-07-13), and `flip_cursor` has always drawn
//! this way for the same reason.
//!
//! ## Free when there is no physics
//!
//! Nothing is drawn for a scene with no bodies, so a painter or vector user
//! never sees physics chrome and never pays for it. The toggle (`B`) exists
//! for the case where the outlines are in the way; W2 moves it into the
//! physics panel, reading this same flag.

use ph2d_ecs::{SimWorld, Transform};
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point, VectorScene};

use super::physics_overlay_joints::{JOINT_RGBA, joint_marks};

/// Outline thickness, in screen px. Thinner than the selection halo (2 px):
/// a collider is standing information, not a thing you just did.
const OUTLINE_PX: f64 = 1.5; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// How many segments approximate a circle. 32 is smooth at any zoom a body
/// is readable at, and the path is rebuilt per frame anyway.
pub(super) const CIRCLE_SEGS: u32 = 32;

/// Static bodies — the scenery. Cool green, the Unity/Box2D convention.
const STATIC_RGBA: [f32; 4] = [0.36, 0.85, 0.52, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// Dynamic bodies — the things that move. Cyan, chosen to stay clear of the
/// amber selection halo: a selected dynamic body must still read as selected.
const DYNAMIC_RGBA: [f32; 4] = [0.35, 0.80, 1.0, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// Kinematic bodies — moved by the scene, not by the solver (a baked body, an
/// animated platform). Violet: it must not read as either neighbour, because
/// the whole question the overlay answers here is *who is driving this*. Box2D
/// and Unity both give the kind its own colour for the same reason.
const KINEMATIC_RGBA: [f32; 4] = [0.72, 0.55, 1.0, 0.85]; // LITERAL-COLOR-OK: overlay de collider

/// The outline of one collider, **in screen pixels**.
///
/// A ball also gets a **spoke** from centre to rim. Without it a rolling
/// circle is indistinguishable from a still one — the outline is rotationally
/// symmetric, so the very motion the collider exists to produce would be
/// invisible. (Box2D's debug draw carries the same spoke, for the same
/// reason.) The spoke is a second subpath, so it does not close into the rim.
pub(crate) fn collider_outline(
    shape: ColliderShape,
    x: f32,
    y: f32,
    rotation: f32,
    camera: &Camera2d,
    window: WindowSize,
) -> BezPath {
    let to_screen = |wx: f32, wy: f32| {
        let (sx, sy) = camera.world_to_screen([wx, wy], window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    // Body rotation, applied in WORLD space before the camera — so the
    // outline turns with the body instead of merely following it.
    let (sin_r, cos_r) = rotation.sin_cos();
    let place =
        |lx: f32, ly: f32| to_screen(x + lx * cos_r - ly * sin_r, y + lx * sin_r + ly * cos_r);

    let mut path = BezPath::new();
    match shape {
        ColliderShape::Ball { radius } => {
            for i in 0..CIRCLE_SEGS {
                let a = f32::from(i as u16) * std::f32::consts::TAU / CIRCLE_SEGS as f32;
                let (s, c) = a.sin_cos();
                let p = place(c * radius, s * radius);
                if i == 0 {
                    path.move_to(p);
                } else {
                    path.line_to(p);
                }
            }
            path.close_path();
            // The spoke: centre → rim along the body's own x axis.
            path.move_to(place(0.0, 0.0));
            path.line_to(place(radius, 0.0));
        }
        ColliderShape::Cuboid { half_x, half_y } => {
            path.move_to(place(-half_x, -half_y));
            path.line_to(place(half_x, -half_y));
            path.line_to(place(half_x, half_y));
            path.line_to(place(-half_x, half_y));
            path.close_path();
        }
    }
    path
}

/// **What to draw, decided once.** Pure: the toggle and the "is there any
/// physics here at all" question are answered here and returned as data, not
/// resolved inside a paint loop. That is the repo's `hit_plan` shape — a
/// refusal that lives in a loop cannot be tested, and an overlay that quietly
/// draws when it was switched off is exactly the kind of thing nobody notices
/// until it is in a screenshot.
pub(crate) fn outlines(
    show: bool,
    sim: &mut SimWorld,
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    if !show {
        return Vec::new();
    }
    let mut q = sim
        .world_mut()
        .query::<(&RigidBody, &Collider, &Transform)>();
    let world = sim.world();
    q.iter(world)
        .map(|(rb, col, t)| {
            let path = collider_outline(
                col.shape,
                t.translation.x,
                t.translation.y,
                t.rotation,
                camera,
                window,
            );
            let rgba = match rb.kind {
                BodyKind::Static => STATIC_RGBA,
                BodyKind::Dynamic => DYNAMIC_RGBA,
                BodyKind::Kinematic => KINEMATIC_RGBA,
            };
            (path, rgba)
        })
        .collect()
}

/// Paint them. No-op when [`outlines`] returns nothing.
pub(super) fn draw(
    show: bool,
    sim: &mut SimWorld,
    joint_anchors: &[([f32; 2], [f32; 2])],
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for (path, rgba) in outlines(show, sim, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
    // Joints ON TOP of the colliders: the link runs between two bodies and
    // would otherwise be hidden by whichever outline was drawn last.
    for path in joint_marks(show, joint_anchors, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(JOINT_RGBA)),
            None,
            &path,
        );
    }
}

#[cfg(test)]
mod tests {
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

    use super::{DYNAMIC_RGBA, STATIC_RGBA, collider_outline, outlines};
    use ph2d_host::WindowSize;
    use ph2d_physics_ecs::ColliderShape;
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
            ColliderShape::Ball { radius: 1.0 },
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
            ColliderShape::Cuboid {
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
        let square = ColliderShape::Cuboid {
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
        let ball = ColliderShape::Ball { radius: 0.5 };
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
        let ball = ColliderShape::Ball { radius: 1.0 };
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
            outlines(false, &mut sim, &camera(), window()).is_empty(),
            "the overlay drew while switched off"
        );
        assert_eq!(
            outlines(true, &mut sim, &camera(), window()).len(),
            2,
            "the overlay drew nothing while switched on"
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
            outlines(true, &mut sim, &camera(), window()).is_empty(),
            "physics chrome leaked into a scene with no bodies"
        );
    }

    /// Scenery and movers are told apart at a glance. Without this the floor
    /// and the falling body draw identically, and "which of these can move?"
    /// — the first question you ask a physics scene — has no answer on screen.
    #[test]
    fn static_and_dynamic_bodies_are_drawn_in_different_colours() {
        let mut sim = physics_scene();
        let out = outlines(true, &mut sim, &camera(), window());
        let colours: Vec<[f32; 4]> = out.iter().map(|(_, c)| *c).collect();
        assert!(colours.contains(&STATIC_RGBA), "no static body was drawn");
        assert!(colours.contains(&DYNAMIC_RGBA), "no dynamic body was drawn");
        assert_ne!(
            STATIC_RGBA, DYNAMIC_RGBA,
            "static and dynamic share a colour — the distinction is invisible"
        );
    }
}
