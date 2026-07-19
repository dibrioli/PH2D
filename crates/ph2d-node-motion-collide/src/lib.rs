#![forbid(unsafe_code)]
//! `motion.collide` — **push apart**: relax a layout so no two instances overlap, the
//! Cinema 4D "Push Apart Effector" / a circle-packing separation (Motion Nodes M3,
//! distributions — doc 01 §3 / doc 26). Distinct from `motion.voronoi` (which spreads
//! points to *uniform density* via Lloyd/CVT): this enforces a *hard radius* — every
//! instance is a disc of radius `radius`, and overlapping pairs are pushed off each
//! other until they merely touch.
//!
//! **Algorithm — the Position Based Dynamics non-penetration contact constraint**
//! (Müller et al., *Position Based Dynamics*, 2007; the relaxation is Jakobsen,
//! *Advanced Character Physics*, 2001). For each pair closer than `2·radius`, the
//! constraint gradient is the contact normal (the unit vector between them) and the
//! correction is half the penetration each, moved apart along that normal — so the pair
//! ends up touching with their midpoint preserved. A **pure relaxation of the input each
//! cook** (no state, like the Voronoi's Lloyd), so a `radius` value input that breathes
//! makes the packing expand and contract — deterministic and replay-safe (HR-5:
//! arithmetic + `sqrt`, no trig). `Effect::Pure`.
//!
//! ## The sweep is AVERAGED JACOBI, not Gauss–Seidel (ADR-0134 Fase 5)
//!
//! Each `iterations` sweep reads ONE snapshot of the positions, accumulates every
//! contact's requested correction per disc, and then applies the **average** of what
//! that disc's contacts asked for (mass splitting — Macklin & Müller, *Unified Particle
//! Physics*, 2014, which is what FleX ships). Averaging is what makes Jacobi stable:
//! summing raw would launch a disc with many contacts across the scene, because every
//! neighbour independently asks for the full push.
//!
//! It replaced an in-place Gauss–Seidel sweep, and the reason is **correctness before
//! speed**: Gauss–Seidel mutates `q[i]`/`q[j]` inside the pair loop, so each pair sees
//! the corrections of pairs already visited — which makes the result depend on the
//! **index order of the stream**. Measured on a crowded cloud of 256 discs, the same
//! SET of points merely listed in a different order packed up to **6.11 world units**
//! apart (1018 % of a disc diameter); the artist neither controls nor sees that order.
//! Averaged Jacobi is order-independent (measured: **0.0**), and on the shipped default
//! of 8 iterations it also packs BETTER (min gap 0.270 vs 0.050 of the required
//! `2·radius`; Gauss–Seidel only overtakes past ~32 iterations on a pathological cloud).
//! It is additionally the scheme a GPU can run at all — every thread reads the same
//! snapshot — which is what lets the spatial-grid port exist.
//!
//! O(n²·iterations) here; the device path (spatial hash on the GPU) is ADR-0134.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `spread` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";
/// The inverse-mass column (PBD's `w = 1/m`) that `motion.pin_constraint` writes:
/// `1` = free (the default when absent — every pre-pin packing is unchanged),
/// `0` = pinned. A string convention shared by the module's solvers, spelled
/// locally by each reader (like `P` / `falloff`) rather than coupling the crates.
const INV_MASS_COL: &str = "inv_mass";

/// Below this a pair is treated as coincident (the normal is undefined).
const EPS: f32 = 1e-9;
/// A hard cap on the relaxation sweeps.
const MAX_ITERATIONS: i64 = 64;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.collide"),
    name: "motion.collide",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // A multiplier on `radius` (animatable): unconnected reads as 1. A `value.lfo`
        // makes the packing breathe.
        PortSpec {
            name: "spread",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // The disc radius: pairs closer than 2·radius are pushed apart.
        ParamSpec {
            name: "radius",
            default: 0.3,
        },
        // Averaged-Jacobi sweeps over all pairs (more = tighter packing).
        ParamSpec {
            name: "iterations",
            default: 8.0,
        },
        // Relaxation factor per sweep (1 = full correction; <1 softens/settles).
        ParamSpec {
            name: "strength",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The `spread` multiplier: unconnected (empty) → 1.0; else the first element.
fn spread_amount(vals: &[f32]) -> f32 {
    vals.first().copied().unwrap_or(1.0)
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Push apart the discs so no two are closer than `2·radius`, sweeping every pair
/// `iterations` times. Returns the relaxed positions. A pure function — the whole
/// node.
///
/// `w` is the per-element inverse mass (PBD's `w = 1/m`, written by
/// `motion.pin_constraint`; all-`1` when no pin is wired). The contact correction
/// is split between the pair **in proportion to their `w`s**, which is the
/// constraint-projection rule of Müller et al. 2007 — with two free elements each
/// takes half (the midpoint of the pair is preserved, bit-for-bit as before the
/// pin existed), and against a pinned element (`w = 0`, infinite mass) the free one
/// takes the whole penetration and the pin does not budge. That is what makes a
/// pinned disc an OBSTACLE the others pack around.
fn push_apart(
    p: &[[f32; 2]],
    w: &[f32],
    radius: f32,
    iterations: usize,
    strength: f32,
) -> Vec<[f32; 2]> {
    let n = p.len();
    let mut q = p.to_vec();
    let min_dist = 2.0 * radius;
    if n < 2 || min_dist <= 0.0 || strength <= 0.0 {
        return q;
    }
    let min_d2 = min_dist * min_dist;
    // Averaged-Jacobi scratch, allocated ONCE: the summed correction each disc is
    // asked for this sweep, and how many contacts asked.
    let mut delta = vec![[0.0f32; 2]; n];
    let mut contacts = vec![0u32; n];
    for _ in 0..iterations {
        delta.fill([0.0, 0.0]);
        contacts.fill(0);
        // ── gather pass: every pair reads the SAME snapshot `q` ──
        for i in 0..n {
            for j in (i + 1)..n {
                // Two immovable discs (or two infinitely heavy ones) have no
                // correction to share — the constraint simply cannot be met.
                let sum_w = w[i] + w[j];
                if sum_w <= 0.0 {
                    continue;
                }
                let dx = q[j][0] - q[i][0];
                let dy = q[j][1] - q[i][1];
                let d2 = dx * dx + dy * dy;
                if d2 >= min_d2 {
                    continue;
                }
                let (nx, ny, penetration) = if d2 > EPS {
                    let d = d2.sqrt();
                    (dx / d, dy / d, min_dist - d)
                } else {
                    // Coincident: split along a deterministic axis (x for even i+j,
                    // y otherwise) so the relaxation stays replay-stable (HR-5, no rng).
                    // ⚠️ This is the ONE order-dependent corner left, and it is
                    // irreducible: two EXACTLY coincident points carry nothing
                    // intrinsic to break the symmetry with, so the index is the only
                    // handle there is. Measure-zero — any jitter escapes it.
                    if (i + j) % 2 == 0 {
                        (1.0, 0.0, min_dist)
                    } else {
                        (0.0, 1.0, min_dist)
                    }
                };
                // Each disc is ASKED to move its SHARE of the penetration:
                // w_i / (w_i + w_j). Both free = half each, so the pair's midpoint is
                // preserved exactly as before.
                let push_i = penetration * (w[i] / sum_w) * strength;
                let push_j = penetration * (w[j] / sum_w) * strength;
                delta[i][0] -= nx * push_i;
                delta[i][1] -= ny * push_i;
                delta[j][0] += nx * push_j;
                delta[j][1] += ny * push_j;
                contacts[i] += 1;
                contacts[j] += 1;
            }
        }
        // ── apply pass: each disc takes the AVERAGE of what its contacts asked ──
        // Averaging (mass splitting, Macklin & Müller 2014) is what makes Jacobi
        // stable: summing raw would launch a disc with many contacts across the
        // scene, because every neighbour independently asks for the full push.
        for i in 0..n {
            let c = contacts[i];
            if c > 0 {
                let inv = 1.0 / c as f32;
                q[i][0] += delta[i][0] * inv;
                q[i][1] += delta[i][1] * inv;
            }
        }
    }
    q
}

/// The inverse-mass column (`motion.pin_constraint`), widened to `n` and made
/// safe: absent reads as free (`1`), and a negative or non-finite weight from a
/// hand-edited document reads as pinned (`0`) rather than INVERTING the push.
fn inv_mass(s: &Stream, n: usize) -> Vec<f32> {
    match s.get(INV_MASS_COL) {
        Some(Column::Scalar(v)) if v.len() == n => v
            .iter()
            .map(|w| if w.is_finite() { w.max(0.0) } else { 0.0 })
            .collect(),
        _ => vec![1.0; n],
    }
}

struct MotionCollide;

impl NodeOp for MotionCollide {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let base_radius = ctx.param("radius");
        let iterations = (ctx.param("iterations").round() as i64).clamp(0, MAX_ITERATIONS) as usize;
        let strength = ctx.param("strength");
        let spread = spread_amount(&scalar_col(ctx.input(1), VALUE_COL));
        let radius = base_radius * spread;
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let w = inv_mass(input, n);
        let out_p = push_apart(&p, &w, radius, iterations, strength);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            if name != "P" {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(out_p));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionCollide))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Collide",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.0,
        max: 5.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "iterations",
        label: "Iterations",
        min: 0.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
        let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
        (dx * dx + dy * dy).sqrt()
    }

    /// Relax with every element free (`w = 1`) — the no-pin case, which is what
    /// the packing behaved like before `motion.pin_constraint` existed.
    fn push_apart_free(
        p: &[[f32; 2]],
        radius: f32,
        iterations: usize,
        strength: f32,
    ) -> Vec<[f32; 2]> {
        push_apart(p, &vec![1.0; p.len()], radius, iterations, strength)
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

    /// **The measurement that decides the solver** (ADR-0134 Fase 5): packing quality
    /// and ORDER-DEPENDENCE of the current scheme. Run:
    ///   cargo test -p ph2d-node-motion-collide -- --ignored --nocapture
    #[test]
    #[ignore = "measurement, not a gate"]
    fn measure_packing_and_order_dependence() {
        let radius = 0.3;
        let n = 256;
        let p = crowded_cloud(n);
        eprintln!("\nstart: min gap = {:.4}× of 2·radius", min_gap_ratio(&p, radius));
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
    /// averaged Jacobi bought (ADR-0134 Fase 5). Pack a crowded cloud, then pack the
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
        assert!(travel > 0.1, "the fixture must actually pack: travel {travel}");

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
        let out = push_apart(&p, &[0.0, 1.0], 0.3, 8, 1.0);
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
        assert_eq!(push_apart(&p, &[0.0, 0.0], 0.3, 8, 1.0), p);
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
                        .with("size", Column::Vec2(vec![[0.4, 0.4], [0.4, 0.4]])),
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
                assert!((d - 0.6).abs() < 1e-2, "separated through the cook: {d}");
            }
            _ => panic!("P"),
        }
    }
}
