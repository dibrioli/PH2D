//! **Que FORMA um collider desenha** — a geometria do contorno, separada do
//! passe que a pinta.
//!
//! Irmão do `physics_overlay.rs` pelo cap de 600 LOC da shell, cortado por
//! responsabilidade: *que figura é esta* × *o que o passe desenha e de que cor*.
//! A ida ao teto foi a W-Compound, quando o passe passou a perguntar **de quem**
//! cada forma é (um corpo composto tem mais de uma).

use ph2d_host::WindowSize;
use ph2d_physics_ecs::{ShapeDesc, capsule_vertices, ellipse_vertices};
use ph2d_render::Camera2d;
use ph2d_vector::{BezPath, Point};

use super::physics_overlay::CIRCLE_SEGS;

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
