//! **Publishing the drawn shapes into the cook** (doc 65) — the shell half of `motion.path`.
//!
//! Every vector path that has a **name** goes into the `Cook`'s external table under that name, as a
//! polyline in **world space**. The node reads it by name (`ctx.external("Track")`), and the memo
//! sees the curve's revision, so dragging the shape re-cooks the set that walks it.
//!
//! ## Three decisions, all of them about where a boundary is
//!
//! **1. The graph gets a POLYLINE, not a Bézier.** Flattening is the shell's job because the shell
//! is the one that already owns the curve — `ph2d-nodegraph` is a leaf substrate with zero deps, and
//! teaching it about `VecPath` would make it the vector module's downstream. The node walks arc-
//! length, and arc-length on a polyline is `sqrt` and addition; on a cubic it is an integral. The
//! cheap thing is also the honest one.
//!
//! **2. The name is the artist's, and it is the WHOLE reference.** A shape with no name is not
//! published — there is nothing to type into the node. Draw it, name it in the Hierarchy, and it is
//! visible to the graph. Rename it and the node's `Shape` field stops matching, which is exactly
//! what renaming a thing you referred to by name means. (An id would have been a second identity to
//! keep in sync with the one the artist can see. There is no picker to build and nowhere for the two
//! to disagree.)
//!
//! **3. Two shapes with the same name is the artist's business, not ours.** The last one in scene
//! order wins, deterministically. Refusing to publish either would punish a typo with silence.
//!
//! The world affine matters: the geometry a path stores is LOCAL and its pose lives on the ECS
//! `Transform` (ADR-0111). A polyline published in local space would put the instances where the
//! shape *was authored*, not where it *is* — and it would look right until the artist moved it.

use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_vec_scene::{VecScene, VecXforms, xform_of};

/// How finely a curve is flattened, in world units. A segment is split until its chord is within
/// this of the true curve — which is what "flatten to tolerance" means everywhere else in this repo
/// (`paint/curve_offset.rs` does the same thing for the same reason).
const FLATTEN_TOL: f64 = 0.01;

/// The deepest a segment subdivides. A cubic needs ~2^5 pieces for the tolerance above at any sane
/// scale, and the cap is what stops a degenerate control net (a cusp, a NaN from a corrupt file)
/// from subdividing until the heap gives out.
const MAX_DEPTH: u32 = 5;

/// Is this cubic flat enough to be its own chord? The distance from each control point to the chord
/// — the standard flatness test, and the one the offset code already uses.
fn flat_enough(p: [[f64; 2]; 4]) -> bool {
    let (dx, dy) = (p[3][0] - p[0][0], p[3][1] - p[0][1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < f64::EPSILON {
        // A degenerate chord (the ends coincide): flat only if the handles are on top of them too,
        // or a loop would be flattened away to a point.
        return [p[1], p[2]]
            .iter()
            .all(|q| (q[0] - p[0][0]).hypot(q[1] - p[0][1]) < FLATTEN_TOL);
    }
    [p[1], p[2]].iter().all(|q| {
        let d = ((q[0] - p[0][0]) * dy - (q[1] - p[0][1]) * dx).abs() / len;
        d < FLATTEN_TOL
    })
}

/// Append the flattened cubic (excluding its first point — the caller already pushed it).
fn flatten(p: [[f64; 2]; 4], depth: u32, out: &mut Vec<[f32; 2]>) {
    if depth >= MAX_DEPTH || flat_enough(p) {
        out.push([p[3][0] as f32, p[3][1] as f32]);
        return;
    }
    // Split at the midpoint (de Casteljau), and recurse on both halves.
    let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let (ab, bc, cd) = (mid(p[0], p[1]), mid(p[1], p[2]), mid(p[2], p[3]));
    let (abc, bcd) = (mid(ab, bc), mid(bc, cd));
    let m = mid(abc, bcd);
    flatten([p[0], ab, abc, m], depth + 1, out);
    flatten([m, bcd, cd, p[3]], depth + 1, out);
}

/// One path, flattened into a **world-space** polyline.
///
/// It walks the **COOKED** geometry (`VecPath::cooked()`, ADR-0121) — the same thing the renderer,
/// the hit-test and the boolean walk. The document keeps the sharp corner and the radius; the world
/// consumes the rounded one, and a node that walked the *authored* verts would put instances on a
/// corner the artist cannot see.
pub(super) fn polyline(
    scene: &VecScene,
    xforms: &VecXforms,
    id: ph2d_vec_scene::VecPathId,
) -> Vec<[f32; 2]> {
    let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    let cooked = path.cooked();
    if cooked.verts.len() < 2 {
        return Vec::new();
    }
    let x = xform_of(xforms, id);
    let w = |p: [f64; 2]| x.apply(p);

    let mut out: Vec<[f32; 2]> = Vec::new();
    let first = w(cooked.verts[0].anchor);
    out.push([first[0] as f32, first[1] as f32]);

    let n = cooked.verts.len();
    let last = if cooked.closed { n } else { n - 1 };
    for i in 0..last {
        let a = &cooked.verts[i];
        let b = &cooked.verts[(i + 1) % n];
        flatten(
            [w(a.anchor), w(a.out_handle), w(b.in_handle), w(b.anchor)],
            0,
            &mut out,
        );
    }
    out
}

/// Publish every **named** shape into the cook, and forget the ones that are gone.
///
/// Called once a frame, before the pump. Republishing is free: the external's revision is a hash of
/// its content, so a shape nobody touched invalidates nothing (`ph2d_nodegraph::external`).
pub(super) fn publish(
    cook: &mut ph2d_nodegraph::cook::Cook,
    sim: &SimWorld,
    scene: &VecScene,
    map: &crate::vec_entities::VecEntityMap,
    xforms: &VecXforms,
) {
    // Clear first: a shape the artist deleted (or un-named) must stop being visible to the graph,
    // and a stale entry would keep a `motion.path` walking a curve that is not on the canvas.
    cook.clear_externals();
    let w = sim.world();
    for (&id, &bits) in map {
        let e = Entity::from_bits(bits);
        let Ok(entity) = w.get_entity(e) else {
            continue;
        };
        let Some(name) = entity.get::<Name>().map(|n| n.0.clone()) else {
            continue; // unnamed: there is nothing for the artist to type into the node
        };
        if name.trim().is_empty() {
            continue;
        }
        let pts = polyline(scene, xforms, id);
        if pts.len() < 2 {
            continue; // not a curve: a single point has no arc to walk
        }
        cook.set_external(name, Stream::new(pts.len()).with("P", Column::Vec2(pts)));
    }
}
