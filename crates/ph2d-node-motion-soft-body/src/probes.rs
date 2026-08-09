//! **Measurements** for `motion.soft_body` — what a tick costs, and what the
//! body's enclosed area actually does under load. None of this is a gate: every
//! test here is `#[ignore]` and prints a table, because the numbers that pick a
//! cap, a slider band and a law have to come from the product before a line is
//! written (CLAUDE.md §0). Split from `lib.rs` at the 700-LOC cap.

use super::*;

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
            let mut s = simulate(anchor(t), &state, t, p);
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
            let mut s = simulate(anchor, &state, t, &p);
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
            pin: true,
        };
        let mut state = Stream::new(0);
        let mut worst = 1.0f32;
        for k in 0..=300usize {
            let t = k as f32 / 60.0;
            let s = simulate([0.0, 0.0], &state, t, &p);
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
            let sc = pressure_scale(&squashed, 8, 8, a0, g, k);
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
        let rest = rest_shape(side, side, 1.0);
        let pred: Vec<[f32; 2]> = rest.iter().map(|p| [p[0] * 1.1, p[1] * 0.9]).collect();
        let t0 = std::time::Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(shape_goals(&pred, &rest, 0.3, 1.0));
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
                [0.0, 0.0],
                &rest,
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
