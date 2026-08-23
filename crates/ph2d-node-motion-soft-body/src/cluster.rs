//! **Cluster-based shape matching** — Müller et al. 2005 §4.3, the section of the
//! very paper this node already implements that turns a body which can only
//! translate and rotate into one that can BEND.
//!
//! A single shape match fits ONE best-fit frame `(M, c)` to the whole cloud, so
//! every goal it projects is the rest shape carried rigidly (or, with `stretch`,
//! sheared uniformly). MEASURED (`how_much_can_a_long_body_bend_today`), the
//! spine of a 32×4 body deviates from a straight line by **0,0000** of its own
//! length at every stiffness and with the linear mode fully on: not *stiff*, but
//! structurally unable — which is the plan's *"uma cobra balança como uma placa"*
//! with a number under it.
//!
//! The paper's answer is to cover the body with **overlapping** regions, match
//! each one on its own, and let a particle's goal be the average of the goals
//! from every region containing it. The overlap is the whole mechanism: regions
//! that merely tiled would hinge at their seams, because two frames meeting at a
//! shared edge agree about nothing. A particle that belongs to two frames pulls
//! them into agreement, and a chain of such particles is a body that bends
//! smoothly.
//!
//! ## The partition
//!
//! One knob, `clusters`, says how many fit along the body's LONGER side; the
//! shorter side takes a proportional count so a cluster stays roughly square.
//! That is not tidiness — a cluster is meant to be a REGION of the body, and
//! bands that run the full width of a square jelly would let it bend one way and
//! not the other, which an artist reads as the body being broken rather than
//! soft.

use crate::shape::shape_goals_weighted;

/// Smallest span, in particles, that a cluster may cover on an axis. Below two
/// there is nothing for a frame to be fitted to: a single particle gives
/// `A_pq = 0`, whose polar factor is the identity, so the "cluster" projects the
/// particle onto itself and constrains nothing at all.
const MIN_SPAN: usize = 2;

/// How many clusters each axis is cut into, given the knob and the mesh.
///
/// `clusters` counts along the LONGER axis; the shorter one gets the count that
/// keeps a cluster roughly square, and never fewer than one. Both are then capped
/// so no cluster falls under `MIN_SPAN`, which is what makes the knob safe to
/// drag past the point where the body runs out of particles to divide.
pub(crate) fn counts(rows: usize, cols: usize, clusters: usize) -> (usize, usize) {
    let (long, short) = (rows.max(cols), rows.min(cols));
    let n_long = clusters.clamp(1, (long / MIN_SPAN).max(1));
    // Proportional, rounded, at least one — integer arithmetic so the split is the
    // same number on every machine (this node replays bit-for-bit, HR-5).
    let n_short = ((clusters * short + long / 2) / long).clamp(1, (short / MIN_SPAN).max(1));
    if rows >= cols {
        (n_long, n_short)
    } else {
        (n_short, n_long)
    }
}

/// The half-open span `[lo, hi)` of cluster `j` of `n` along an axis of `len`
/// particles, GROWN by half a band on each side so consecutive clusters overlap.
///
/// The growth is the point. `n` equal bands laid end to end would give every
/// particle exactly one frame and the body would hinge at the seams; extending
/// each band by half its own width puts the particles near a seam inside both
/// neighbours, and their averaged goal is what carries curvature across it.
pub(crate) fn span(j: usize, n: usize, len: usize) -> (usize, usize) {
    let lo = j * len / n;
    let hi = (j + 1) * len / n;
    let pad = ((hi - lo) / 2).max(1);
    (lo.saturating_sub(pad), (hi + pad).min(len))
}

/// The goal for every particle as the AVERAGE over the clusters containing it —
/// the paper's `gᵢ = (1/nᵢ) Σ_c gᵢ^c`.
///
/// Each cluster is matched exactly as the whole body is, on its own sub-mesh:
/// its rest points are re-centred on its OWN rest centroid, because
/// `shape_goals` is written to the paper's `qᵢ = xᵢ⁰ − c₀` and would otherwise
/// read a cluster's offset from the body's centre as a deformation to undo.
///
/// ⚠️ The pressure `scale` is passed through UNCHANGED to every cluster rather
/// than recomputed per region, and that is physics rather than economy: the gas
/// inside a body is at one pressure everywhere. What differs between regions is
/// how much each one has been squashed, and that is already carried by each
/// cluster's own frame.
/// A porta UNIFORME (ver `shape::shape_goals`): a lei com o peso neutro, falada
/// pelos oráculos e por nenhum caminho de produção.
#[cfg(test)]
pub(crate) fn cluster_goals(
    pred: &[[f32; 2]],
    rest: &[[f32; 2]],
    buckets: &[Vec<usize>],
    beta: f32,
    scale: f32,
) -> Vec<[f32; 2]> {
    cluster_goals_weighted(pred, rest, buckets, beta, scale, None)
}

/// A mesma partição, com o **peso por partícula** atravessando para cada região.
///
/// ⚠️ **O peso não pode entrar só no ajuste de cada região — ele tem de entrar na
/// MÉDIA que junta as regiões também.** Uma partícula que não pertence ao corpo
/// aparece em até quatro regiões vizinhas, e sem o peso na costura ela voltaria a
/// arrastar o goal dos vizinhos pela porta de trás.
pub(crate) fn cluster_goals_weighted(
    pred: &[[f32; 2]],
    rest: &[[f32; 2]],
    buckets: &[Vec<usize>],
    beta: f32,
    scale: f32,
    w: Option<&[f32]>,
) -> Vec<[f32; 2]> {
    let n = rest.len();
    let mut sum = vec![[0.0f32; 2]; n];
    let mut hits = vec![0u32; n];

    // Scratch reused across clusters: a 512² body at four clusters a side would
    // otherwise churn sixteen allocations per tick, per particle plane.
    let (mut sub_pred, mut sub_rest) = (Vec::new(), Vec::new());
    let mut sub_w: Vec<f32> = Vec::new();

    for idx in buckets {
        // ⚠️ **Uma região com menos de `MIN_SPAN` membros não se ajusta — ela
        // DILUI.** O ajuste sobre uma partícula é a identidade, que a projecta
        // sobre si mesma; média-la com as regiões reais enfraquece a única
        // restrição que aquela partícula tinha. Sobre a grelha autorada isto
        // nunca dispara (cada banda cobre `MIN_SPAN` índices por eixo, logo
        // quatro membros), e há gate a dizê-lo — a guarda existe para a nuvem,
        // onde uma banda pode calhar quase vazia.
        if idx.len() < MIN_SPAN {
            continue;
        }
        sub_pred.clear();
        sub_rest.clear();
        sub_w.clear();
        let mut centre = [0.0f32; 2];
        let mut total = 0.0f32;
        for &i in idx {
            sub_pred.push(pred[i]);
            sub_rest.push(rest[i]);
            let k = w.map_or(1.0, |w| w[i]);
            sub_w.push(k);
            centre[0] += k * rest[i][0];
            centre[1] += k * rest[i][1];
            total += k;
        }
        // ⚠️ **O centro sai da MESMA divisão de sempre quando não há pesos** —
        // `Σ 1.0` sobre `m` partículas é exactamente `m as f32` —, e o
        // `shape_goals_weighted` passa a fazer a subtração que este laço fazia
        // à mão. Mesma expressão, mesma ordem, mesmos bits.
        if total <= 1e-6 {
            continue; // região sem membro nenhum: nada a ajustar
        }
        centre = [centre[0] / total, centre[1] / total];
        let ws = w.map(|_| sub_w.as_slice());
        for (k, g) in shape_goals_weighted(&sub_pred, &sub_rest, beta, scale, ws, centre)
            .into_iter()
            .enumerate()
        {
            let i = idx[k];
            sum[i][0] += g[0];
            sum[i][1] += g[1];
            hits[i] += 1;
        }
    }

    for i in 0..n {
        // A particle no cluster reached keeps its predicted position, which is the
        // goal that constrains nothing — the partition covers the mesh by
        // construction, so this is the answer to a question that cannot be asked
        // rather than a fallback anyone relies on.
        let h = hits[i] as f32;
        if hits[i] == 0 {
            sum[i] = pred[i];
        } else {
            sum[i] = [sum[i][0] / h, sum[i][1] / h];
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::BodyLayout;
    use crate::shape::rest_shape;
    use crate::shape::shape_goals;

    /// Bend the rest mesh around a circle of `radius`. ⚠️ `radius − q[0]`, not
    /// `+`: the Jacobian of the `+` version has determinant `−r/R`, so it turns the
    /// body INSIDE OUT, and the polar factor is documented to return a proper
    /// rotation always. The first version of this fixture used `+` and every
    /// cluster reported the same residual, because each was equally unable to
    /// mirror — a fixture that contained a reflection instead of a curve, reading
    /// exactly like a feature that does nothing.
    fn arc(rest: &[[f32; 2]], radius: f32) -> Vec<[f32; 2]> {
        rest.iter()
            .map(|q| {
                let a = q[1] / radius;
                let r = radius - q[0];
                [r * a.sin(), r * a.cos() - radius]
            })
            .collect()
    }

    fn rms_to(goals: &[[f32; 2]], target: &[[f32; 2]]) -> f32 {
        (goals
            .iter()
            .zip(target)
            .map(|(g, b)| (g[0] - b[0]).powi(2) + (g[1] - b[1]).powi(2))
            .sum::<f32>()
            / goals.len() as f32)
            .sqrt()
    }

    /// **The headline.** A single frame cannot express a bend at all — its goal is
    /// the rest shape carried rigidly — so this bends a body around a known arc and
    /// asks how far each model's goal lands from it.
    ///
    /// The control is the point: the one-cluster number is the arc's own sagitta,
    /// and it is what the node could do before. MEASURED, 32×4 around 1,2 rad:
    /// 1 → **1,075** · 2 → 0,503 · 4 → 0,135 · 8 → 0,044 · 16 → **0,017**, a
    /// monotone fall of 63× that tracks the `1/n²` a chord-against-arc error has.
    #[test]
    fn clusters_let_the_body_follow_a_curve_that_one_frame_cannot() {
        let (rows, cols) = (32usize, 4usize);
        let rest = rest_shape(rows, cols, 0.7);
        let bent = arc(&rest, (rows as f32 - 1.0) * 0.7 / 1.2);

        let single = rms_to(&shape_goals(&bent, &rest, 0.0, 1.0), &bent);
        assert!(
            single > 0.9,
            "CONTROLE: um frame so NAO consegue seguir o arco, e errou {single}"
        );

        let mut prev = single;
        for n in [2usize, 4, 8, 16] {
            let r = rms_to(
                &cluster_goals(
                    &bent,
                    &rest,
                    &BodyLayout::from_grid(rows, cols, 0.7).buckets(n),
                    0.0,
                    1.0,
                ),
                &bent,
            );
            assert!(
                r < prev,
                "{n} clusters tem de seguir melhor que o anterior: {prev} -> {r}"
            );
            prev = r;
        }
        assert!(prev < single / 20.0, "16 clusters: {single} -> {prev}");
    }

    /// The partition COVERS the mesh and its pieces OVERLAP — the two halves of
    /// what makes a clustered body bend smoothly instead of hinging.
    ///
    /// Coverage is not decoration: a particle no frame reached would keep its
    /// predicted position, which constrains nothing, and a body with such a hole
    /// would tear there. Overlap is the mechanism itself — bands laid end to end
    /// give every particle exactly one frame, and two frames meeting at a seam
    /// agree about nothing.
    #[test]
    fn every_particle_has_a_frame_and_the_seams_have_two() {
        for (rows, cols) in [(32usize, 4usize), (16, 8), (8, 8), (5, 3)] {
            for want in [2usize, 3, 4, 8] {
                let (nr, nc) = counts(rows, cols, want);
                let mut hits = vec![0u32; rows * cols];
                for rj in 0..nr {
                    let (r0, r1) = span(rj, nr, rows);
                    for cj in 0..nc {
                        let (c0, c1) = span(cj, nc, cols);
                        for r in r0..r1 {
                            for c in c0..c1 {
                                hits[r * cols + c] += 1;
                            }
                        }
                    }
                }
                assert!(
                    hits.iter().all(|&h| h >= 1),
                    "{rows}x{cols} @{want} ({nr}x{nc}): particula sem frame nenhum"
                );
                if nr > 1 || nc > 1 {
                    assert!(
                        hits.iter().any(|&h| h >= 2),
                        "{rows}x{cols} @{want}: nenhuma sobreposicao — as bandas so ladrilham"
                    );
                }
            }
        }
    }

    /// A cluster is a REGION, so the split stays roughly square rather than cutting
    /// bands across the whole width — a square jelly banded one way bends one way
    /// and not the other, which reads as broken rather than soft. And no cluster is
    /// ever cut below the two particles a frame needs: under that, `A_pq` is zero,
    /// its polar factor is the identity, and the "cluster" projects each particle
    /// onto itself while constraining nothing.
    #[test]
    fn the_split_keeps_clusters_square_and_never_starves_one() {
        // A long body divides along its LENGTH; its narrow side is left whole.
        assert_eq!(counts(32, 4, 4), (4, 1));
        assert_eq!(counts(4, 32, 4), (1, 4), "e o mesmo corpo deitado");
        // A square one divides both ways, or it could only bend on one axis.
        assert_eq!(counts(16, 16, 4), (4, 4));
        // Asked for more pieces than there are particles, it stops where a frame
        // still has something to fit.
        assert_eq!(counts(32, 4, 1000), (16, 2));
        for (rows, cols) in [(32usize, 4usize), (16, 8), (5, 3), (2, 2)] {
            for want in [1usize, 2, 7, 64, 1000] {
                let (nr, nc) = counts(rows, cols, want);
                assert!(nr >= 1 && nc >= 1);
                for (n, len) in [(nr, rows), (nc, cols)] {
                    for j in 0..n {
                        let (lo, hi) = span(j, n, len);
                        assert!(
                            hi - lo >= MIN_SPAN.min(len),
                            "{rows}x{cols}@{want}: cluster de {} particulas num eixo de {len}",
                            hi - lo
                        );
                    }
                }
            }
        }
    }

    /// One cluster is the whole body, so the clustered path reproduces the single
    /// frame. The product never takes this route — `step` skips straight to
    /// `shape_goals` — but the agreement is what says the re-centring and the
    /// averaging add nothing of their own.
    ///
    /// Near-equality rather than bit-equality, and the difference is the point: the
    /// rest shape is built centred on the origin, so subtracting its own centroid
    /// subtracts a sum of floats that is *almost* zero. That residue is why the
    /// early-out exists rather than being an optimisation.
    #[test]
    fn one_cluster_is_the_body_itself() {
        let (rows, cols) = (9usize, 6usize);
        let rest = rest_shape(rows, cols, 0.7);
        let bent = arc(&rest, (rows as f32 - 1.0) * 0.7 / 0.8);
        let one = cluster_goals(
            &bent,
            &rest,
            &BodyLayout::from_grid(rows, cols, 0.7).buckets(1),
            0.0,
            1.0,
        );
        let plain = shape_goals(&bent, &rest, 0.0, 1.0);
        for (a, b) in one.iter().zip(&plain) {
            assert!(
                (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
                "{a:?} vs {b:?}"
            );
        }
    }

    /// **O PESO ATRAVESSA A PARTIÇÃO TAMBÉM** — e é preciso dizê-lo, porque a
    /// rota agrupada é a SEGUNDA porta do ajuste: ela ganha o peso no ajuste de
    /// cada região **e** na média que junta as regiões.
    ///
    /// ⚠️ Um corpo LONGO com poucas regiões é a fixture: numa região só, a
    /// partícula solta arrastaria o quadro global, e o gate não distinguiria as
    /// duas rotas.
    #[test]
    fn the_weight_crosses_the_partition() {
        let (rows, cols) = (4, 16);
        let rest = crate::shape::rest_shape(rows, cols, 0.7);
        let n = rest.len();
        let mut w = vec![1.0f32; n];
        w[n - 1] = 0.0;
        let base: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 1.05, q[1] * 0.95]).collect();

        let goal_of = |runaway: [f32; 2]| {
            let mut pred = base.clone();
            pred[n - 1] = runaway;
            cluster_goals_weighted(
                &pred,
                &rest,
                &BodyLayout::from_grid(rows, cols, 0.7).buckets(4),
                0.0,
                1.0,
                Some(&w),
            )
        };
        let near = goal_of(base[n - 1]);
        let far = goal_of([120.0, 90.0]);
        let worst = near
            .iter()
            .zip(&far)
            .take(n - 1)
            .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            worst < 1e-4,
            "a partícula solta não pode arrastar região nenhuma; {worst:.6}"
        );

        // CONTROLE: com peso cheio, a mesma fuga move o quadro da região dela.
        let uniform = |runaway: [f32; 2]| {
            let mut pred = base.clone();
            pred[n - 1] = runaway;
            cluster_goals(
                &pred,
                &rest,
                &BodyLayout::from_grid(rows, cols, 0.7).buckets(4),
                0.0,
                1.0,
            )
        };
        let moved = uniform(base[n - 1])
            .iter()
            .zip(&uniform([120.0, 90.0]))
            .take(n - 1)
            .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            moved > 1.0,
            "o controle tem de mover (senão o gate é vácuo); {moved:.4}"
        );
    }
}
