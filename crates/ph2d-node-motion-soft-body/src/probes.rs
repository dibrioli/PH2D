//! **Measurements** for `motion.soft_body` — what a tick costs, and what the
//! body's enclosed area actually does under load. None of this is a gate: every
//! test here is `#[ignore]` and prints a table, because the numbers that pick a
//! cap, a slider band and a law have to come from the product before a line is
//! written (CLAUDE.md §0). Split from `lib.rs` at the 700-LOC cap.

use super::*;
use crate::shape::shape_goals;

/// MEASUREMENT, before a line of `pressure` is written: **does the body's area
/// actually deviate from its rest area?** The plan calls `pressure` a term
/// inside the goal projection, but the goal is either the RIGID rest shape or
/// the paper's AREA-PRESERVED linear map — both already carry the rest area
/// exactly. So if the live cloud tracked its goal, an area-restoring term
/// would be inert, and the honest answer would be *there is nothing to
/// build*.
///
/// What the cloud does instead is lag the goal by `stiffness`, and that lag
/// is where a volume can be lost. This prints the ratio so the LAW is chosen
/// from a number.
///
///   cargo test -p ph2d-node-motion-soft-body -- --ignored --nocapture pressure
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn what_does_the_area_do_under_load() {
    let (rows, cols) = (8usize, 8usize);
    let rest = rest_shape(rows, cols, 0.7);
    let a0 = boundary_area(&rest, rows, cols);
    eprintln!("  area de repouso = {a0:.4} (assinada; negativa = anel horario)");

    let run = |name: &str, p: &Params, anchor: fn(f32) -> [f32; 2], squeeze: f32| {
        let mut state = Stream::new(0);
        let mut out = String::new();
        for k in 0..=240usize {
            let t = k as f32 / 60.0;
            let mut s = simulate(anchor(t), &state, &[], t, p);
            if squeeze != 0.0 {
                // An inward accel field: every particle pulled toward the
                // centroid, which is what a `force.attractor` in the state
                // chain does to a body it sits on top of.
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
                s = s.with("accel", Column::Vec2(acc));
            }
            state = s;
            if k % 60 == 0 {
                let pos = vec2_col(&state, "P");
                let a = boundary_area(&pos, rows, cols);
                out.push_str(&format!(" t={t:.1}s {:.3}", a / a0));
            }
        }
        eprintln!("  {name:<34}{out}");
    };

    let base = |stiff: f32, pin: bool| Params {
        rows,
        cols,
        spacing: 0.7,
        gravity: 12.0,
        stiffness: stiff,
        beta: 0.0,
        damping: 0.03,
        pressure: 0.0,
        clusters: 1,
        pin,
    };
    eprintln!("  (razao area/repouso; 1.000 = defende o volume sozinho)");
    run(
        "pendurado, stiffness 0.4",
        &base(0.4, true),
        |_| [0.0, 0.0],
        0.0,
    );
    run(
        "pendurado, stiffness 0.1",
        &base(0.1, true),
        |_| [0.0, 0.0],
        0.0,
    );
    run(
        "queda livre (sem pino)",
        &base(0.4, false),
        |_| [0.0, 0.0],
        0.0,
    );
    run(
        "ancora sacudida",
        &base(0.4, true),
        |t| [(t * 9.0).sin() * 2.0, 0.0],
        0.0,
    );
    run(
        "espremido (attractor 40)",
        &base(0.4, true),
        |_| [0.0, 0.0],
        40.0,
    );
    run(
        "espremido (attractor 120)",
        &base(0.4, true),
        |_| [0.0, 0.0],
        120.0,
    );
    let mut squash = base(0.4, true);
    squash.beta = 1.0;
    run("espremido, linear (beta 1)", &squash, |_| [0.0, 0.0], 120.0);
}

/// CALIBRATION: the area the body settles at for a sweep of `pressure`, in the
/// two regimes the probe above proved lossy. This is what picks the slider's
/// comfortable range and the typable ceiling — a hard max above what the
/// kernel HONOURS is a box that accepts and lies (doc 88 B2).
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture calibrat
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn what_each_pressure_settles_at() {
    let (rows, cols) = (8usize, 8usize);
    let rest = rest_shape(rows, cols, 0.7);
    let a0 = boundary_area(&rest, rows, cols);

    let settle = |gain: f32, squeeze: f32, shake: bool| -> (f32, f32) {
        let p = Params {
            rows,
            cols,
            spacing: 0.7,
            gravity: 12.0,
            stiffness: 0.4,
            beta: 0.0,
            damping: 0.03,
            pressure: gain,
            clusters: 1,
            pin: true,
        };
        let mut state = Stream::new(0);
        let (mut worst, mut last) = (1.0f32, 1.0f32);
        for k in 0..=300usize {
            let t = k as f32 / 60.0;
            let anchor = if shake {
                [(t * 9.0).sin() * 2.0, 0.0]
            } else {
                [0.0, 0.0]
            };
            let mut s = simulate(anchor, &state, &[], t, &p);
            if squeeze != 0.0 {
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
                s = s.with("accel", Column::Vec2(acc));
            }
            state = s;
            if k > 60 {
                let r = boundary_area(&vec2_col(&state, "P"), rows, cols) / a0;
                last = r;
                if (r - 1.0).abs() > (worst - 1.0).abs() {
                    worst = r;
                }
            }
        }
        (last, worst)
    };

    eprintln!("  pressure | espremido(120): fim / pior | sacudido: fim / pior");
    for gain in [0.0f32, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
        let (a, aw) = settle(gain, 120.0, false);
        let (b, bw) = settle(gain, 0.0, true);
        eprintln!("  {gain:>8.2} | {a:>10.3} / {aw:.3} | {b:>10.3} / {bw:.3}");
    }
}

/// The question the sweep above RAISED: the body travels `stiffness` of the way
/// to its goal each step, so the closed loop's gain looks like
/// `stiffness × pressure` — which would mean the largest usable pressure is a
/// function of ANOTHER knob, the shape this repo calls an ergonomics bug
/// rather than a value to pick.
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture coupled
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn is_the_useful_pressure_coupled_to_stiffness() {
    let (rows, cols) = (8usize, 8usize);
    let rest = rest_shape(rows, cols, 0.7);
    let a0 = boundary_area(&rest, rows, cols);

    let settle = |gain: f32, stiff: f32| -> f32 {
        let p = Params {
            rows,
            cols,
            spacing: 0.7,
            gravity: 12.0,
            stiffness: stiff,
            beta: 0.0,
            damping: 0.03,
            pressure: gain,
            clusters: 1,
            pin: true,
        };
        let mut state = Stream::new(0);
        let mut worst = 1.0f32;
        for k in 0..=300usize {
            let t = k as f32 / 60.0;
            let s = simulate([0.0, 0.0], &state, &[], t, &p);
            let pos = vec2_col(&s, "P");
            let n = pos.len() as f32;
            let c = pos
                .iter()
                .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
            let c = [c[0] / n, c[1] / n];
            let acc: Vec<[f32; 2]> = pos
                .iter()
                .map(|q| [(c[0] - q[0]) * 120.0, (c[1] - q[1]) * 120.0])
                .collect();
            state = s.with("accel", Column::Vec2(acc));
            if k > 60 {
                let r = boundary_area(&vec2_col(&state, "P"), rows, cols) / a0;
                if (r - 1.0).abs() > (worst - 1.0).abs() {
                    worst = r;
                }
            }
        }
        worst
    };

    // Where does the single-step clamp actually bite? The band it guards has to
    // be stated as a fact, not as a hope.
    {
        let rest = rest_shape(8, 8, 0.7);
        let a0 = boundary_area(&rest, 8, 8);
        let ring = crate::layout::grid_ring(8, 8);
        let squashed: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 0.92, q[1] * 0.92]).collect();
        eprint!("  escala pedida (corpo 8% menor)  ");
        for (k, g) in [
            (0.4f32, 1.0f32),
            (0.4, 2.0),
            (0.4, 4.0),
            (0.1, 1.0),
            (0.1, 2.0),
            (0.1, 4.0),
        ] {
            let sc = pressure_scale(&squashed, &ring, a0, g, k);
            eprint!(" k={k:.1}/p={g:.0}:{sc:.2}");
        }
        eprintln!();
    }
    eprintln!("  pior razao de area (1.000 = perfeito; >1.3 = estourou)");
    eprint!("  stiff\\press ");
    for gain in [0.5f32, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0] {
        eprint!("{gain:>8.1}");
    }
    eprintln!();
    for stiff in [0.1f32, 0.2, 0.4, 0.7, 1.0] {
        eprint!("  {stiff:>10.2} ");
        for gain in [0.5f32, 1.0, 1.5, 2.0, 3.0, 4.0, 6.0] {
            eprint!("{:>8.3}", settle(gain, stiff));
        }
        eprintln!();
    }
}

/// MEASUREMENT (a fila §2 do handoff da linha GPU): o cap do nó ERA
/// `MAX_SIDE = 40` (1600 partículas). O custo por tick é o shape-matching —
/// DUAS passadas lineares sobre a nuvem (centroide, depois `A_pq`/`A_qq`) —
/// então a pergunta do §0.0 é: esse cap é de CUSTO ou é escolha?
///
/// Mede as DUAS coisas: o núcleo (`shape_goals`) e o **tick inteiro**
/// (`step`, que é o que o orçamento do HR-4 de fato paga — predição,
/// projeção ao objetivo e leitura de velocidade, todas lineares por cima).
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn what_does_the_shape_match_cost_per_tick() {
    const REPS: u32 = 20;
    for &side in &[40usize, 100, 316, 512, 724, 1000] {
        let n = side * side;
        let layout = crate::layout::BodyLayout::from_grid(side, side, 1.0);
        let rest = &layout.rest;
        let pred: Vec<[f32; 2]> = rest.iter().map(|p| [p[0] * 1.1, p[1] * 0.9]).collect();
        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(shape_goals(&pred, rest, 0.3, 1.0));
        }
        let core = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);

        let p = super::Params {
            rows: side,
            cols: side,
            spacing: 1.0,
            gravity: 9.8,
            stiffness: 0.6,
            beta: 0.3,
            damping: 0.02,
            pressure: 0.0,
            clusters: 1,
            pin: true,
        };
        let vel = vec![[0.0f32; 2]; n];
        // The zeros the product materialises when no force reaches the body
        // — `simulate` always hands `step` a full-length slice, so measuring
        // with one is measuring what ships.
        let accel = vec![[0.0f32; 2]; n];
        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(super::step(
                &pred,
                &vel,
                &accel,
                &[],
                None,
                [0.0, 0.0],
                &layout,
                1.0 / 60.0,
                &p,
            ));
        }
        let tick = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
        eprintln!(
            "  {side:>4}x{side:<4} = {n:>9} partículas: núcleo {core:>7.3} ms · TICK {tick:>7.3} ms"
        );
    }
}

/// MEASUREMENT, before a line of clusters is written: **does a long body actually
/// move like a plate?** The plan says a 32×4 snake "balança como uma placa"
/// because shape matching fits ONE rigid frame to the whole cloud, so the goal it
/// projects is the rest shape rotated — a shape with no bend in it anywhere.
///
/// The statistic is the SPINE's curvature: take each row's centroid, fit a line
/// through them, and report the worst distance from it, in units of the body's own
/// length. A body that only translates and rotates reads ~0 whatever it is doing;
/// a body that bends reads a number.
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture spine
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn how_much_can_a_long_body_bend_today() {
    let bend_of = |rows: usize, cols: usize, stiff: f32, beta: f32, shake: f32| -> f32 {
        let p = Params {
            rows,
            cols,
            spacing: 0.7,
            gravity: 12.0,
            stiffness: stiff,
            beta,
            damping: 0.03,
            pressure: 0.0,
            clusters: 1,
            pin: true,
        };
        let mut state = Stream::new(0);
        let mut worst = 0.0f32;
        for k in 0..=300usize {
            let t = k as f32 / 60.0;
            state = simulate([(t * 7.0).sin() * shake, 0.0], &state, &[], t, &p);
            if k > 60 {
                worst = worst.max(spine_bend(&vec2_col(&state, "P"), rows, cols));
            }
        }
        worst
    };
    eprintln!("  desvio maximo da espinha em relacao a RETA, como fracao do comprimento");
    for (r, c, name) in [
        (32usize, 4usize, "cobra 32x4"),
        (16, 8, "corpo 16x8"),
        (8, 8, "quadrado 8x8"),
    ] {
        eprintln!(
            "  {name:<14} rigido={:.4}  linear(beta 1)={:.4}  mole(stiff .1)={:.4}  sacudido={:.4}",
            bend_of(r, c, 0.4, 0.0, 0.0),
            bend_of(r, c, 0.4, 1.0, 0.0),
            bend_of(r, c, 0.1, 0.0, 0.0),
            bend_of(r, c, 0.4, 0.0, 2.5),
        );
    }
}

/// The worst distance from a row-centroid to the straight line through the first
/// and last of them, as a fraction of the body's rest length. Zero for anything
/// that is only translated, rotated or uniformly scaled — which is exactly the set
/// of poses a single rigid frame can produce.
fn spine_bend(pos: &[[f32; 2]], rows: usize, cols: usize) -> f32 {
    if rows < 3 || pos.len() < rows * cols {
        return 0.0;
    }
    let centre = |r: usize| {
        let s = pos[r * cols..(r + 1) * cols]
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        [s[0] / cols as f32, s[1] / cols as f32]
    };
    let (a, b) = (centre(0), centre(rows - 1));
    let d = [b[0] - a[0], b[1] - a[1]];
    let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
    if len < 1e-6 {
        return 0.0;
    }
    let mut worst = 0.0f32;
    for r in 1..rows - 1 {
        let p = centre(r);
        // |(p − a) × d| / |d| — the perpendicular distance to the chord.
        let off = ((p[0] - a[0]) * d[1] - (p[1] - a[1]) * d[0]).abs() / len;
        worst = worst.max(off);
    }
    worst / len
}

/// CALIBRATION: what each cluster count buys in BEND, and what it costs per tick.
/// The knob's slider band and its typable ceiling come from here, and so does the
/// honest statement about the node's 512² cap — which was measured against a
/// SINGLE shape match, and clusters overlap on purpose.
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture clusters_buy
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn what_clusters_buy_and_what_they_cost() {
    let bend_of = |rows: usize, cols: usize, n: usize| -> f32 {
        let p = Params {
            rows,
            cols,
            spacing: 0.7,
            gravity: 12.0,
            stiffness: 0.4,
            beta: 0.0,
            damping: 0.03,
            pressure: 0.0,
            clusters: n,
            pin: true,
        };
        let mut state = Stream::new(0);
        let mut worst = 0.0f32;
        for k in 0..=300usize {
            let t = k as f32 / 60.0;
            // ⚠️ The anchor SHAKES, and the earlier version of this probe did not.
            // A body hanging from a symmetric pin under uniform gravity has no
            // reason to curve: every cluster sees the same rotation and the spine
            // is straight whatever the model can express. It reported 0,0000 for
            // every cluster count and would have read as the feature being inert.
            state = simulate([(t * 7.0).sin() * 2.5, 0.0], &state, &[], t, &p);
            if k > 60 {
                worst = worst.max(spine_bend(&vec2_col(&state, "P"), rows, cols));
            }
        }
        worst
    };
    eprintln!("  desvio da espinha (fracao do comprimento), ancora SACUDIDA");
    for (r, c, name) in [
        (32usize, 4usize, "cobra 32x4"),
        (16, 8, "corpo 16x8"),
        (8, 8, "quadrado 8x8"),
    ] {
        let mut line = String::new();
        for n in [1usize, 2, 3, 4, 6, 8, 12, 16] {
            line.push_str(&format!(" {n}:{:.4}", bend_of(r, c, n)));
        }
        eprintln!("  {name:<14}{line}");
    }

    eprintln!("  custo de um TICK (ms) -- o cap de 512 do no foi medido com 1 cluster");
    for side in [64usize, 128, 256, 512] {
        let mut line = String::new();
        for n in [1usize, 2, 4, 8] {
            let p = Params {
                rows: side,
                cols: side,
                spacing: 1.0,
                gravity: 9.8,
                stiffness: 0.6,
                beta: 0.3,
                damping: 0.02,
                pressure: 0.0,
                clusters: n,
                pin: true,
            };
            let layout = crate::layout::BodyLayout::from_grid(side, side, 1.0);
            let rest = &layout.rest;
            let pred: Vec<[f32; 2]> = rest.iter().map(|q| [q[0] * 1.1, q[1] * 0.9]).collect();
            let vel = vec![[0.0f32; 2]; side * side];
            let accel = vec![[0.0f32; 2]; side * side];
            const REPS: u32 = 10;
            let t0 = std::time::Instant::now();
            for _ in 0..REPS {
                std::hint::black_box(step(
                    &pred,
                    &vel,
                    &accel,
                    &[],
                    None,
                    [0.0, 0.0],
                    &layout,
                    1.0 / 60.0,
                    &p,
                ));
            }
            let ms = t0.elapsed().as_secs_f64() * 1e3 / f64::from(REPS);
            line.push_str(&format!(" {n}:{ms:>7.3}"));
        }
        eprintln!("  {side:>4}x{side:<4}{line}");
    }
}

/// Does a clustered match REPRESENT a bend at all? Bend the rest mesh into a known
/// arc, ask for the goals, and report how far they land from the arc. One global
/// frame can only answer with a straight body, so its number is the arc's own
/// sagitta; a clustered match should follow.
///
///   cargo test -p ph2d-node-motion-soft-body --release -- --ignored --nocapture can_a_cluster
#[test]
#[ignore = "measurement — run alone with --nocapture"]
fn can_a_clustered_match_follow_an_arc() {
    let (rows, cols) = (32usize, 4usize);
    let rest = rest_shape(rows, cols, 0.7);
    // Bend the whole mesh around a circle: map y (along the length) to an angle.
    let h = (rows as f32 - 1.0) * 0.7;
    let radius = h / 1.2; // ~1.2 rad of arc — a clear, non-subtle curve
    let bent: Vec<[f32; 2]> = rest
        .iter()
        .map(|q| {
            let a = q[1] / radius;
            let r = radius - q[0];
            [r * a.sin(), r * a.cos() - radius]
        })
        .collect();
    eprintln!("  distancia RMS entre o GOAL e o arco real (unidades de mundo)");
    for n in [1usize, 2, 3, 4, 6, 8, 12, 16] {
        let goals = if n > 1 {
            crate::cluster::cluster_goals(
                &bent,
                &rest,
                &crate::layout::BodyLayout::from_grid(rows, cols, 0.7).buckets(n),
                0.0,
                1.0,
            )
        } else {
            shape_goals(&bent, &rest, 0.0, 1.0)
        };
        let rms = (goals
            .iter()
            .zip(&bent)
            .map(|(g, b)| (g[0] - b[0]).powi(2) + (g[1] - b[1]).powi(2))
            .sum::<f32>()
            / goals.len() as f32)
            .sqrt();
        let (nr, nc) = crate::cluster::counts(rows, cols, n);
        eprint!("  {n}({nr}x{nc}):{rms:.3}");
    }
    eprintln!();
}
