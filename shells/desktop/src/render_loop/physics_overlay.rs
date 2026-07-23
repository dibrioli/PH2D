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

use super::physics_overlay_contacts::{
    CONTACT_FLASH_RGBA, CONTACT_RGBA, WATERLINE_RGBA, contact_flashes, contact_marks,
    waterline_marks,
};
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

/// The **initial-velocity arrow** — a launch the artist armed but cannot see,
/// because a still body at t=0 is not moving. Yellow, a hue no collider or joint
/// uses. Only linear velocity is drawn (an angular spin has no direction to point).
const VELOCITY_RGBA: [f32; 4] = [0.96, 0.92, 0.26, 0.95]; // LITERAL-COLOR-OK: overlay de velocidade inicial

/// How far ahead the arrow reaches, in seconds of travel: the tip is where the
/// body would be a quarter-second after launch. A time, not a raw length, so the
/// arrow scales with the world like the collider does — faster is longer.
const ARROW_SECONDS: f32 = 0.25;

/// Arrowhead length, screen px — chrome, constant size like the outline stroke.
const ARROW_HEAD_PX: f64 = 9.0; // LITERAL-PX-OK: chrome de overlay

/// An arrow for one **velocity** at one world point, in screen pixels, or `None` if
/// it is (near) zero — a still body draws no arrow.
///
/// Built in screen space on purpose (the module's rule — see the header): the
/// shaft is the world vector projected through the camera, so its length
/// tracks speed and zoom, but the arrowhead is a constant-size screen ornament,
/// so it never balloons the way a world-space stroke width would.
///
/// Two callers: the authored launch, and the force zone below — which converts its
/// newtons into the velocity they would give a reference body, so both arrows are
/// read against the SAME ruler instead of inventing a second one.
fn velocity_arrow(
    cx: f32,
    cy: f32,
    linvel: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    if linvel[0].hypot(linvel[1]) < 1e-3 {
        return None;
    }
    let to_screen = |wx: f32, wy: f32| {
        let (sx, sy) = camera.world_to_screen([wx, wy], window);
        Point::new(f64::from(sx), f64::from(sy))
    };
    let tail = to_screen(cx, cy);
    let tip = to_screen(
        cx + linvel[0] * ARROW_SECONDS,
        cy + linvel[1] * ARROW_SECONDS,
    );
    let (dx, dy) = (tip.x - tail.x, tip.y - tail.y);
    let len = dx.hypot(dy);

    let mut path = BezPath::new();
    path.move_to(tail);
    path.line_to(tip);
    if len > 1e-6 {
        // Head: two barbs from the tip, back along the shaft rotated ±25°.
        let (bx, by) = (-dx / len, -dy / len);
        let (s, c) = (25.0_f64).to_radians().sin_cos();
        for sign in [1.0, -1.0] {
            let (rx, ry) = (bx * c - by * (sign * s), bx * (sign * s) + by * c);
            path.move_to(tip);
            path.line_to(Point::new(
                tip.x + rx * ARROW_HEAD_PX,
                tip.y + ry * ARROW_HEAD_PX,
            ));
        }
    }
    Some(path)
}

/// The force-zone arrow — **orange**, a hue no collider, joint or launch uses.
const EFFECTOR_RGBA: [f32; 4] = [1.0, 0.62, 0.16, 0.95]; // LITERAL-COLOR-OK: overlay de campo de forca

/// The mass a force is drawn AGAINST, in kg.
///
/// A force is not a length, so any arrow for it is a statement about SOME body — that
/// is what "resisted by mass" means, and it is the feature, not a wart. One kilogram
/// is the honest reference: at 1 kg the newtons ARE the acceleration, so the arrow
/// reads directly off the number in the row.
const EFFECTOR_REFERENCE_KG: f32 = 1.0;

/// The arrow for one force zone's push. `None` for a zone that pushes nothing.
///
/// The force is converted into the velocity it would give the reference body in
/// [`ARROW_SECONDS`], and then drawn by the SAME function the launch arrow uses —
/// so the two are read against one ruler. A second length scale would be a second
/// answer to "how long is a strong one".
fn effector_arrow(
    cx: f32,
    cy: f32,
    force: [f32; 2],
    camera: &Camera2d,
    window: WindowSize,
) -> Option<BezPath> {
    let k = ARROW_SECONDS / EFFECTOR_REFERENCE_KG;
    velocity_arrow(cx, cy, [force[0] * k, force[1] * k], camera, window)
}

/// **What to draw, decided once.** Pure: the toggle and the "is there any
/// physics here at all" question are answered here and returned as data, not
/// resolved inside a paint loop. That is the repo's `hit_plan` shape — a
/// refusal that lives in a loop cannot be tested, and an overlay that quietly
/// draws when it was switched off is exactly the kind of thing nobody notices
/// until it is in a screenshot.
/// `show_velocity` adds the initial-velocity arrow to each body that has a
/// launch armed. It is separate from `show` because the arrow is only truthful
/// while the bodies are at their AUTHORED rest (before the sim steps): once a
/// body has moved, its live velocity is no longer the authored one, so the
/// caller passes `false` while the clock is running.
pub(crate) fn outlines(
    show: bool,
    show_velocity: bool,
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
    let mut out = Vec::new();
    for (e, rb, col) in q.iter(world) {
        let Some(t) = ph2d_ecs::world_transform_into(world, e, &mut chain) else {
            continue;
        };
        // The collider offset, placed exactly where the solver puts it. The
        // bridge folds the body's SIGNED scale into the offset (a flip mirrors it)
        // and rapier rotates the result with the body; the overlay does the same
        // so the wireframe sits on the collider, not the sprite centre. Reading
        // `col.offset` any other way here is how the outline and the solver would
        // come to disagree about WHERE the collider is.
        let (ox, oy) = (col.offset[0] * t.scale.x, col.offset[1] * t.scale.y);
        let (sin_r, cos_r) = t.rotation.sin_cos();
        let (wox, woy) = (ox * cos_r - oy * sin_r, ox * sin_r + oy * cos_r);
        // The SAME resolution the bridge does — so the outline is drawn at the
        // size (and shape: circle vs ellipse) the solver actually simulates
        // under this body's world scale.
        let path = collider_outline(
            scaled_shape(col.shape, t.scale),
            t.translation.x + wox,
            t.translation.y + woy,
            t.rotation,
            camera,
            window,
        );
        out.push((
            path,
            outline_rgba(col.is_sensor, triggered.contains(&e), rb.kind),
        ));
        // The armed launch, when the bodies are at rest (so `t` is the authored
        // position the arrow springs from). Absent component = no arrow.
        if show_velocity
            && let Some(iv) = world.get::<ph2d_physics_ecs::InitialVelocity>(e)
            && let Some(arrow) =
                velocity_arrow(t.translation.x, t.translation.y, iv.linvel, camera, window)
        {
            out.push((arrow, VELOCITY_RGBA));
        }
        // The force zone's push — WHICH WAY DOES THIS BLOW, and how hard. A zone is a
        // sensor, so it is already magenta; the arrow is what makes it a *directed*
        // area rather than a region that merely notices things.
        //
        // Unlike the launch above it is drawn whether or not the clock is running: a
        // force is a property of the AREA, authored once, and it does not stop being
        // true because the simulation started (the launch arrow is hidden while
        // playing precisely because it stops being true the moment the body moves).
        if show
            && let Some(a) = world.get::<ph2d_physics_ecs::AreaEffector>(e)
            && let Some(arrow) = effector_arrow(
                t.translation.x + wox,
                t.translation.y + woy,
                a.force,
                camera,
                window,
            )
        {
            out.push((arrow, EFFECTOR_RGBA));
        }
    }
    out
}

/// Paint them. No-op when [`outlines`] returns nothing.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    show: bool,
    show_velocity: bool,
    sim: &mut SimWorld,
    joint_anchors: &[([f32; 2], [f32; 2])],
    contacts: &[ph2d_physics_ecs::BodyContact],
    flashes: &[ph2d_physics_ecs::ContactFlash],
    waterlines: &[([f32; 2], [f32; 2])],
    triggered: &[ph2d_ecs::Entity],
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    use ph2d_vector::{Affine, Brush, Color, Stroke};
    for (path, rgba) in outlines(show, show_velocity, sim, triggered, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(rgba)),
            None,
            &path,
        );
    }
    // A linha d'água ANTES dos corpos: ela é o cenário (onde a superfície está), e o
    // que se lê por cima dela são as coisas que boiam.
    for path in waterline_marks(show, waterlines, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(WATERLINE_RGBA)),
            None,
            &path,
        );
    }
    // Contacts ON TOP of everything: a contact is the smallest mark on screen (a few
    // pixels) and it sits exactly ON the outlines of the two bodies that meet, so any
    // other order buries it under the shapes it is describing.
    for path in contact_marks(show, contacts, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(CONTACT_RGBA)),
            None,
            &path,
        );
    }
    // And the BEGIN-flash on top of the standing marks: it lives a few ticks and it is
    // the visible half of the contact-events channel (`contact_flashes`). Last, so the
    // spark is never buried by the very cross it is announcing.
    for path in contact_flashes(show, flashes, camera, window) {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(Color::new(CONTACT_FLASH_RGBA)),
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

#[cfg(test)]
#[path = "physics_overlay_scene_tests.rs"]
mod scene_tests;
