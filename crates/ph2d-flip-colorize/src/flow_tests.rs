//! Gates for the Colorize C2 max-flow (`docs/Flip/09 §7.1`) + the measurement bench.
//!
//! The correctness oracle is an **independent** Edmonds–Karp (BFS-augmenting) max-flow,
//! written from scratch here — a second implementation of the same DEFINITION, so a bug in
//! the BK solver can't hide behind a matching bug in its own test (the sibling trap the
//! audio/painter lines paid for). BK is verified to give the exact same flow value on many
//! pseudo-random small grids *and* on the real `lazybrush_binary` graph; the cut it reads
//! back is checked to weigh exactly the flow. Then the bench measures the real product
//! grid — the number `09 §7.1`/`§7.2` reserves before any UI.

use super::{Flow, lazybrush_binary};

/// The opposite of grid neighbour direction `d` (`0`=E, `1`=W, `2`=S, `3`=N ⇒ a single bit).
///
/// It lives **in the gates**, not in the solver: the engine is a general graph and has no
/// notion of a direction: leaving this next to it would assert otherwise. The grid layout is
/// the ORACLE's language now.
const fn opp(d: usize) -> usize {
    d ^ 1
}
use ph2d_core::Vec2;
use ph2d_flip_fill::Grid;

/// splitmix64 — the crate's deterministic PRNG idiom (HR-5), for varied fixtures without a
/// dependency. A gate that always sees the same tiny graph proves nothing.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[inline]
fn neighbour(w: usize, h: usize, i: usize, d: usize) -> Option<usize> {
    let x = i % w;
    let y = i / w;
    match d {
        0 if x + 1 < w => Some(i + 1),
        1 if x > 0 => Some(i - 1),
        2 if y + 1 < h => Some(i + w),
        3 if y > 0 => Some(i - w),
        _ => None,
    }
}

/// The independent oracle: max-flow by repeated BFS augmenting paths (Edmonds–Karp) on the
/// same implicit grid, from a snapshot of the initial capacities. A super-source feeds
/// every node with `t0 > 0`; a super-sink drains every node with `t0 < 0`. Terminal reverse
/// arcs are omitted **on purpose** — an s→t augmenting path can never traverse `i→source`
/// (that would revisit the start) nor `sink→i` (the path ends at the sink), so they carry
/// no flow and their omission is exact. Small grids only.
fn edmonds_karp(w: usize, h: usize, res0: &[i32], t0: &[i32]) -> i64 {
    let n = w * h;
    let mut res = res0.to_vec();
    let mut src: Vec<i32> = (0..n).map(|i| t0[i].max(0)).collect();
    let mut snk: Vec<i32> = (0..n).map(|i| (-t0[i]).max(0)).collect();
    let mut flow: i64 = 0;

    loop {
        // par: -1 unvisited, -2 reached from the super-source, else the parent node index.
        let mut par = vec![-1i32; n];
        let mut pdir = vec![0usize; n];
        let mut q = std::collections::VecDeque::new();
        for i in 0..n {
            if src[i] > 0 {
                par[i] = -2;
                q.push_back(i);
            }
        }
        let mut sink = None;
        while let Some(i) = q.pop_front() {
            if snk[i] > 0 {
                sink = Some(i);
                break;
            }
            for d in 0..4 {
                if res[4 * i + d] > 0
                    && let Some(qq) = neighbour(w, h, i, d)
                    && par[qq] == -1
                {
                    par[qq] = i as i32;
                    pdir[qq] = d;
                    q.push_back(qq);
                }
            }
        }
        let Some(i) = sink else { break };

        // Bottleneck over: the sink t-link, the n-links up to the source-attached node, and
        // that node's source t-link.
        let mut b = snk[i];
        {
            let mut node = i;
            while par[node] >= 0 {
                let p = par[node] as usize;
                b = b.min(res[4 * p + pdir[node]]);
                node = p;
            }
            b = b.min(src[node]);
        }
        // Push it.
        snk[i] -= b;
        {
            let mut node = i;
            while par[node] >= 0 {
                let p = par[node] as usize;
                let d = pdir[node];
                res[4 * p + d] -= b;
                let child = neighbour(w, h, p, d).expect("arc head");
                res[4 * child + opp(d)] += b;
                node = p;
            }
            src[node] -= b;
        }
        flow += i64::from(b);
    }
    flow
}

/// The value of the cut described by `source_side`: sum of `V_pq` over adjacent pairs split
/// by it, plus the terminal links crossed. Must equal the max-flow (max-flow ≡ min-cut).
fn cut_value(w: usize, h: usize, res0: &[i32], t0: &[i32], side: &[bool]) -> i64 {
    let mut v: i64 = 0;
    for i in 0..w * h {
        // Source t-link is cut if i is on the sink side; sink t-link if i is on source side.
        if t0[i] > 0 && !side[i] {
            v += i64::from(t0[i]);
        }
        if t0[i] < 0 && side[i] {
            v += i64::from(-t0[i]);
        }
        // n-link i→q counts once, when i is source-side and q is sink-side.
        for d in [0usize, 2] {
            if let Some(q) = neighbour(w, h, i, d) {
                if side[i] && !side[q] {
                    v += i64::from(res0[4 * i + d]);
                }
                if side[q] && !side[i] {
                    v += i64::from(res0[4 * q + opp(d)]);
                }
            }
        }
    }
    v
}

/// BK ≡ the independent oracle on many pseudo-random small grids. This is the load-bearing
/// correctness gate: varied capacities, varied terminals, both trees exercised.
#[test]
fn bk_matches_edmonds_karp_on_random_grids() {
    let mut seed = 0xC0FF_EE00_1234_5678u64;
    for &(w, h) in &[
        (4, 4),
        (5, 3),
        (6, 6),
        (7, 5),
        (8, 8),
        (3, 9),
        (10, 10),
        (9, 7),
    ] {
        let n = w * h;
        for _trial in 0..16 {
            // As capacidades nascem em layout de GRADE (`4·i + d`) — que é o que o oráculo
            // lê. Ele não conhece a representação interna do motor, e é por isso que ele
            // continua sendo oráculo depois do motor virar grafo geral.
            let mut caps = vec![0i32; 4 * n];
            for i in 0..n {
                for d in [0usize, 2] {
                    if let Some(q) = neighbour(w, h, i, d) {
                        let c = (splitmix(&mut seed) % 7) as i32;
                        caps[4 * i + d] = c;
                        caps[4 * q + opp(d)] = c;
                    }
                }
            }
            let mut f = Flow::grid_4conn(w, h, |i, q| {
                let d = if q == i + 1 { 0 } else { 2 };
                caps[4 * i + d]
            });
            let a = (splitmix(&mut seed) as usize) % n;
            let mut b = (splitmix(&mut seed) as usize) % n;
            if b == a {
                b = (b + 1) % n;
            }
            let k = 2 * (w + h) as i32;
            f.set_tlink(a, k, 0);
            f.set_tlink(b, 0, k);
            // A few weak extra terminals so the instance isn't a clean two-seed cut.
            for _ in 0..3 {
                let j = (splitmix(&mut seed) as usize) % n;
                if j != a && j != b {
                    let mag = (splitmix(&mut seed) % 4) as i32;
                    if splitmix(&mut seed) & 1 == 0 {
                        f.set_tlink(j, mag, 0);
                    } else {
                        f.set_tlink(j, 0, mag);
                    }
                }
            }

            let res0 = caps.clone();
            let t0 = f.t.clone();
            let bk = f.max_flow();
            let ek = edmonds_karp(w, h, &res0, &t0);
            assert_eq!(bk, ek, "BK != Edmonds-Karp on {w}x{h}");

            // And the cut it reads back must weigh exactly the flow.
            let side = f.source_side();
            assert_eq!(
                cut_value(w, h, &res0, &t0, &side),
                bk,
                "cut value != flow on {w}x{h}"
            );
        }
    }
}

/// A hand-checked bottleneck: source seed and sink seed joined by a single 3-cell bridge of
/// capacities 5,2,9 → the min-cut is the middle arc, value 2. Built through [`Flow::build`],
/// so it also pins the general-graph door the region cut uses (`§8`) — a chain is not a grid.
#[test]
fn bk_finds_the_hand_computed_bottleneck() {
    let mut f = Flow::build(4, [(0, 1, 5), (1, 2, 2), (2, 3, 9)]);
    f.set_tlink(0, 100, 0);
    f.set_tlink(3, 0, 100);
    assert_eq!(f.max_flow(), 2);
}

/// The general graph is not a grid, and the CSR must survive **irregular degree** — the whole
/// reason the region graph needs it (a region can border two neighbours or twenty). A star
/// whose centre is the only path from source to sink: the cut is the cheaper of its two
/// spokes, which no 4-neighbour layout could even represent at this degree.
#[test]
fn bk_cuts_a_high_degree_star() {
    // 0 = source seed · 1..=5 = leaves into the hub 6 · 7 = sink seed.
    let mut edges = vec![(6u32, 7u32, 3i32)]; // hub → sink, the throat
    for leaf in 1..=5u32 {
        edges.push((0, leaf, 10));
        edges.push((leaf, 6, 10));
    }
    let mut f = Flow::build(8, edges);
    f.set_tlink(0, 100, 0);
    f.set_tlink(7, 0, 100);
    assert_eq!(
        f.max_flow(),
        3,
        "o gargalo e' o arco do hub para o sumidouro"
    );
    let side = f.source_side();
    assert!(side[0] && side[6] && !side[7], "o corte isola o sumidouro");
}

/// The `V_pq` law of `09 §3` in grid layout (`4·i + d`), for the oracle — an **independent
/// restatement**, deliberately not a call into `lazybrush_binary`.
fn grid_caps(g: &Grid, v_white: i32, v_ink: i32) -> Vec<i32> {
    let (w, h) = (g.w, g.h);
    let is_ink = |i: usize| g.flags[i] & ph2d_flip_fill::BOUNDARY != 0;
    let mut caps = vec![0i32; 4 * w * h];
    for i in 0..w * h {
        for d in [0usize, 2] {
            let Some(q) = neighbour(w, h, i, d) else {
                continue;
            };
            let c = if is_ink(i) || is_ink(q) {
                v_ink
            } else {
                v_white
            };
            caps[4 * i + d] = c;
            caps[4 * q + opp(d)] = c;
        }
    }
    caps
}

/// BK ≡ the oracle on the real `lazybrush_binary` graph over a product `Grid`: a box, seed
/// inside vs outside. Exercises the graph builder, not just the solver.
#[test]
fn bk_cuts_a_real_boxed_grid_like_the_reference() {
    let world = 1.0f32;
    let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(world, world), 16.0, 2, 64);
    let (a, b) = (0.25 * world, 0.75 * world);
    for (p, q) in [
        (Vec2::new(a, a), Vec2::new(b, a)),
        (Vec2::new(b, a), Vec2::new(b, b)),
        (Vec2::new(b, b), Vec2::new(a, b)),
        (Vec2::new(a, b), Vec2::new(a, a)),
    ] {
        g.stroke_capsule(p, q, 0.0);
    }
    let (ix, iy) = g.pixel_of(Vec2::new(0.5, 0.5)).expect("inside pixel");
    let inside = iy * g.w + ix;
    let outside = g.w + 1; // (1,1): in the margin, outside the box, not ink

    let mut f = lazybrush_binary(&g, &[inside], &[outside], 8, 1);
    // O oráculo recebe as capacidades em layout de GRADE, **re-enunciadas aqui** a partir da
    // lei do `09 §3` (tinta barata, papel caro). Lê-las de dentro do `Flow` seria pedir ao
    // motor a resposta que o oráculo existe para conferir.
    let res0 = grid_caps(&g, 8, 1);
    let t0 = f.t.clone();
    let bk = f.max_flow();
    let ek = edmonds_karp(g.w, g.h, &res0, &t0);
    assert_eq!(bk, ek, "BK != Edmonds-Karp on the boxed grid");
    // The inside seed stays source-side, the outside seed sink-side (the cut encircles it).
    let side = f.source_side();
    assert!(
        side[inside] && !side[outside],
        "the cut must separate the seeds"
    );
    assert_eq!(cut_value(g.w, g.h, &res0, &t0, &side), bk);
}

/// **The régua** (`--release --ignored --nocapture`): the cost of ONE binary LazyBrush cut
/// on the real product grid, with the product numbers (mirrors
/// `measure_the_product_grid_and_ball_cost`). Prints the table; asserts no wall-clock number
/// (a timing gate would measure the build profile, not the product — CLAUDE.md §0.0). The
/// 16 ms line decides sync vs the `progress` bar; 2 s decides whether C2 exists in this form
/// (`09 §7.2`).
#[test]
#[ignore = "régua de medição — rode com --release --ignored --nocapture"]
fn measure_the_binary_cut_cost() {
    let px_to_world = 10.0f32 / 1080.0;
    let precision = 1.6 / px_to_world;

    println!("\n=== corte binario (LazyBrush C2, 09 §7.1) — grade do PRODUTO ===");
    println!(
        "{:>10} {:>12} {:>9} {:>12} {:>11} {:>11}",
        "tela(px)", "grade", "Mpix", "fluxo", "build(ms)", "corte(ms)"
    );
    for &screen in &[512.0f32, 1080.0, 1920.0, 3840.0] {
        let world = screen * px_to_world;
        let mut g = Grid::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(world, world),
            precision,
            20,
            4096,
        );
        let (a, b) = (world * 0.25, world * 0.75);
        for (p, q) in [
            (Vec2::new(a, a), Vec2::new(b, a)),
            (Vec2::new(b, a), Vec2::new(b, b)),
            (Vec2::new(b, b), Vec2::new(a, b)),
            (Vec2::new(a, b), Vec2::new(a, a)),
        ] {
            g.stroke_capsule(p, q, 0.0);
        }
        // Scribbles, NOT single pixels (see `lazybrush_binary`): a horizontal line across
        // the middle of the box interior vs a horizontal line in the outside region below
        // it. Both span half the box width, so the min-cut runs along the ink walls — a
        // real region cut with balanced trees, not the degenerate "fence off one pixel".
        let px_line = |y: f32, dst: &mut Vec<usize>| {
            let (x0, x1) = (a + (b - a) * 0.25, a + (b - a) * 0.75);
            let steps = (((x1 - x0) * g.scale) as usize).max(1);
            for s in 0..=steps {
                let x = x0 + (x1 - x0) * (s as f32 / steps as f32);
                if let Some((px, py)) = g.pixel_of(Vec2::new(x, y)) {
                    dst.push(py * g.w + px);
                }
            }
        };
        let mut source = Vec::new();
        let mut sink = Vec::new();
        px_line(world * 0.5, &mut source); // inside the box
        px_line(world * 0.10, &mut sink); // below the box, outside
        let mpix = (g.w * g.h) as f64 / 1.0e6;

        let t0 = std::time::Instant::now();
        let mut f = lazybrush_binary(&g, &source, &sink, 8, 1);
        let build = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = std::time::Instant::now();
        let flow = f.max_flow();
        let solve = t1.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{screen:>10.0} {:>5}x{:<6} {mpix:>9.2} {flow:>12} {build:>11.1} {solve:>11.1}",
            g.w, g.h
        );
    }
    println!(
        "\n(UMA cor contra o resto = 1 corte; o multiway guloso e' ~n_labels desses — 09 §3)\n\
         (>16 ms/frame => progress bar; >2 s => C2 muda de forma — 09 §7.2)\n"
    );
}
