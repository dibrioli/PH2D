//! **The points on a collider an artist aims at** — [`ShapeDesc::snap_points`],
//! the candidate set a joint anchor snaps to (W-J2).
//!
//! The property these gates defend is not the count: it is that **every point is
//! ON the shape**. Offering a corner on a circle would put the magnet somewhere
//! the body is not, and the artist would place an anchor outside the collider
//! believing they had snapped it to the rim.

use ph2d_physics::ShapeDesc;

fn points(shape: ShapeDesc) -> Vec<[f32; 2]> {
    let mut out = [[0.0f32; 2]; ShapeDesc::MAX_SNAP_POINTS];
    let n = shape.snap_points(&mut out);
    out[..n].to_vec()
}

/// The shape's own ruler: `0` at the centre, `1` on the boundary
/// ([`ShapeDesc::radial_fraction`]). Reusing it rather than re-deriving each
/// shape's edge is the point — a second opinion about where the boundary is is
/// exactly the drift a snap target would hide.
fn fraction(shape: ShapeDesc, p: [f32; 2]) -> f32 {
    shape.radial_fraction(p)
}

const SHAPES: &[ShapeDesc] = &[
    ShapeDesc::Ball { radius: 0.7 },
    ShapeDesc::Cuboid {
        half_x: 0.5,
        half_y: 0.2,
    },
    ShapeDesc::Ellipse { rx: 0.9, ry: 0.3 },
    ShapeDesc::Capsule {
        half_height: 0.4,
        radius: 0.25,
    },
    ShapeDesc::Stadium {
        half_height: 0.4,
        rx: 0.3,
        ry: 0.2,
    },
];

/// **Every snap point is inside the shape or exactly on its boundary** — never
/// outside it.
///
/// Mutation-tested: giving a `Ball` the cuboid's four corners (the "one table for
/// all shapes" shortcut) puts them at fraction `√2 ≈ 1.414` and this goes red.
#[test]
fn every_snap_point_lies_on_the_shape() {
    for &shape in SHAPES {
        for p in points(shape) {
            let t = fraction(shape, p);
            assert!(
                t <= 1.0 + 1e-5,
                "{shape:?} offered {p:?}, which is {t:.4} of the way out — a snap target \
                 outside the collider is a place the body is not"
            );
        }
    }
}

/// **The centre is always offered, and it is always first.** It is the anchor an
/// artist reaches for most (a pin through the middle), and the one point every
/// shape has.
#[test]
fn the_centre_is_always_the_first_candidate() {
    for &shape in SHAPES {
        let pts = points(shape);
        assert_eq!(pts[0], [0.0, 0.0], "{shape:?} must offer its centre first");
    }
}

/// **Each shape offers exactly the points it has.** A cuboid has corners and edge
/// midpoints; a round shape has neither, and inventing them is the failure the
/// first gate catches. The counts are pinned so a shape cannot quietly lose its
/// extremes and keep only the centre — which would pass every other assertion
/// here.
#[test]
fn each_family_offers_its_own_extremes() {
    assert_eq!(points(SHAPES[0]).len(), 5, "ball: centre + 4 rim");
    assert_eq!(points(SHAPES[1]).len(), 9, "cuboid: centre + 4 + 4");
    assert_eq!(points(SHAPES[2]).len(), 5, "ellipse: centre + 4 rim");
    assert_eq!(points(SHAPES[3]).len(), 7, "capsule: centre + 2 caps + 4");
    assert_eq!(points(SHAPES[4]).len(), 7, "stadium: centre + 2 caps + 4");
    for &shape in SHAPES {
        let pts = points(shape);
        let reach = pts[1..]
            .iter()
            .map(|p| (p[0] * p[0] + p[1] * p[1]).sqrt())
            .fold(0.0f32, f32::max);
        assert!(
            reach > 1e-3,
            "{shape:?} collapsed all its candidates onto the centre — the extremes \
             are the whole reason the set exists"
        );
    }
}

/// **A cuboid's corners are its corners** — the same nine points the pivot
/// handle's snap already offers, so two point handles in one editor agree about
/// what a box can be snapped to.
#[test]
fn a_cuboid_offers_the_same_nine_points_the_pivot_handle_does() {
    let pts = points(ShapeDesc::Cuboid {
        half_x: 2.0,
        half_y: 1.0,
    });
    // Same order as `ph2d_editor::pivot_snap_candidates`: centre, TL, TR, BL, BR,
    // T, R, B, L.
    assert_eq!(
        pts,
        vec![
            [0.0, 0.0],
            [-2.0, 1.0],
            [2.0, 1.0],
            [-2.0, -1.0],
            [2.0, -1.0],
            [0.0, 1.0],
            [2.0, 0.0],
            [0.0, -1.0],
            [-2.0, 0.0],
        ]
    );
}

/// **A capsule offers its cap centres.** They are where a limb pivots, they are
/// invisible on the outline, and they are therefore exactly the point an artist
/// cannot hit by eye — which is what a magnet is for.
#[test]
fn a_capsule_offers_its_cap_centres() {
    let pts = points(ShapeDesc::Capsule {
        half_height: 0.4,
        radius: 0.25,
    });
    assert!(
        pts.contains(&[0.0, 0.4]) && pts.contains(&[0.0, -0.4]),
        "the two cap centres must be offered, got {pts:?}"
    );
    // …and the poles, which are on the boundary rather than inside it.
    assert!(pts.contains(&[0.0, 0.65]) && pts.contains(&[0.0, -0.65]));
}

/// **A degenerate shape still answers.** A zero half-extent has no interior, and
/// the honest reply is the centre (with the extremes collapsed onto it) rather
/// than a panic or an empty set the caller has to special-case.
#[test]
fn a_degenerate_shape_answers_with_its_centre() {
    let pts = points(ShapeDesc::Cuboid {
        half_x: 0.0,
        half_y: 0.0,
    });
    assert_eq!(pts.len(), 9);
    assert!(pts.iter().all(|p| *p == [0.0, 0.0]));
}
