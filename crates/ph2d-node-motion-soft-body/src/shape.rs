//! Shape-matching geometry for `motion.soft_body` (split from `lib.rs` for the LOC
//! cap): the rest mesh, the 2D polar decomposition, and the best-fit transform.
//!
//! Faithful to Müller et al. 2005 (§3.3–3.4), verified against the authors' own
//! equations:
//! - rest/current centroids are the (mass-weighted) centres of mass; here the mesh
//!   is uniform, so `wᵢ = mᵢ = 1` and the weights drop out (the paper's `wᵢ = mᵢ`).
//! - `A_pq = Σ pᵢ qᵢᵀ` with `pᵢ = xᵢ − c`, `qᵢ = xᵢ⁰ − c₀`; the **rigid** rotation
//!   `R` is the polar factor of `A_pq` (`R = A_pq S⁻¹`, `S = √(A_pqᵀA_pq)`), which in
//!   2D has the closed form `(cos,sin) ∝ (A₀₀+A₁₁, A₁₀−A₀₁)` — one `sqrt`, no trig.
//! - the **linear** mode blends in `A = A_pq A_qq⁻¹` (the least-squares linear map),
//!   area-preserved by `A / √det(A)` (the paper's `A / det(A)^{1/d}`, d = 2), giving
//!   `M = β·A + (1−β)·R` for squash-and-stretch (`β = 0` ⇒ pure rigid).
//!
//! The goal is `gᵢ = M·qᵢ + c`. Transcendental-free (HR-5: `sqrt` only).

/// Below this a magnitude / determinant is treated as zero (skip the divide).
const EPS: f32 = 1e-6;

/// The rest shape: a `rows×cols` grid centred on the origin (so its centroid is 0,
/// which the shape match assumes). Row 0 is the TOP (max y). Row-major, so
/// `0..cols` is the top row.
pub(crate) fn rest_shape(rows: usize, cols: usize, spacing: f32) -> Vec<[f32; 2]> {
    let (w, h) = ((cols as f32 - 1.0) * spacing, (rows as f32 - 1.0) * spacing);
    let mut q = Vec::with_capacity(rows * cols);
    for r in 0..rows {
        for c in 0..cols {
            q.push([c as f32 * spacing - w * 0.5, h * 0.5 - r as f32 * spacing]);
        }
    }
    q
}

/// The best-fit rotation `(cos, sin)` — the 2D polar decomposition of `A_pq`, closed
/// form (HR-5: one `sqrt`). Identity when the cloud is degenerate. `R` is ALWAYS a
/// proper rotation (det +1) by construction, so no reflection-flip is needed (the
/// 3D case's `det(R)` check is moot in 2D).
pub(crate) fn polar_rotation(apq: [f32; 4]) -> (f32, f32) {
    // A_pq = [[a00,a01],[a10,a11]]; R minimising ‖A − R‖ has cos ∝ a00+a11,
    // sin ∝ a10−a01 (the analytic polar factor).
    let (c, s) = (apq[0] + apq[3], apq[2] - apq[1]);
    let mag = (c * c + s * s).sqrt();
    if mag < EPS {
        (1.0, 0.0)
    } else {
        (c / mag, s / mag)
    }
}

fn det2(m: [f32; 4]) -> f32 {
    m[0] * m[3] - m[1] * m[2]
}

/// `a · b` for 2×2 row-major matrices.
fn matmul2(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
    ]
}

/// `m⁻¹` (2×2), or `None` if singular.
fn inv2(m: [f32; 4]) -> Option<[f32; 4]> {
    let det = det2(m);
    if det.abs() < EPS {
        return None;
    }
    let inv = 1.0 / det;
    Some([m[3] * inv, -m[1] * inv, -m[2] * inv, m[0] * inv])
}

/// The best-fit transform `M` mapping rest → deformed: the rigid rotation `R` when
/// `beta = 0`, else the Müller 2005 blend `β·A + (1−β)·R` with `A = A_pq·A_qq⁻¹`
/// area-preserved (so a linear/quadratic deformation can squash & stretch while the
/// rigid part anchors it). Falls back to `R` if the linear map is degenerate.
fn best_transform(apq: [f32; 4], aqq: [f32; 4], beta: f32) -> [f32; 4] {
    let (cos, sin) = polar_rotation(apq);
    let r = [cos, -sin, sin, cos];
    if beta < EPS {
        return r;
    }
    let Some(aqq_inv) = inv2(aqq) else {
        return r;
    };
    let mut a = matmul2(apq, aqq_inv);
    // Area preservation: divide by √det so det(A) → 1 (the paper's det(A)^{1/d}, d=2).
    let det = det2(a);
    if det <= EPS {
        return r; // inverted / collapsed → keep the rigid part
    }
    let s = det.sqrt();
    a = [a[0] / s, a[1] / s, a[2] / s, a[3] / s];
    [
        beta * a[0] + (1.0 - beta) * r[0],
        beta * a[1] + (1.0 - beta) * r[1],
        beta * a[2] + (1.0 - beta) * r[2],
        beta * a[3] + (1.0 - beta) * r[3],
    ]
}

/// The shape-match goal for every particle: `gᵢ = M·qᵢ + c`, the rest shape mapped
/// into the deformed cloud's best-fit frame `M` (rigid + `beta`·linear). Pure (no
/// state) — the falsifier for the transform math.
pub(crate) fn shape_goals(pred: &[[f32; 2]], rest: &[[f32; 2]], beta: f32) -> Vec<[f32; 2]> {
    let n = pred.len();
    if n == 0 {
        return Vec::new();
    }
    // Centroid of the deformed cloud (the rest centroid is 0 by construction).
    let mut c = [0.0f32; 2];
    for p in pred {
        c[0] += p[0];
        c[1] += p[1];
    }
    c = [c[0] / n as f32, c[1] / n as f32];
    // A_pq = Σ (predᵢ − c)(qᵢ)ᵀ, and A_qq = Σ qᵢ qᵢᵀ (needed only for the linear mode).
    let mut apq = [0.0f32; 4];
    let mut aqq = [0.0f32; 4];
    for i in 0..n {
        let (dx, dy) = (pred[i][0] - c[0], pred[i][1] - c[1]);
        let (qx, qy) = (rest[i][0], rest[i][1]);
        apq[0] += dx * qx;
        apq[1] += dx * qy;
        apq[2] += dy * qx;
        apq[3] += dy * qy;
        aqq[0] += qx * qx;
        aqq[1] += qx * qy;
        aqq[2] += qy * qx;
        aqq[3] += qy * qy;
    }
    let m = best_transform(apq, aqq, beta);
    rest.iter()
        .map(|q| {
            [
                m[0] * q[0] + m[1] * q[1] + c[0],
                m[2] * q[0] + m[3] * q[1] + c[1],
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The polar decomposition is CORRECT: a rest shape placed as a pure rigid pose
    /// (rotation + translation) shape-matches to ITSELF — every goal equals its
    /// predicted position, so a rigid body feels no spurious deformation. FALSIFIED
    /// by a wrong `(cos, sin)`: the goals would twist away from the rigid pose.
    #[test]
    fn shape_match_is_rigid_invariant() {
        let rest = rest_shape(3, 3, 0.7);
        // A known rigid pose: rotate every rest point by ~37° and translate.
        let (c, s) = (0.79864_f32, 0.60181_f32); // cos/sin 37°
        let posed: Vec<[f32; 2]> = rest
            .iter()
            .map(|q| [c * q[0] - s * q[1] + 5.0, s * q[0] + c * q[1] - 2.0])
            .collect();
        let goals = shape_goals(&posed, &rest, 0.0);
        for (g, p) in goals.iter().zip(&posed) {
            assert!(
                (g[0] - p[0]).abs() < 1e-3 && (g[1] - p[1]).abs() < 1e-3,
                "rigid pose is its own goal: {g:?} vs {p:?}"
            );
        }
    }

    /// Rigid recovery: yank one corner far out and the RIGID match pulls its goal back
    /// toward the rest shape. FALSIFIED by no recovery (the goal would stay at the yank).
    #[test]
    fn rigid_mode_recovers_the_shape() {
        let rest = rest_shape(3, 3, 0.7);
        let mut deformed = rest.clone();
        deformed[8] = [10.0, 10.0]; // yank the last corner far away
        let goals = shape_goals(&deformed, &rest, 0.0);
        let sq = |a: [f32; 2], b: [f32; 2]| {
            let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
            dx * dx + dy * dy
        };
        assert!(
            sq(goals[8], rest[8]) < sq(deformed[8], rest[8]) * 0.25,
            "the rigid goal snaps back toward rest"
        );
    }

    /// The LINEAR mode (`beta`) lets the body squash & stretch: under an area-preserving
    /// shear (stretch X, compress Y), the rigid match (`beta = 0`) has no rotation and
    /// snaps every goal back to the REST shape, while the linear match (`beta = 1`)
    /// FOLLOWS the stretch — the goal tracks the deformed cloud. This is the Müller 2005
    /// linear-deformation richness that pure rigid shape matching lacks.
    #[test]
    fn linear_mode_follows_an_area_preserving_stretch() {
        let rest = rest_shape(4, 4, 0.7);
        // Area-preserving diagonal stretch (det = 1.5 · 1/1.5 = 1).
        let (sx, sy) = (1.5f32, 1.0 / 1.5);
        let deformed: Vec<[f32; 2]> = rest.iter().map(|q| [sx * q[0], sy * q[1]]).collect();

        let rigid = shape_goals(&deformed, &rest, 0.0);
        let linear = shape_goals(&deformed, &rest, 1.0);
        // A non-central corner, where the stretch bites hardest.
        let i = 0;
        let sq = |a: [f32; 2], b: [f32; 2]| {
            let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
            dx * dx + dy * dy
        };
        // Rigid snaps back to rest (far from the deformed cloud)…
        assert!(
            sq(rigid[i], rest[i]) < 1e-4,
            "rigid ignores the stretch → goal = rest: {:?}",
            rigid[i]
        );
        // …linear follows the stretch (goal ≈ the deformed position).
        assert!(
            sq(linear[i], deformed[i]) < 1e-3,
            "linear follows the stretch → goal ≈ deformed: {:?} vs {:?}",
            linear[i],
            deformed[i]
        );
    }

    /// Area preservation: a big UNIFORM scale (which the linear map would happily
    /// follow) is normalised away, so the linear goal keeps the rest AREA rather than
    /// ballooning — exactly the paper's `A / det(A)^{1/d}` guard.
    #[test]
    fn linear_mode_preserves_area_under_uniform_scale() {
        let rest = rest_shape(4, 4, 0.7);
        let blown: Vec<[f32; 2]> = rest.iter().map(|q| [3.0 * q[0], 3.0 * q[1]]).collect();
        let linear = shape_goals(&blown, &rest, 1.0);
        // The uniform ×3 is removed → the goal spread matches rest, not ×3.
        let spread = |v: &[[f32; 2]]| v.iter().map(|p| p[0].abs()).fold(0.0, f32::max);
        assert!(
            (spread(&linear) - spread(&rest)).abs() < 0.05,
            "area-preserved: goal spread {} ≈ rest {}",
            spread(&linear),
            spread(&rest)
        );
    }
}
