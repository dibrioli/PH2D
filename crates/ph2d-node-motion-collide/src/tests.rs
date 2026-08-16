//! Gates do `motion.collide` — extraídos do `lib.rs` no teto de LOC do HR-18.
//!
//! ⚠️ **Este arquivo nasceu de um vermelho LATENTE desta própria linha:** a wave
//! dos tetos levou o `lib.rs` de 682 para 741, e o gate que o mede
//! (`architecture_workspace_file_loc_cap`) mora na `ph2d-editor-core` — nenhum
//! `cargo test -p ph2d-node-motion-collide` o alcança, então a suíte do crate
//! ficou verde por cima. É a mesma causa estrutural que esta casa já registrou
//! quatro vezes; o achado é o arquivo, não o teto.
//!
//! Filho (`#[path]`), não irmão: `use super::*` continua alcançando os privados
//! (`push_apart`, `spread_amount`, `inv_mass`) que os gates medem.

use super::*;

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

/// Relax with a UNIFORM radius and no falloff — the shape the node had before the
/// per-element radius existed, and the fixture almost every gate below wants.
fn push_apart_w(
    p: &[[f32; 2]],
    w: &[f32],
    radius: f32,
    iterations: usize,
    strength: f32,
) -> Vec<[f32; 2]> {
    let n = p.len();
    push_apart(p, w, &vec![radius; n], &vec![1.0; n], iterations, strength)
}

/// Relax with every element free (`w = 1`) — the no-pin case, which is what
/// the packing behaved like before `motion.pin_constraint` existed.
fn push_apart_free(p: &[[f32; 2]], radius: f32, iterations: usize, strength: f32) -> Vec<[f32; 2]> {
    push_apart_w(p, &vec![1.0; p.len()], radius, iterations, strength)
}

/// A crowded cloud: `n` discs on a jittered lattice far tighter than `2·radius`,
/// so almost every pair starts overlapping. Deterministic (a hashed jitter), so
/// the measurement is reproducible.
fn crowded_cloud(n: usize) -> Vec<[f32; 2]> {
    let side = (n as f32).sqrt().ceil() as usize;
    (0..n)
        .map(|i| {
            let (gx, gy) = ((i % side) as f32, (i / side) as f32);
            // splitmix-ish integer hash → jitter in [-0.5, 0.5]
            let mut h = (i as u32).wrapping_mul(0x9e37_79b9);
            h ^= h >> 16;
            h = h.wrapping_mul(0x7feb_352d);
            h ^= h >> 15;
            let jx = (h >> 8) as f32 / 16_777_216.0 - 0.5;
            let jy = (h & 0xffff) as f32 / 65_536.0 - 0.5;
            // Lattice pitch 0.25 with radius 0.3 ⇒ min_dist 0.6: heavy overlap.
            [(gx + jx) * 0.25, (gy + jy) * 0.25]
        })
        .collect()
}

/// The smallest pairwise distance in the cloud, as a FRACTION of `2·radius`.
/// 1.0 = the constraint is met everywhere; below 1.0 = residual overlap.
fn min_gap_ratio(p: &[[f32; 2]], radius: f32) -> f32 {
    let min_dist = 2.0 * radius;
    let mut m = f32::MAX;
    for (i, a) in p.iter().enumerate() {
        for b in &p[i + 1..] {
            m = m.min(dist(*a, *b));
        }
    }
    m / min_dist
}

/// **The measurement that decides the solver** (ADR-0140 Fase 5): packing quality
/// and ORDER-DEPENDENCE of the current scheme. Run:
///   cargo test -p ph2d-node-motion-collide -- --ignored --nocapture
#[test]
#[ignore = "measurement, not a gate"]
fn measure_packing_and_order_dependence() {
    let radius = 0.3;
    let n = 256;
    let p = crowded_cloud(n);
    eprintln!(
        "\nstart: min gap = {:.4}× of 2·radius",
        min_gap_ratio(&p, radius)
    );
    eprintln!("  {:>6}  {:>12}", "iters", "min gap ratio");
    for &iters in &[1usize, 2, 4, 8, 16, 32, 64] {
        let out = push_apart_free(&p, radius, iters, 1.0);
        eprintln!("  {iters:>6}  {:>12.4}", min_gap_ratio(&out, radius));
    }

    // A REALISTIC cloud too: the brutal one above starts at 6% of the required
    // gap, which no scheme fully resolves. This one merely overlaps.
    let loose: Vec<[f32; 2]> = crowded_cloud(n)
        .iter()
        .map(|q| [q[0] * 2.0, q[1] * 2.0])
        .collect();
    eprintln!(
        "\nrealistic cloud: start {:.4}  →  8 iters {:.4}  →  32 iters {:.4}",
        min_gap_ratio(&loose, radius),
        min_gap_ratio(&push_apart_free(&loose, radius, 8, 1.0), radius),
        min_gap_ratio(&push_apart_free(&loose, radius, 32, 1.0), radius),
    );

    // ORDER DEPENDENCE: pack the SAME set of points presented in a different
    // order, then un-permute and compare. A relaxation of a SET should not care
    // how the set was listed. TWO different permutations, because a single one
    // reading 0 could be a fixture accident rather than a property.
    let a = push_apart_free(&p, radius, 8, 1.0);
    let travel = (0..n).map(|i| dist(a[i], p[i])).fold(0.0f32, f32::max);
    for (label, step, off) in [("perm A", 97usize, 13usize), ("perm B", 181, 7)] {
        let perm: Vec<usize> = (0..n).map(|i| (i * step + off) % n).collect();
        // A stride coprime with `n` is a bijection — assert it, so a silent
        // non-permutation cannot make this read 0 by collapsing the comparison.
        let mut seen = vec![false; n];
        for &s in &perm {
            assert!(!seen[s], "{label} is not a permutation");
            seen[s] = true;
        }
        let mut shuffled = vec![[0.0f32; 2]; n];
        for (k, &src) in perm.iter().enumerate() {
            shuffled[k] = p[src];
        }
        let b = push_apart_free(&shuffled, radius, 8, 1.0);
        let mut worst = 0.0f32;
        for (k, &src) in perm.iter().enumerate() {
            worst = worst.max(dist(a[src], b[k]));
        }
        eprintln!(
            "order dependence ({label}): worst |Δpos| = {worst:.6} \
             ({:.1}% of 2·radius)  [points travelled up to {travel:.4}]",
            100.0 * worst / (2.0 * radius)
        );
    }
}

/// **The packing is a fact about the SET, not about the listing** — the property
/// averaged Jacobi bought (ADR-0140 Fase 5). Pack a crowded cloud, then pack the
/// same points presented in a different order, un-permute, and compare.
///
/// The in-place Gauss–Seidel this replaced fails here by **6.11 world units**
/// (1018 % of a disc diameter), because each pair saw the corrections of pairs
/// already visited. The tolerance is a tight ε rather than the exact `0.0` this
/// fixture happens to produce: reordering changes the ORDER of a float summation,
/// which is allowed to move the last bits — pinning bit-equality would be
/// over-fitting this cloud.
#[test]
fn the_packing_does_not_depend_on_the_order_of_the_stream() {
    let radius = 0.3;
    let n = 256;
    let p = crowded_cloud(n);
    let a = push_apart_free(&p, radius, 8, 1.0);
    // The cloud must actually MOVE, or "unchanged under permutation" is vacuous.
    let travel = (0..n).map(|i| dist(a[i], p[i])).fold(0.0f32, f32::max);
    assert!(
        travel > 0.1,
        "the fixture must actually pack: travel {travel}"
    );

    for (step, off) in [(97usize, 13usize), (181, 7)] {
        let perm: Vec<usize> = (0..n).map(|i| (i * step + off) % n).collect();
        // Coprime stride ⇒ bijection. Asserted, so a silent non-permutation
        // cannot make this pass by collapsing the comparison.
        let mut seen = vec![false; n];
        for &s in &perm {
            assert!(!seen[s], "not a permutation");
            seen[s] = true;
        }
        let shuffled: Vec<[f32; 2]> = perm.iter().map(|&src| p[src]).collect();
        let b = push_apart_free(&shuffled, radius, 8, 1.0);
        for (k, &src) in perm.iter().enumerate() {
            let d = dist(a[src], b[k]);
            assert!(
                d < 1e-4,
                "element {src} packed differently when listed at {k}: |Δ| {d}"
            );
        }
    }
}

/// Two overlapping discs are pushed apart until they merely touch (distance =
/// 2·radius). FALSIFIED if separation were skipped (they stay overlapping).
#[test]
fn overlapping_discs_separate_to_touching() {
    let p = vec![[0.0, 0.0], [0.1, 0.0]]; // 0.1 apart, radius 0.3 → must reach 0.6
    let out = push_apart_free(&p, 0.3, 8, 1.0);
    assert!(
        (dist(out[0], out[1]) - 0.6).abs() < 1e-3,
        "separated to touching: {}",
        dist(out[0], out[1])
    );
}

/// Already-separated discs are left untouched (the constraint only fires on
/// penetration).
#[test]
fn separated_discs_are_unchanged() {
    let p = vec![[0.0, 0.0], [5.0, 0.0]];
    let out = push_apart_free(&p, 0.3, 8, 1.0);
    assert_eq!(out, p, "no overlap -> identity");
}

/// The correction is symmetric: the pair's midpoint is preserved (each disc moves
/// half the penetration).
#[test]
fn separation_preserves_the_midpoint() {
    let p = vec![[1.0, 2.0], [1.2, 2.0]];
    let mid0 = [(p[0][0] + p[1][0]) * 0.5, (p[0][1] + p[1][1]) * 0.5];
    let out = push_apart_free(&p, 0.4, 8, 1.0);
    let mid1 = [(out[0][0] + out[1][0]) * 0.5, (out[0][1] + out[1][1]) * 0.5];
    assert!((mid0[0] - mid1[0]).abs() < 1e-4 && (mid0[1] - mid1[1]).abs() < 1e-4);
}

/// A crowded cluster ends up with no overlapping pair (the packing objective).
#[test]
fn a_cluster_packs_without_overlap() {
    // 9 points bunched inside a 0.4-wide box; radius 0.25 → min_dist 0.5.
    let mut p = Vec::new();
    for i in 0..3 {
        for j in 0..3 {
            p.push([i as f32 * 0.2, j as f32 * 0.2]);
        }
    }
    let out = push_apart_free(&p, 0.25, 32, 1.0);
    for a in 0..out.len() {
        for b in (a + 1)..out.len() {
            assert!(
                dist(out[a], out[b]) > 0.5 - 2e-2,
                "pair {a},{b} still overlaps: {}",
                dist(out[a], out[b])
            );
        }
    }
}

/// Coincident points are split deterministically (no rng): two runs match, and the
/// pair ends up touching rather than stuck on top of each other.
#[test]
fn coincident_points_split_deterministically() {
    let p = vec![[0.0, 0.0], [0.0, 0.0]];
    let a = push_apart_free(&p, 0.3, 8, 1.0);
    let b = push_apart_free(&p, 0.3, 8, 1.0);
    assert_eq!(a, b, "deterministic");
    assert!(
        dist(a[0], a[1]) > 0.5,
        "no longer coincident: {}",
        dist(a[0], a[1])
    );
}

/// A **pinned** disc (`motion.pin_constraint`'s `inv_mass = 0`) is an OBSTACLE:
/// it does not budge, and the free one takes the WHOLE penetration (PBD's
/// proportional split). FALSIFIED if the weights were ignored — the pin would
/// be shoved half a penetration off its anchor, exactly the bug that makes a
/// "fixed" obstacle drift.
#[test]
fn a_pinned_disc_does_not_move_and_the_free_one_takes_it_all() {
    let p = vec![[0.0, 0.0], [0.1, 0.0]];
    let out = push_apart_w(&p, &[0.0, 1.0], 0.3, 8, 1.0);
    assert_eq!(out[0], [0.0, 0.0], "the pinned disc held its ground");
    assert!(
        (dist(out[0], out[1]) - 0.6).abs() < 1e-3,
        "and the pair still ends up touching: {}",
        dist(out[0], out[1])
    );
}

/// Two pinned discs that overlap simply stay overlapping — the constraint
/// cannot be met, and neither may move (no division by a zero weight sum).
#[test]
fn two_pinned_discs_have_no_correction_to_share() {
    let p = vec![[0.0, 0.0], [0.1, 0.0]];
    assert_eq!(push_apart_w(&p, &[0.0, 0.0], 0.3, 8, 1.0), p);
}

/// `strength` 0 (or radius 0) is the identity.
#[test]
fn zero_strength_is_the_identity() {
    let p = vec![[0.0, 0.0], [0.1, 0.0]];
    assert_eq!(push_apart_free(&p, 0.3, 8, 0.0), p);
    assert_eq!(push_apart_free(&p, 0.0, 8, 1.0), p);
}

/// Deterministic + cooks through the registry: count and columns pass through, and
/// the overlapping pair separates. `spread` scales the radius.
#[test]
fn registers_and_separates_through_the_cook() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.collide.test.src"),
        name: "motion.collide.test.src",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [0.1, 0.0]]))
                    .with("size", Column::Vec2(vec![[1.0, 1.0], [1.0, 1.0]])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionCollide),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.collide.test.src");
    let c = g.add_node("motion.collide");
    g.set_param(c, "radius", 0.3);
    g.connect(Edge {
        from: (src, 0),
        to: (c, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
    let s = out[0].as_stream();
    assert_eq!(s.count(), 2, "count preserved");
    assert!(s.get("size").is_some(), "columns pass through");
    match s.get("P").unwrap() {
        Column::Vec2(v) => {
            let d = dist(v[0], v[1]);
            // 2·radius·1 — o CONTROLE do raio por elemento: com `size` na
            // identidade a lei e' a de sempre, byte a byte.
            assert!((d - 0.6).abs() < 1e-2, "separated through the cook: {d}");
        }
        _ => panic!("P"),
    }
}
