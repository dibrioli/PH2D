//! Os gates da LEI do ajuste de forma — o `shape.rs` cruzou o teto de LOC com o
//! peso por partícula, e o corte é por RESPONSABILIDADE: o pai responde *qual é
//! a lei*, este arquivo responde *como sabemos que ela é essa*.
//!
//! ⚠️ Segue **FILHO** (`#[path]` dentro de `mod tests`), então o `use super::*`
//! alcança os privados do pai — um irmão de topo não alcançaria o `det2`, o
//! `inv2` nem o oráculo congelado `shape_goals_as_it_shipped`.

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

/// **UMA PARTÍCULA DE PESO ZERO NÃO DEFINE A FORMA** — e o oráculo é a
/// INVARIÂNCIA, não um número: leve-a para onde quiser e o goal de quem
/// pertence ao corpo não se mexe.
///
/// ⚠️ **É este gate que separa *o peso alcança o AJUSTE* de *o peso alcança só
/// o PUXÃO*.** Com os pesos ignorados no ajuste, arrastar a partícula solta
/// arrasta o quadro inteiro atrás dela — que é exactamente o corpo a derreter
/// para acompanhar quem se soltou dele.
#[test]
fn a_particle_of_zero_weight_does_not_define_the_shape() {
    let rest = rest_shape(4, 4, 0.7);
    let n = rest.len();
    let mut w = vec![1.0f32; n];
    w[0] = 0.0; // a partícula solta
    let base: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 1.1, q[1] * 0.9]).collect();

    let goal_of = |runaway: [f32; 2]| {
        let mut pred = base.clone();
        pred[0] = runaway;
        let c0 = weighted_rest_centroid(&rest, Some(&w));
        shape_goals_weighted(&pred, &rest, 0.35, 1.0, Some(&w), c0)
    };
    let near = goal_of(base[0]);
    let far = goal_of([90.0, -70.0]);
    let worst = near
        .iter()
        .zip(&far)
        .skip(1) // a própria solta: o goal dela existe, só não a puxa
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-5,
        "o goal dos membros tem de ser cego a onde a partícula solta está; pior desvio {worst:.6}"
    );
}

/// **O CONTROLE do gate acima:** com peso CHEIO a mesma partícula arrasta o
/// quadro. Sem esta metade, *"o goal não se mexe"* seria satisfeito por um
/// ajuste que ignorasse a nuvem inteira.
#[test]
fn a_particle_of_full_weight_does_drag_the_shape() {
    let rest = rest_shape(4, 4, 0.7);
    let base: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 1.1, q[1] * 0.9]).collect();
    let goal_of = |runaway: [f32; 2]| {
        let mut pred = base.clone();
        pred[0] = runaway;
        shape_goals(&pred, &rest, 0.35, 1.0)
    };
    let near = goal_of(base[0]);
    let far = goal_of([90.0, -70.0]);
    let worst = near
        .iter()
        .zip(&far)
        .skip(1)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        worst > 1.0,
        "com peso cheio a fuga TEM de mover o quadro (senão o gate irmão é vácuo); {worst:.4}"
    );
}

/// **O CENTROIDE DE REPOUSO É PONDERADO, e um corpo em repouso fica onde
/// está.** Sem a subtração de `c₀` o `q` mede a partir do centro GEOMÉTRICO
/// enquanto o `c` mede o centro de MASSA, e os dois desalinhados deslocam o
/// corpo inteiro — parado, sem ninguém tocar em nada.
#[test]
fn a_weighted_body_at_rest_stays_where_it_is() {
    let rest = rest_shape(4, 4, 0.7);
    let n = rest.len();
    // Pesos deliberadamente enviesados para UM lado: o centroide ponderado
    // sai longe do geométrico, que é onde o defeito é grande.
    let w: Vec<f32> = (0..n).map(|i| if i % 4 == 0 { 1.0 } else { 0.1 }).collect();
    let c0 = weighted_rest_centroid(&rest, Some(&w));
    assert!(
        c0[0].abs() > 0.05,
        "a fixture tem de ter centroide ponderado LONGE do geométrico, senão nada prova; {c0:?}"
    );
    let goals = shape_goals_weighted(&rest, &rest, 0.0, 1.0, Some(&w), c0);
    let worst = goals
        .iter()
        .zip(&rest)
        .map(|(g, r)| (g[0] - r[0]).abs().max((g[1] - r[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-5,
        "um corpo ponderado em repouso tem de ter goal == repouso; pior desvio {worst:.6}"
    );
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
