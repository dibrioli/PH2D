//! **A shape as a polyline** — moved verbatim from `ph2d-app-motion` (`motion_bridge_shapes.rs`) so the
//! Follow Path constraint walks the same flattening as `motion.path` (W2, one owner).

use crate::{VecPathId, VecScene, Xform};

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

/// ⭐ **One path, flattened into a polyline under the affine `x`** (its world affine ⇒ world space) — the
/// one door of `motion.path` and of the Follow Path constraint.
///
/// It walks the **COOKED** geometry (`VecPath::cooked()`, ADR-0121) — the same thing the renderer,
/// the hit-test and the boolean walk. The document keeps the sharp corner and the radius; the world
/// consumes the rounded one, and a node that walked the *authored* verts would put instances on a
/// corner the artist cannot see.
pub fn world_polyline(scene: &VecScene, id: VecPathId, x: &Xform) -> Vec<[f32; 2]> {
    let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    let cooked = path.cooked();
    if cooked.verts.len() < 2 {
        return Vec::new();
    }
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
