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

/// The SIGNED area enclosed by the mesh's boundary ring, by the shoelace formula.
///
/// This is the grandeza a pressure term defends, and it is deliberately the
/// **boundary**, not the sum of cell areas: a soft body's volume is what its
/// outline encloses, and the ring costs `O(rows + cols)` where a cell sum costs
/// `O(rows · cols)` — the shape match itself is the only linear pass this node is
/// allowed to have (see `MAX_SIDE`, whose 512² cap was measured against exactly
/// one of them).
///
/// SIGNED on purpose. The traversal is top row left→right, right column down,
/// bottom row right→left, left column up — clockwise in this y-up frame, so a
/// healthy body reports a NEGATIVE number and the caller compares the sign
/// against the rest shape's. Taking `abs()` here would report a body turned
/// inside-out as perfectly healthy, which is precisely the state where an
/// area-restoring term would push in the wrong direction.
pub(crate) fn boundary_area(pos: &[[f32; 2]], rows: usize, cols: usize) -> f32 {
    if rows < 2 || cols < 2 || pos.len() < rows * cols {
        return 0.0;
    }
    let at = |r: usize, c: usize| pos[r * cols + c];
    let mut sum = 0.0f32;
    let mut prev = at(0, 0);
    // The ring, once around; `fold` accumulates the cross product of consecutive
    // vertices (the shoelace), and the final edge closes back onto the start.
    let mut edge = |p: [f32; 2], prev: &mut [f32; 2]| {
        sum += prev[0] * p[1] - p[0] * prev[1];
        *prev = p;
    };
    for c in 1..cols {
        edge(at(0, c), &mut prev);
    }
    for r in 1..rows {
        edge(at(r, cols - 1), &mut prev);
    }
    for c in (0..cols - 1).rev() {
        edge(at(rows - 1, c), &mut prev);
    }
    for r in (1..rows - 1).rev() {
        edge(at(r, 0), &mut prev);
    }
    edge(at(0, 0), &mut prev); // close the ring
    sum * 0.5
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

/// How far a single step's pressure term may scale the goal frame, and the band
/// is MEASURED rather than assumed — including the corner where it is REACHED,
/// because a guard whose doc claims it never fires is a guard nobody re-checks.
///
/// The scale asked of a body 8% under its rest area
/// (`is_the_useful_pressure_coupled_to_stiffness`): at the default stiffness it
/// is `1,12` for a pressure of 1 and `1,48` at 4 — nowhere near. At
/// `stiffness = 0,1`, where the `(1−k)/k` factor is nine, the same body asks
/// `1,72 · 2,44 · 3,88`, so the top of the typable range does arrive here. That
/// is the case this exists for: the deficit divided by a small travel is
/// unbounded, and without a ceiling one step could scale the goal by thousands
/// and the body would be gone before the next tick could pull it back.
const MAX_PRESSURE_SCALE: f32 = 4.0;

/// The uniform LINEAR scale that a pressure of `gain` asks of the goal frame, so
/// that a body which has lost volume gets a goal bigger than its rest shape and
/// pushes back out.
///
/// `gain` is the WEIGHT of the correction, not a target volume: `0` asks for
/// nothing (and the caller does not even reach here), `1` asks for exactly the
/// rest area. This differs from Vellum's Balloon, where `pressure` multiplies the
/// TARGET volume and `0` means *collapse to nothing* — a neutral that cannot be
/// the off switch of an opt-in term. The reference's *name* is kept because the
/// mechanism is the reference's mechanism (an internal pressure defending a
/// volume); the reference's *zero* is not, because in this codebase a term that
/// ships off must be byte-identical when off.
///
/// Returns exactly `1.0` — no correction — when the body is inverted or
/// collapsed. That is not defensive padding: on a cloud whose boundary has
/// turned inside-out the signed area flips, and an area-restoring term would
/// read the deficit backwards and drive it FURTHER inside-out.
///
/// ⚠️ **`travel` is what keeps `gain` meaning one thing**, and the factor it
/// produces is DERIVED rather than tuned. The body never reaches its goal — it
/// moves `travel` (the `stiffness`) of the way there — so the linear size it
/// lands on is `(1−k)·L + k·s·L₀`. Setting that equal to `L₀` and solving gives
/// the scale that restores the rest area in exactly one step:
///
/// ```text
///   s* = (1 − (1−k)·u) / k = 1 + (1−k)/k · (1 − u),   u = L/L₀ = √(A/A₀)
/// ```
///
/// so the term is `s = 1 + gain·(s* − 1)`: `gain = 0` asks nothing, `gain = 1`
/// asks for the whole deficit, and both mean the same thing at every stiffness.
///
/// ⚠️ **The `(1−k)` numerator is the part I first got wrong, and the measurement
/// is what said so.** A gain merely DIVIDED by `k` still diverged at the top of
/// the stiffness range — to **12,3×** the rest area at `stiffness = 1` — because
/// at full stiffness the body *becomes* its goal, whose area is already `A₀`:
/// there is nothing left to correct and any push is pure overshoot. The
/// `(1−k)` factor is that fact, and it makes the term vanish there by
/// ARITHMETIC instead of by a special case.
///
/// ⚠️ And the zero-travel guard is CORRECTNESS, not hygiene: at `k = 0` the
/// factor is `+∞`, and a body sitting at exactly its rest area has `1 − u = 0`,
/// so `∞ · 0` is **NaN** — which would reach the goal, trip the node's
/// non-finite guard, and collapse the body onto its pin. The honest answer at
/// zero travel is that the goal is never consulted, so pressure cannot act.
pub(crate) fn pressure_scale(
    pred: &[[f32; 2]],
    rows: usize,
    cols: usize,
    rest_area: f32,
    gain: f32,
    travel: f32,
) -> f32 {
    if travel < EPS {
        return 1.0;
    }
    let area = boundary_area(pred, rows, cols);
    // Same sign = the ring still winds the way the rest shape winds; both
    // magnitudes non-degenerate.
    if area.abs() < EPS || rest_area.abs() < EPS || (area > 0.0) != (rest_area > 0.0) {
        return 1.0;
    }
    // Area scales as the SQUARE of a linear scale, so the deficit the goal has to
    // answer is measured on the linear ratio — the same `det(A)^{1/d}` (d = 2)
    // crossing the paper's own area preservation makes a few lines above. Feeding
    // the area ratio in raw would over-correct by its own square root.
    let u = (area / rest_area).sqrt();
    let deadbeat = (1.0 - travel) / travel * (1.0 - u);
    (1.0 + gain * deadbeat).clamp(1.0 / MAX_PRESSURE_SCALE, MAX_PRESSURE_SCALE)
}

/// The shape-match goal for every particle: `gᵢ = scale·M·qᵢ + c`, the rest shape
/// mapped into the deformed cloud's best-fit frame `M` (rigid + `beta`·linear),
/// then scaled about that frame's own centre by the pressure term. Pure (no
/// state) — the falsifier for the transform math.
///
/// ⚠️ `scale = 1.0` is byte-identical to the goal without pressure, and by
/// ARITHMETIC rather than by a branch: `x * 1.0 == x` exactly in IEEE-754 for
/// every finite `x`. The caller still skips the whole term when the gain is
/// zero, because the boundary shoelace that produces `scale` is work nobody
/// asked for.
///
/// Scaling `M` and scaling the goals about `c` are the same operation, and that
/// is a fact about this rest shape rather than a convenience: `rest_shape` is
/// centred on the origin, so `Σ M·qᵢ = M·Σ qᵢ = 0` and the goal cloud's centroid
/// IS `c`. A rest shape built off-centre would make these two differ, and the
/// pressure would translate the body while inflating it.
pub(crate) fn shape_goals(
    pred: &[[f32; 2]],
    rest: &[[f32; 2]],
    beta: f32,
    scale: f32,
) -> Vec<[f32; 2]> {
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
    let b = best_transform(apq, aqq, beta);
    let m = [b[0] * scale, b[1] * scale, b[2] * scale, b[3] * scale];
    rest.iter()
        .map(|q| {
            [
                m[0] * q[0] + m[1] * q[1] + c[0],
                m[2] * q[0] + m[3] * q[1] + c[1],
            ]
        })
        .collect()
}

/// The goal projection **exactly as it shipped before the pressure term** — the
/// body of `shape_goals` from `HEAD`, verbatim, under `cfg(test)` so it is a
/// frozen ORACLE and never a second door a caller could wander into (the
/// `serial_side` / `warp_axis` precedent). Its whole job is to make *pressure
/// off is byte-identical* a measured fact rather than an argument about
/// IEEE-754.
#[cfg(test)]
fn shape_goals_as_it_shipped(pred: &[[f32; 2]], rest: &[[f32; 2]], beta: f32) -> Vec<[f32; 2]> {
    let n = pred.len();
    if n == 0 {
        return Vec::new();
    }
    let mut c = [0.0f32; 2];
    for p in pred {
        c[0] += p[0];
        c[1] += p[1];
    }
    c = [c[0] / n as f32, c[1] / n as f32];
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
        let goals = shape_goals(&posed, &rest, 0.0, 1.0);
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
        let goals = shape_goals(&deformed, &rest, 0.0, 1.0);
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

        let rigid = shape_goals(&deformed, &rest, 0.0, 1.0);
        let linear = shape_goals(&deformed, &rest, 1.0, 1.0);
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

    /// The ring encloses the MESH's area, and it encloses it exactly. A rest grid
    /// is a rectangle of `(cols−1)·(rows−1)·spacing²`, which the shoelace has to
    /// reproduce to the float — and the SIGN is asserted alongside it, because the
    /// pressure term compares its sign against this one to tell a healthy body from
    /// one turned inside-out. A traversal bug that skipped or doubled a corner would
    /// land somewhere plausible; the closed form does not admit "plausible".
    #[test]
    fn the_ring_encloses_the_meshes_own_area() {
        for (rows, cols, sp) in [
            (2usize, 2usize, 1.0f32),
            (3, 7, 0.5),
            (8, 8, 0.7),
            (5, 2, 2.0),
        ] {
            let rest = rest_shape(rows, cols, sp);
            let a = boundary_area(&rest, rows, cols);
            let want = (cols as f32 - 1.0) * (rows as f32 - 1.0) * sp * sp;
            assert!(
                (a.abs() - want).abs() < 1e-4,
                "{rows}x{cols}@{sp}: |{a}| deveria ser {want}"
            );
            assert!(a < 0.0, "o anel de repouso e HORARIO neste frame y-up: {a}");
        }
        // Turn the body inside out (mirror x) and the sign flips — which is the
        // fact the pressure term's guard is built on.
        let rest = rest_shape(4, 4, 0.7);
        let flipped: Vec<[f32; 2]> = rest.iter().map(|q| [-q[0], q[1]]).collect();
        assert!(
            boundary_area(&flipped, 4, 4) > 0.0,
            "espelhado inverte o sinal"
        );
    }

    /// **Every boundary particle is on the ring, and no interior one is** — the two
    /// halves of what `boundary_area` claims to be, asserted by MOVING each particle
    /// in turn and asking whether the number noticed.
    ///
    /// ⚠️ This exists because the closed-form gate above could not see a traversal
    /// that SKIPS a vertex: a rest mesh is a rectangle, so every edge particle is
    /// collinear with its neighbours and dropping one leaves the enclosed area
    /// exactly where it was. The gate was green over a ring with a hole in it. What
    /// cannot be faked is influence — a vertex the walk never visits cannot move the
    /// answer, however it is nudged.
    ///
    /// The second half is not symmetry either: *the boundary, not the sum of cells*
    /// is the decision that makes this `O(rows + cols)` instead of `O(rows · cols)`,
    /// and it is what lets the term ride inside a node whose 512² cap was measured
    /// against exactly one linear pass.
    #[test]
    fn the_ring_is_the_boundary_and_the_whole_boundary() {
        let (rows, cols) = (5usize, 6usize);
        let rest = rest_shape(rows, cols, 0.7);
        let base = boundary_area(&rest, rows, cols);
        // ⚠️ TWO directions, not one. A vertex's contribution to the shoelace
        // changes by `d · perp(next − prev)`, which is ZERO when the nudge runs
        // ALONG the edge through it — and the ring's own corners have diagonal
        // neighbours, so a single diagonal nudge reports the corner as
        // uninfluential and the gate accuses a healthy walk. Any vertex with two
        // distinct neighbours answers to at least one axis.
        for r in 0..rows {
            for c in 0..cols {
                let moved_by = |d: [f32; 2]| {
                    let mut m = rest.clone();
                    m[r * cols + c] = [rest[r * cols + c][0] + d[0], rest[r * cols + c][1] + d[1]];
                    (boundary_area(&m, rows, cols) - base).abs()
                };
                let felt = moved_by([1.5, 0.0]).max(moved_by([0.0, 1.5]));
                let on_ring = r == 0 || c == 0 || r == rows - 1 || c == cols - 1;
                if on_ring {
                    assert!(
                        felt > 1e-3,
                        "({r},{c}) esta na borda e o anel nao a visitou"
                    );
                } else {
                    assert!(felt < 1e-6, "({r},{c}) e INTERIOR e mexeu na area: {felt}");
                }
            }
        }
    }

    /// **The headline, as an ORACLE rather than as the formula.** A pressure of 1
    /// is *restore the rest volume in one step*, so this squashes a body by a known
    /// amount, asks for the scale, performs the step the node performs — land at
    /// `pred + stiffness·(goal − pred)` — and then MEASURES the area it arrived at.
    /// Nothing here knows `(1−k)/k` or the square root; it only knows what the
    /// answer has to be.
    ///
    /// That is what makes it catch three different mistakes at once: dropping the
    /// √ (the correction is then the area deficit, not the linear one, and it
    /// overshoots by its own square root), dropping the `(1−k)` (it overshoots
    /// harder the stiffer the body, to 12× at `stiffness = 1`), and inverting the
    /// direction (it drives the deficit the wrong way).
    ///
    /// ⚠️ **"One step" has a reach, and this gate is where I found its edge.** A
    /// body 25% over its rest area at `stiffness = 0,15` would need a goal scaled
    /// by **−0,42** to come back in one step — negative, which is to say the goal
    /// mirrored through its own centre. Moving 15% of the way somewhere simply
    /// cannot shrink you by 20%. So the exact claim is made only where the term
    /// was not clamped, and the test asks the RESULT whether it was clamped rather
    /// than re-deriving the condition from the law it is testing.
    #[test]
    fn a_pressure_of_one_restores_the_volume_in_a_single_step() {
        let (rows, cols) = (6usize, 6usize);
        let rest = rest_shape(rows, cols, 0.7);
        let a0 = boundary_area(&rest, rows, cols);
        let (lo, hi) = (1.0 / MAX_PRESSURE_SCALE, MAX_PRESSURE_SCALE);
        let mut exact = 0usize;
        for k in [0.15f32, 0.4, 0.7, 0.95] {
            for u in [0.80f32, 0.93, 1.06, 1.25] {
                let pred: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * u, q[1] * u]).collect();
                let s = pressure_scale(&pred, rows, cols, a0, 1.0, k);
                let goals = shape_goals(&pred, &rest, 0.0, s);
                let landed: Vec<[f32; 2]> = pred
                    .iter()
                    .zip(&goals)
                    .map(|(p, g)| [p[0] + (g[0] - p[0]) * k, p[1] + (g[1] - p[1]) * k])
                    .collect();
                let r = boundary_area(&landed, rows, cols) / a0;
                let before = u * u; // the area ratio it started at

                // ALWAYS: it moves toward the rest area, and never past it. This half
                // holds even where the correction is out of reach, and it is the half
                // that catches a sign inversion.
                assert!(
                    (r - 1.0).abs() <= (before - 1.0).abs() + 1e-4,
                    "k={k} u={u}: {before} -> {r} afastou-se do repouso"
                );
                assert!(
                    (r - 1.0) * (before - 1.0) >= -1e-4,
                    "k={k} u={u}: {before} -> {r} passou do repouso para o outro lado"
                );

                // WHERE NOTHING WAS CLAMPED: exactly one step, to the float. The test
                // reads the clamp off the answer instead of recomputing when it bites.
                if s > lo + 1e-4 && s < hi - 1e-4 {
                    exact += 1;
                    assert!(
                        (r - 1.0).abs() < 1e-3,
                        "k={k} u={u}: nada foi limitado (escala {s}) e pousou em {r}"
                    );
                }
            }
        }
        // The fixture has to CONTAIN the exact case, or the paragraph above is a
        // claim about an empty set.
        assert!(exact >= 12, "so {exact} celulas exerceram o passo exato");
    }

    /// The three states where the term must ask for NOTHING, and none of them is
    /// padding: a body already at its rest area (asking for anything would make the
    /// volume the one thing a resting body cannot leave alone), a body whose ring
    /// has turned inside-out (the deficit reads backwards there and the correction
    /// would drive it further in), and zero travel — where the factor is `+∞`, the
    /// deficit of a resting body is `0`, and `∞ · 0` is **NaN**, which would reach
    /// the goal and trip the node's own non-finite guard into collapsing the body
    /// onto its pin.
    #[test]
    fn the_term_asks_for_nothing_where_it_has_nothing_to_say() {
        let (rows, cols) = (5usize, 5usize);
        let rest = rest_shape(rows, cols, 0.7);
        let a0 = boundary_area(&rest, rows, cols);

        assert_eq!(
            pressure_scale(&rest, rows, cols, a0, 1.0, 0.4),
            1.0,
            "corpo na area de repouso"
        );
        assert_eq!(
            pressure_scale(&rest, rows, cols, a0, 0.0, 0.4),
            1.0,
            "ganho zero"
        );

        let squashed: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 0.8, q[1] * 0.8]).collect();
        assert_eq!(
            pressure_scale(&squashed, rows, cols, a0, 1.0, 0.0),
            1.0,
            "travel zero: o goal nunca e consultado, entao a pressao nao pode agir"
        );
        let s = pressure_scale(&squashed, rows, cols, a0, 1.0, 1e-9);
        assert!(s.is_finite(), "travel ~0 nunca produz NaN/inf: {s}");

        let inside_out: Vec<[f32; 2]> = rest.iter().map(|q| [-q[0] * 0.8, q[1] * 0.8]).collect();
        assert_eq!(
            pressure_scale(&inside_out, rows, cols, a0, 1.0, 0.4),
            1.0,
            "corpo do avesso: recuar, nunca empurrar mais fundo"
        );
    }

    /// **Off is off, and it is off to the BIT** — against the projection as it
    /// shipped, not against an argument. Adversarial clouds on purpose: a rotation
    /// (so `M` is not the identity and the multiply lands on every entry), the
    /// linear mode (so `beta` is live), and coordinates far from the origin (where
    /// a lost bit would show first).
    #[test]
    fn the_goal_without_pressure_is_the_goal_that_shipped() {
        let rest = rest_shape(5, 4, 0.7);
        let (c, s) = (0.79864_f32, 0.60181_f32);
        for beta in [0.0f32, 0.35, 1.0] {
            for offset in [0.0f32, 137.5] {
                let pred: Vec<[f32; 2]> = rest
                    .iter()
                    .map(|q| {
                        [
                            (c * q[0] - s * q[1]) * 1.21 + offset,
                            (s * q[0] + c * q[1]) * 0.83 - offset,
                        ]
                    })
                    .collect();
                let now = shape_goals(&pred, &rest, beta, 1.0);
                let then = shape_goals_as_it_shipped(&pred, &rest, beta);
                assert_eq!(now, then, "beta={beta} offset={offset}");
            }
        }
    }

    /// Area preservation: a big UNIFORM scale (which the linear map would happily
    /// follow) is normalised away, so the linear goal keeps the rest AREA rather than
    /// ballooning — exactly the paper's `A / det(A)^{1/d}` guard.
    #[test]
    fn linear_mode_preserves_area_under_uniform_scale() {
        let rest = rest_shape(4, 4, 0.7);
        let blown: Vec<[f32; 2]> = rest.iter().map(|q| [3.0 * q[0], 3.0 * q[1]]).collect();
        let linear = shape_goals(&blown, &rest, 1.0, 1.0);
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
