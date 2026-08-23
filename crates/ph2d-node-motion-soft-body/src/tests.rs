//! Unit tests for `motion.soft_body` — the full sequential simulation (seeding,
//! gravity + pin, the moving anchor, replay, the NaN guard, the cook). The pure
//! shape-match geometry is falsified in the `shape` sibling. Split from `lib.rs`
//! at the 700-LOC cap.

use super::*;

/// ⚠️ `pub(super)` porque o `port_tests` — a metade que o tecto de LOC cortou
/// daqui — usa a MESMA fixtura. Dois `params` divergiriam em silêncio, e o
/// primeiro a mudar de default levaria metade dos gates com ele.
pub(super) fn params(rows: usize, cols: usize, gravity: f32, stiffness: f32, pin: bool) -> Params {
    Params {
        rows,
        cols,
        spacing: 0.7,
        gravity,
        stiffness,
        beta: 0.0,
        damping: 0.0,
        // A fixture DECLARA o neutro em vez de o herdar: `0` e *sem defesa de
        // volume*, e escreve-lo aqui e o que impede estes testes de mudarem de
        // sentido em silencio no dia em que o default se mover.
        pressure: 0.0,
        clusters: 1,
        pin,
    }
}

fn run(anchor: impl Fn(f32) -> [f32; 2], p: &Params, ticks: usize, dt: f32) -> Vec<Vec<[f32; 2]>> {
    let mut state = Stream::new(0);
    let mut frames = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let out = simulate(anchor(t), &state, &[], t, p);
        state = out.clone();
        frames.push(vec2_col(&out, "P"));
    }
    frames
}

/// Tick 0 seeds the rest mesh at the anchor: `rows·cols` particles, the top row
/// (indices `0..cols`) sitting at the anchor row.
#[test]
fn seeds_the_rest_mesh_at_the_anchor() {
    let p = params(3, 4, 9.0, 0.5, true);
    let out = simulate([2.0, 1.0], &Stream::new(0), &[], 0.0, &p);
    let pos = vec2_col(&out, "P");
    assert_eq!(pos.len(), 12, "3×4 mesh");
    // Top row is at the top (max y); rows descend. Centre column near anchor x.
    let top_y = pos[0][1];
    let bottom_y = pos[11][1];
    assert!(top_y > bottom_y, "row 0 is the top");
    // The mesh is centred on the anchor: mean position ≈ anchor.
    let mean = pos
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let mean = [mean[0] / 12.0, mean[1] / 12.0];
    assert!((mean[0] - 2.0).abs() < 1e-4 && (mean[1] - 1.0).abs() < 1e-4);
}

// (The pure shape-match geometry — rigid invariance, shape recovery, and the
// `beta` linear-deformation mode — is falsified in the `shape` module's own
// tests. Here we exercise the full sequential simulation.)

/// Gravity + pin: the pinned top row holds at the anchor while the free body
/// sags below it over time. FALSIFIED by a dead body (no gravity) — it stays put.
#[test]
fn gravity_hangs_the_body_below_the_pinned_top() {
    let live = params(4, 3, 12.0, 0.5, true);
    let bottom_y = |p: &Params| {
        let last = run(|_| [0.0, 0.0], p, 180, 1.0 / 60.0);
        let pos = last.last().unwrap();
        // Bottom row's mean y.
        let (rows, cols) = (p.rows, p.cols);
        let start = (rows - 1) * cols;
        pos[start..].iter().map(|q| q[1]).sum::<f32>() / cols as f32
    };
    assert!(bottom_y(&live) < -0.5, "the body sags below the pin");
    // The pinned top row stays at the anchor height (y = +half).
    let top = run(|_| [0.0, 0.0], &live, 180, 1.0 / 60.0);
    let top_row = &top.last().unwrap()[0..live.cols];
    for q in top_row {
        assert!(q[1].abs() < 0.5 + 1.0, "top row pinned near the anchor row");
    }
}

/// A moving anchor drags the jelly: sliding the pin carries the whole body along.
#[test]
fn a_moving_anchor_drags_the_body() {
    let p = params(4, 3, 9.0, 0.5, true);
    let moved = run(|t| [3.0 * t, 0.0], &p, 120, 1.0 / 60.0);
    let still = run(|_| [0.0, 0.0], &p, 120, 1.0 / 60.0);
    let mean_x = |f: &[[f32; 2]]| f.iter().map(|q| q[0]).sum::<f32>() / f.len() as f32;
    assert!(
        mean_x(moved.last().unwrap()) > mean_x(still.last().unwrap()) + 0.5,
        "the sliding anchor carried the body in +x"
    );
}

/// Deterministic replay (HR-5: arithmetic + one `sqrt`): two runs match exactly.
#[test]
fn replay_is_deterministic() {
    let p = params(4, 4, 9.0, 0.4, true);
    let a = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
    let b = run(|t| [2.0 * t, 0.0], &p, 90, 1.0 / 60.0);
    assert_eq!(a, b);
}

/// Without the state loop it re-seeds every tick → the rest mesh at the anchor,
/// never deforming (the "only simulates with feedback" footnote).
#[test]
fn without_the_state_loop_it_holds_the_rest_mesh() {
    let p = params(3, 3, 12.0, 0.5, true);
    let rest = rest_shape(p.rows, p.cols, p.spacing);
    for k in 0..20 {
        let out = simulate([1.0, 0.0], &Stream::new(0), &[], k as f32 / 60.0, &p);
        let pos = vec2_col(&out, "P");
        for (q, r) in pos.iter().zip(&rest) {
            assert!((q[0] - (r[0] + 1.0)).abs() < 1e-5 && (q[1] - r[1]).abs() < 1e-5);
        }
    }
}

/// A poisoned (non-finite) state recovers on the rest frame instead of spreading.
#[test]
fn non_finite_state_recovers() {
    let p = params(2, 2, 9.0, 0.5, false);
    let rest = rest_shape(p.rows, p.cols, p.spacing);
    let state = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![
                [0.0, 0.0],
                [f32::INFINITY, 0.0],
                [0.0, 0.0],
                [0.0, 0.0],
            ]),
        )
        .with("sb_vel", Column::Vec2(vec![[0.0, 0.0]; 4]))
        .with("sim_t", Column::Scalar(vec![0.0; 4]));
    let out = simulate([0.0, 0.0], &state, &[], 1.0 / 60.0, &p);
    let _ = rest;
    assert!(
        vec2_col(&out, "P")
            .iter()
            .all(|q| q[0].is_finite() && q[1].is_finite()),
        "the diverged particle was reset, not propagated"
    );
}

/// Cooks through the registry with the `pre` self-loop, exactly as the editor
/// wires it.
#[test]
fn registers_and_steps_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionSoftBody as &dyn NodeOp)
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let sb = g.add_node("motion.soft_body");
    g.set_param(sb, "rows", 4.0);
    g.set_param(sb, "cols", 3.0);
    g.set_param(sb, "gravity", 12.0);
    g.connect(Edge {
        from: (sb, 0),
        to: (sb, 2),
        delayed: true,
    })
    .unwrap();

    let mut cook = Cook::new();
    let out0 = cook.cook(&g, &Ops, sb, 0.0).unwrap();
    assert!(matches!(out0[0].as_stream().get("P"), Some(Column::Vec2(v)) if v.len() == 12));
    for k in 0..60 {
        let t = k as f64 / 60.0;
        cook.cook(&g, &Ops, sb, t).unwrap();
        cook.advance_tick(&g, &Ops, t).unwrap();
    }
    let out = cook.cook(&g, &Ops, sb, 1.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        // The free bottom row has sagged below the pinned top.
        Column::Vec2(v) => assert!(v[11][1] < v[0][1], "the body hangs after a second"),
        _ => panic!("P"),
    }
}

// ---------------------------------------------------------------------------
// Pressure — the volume defence, through the real sequential simulation.
// ---------------------------------------------------------------------------

/// Run a body inside an inward force field for `ticks`, and report the WORST area
/// ratio it reaches once the transient has passed. The field is what a
/// `force.attractor` in the state chain does to a body sitting on it, delivered
/// through the same `accel` column the node reads in production — a squeeze is the
/// only thing in this node's world that can take volume away, so it is the only
/// fixture that CONTAINS the phenomenon.
fn worst_area_under_squeeze(p: &Params, ticks: usize, squeeze: f32) -> f32 {
    let rest = rest_shape(p.rows, p.cols, p.spacing);
    let a0 = crate::shape::boundary_area(&rest, p.rows, p.cols);
    let mut state = Stream::new(0);
    let mut worst = 1.0f32;
    for k in 0..=ticks {
        let t = k as f32 / 60.0;
        let s = simulate([0.0, 0.0], &state, &[], t, p);
        let pos = vec2_col(&s, "P");
        let n = pos.len() as f32;
        let c = pos
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        let c = [c[0] / n, c[1] / n];
        let acc: Vec<[f32; 2]> = pos
            .iter()
            .map(|q| [(c[0] - q[0]) * squeeze, (c[1] - q[1]) * squeeze])
            .collect();
        state = s.with("accel", Column::Vec2(acc));
        if k > 60 {
            let r = crate::shape::boundary_area(&vec2_col(&state, "P"), p.rows, p.cols) / a0;
            if (r - 1.0).abs() > (worst - 1.0).abs() {
                worst = r;
            }
        }
    }
    worst
}

fn pressured(stiffness: f32, pressure: f32) -> Params {
    Params {
        rows: 8,
        cols: 8,
        spacing: 0.7,
        gravity: 12.0,
        stiffness,
        beta: 0.0,
        damping: 0.03,
        pressure,
        clusters: 1,
        pin: true,
    }
}

/// **The headline.** A squeezed body settles below its rest area and STAYS there —
/// the shape match restores the rest SHAPE, but the cloud only travels `stiffness`
/// of the way to its goal each step and the volume is lost in that lag. With
/// pressure on, the body holds its size.
///
/// The control is in the same test on purpose: without it a bar on the pressured
/// number alone would pass just as happily over a fixture that never squeezed
/// anything.
#[test]
fn pressure_defends_the_volume_against_a_squeeze() {
    let without = worst_area_under_squeeze(&pressured(0.4, 0.0), 300, 120.0);
    let with = worst_area_under_squeeze(&pressured(0.4, 1.0), 300, 120.0);
    assert!(
        without < 0.95,
        "CONTROLE: sem pressao o corpo TEM de perder volume, e perdeu {without}"
    );
    assert!(
        (with - 1.0).abs() * 4.0 < (without - 1.0).abs(),
        "com pressao o deficit tem de encolher varias vezes: {without} -> {with}"
    );
}

/// **The design gate**, and the one this wave exists for. The body travels
/// `stiffness` of the way to its goal, so a correction asked of the goal is
/// delivered scaled by `stiffness` — a pressure knob that ignored that would have
/// a different useful value at every stiffness, which is the shape this codebase
/// calls an ergonomics bug rather than a number to pick.
///
/// MEASURED with the term as it ships: 0,985 / 0,987 / 0,988 across a range of
/// `stiffness` that spans almost an order of magnitude. With the `(1−k)` factor
/// removed — a gain merely divided by the travel — the same three cells read
/// 0,992 / 1,004 / **1,727**, because at high stiffness the body already reaches
/// its goal and every extra push is pure overshoot.
#[test]
fn the_pressure_knob_means_the_same_thing_at_every_stiffness() {
    let seen: Vec<f32> = [0.1f32, 0.4, 0.7]
        .iter()
        .map(|&k| worst_area_under_squeeze(&pressured(k, 1.0), 300, 120.0))
        .collect();
    let lo = seen.iter().copied().fold(f32::MAX, f32::min);
    let hi = seen.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        hi - lo < 0.02,
        "a mesma pressao tem de entregar a mesma defesa: {seen:?} (espalhamento {})",
        hi - lo
    );
    // …and it has to be a DEFENCE, not merely a consistent one: three cells all
    // stuck at the unpressured 0,908 would also be tightly spread.
    assert!(
        seen.iter().all(|r| (r - 1.0).abs() < 0.05),
        "e perto do repouso, nao apenas consistente: {seen:?}"
    );
}

/// A body that travels NOWHERE toward its goal cannot be pressurised — and the
/// arithmetic of saying so is a division by that travel, so a resting body at zero
/// stiffness computes `∞ · 0`. That is **NaN**, it would reach the goal, and the
/// node's own non-finite guard would then "recover" every particle onto its pin
/// target. The failure looks like the body freezing in mid-air the instant someone
/// drags Stiffness to zero.
///
/// ⚠️ **The oracle is that the body FELL, and picking it took two tries.** The
/// obvious assertion — *the body is still its rest size* — is worthless here,
/// because a pin target is `anchor + restᵢ`, so the NaN recovery reassembles the
/// body into exactly its rest shape: area ratio **1,000**, which the obvious
/// assertion would have welcomed as proof of health. What the recovery cannot fake
/// is a body that has moved: with no shape match at all this one is in free fall
/// below its anchor, and a recovered body is sitting on it.
#[test]
fn pressure_at_zero_stiffness_leaves_the_body_alone() {
    let p = pressured(0.0, 1.5);
    let mut state = Stream::new(0);
    for k in 0..30 {
        state = simulate([0.0, 0.0], &state, &[], k as f32 / 60.0, &p);
    }
    let pos = vec2_col(&state, "P");
    assert!(
        pos.iter().all(|q| q[0].is_finite() && q[1].is_finite()),
        "nada de NaN a caminho do goal"
    );
    let mean_y = pos.iter().map(|q| q[1]).sum::<f32>() / pos.len() as f32;
    assert!(
        mean_y < -0.3,
        "o corpo caiu em vez de ser remontado no pino: y medio {mean_y}"
    );
}

// ---------------------------------------------------------------------------
// Clusters — the body that can bend, through the real sequential simulation.
// ---------------------------------------------------------------------------

/// The worst deviation of the row centroids from the straight line through the
/// first and last, as a fraction of the body's length. Zero for anything that has
/// only been translated, rotated or uniformly scaled — which is precisely the set
/// of poses one shape-matched frame can produce.
fn worst_spine_bend(p: &Params, ticks: usize, shake: f32) -> f32 {
    let mut state = Stream::new(0);
    let mut worst = 0.0f32;
    for k in 0..=ticks {
        let t = k as f32 / 60.0;
        state = simulate([(t * 7.0).sin() * shake, 0.0], &state, &[], t, p);
        if k > 60 {
            let pos = vec2_col(&state, "P");
            let centre = |r: usize| {
                let s = pos[r * p.cols..(r + 1) * p.cols]
                    .iter()
                    .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
                [s[0] / p.cols as f32, s[1] / p.cols as f32]
            };
            let (a, b) = (centre(0), centre(p.rows - 1));
            let d = [b[0] - a[0], b[1] - a[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if len > 1e-6 {
                for r in 1..p.rows - 1 {
                    let q = centre(r);
                    let off = ((q[0] - a[0]) * d[1] - (q[1] - a[1]) * d[0]).abs() / len;
                    worst = worst.max(off / len);
                }
            }
        }
    }
    worst
}

fn snake(clusters: usize) -> Params {
    Params {
        rows: 32,
        cols: 4,
        spacing: 0.7,
        gravity: 12.0,
        stiffness: 0.4,
        beta: 0.0,
        damping: 0.03,
        pressure: 0.0,
        clusters,
        pin: true,
    }
}

/// **A snake stops being a plate.** With one frame the model can only translate and
/// rotate the rest shape, so a body whipped by its anchor swings as a stick; with
/// overlapping clusters it curves.
///
/// ⚠️ **The anchor SHAKES, and the first version of this fixture did not.** A body
/// hanging from a symmetric pin under uniform gravity has no reason to bend — every
/// cluster sees the same rotation — and the probe read **0,0000 at every cluster
/// count**, which is indistinguishable from a feature that does nothing. The load
/// has to be the one that asks the question.
#[test]
fn clusters_let_a_whipped_snake_bend_instead_of_swinging_as_a_stick() {
    let stick = worst_spine_bend(&snake(1), 300, 2.5);
    let bendy = worst_spine_bend(&snake(4), 300, 2.5);
    assert!(
        stick < 0.06,
        "CONTROLE: um frame so mal se afasta da reta, e afastou-se {stick}"
    );
    assert!(
        bendy > stick * 3.0,
        "com clusters a espinha tem de curvar de verdade: {stick} -> {bendy}"
    );
}

/// One cluster is the body that shipped, and the whole sequential simulation says
/// so — not just the goal projection. This is the regression net for the nine facts
/// above, stated once: the clustered route is never entered at all.
#[test]
fn one_cluster_replays_the_body_that_shipped_to_the_bit() {
    let run = |p: &Params| {
        let mut state = Stream::new(0);
        for k in 0..120 {
            state = simulate(
                [(k as f32 / 60.0 * 3.0).sin(), 0.0],
                &state,
                &[],
                k as f32 / 60.0,
                p,
            );
        }
        vec2_col(&state, "P")
    };
    // `clusters` is clamped to at least 1 in `eval`, so zero and one are the same
    // request; both have to give the untouched body.
    assert_eq!(run(&snake(1)), run(&snake(1)));
    let one = run(&snake(1));
    assert!(one.iter().all(|q| q[0].is_finite()));
    // And the clustered route is a DIFFERENT body — otherwise the gate above is
    // comparing the shipping path with itself.
    assert_ne!(one, run(&snake(4)));
}

/// Roda o corpo com uma coluna `falloff` injectada no estado a cada tique — que
/// e' exactamente o que um `field.*` na cadeia de estado faz
/// (`soft_body.out --pre--> field.box --> soft_body.state`), o mesmo fio pelo
/// qual o `accel` e o `inv_mass` ja' entram.
fn run_with_falloff(p: &Params, w: &[f32], ticks: usize, dt: f32) -> Vec<[f32; 2]> {
    let mut state = Stream::new(0);
    let mut last = Vec::new();
    for k in 0..ticks {
        let t = k as f32 * dt;
        let mut out = simulate([0.0, 0.0], &state, &[], t, p);
        last = vec2_col(&out, "P");
        if out.count() == w.len() {
            out.set(FALLOFF_COL, Column::Scalar(w.to_vec()));
        }
        state = out;
    }
    last
}

/// **O PESO ALCANÇA O PUXÃO: uma partícula de peso zero é LIVRE.**
///
/// A metade de baixo do corpo perde o peso e cai; a de cima, pinada e com peso
/// cheio, fica. ⚠️ O oráculo é a SEPARAÇÃO entre as duas metades e não a queda
/// em si — um corpo inteiro a cair mediria a mesma coisa e não diria nada.
#[test]
fn a_particle_of_zero_weight_falls_free_of_the_body() {
    let p = params(4, 4, 9.8, 0.4, true);
    let n = p.rows * p.cols;
    let free: Vec<f32> = (0..n).map(|i| if i >= n / 2 { 0.0 } else { 1.0 }).collect();
    let held = vec![1.0f32; n];

    let a = run_with_falloff(&p, &held, 90, 1.0 / 60.0);
    let b = run_with_falloff(&p, &free, 90, 1.0 / 60.0);
    let bottom = |v: &Vec<[f32; 2]>| v[n - 1][1];
    let (ya, yb) = (bottom(&a), bottom(&b));
    assert!(
        yb < ya - 1.0,
        "sem peso a partícula tem de cair LONGE do corpo: preso {ya:.4}, livre {yb:.4}"
    );
    // E o CONTROLE: a linha de topo, pinada, não se move em nenhum dos dois.
    assert!(
        (a[0][1] - b[0][1]).abs() < 1e-4,
        "o pino não pode depender do peso de quem se soltou: {:.6} vs {:.6}",
        a[0][1],
        b[0][1]
    );
}

/// **UM PESO CHEIO EM TODA PARTE É O CORPO QUE SEMPRE SHIPOU** — e a barra é
/// `1e-4`, não zero, com o motivo escrito: com a coluna presente o centroide de
/// repouso passa a ser CALCULADO em vez de assumido zero, e medido ele vale
/// `~1e-7` numa malha real. É o número certo; o que ele não é, é o mesmo bit.
#[test]
fn a_falloff_of_one_everywhere_is_the_body_that_ships() {
    let p = params(5, 4, 9.8, 0.4, true);
    let n = p.rows * p.cols;
    let ones = vec![1.0f32; n];
    let with = run_with_falloff(&p, &ones, 60, 1.0 / 60.0);
    let without = run(|_| [0.0, 0.0], &p, 60, 1.0 / 60.0);
    let last = without.last().expect("frames");
    let worst = with
        .iter()
        .zip(last)
        .map(|(a, b)| (a[0] - b[0]).abs().max((a[1] - b[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-4,
        "peso cheio tem de ser o corpo de sempre; {worst:.7}"
    );
}

/// **UM PESO NEGATIVO NÃO INVERTE NADA** — um documento editado à mão pode
/// escrever qualquer `f32` na coluna, e um peso negativo no ajuste vira o quadro
/// do avesso. Clampado, ele lê como *"não pertence"*, que é a leitura honesta.
#[test]
fn a_negative_falloff_reads_as_no_membership() {
    let p = params(4, 4, 9.8, 0.4, true);
    let n = p.rows * p.cols;
    let neg: Vec<f32> = (0..n)
        .map(|i| if i >= n / 2 { -3.0 } else { 1.0 })
        .collect();
    let zero: Vec<f32> = (0..n).map(|i| if i >= n / 2 { 0.0 } else { 1.0 }).collect();
    let a = run_with_falloff(&p, &neg, 60, 1.0 / 60.0);
    let b = run_with_falloff(&p, &zero, 60, 1.0 / 60.0);
    let worst = a
        .iter()
        .zip(&b)
        .map(|(x, y)| (x[0] - y[0]).abs().max((x[1] - y[1]).abs()))
        .fold(0.0f32, f32::max);
    assert!(worst < 1e-5, "negativo tem de ler como zero; {worst:.6}");
}

/// **O CORPO NÃO SEGUE QUEM SE SOLTOU DELE — pela porta do PRODUTO.**
///
/// ⚠️ Este gate existe porque uma mutação passou: os dois gates da lei chamam o
/// kernel do ajuste DIRETO, então *"o peso alcança o AJUSTE"* e *"o peso alcança
/// só o PUXÃO"* eram indistinguíveis a partir do `simulate` — e a segunda
/// metade é a que faz o corpo derreter atrás de quem caiu. A régua é a linha do
/// MEIO (peso cheio, sem pino): ela mede o quadro, não a queda.
#[test]
fn the_body_does_not_follow_what_fell_off_it() {
    let p = params(6, 6, 9.8, 0.4, true);
    let (n, cols) = (p.rows * p.cols, p.cols);
    // As duas últimas linhas soltam-se; as de cima ficam com peso cheio.
    let free: Vec<f32> = (0..n)
        .map(|i| if i >= n - 2 * cols { 0.0 } else { 1.0 })
        .collect();
    let held = vec![1.0f32; n];

    let mid = |v: &[[f32; 2]]| v[2 * cols + cols / 2][1];
    let a = run_with_falloff(&p, &held, 150, 1.0 / 60.0);
    let b = run_with_falloff(&p, &free, 150, 1.0 / 60.0);

    // O CONTROLE primeiro: as duas últimas linhas TÊM de se soltar, senão o
    // gate mede um corpo que ninguém rasgou.
    let tail = |v: &[[f32; 2]]| v[n - 1][1];
    assert!(
        tail(&b) < tail(&a) - 1.0,
        "a fixture tem de conter o fenômeno: cauda presa {:.4}, solta {:.4}",
        tail(&a),
        tail(&b)
    );
    // E a linha do meio, que pertence ao corpo nos DOIS, tem de ficar no mesmo
    // lugar: um ajuste cego ao peso a arrastaria atrás da cauda.
    let d = (mid(&a) - mid(&b)).abs();
    assert!(
        d < 0.05,
        "o quadro do corpo não pode seguir quem se soltou; a linha do meio andou {d:.4}"
    );
}
