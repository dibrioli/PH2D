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

/// A malha de repouso autorada — vive no [`crate::layout`], que é quem sabe a
/// forma do corpo desde que ela deixou de ser sempre uma grelha.
#[cfg(test)]
pub(crate) use crate::layout::grid_rest as rest_shape;

/// A área do anel de uma malha AUTORADA — o atalho dos gates e das sondas,
/// cujas fixturas SÃO grelhas. ⚠️ `#[cfg(test)]` de propósito: o caminho de
/// produção recebe o anel do [`crate::layout::BodyLayout`], que é quem sabe se
/// o corpo é uma malha ou uma nuvem — um atalho alcançável dali seria a
/// segunda resposta à espera de alguém a chamar.
#[cfg(test)]
pub(crate) fn boundary_area(pos: &[[f32; 2]], rows: usize, cols: usize) -> f32 {
    ring_area(pos, &crate::layout::grid_ring(rows, cols))
}

/// The SIGNED area enclosed by the body's boundary ring, by the shoelace formula.
///
/// This is the grandeza a pressure term defends, and it is deliberately the
/// **boundary**, not the sum of cell areas: a soft body's volume is what its
/// outline encloses, and the ring costs `O(rows + cols)` where a cell sum costs
/// `O(rows · cols)` — the shape match itself is the only linear pass this node is
/// allowed to have (see `MAX_SIDE`, whose 512² cap was measured against exactly
/// one of them).
///
/// SIGNED on purpose. The ring winds top row left→right, right column down,
/// bottom row right→left, left column up — clockwise in this y-up frame, so a
/// healthy body reports a NEGATIVE number and the caller compares the sign
/// against the rest shape's. Taking `abs()` here would report a body turned
/// inside-out as perfectly healthy, which is precisely the state where an
/// area-restoring term would push in the wrong direction.
///
/// ⚠️ **O anel entra por argumento, e é isso que separa a LEI do FORNECEDOR.**
/// Quem o produz — o passeio da grelha ou o casco de uma nuvem — vive no
/// [`crate::layout`]; aqui só se soma. A ordem dos índices é load-bearing: a
/// soma é `f32`, e o mesmo anel noutra ordem dá outro número.
pub(crate) fn ring_area(pos: &[[f32; 2]], ring: &[usize]) -> f32 {
    if ring.len() < 3 || ring.iter().any(|&k| k >= pos.len()) {
        return 0.0;
    }
    let mut sum = 0.0f32;
    let mut prev = pos[ring[0]];
    // Once around; each step accumulates the cross product of consecutive
    // vertices (the shoelace), and the final edge closes back onto the start.
    let mut edge = |p: [f32; 2], prev: &mut [f32; 2]| {
        sum += prev[0] * p[1] - p[0] * prev[1];
        *prev = p;
    };
    for &k in &ring[1..] {
        edge(pos[k], &mut prev);
    }
    edge(pos[ring[0]], &mut prev); // close the ring
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
    ring: &[usize],
    rest_area: f32,
    gain: f32,
    travel: f32,
) -> f32 {
    if travel < EPS {
        return 1.0;
    }
    let area = ring_area(pred, ring);
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
/// A porta UNIFORME — a lei com os argumentos neutros.
///
/// ⚠️ `#[cfg(test)]` de propósito: desde que o peso por partícula existe, **nenhum
/// caminho de produção passa por aqui** (o `simulate` chama a lei ponderada com
/// `None` quando a coluna está ausente), e um `pub(crate)` sem chamador é a
/// segunda resposta à espera de alguém a chamar. Ela sobrevive como a voz que os
/// oráculos falam, ao lado do `shape_goals_as_it_shipped`.
#[cfg(test)]
pub(crate) fn shape_goals(
    pred: &[[f32; 2]],
    rest: &[[f32; 2]],
    beta: f32,
    scale: f32,
) -> Vec<[f32; 2]> {
    shape_goals_weighted(pred, rest, beta, scale, None, [0.0, 0.0])
}

/// O centroide de repouso PONDERADO — `[0, 0]` quando não há pesos, e é isso que
/// faz a rota uniforme reduzir **ao bit**.
///
/// ⚠️ **O zero literal não é uma aproximação do valor verdadeiro, e a medição é
/// que decide isto:** o `shape_goals` sempre assumiu *"o centroide de repouso é 0
/// por construção"* e nunca o subtraiu — medido (`is_the_rest_centroid_exactly_zero`),
/// a soma das coordenadas de uma malha 8×8 de espaçamento 0,7 vale **−1,192e-7**,
/// não zero. Calcular o centroide de verdade no caso uniforme seria *mais
/// correto* e moveria todo corpo já autorado por ~1e-7 a cada tique; devolver o
/// zero literal mantém `q = rest − 0.0`, que é a **identidade** em IEEE-754 para
/// todo finito (inclusive `−0,0`), e deixa a lei nova alcançar só quem tem peso.
pub(crate) fn weighted_rest_centroid(rest: &[[f32; 2]], w: Option<&[f32]>) -> [f32; 2] {
    let Some(w) = w else {
        return [0.0, 0.0];
    };
    let mut sum = [0.0f32; 2];
    let mut total = 0.0f32;
    for (q, wi) in rest.iter().zip(w) {
        sum[0] += wi * q[0];
        sum[1] += wi * q[1];
        total += wi;
    }
    if total <= EPS {
        // Nenhuma partícula pertence ao corpo: não há forma a ajustar, e um
        // centroide dividido por zero envenenaria o goal inteiro.
        return [0.0, 0.0];
    }
    [sum[0] / total, sum[1] / total]
}

/// A lei do ajuste, com o **peso por partícula** de Müller 2005 (`wᵢ = mᵢ`).
///
/// ⚠️ **A versão ponderada é a lei, e a uniforme é ela com os argumentos
/// neutros** — não há duas implementações do mesmo ajuste a divergir. E o neutro
/// é exato por ARITMÉTICA, não por um ramo: `1.0 * x` é `x`, `x − 0.0` é `x`,
/// `Σ 1.0` sobre `n` partículas é exactamente `n as f32` (para `n < 2²⁴`), e a
/// divisão que sai disso é a MESMA que o corpo de sempre fazia.
///
/// ⚠️ **O peso entra no CENTROIDE também, e não só nas somas** — sem isso o
/// `q` continuaria medido a partir do centro geométrico enquanto o `c` mede o
/// centro de massa, e os dois desalinhados deslocam o corpo inteiro pelo próprio
/// centroide de repouso ponderado, em repouso, sem ninguém tocar em nada.
pub(crate) fn shape_goals_weighted(
    pred: &[[f32; 2]],
    rest: &[[f32; 2]],
    beta: f32,
    scale: f32,
    w: Option<&[f32]>,
    c0: [f32; 2],
) -> Vec<[f32; 2]> {
    let n = pred.len();
    if n == 0 {
        return Vec::new();
    }
    let wi = |i: usize| w.map_or(1.0, |w| w[i]);
    // Centroid of the deformed cloud, weighted by how much each particle belongs.
    let mut c = [0.0f32; 2];
    let mut total = 0.0f32;
    for (i, p) in pred.iter().enumerate() {
        let k = wi(i);
        c[0] += k * p[0];
        c[1] += k * p[1];
        total += k;
    }
    if total <= EPS {
        // Um corpo sem membro nenhum não tem forma a que voltar: o goal é a
        // própria predição, que é o que "esta partícula é livre" significa.
        return pred.to_vec();
    }
    c = [c[0] / total, c[1] / total];
    // A_pq = Σ wᵢ (predᵢ − c)(qᵢ)ᵀ, and A_qq = Σ wᵢ qᵢ qᵢᵀ (linear mode only).
    let mut apq = [0.0f32; 4];
    let mut aqq = [0.0f32; 4];
    for i in 0..n {
        let k = wi(i);
        let (dx, dy) = (k * (pred[i][0] - c[0]), k * (pred[i][1] - c[1]));
        let (qx, qy) = (rest[i][0] - c0[0], rest[i][1] - c0[1]);
        apq[0] += dx * qx;
        apq[1] += dx * qy;
        apq[2] += dy * qx;
        apq[3] += dy * qy;
        aqq[0] += k * qx * qx;
        aqq[1] += k * qx * qy;
        aqq[2] += k * qy * qx;
        aqq[3] += k * qy * qy;
    }
    let b = best_transform(apq, aqq, beta);
    let m = [b[0] * scale, b[1] * scale, b[2] * scale, b[3] * scale];
    rest.iter()
        .map(|r| {
            let q = [r[0] - c0[0], r[1] - c0[1]];
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
#[path = "shape_tests.rs"]
mod tests;
