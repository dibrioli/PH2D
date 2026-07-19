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

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{
    BodyKind, Collider, RigidBody, ShapeDesc, capsule_vertices, ellipse_vertices, scaled_shape,
};
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

/// A **sensor** (trigger) with nothing inside it — magenta, a hue no body kind
/// or the amber joint uses, so a trigger zone never reads as a solid collider.
/// Dim, because an empty trigger is standing information, not an event.
const SENSOR_IDLE_RGBA: [f32; 4] = [0.95, 0.45, 0.90, 0.55]; // LITERAL-COLOR-OK: overlay de collider

/// A sensor with a body inside it — the SAME magenta, bright and opaque. The
/// jump from idle to this is the whole point of a trigger: you see it fire.
const SENSOR_ACTIVE_RGBA: [f32; 4] = [1.0, 0.55, 0.98, 1.0]; // LITERAL-COLOR-OK: overlay de collider

/// The colour of one collider outline: a sensor is magenta (bright when a body
/// is inside it), and any solid collider is coloured by its body kind. A sensor
/// overrides the kind colour on purpose — "is this a trigger?" is the first
/// thing you need to know about it, ahead of who moves it.
fn outline_rgba(is_sensor: bool, triggered: bool, kind: BodyKind) -> [f32; 4] {
    if is_sensor {
        if triggered {
            SENSOR_ACTIVE_RGBA
        } else {
            SENSOR_IDLE_RGBA
        }
    } else {
        match kind {
            BodyKind::Static => STATIC_RGBA,
            BodyKind::Dynamic => DYNAMIC_RGBA,
            BodyKind::Kinematic => KINEMATIC_RGBA,
        }
    }
}

/// The outline of one collider, **in screen pixels**.
///
/// Takes the **resolved** [`ShapeDesc`] — the same value the bridge hands
/// rapier ([`scaled_shape`]) — not the authored `ColliderShape`. So a
/// non-uniformly scaled ball draws as the ELLIPSE it actually simulates, and
/// the wireframe can never describe a size the solver does not use.
///
/// A round collider (ball or ellipse) also gets a **spoke** from centre to
/// rim. Without it a rolling circle is indistinguishable from a still one —
/// the outline is rotationally symmetric, so the very motion the collider
/// exists to produce would be invisible. (Box2D's debug draw carries the same
/// spoke, for the same reason.) The spoke is a second subpath, so it does not
/// close into the rim.
pub(crate) fn collider_outline(
    shape: ShapeDesc,
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

    // A ring of local points (already ordered) closed into a loop, plus the
    // spoke to the +x rim. Shared by the circle and the ellipse so both round
    // shapes draw the same way.
    let ring = |path: &mut BezPath, verts: &[[f32; 2]], spoke_x: f32| {
        for (i, [lx, ly]) in verts.iter().enumerate() {
            let p = place(*lx, *ly);
            if i == 0 {
                path.move_to(p);
            } else {
                path.line_to(p);
            }
        }
        path.close_path();
        path.move_to(place(0.0, 0.0));
        path.line_to(place(spoke_x, 0.0));
    };

    let mut path = BezPath::new();
    match shape {
        ShapeDesc::Ball { radius } => {
            let verts: Vec<[f32; 2]> = (0..CIRCLE_SEGS)
                .map(|i| {
                    let a = f32::from(i as u16) * std::f32::consts::TAU / CIRCLE_SEGS as f32;
                    let (s, c) = a.sin_cos();
                    [c * radius, s * radius]
                })
                .collect();
            ring(&mut path, &verts, radius);
        }
        // The ellipse traces the SAME polygon the collider is built from
        // (`ellipse_vertices`), so the wireframe sits exactly on the convex
        // hull the solver sees rather than on a smoother curve outside it.
        ShapeDesc::Ellipse { rx, ry } => {
            ring(&mut path, &ellipse_vertices(rx, ry), rx);
        }
        // Both capsule forms trace `capsule_vertices` — the SAME function the
        // collider is built from for the stadium, and (with `rx == ry`) the same
        // stadium outline rapier's exact capsule has. So the wireframe never
        // describes an edge the solver does not collide with. The spoke lands on
        // the +x flank, which is straight at `radius` for the whole segment, so
        // it reads as a rotation guide exactly like the circle's.
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => {
            ring(
                &mut path,
                &capsule_vertices(half_height, radius, radius),
                radius,
            );
        }
        ShapeDesc::Stadium {
            half_height,
            rx,
            ry,
        } => {
            ring(&mut path, &capsule_vertices(half_height, rx, ry), rx);
        }
        ShapeDesc::Cuboid { half_x, half_y } => {
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
    triggered: &[ph2d_ecs::Entity],
    camera: &Camera2d,
    window: WindowSize,
) -> Vec<(BezPath, [f32; 4])> {
    if !show {
        return Vec::new();
    }
    let mut q = sim
        .world_mut()
        .query::<(ph2d_ecs::Entity, &RigidBody, &Collider)>();
    let world = sim.world();
    // ⚠️ WORLD, never the raw `Transform`. The outline annotates a SPRITE, and
    // the sprite is drawn from the composed chain — so an outline placed at the
    // entity's LOCAL pose lands a full parent-offset away from the thing it is
    // supposed to be describing. Under a rig at x = -3 every collider drew at
    // x = 0 instead, so all of them piled up in the middle of the scene, far
    // from their art. That shipped, and it is what Enio saw
    // (`docs/Physics/BUGS_physics.md` #2 — the overlay was a SIXTH reader of
    // "where is this body", missed when the bridge's five were converted).
    //
    // One scratch buffer for the whole pass, so the chain walk allocates once
    // rather than once per body.
    let mut chain = Vec::new();
    q.iter(world)
        .filter_map(|(e, rb, col)| {
            let t = ph2d_ecs::world_transform_into(world, e, &mut chain)?;
            // The SAME resolution the bridge does — so the outline is drawn at
            // the size (and shape: circle vs ellipse) the solver actually
            // simulates under this body's world scale.
            let path = collider_outline(
                scaled_shape(col.shape, t.scale),
                t.translation.x,
                t.translation.y,
                t.rotation,
                camera,
                window,
            );
            let rgba = outline_rgba(col.is_sensor, triggered.contains(&e), rb.kind);
            Some((path, rgba))
        })
        .collect()
}

/// Paint them. No-op when [`outlines`] returns nothing.
pub(super) fn draw(
    show: bool,
    sim: &mut SimWorld,
    joint_anchors: &[([f32; 2], [f32; 2])],
    triggered: &[ph2d_ecs::Entity],
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for (path, rgba) in outlines(show, sim, triggered, camera, window) {
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
#[path = "physics_overlay_tests.rs"]
mod tests;
