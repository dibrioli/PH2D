#![forbid(unsafe_code)]
//! `motion.voronoi` — an **even, organic** point distribution by **Lloyd's
//! relaxation** toward a centroidal Voronoi tessellation (Motion Nodes M3,
//! distributions — doc 01 §3 / doc 23). Starting from a random cloud, each point is
//! repeatedly moved to the centroid of its Voronoi cell; the set converges to a CVT
//! — the even, hexagonal-ish packing that stippling, dart-boards and blue-noise
//! sampling all want. Distinct from `motion.scatter` (one-shot best-candidate blue
//! noise) and from the crystalline `motion.lattice`: Lloyd is *iterated relaxation*,
//! and a `relax` value input plays that relaxation forward so a `value.lfo` makes the
//! cloud **organise into a honeycomb and dissolve back**.
//!
//! **Algorithm — Lloyd's algorithm (Voronoi iteration), grid-approximated.** The
//! exact method computes each point's Voronoi cell (Fortune's sweep) and moves it to
//! the polygon centroid; the standard practical form (the 2002 weighted-Voronoi
//! stippling method; the CPU cousin of the GPU jump-flood) discretises the domain into
//! a `res²` grid, assigns each sample to its nearest point, and moves each point to the
//! mean of its samples — repeated `iterations` times. The grid `res` scales with the
//! count (≈16 samples/point), so the cost tracks the work. Deterministic and
//! transcendental-free (HR-5: squared distances only, no `sqrt`).
//!
//! **Cost (read before animating).** A cook is `O(iterations · res² · count)`.
//! With `relax` UNCONNECTED (a static CVT) the node cooks once and the memo caches it
//! — free thereafter. But **animating `relax`** (a `value.lfo`) re-runs the *whole*
//! relaxation every frame (the raw seed and relaxed CVT are fixed, yet a Pure node
//! can't cache across frames), so keep `count`/`iterations` modest when it is live —
//! this is a genuinely heavier node than the other distributions.
//!
//! A **Source** node (no stream input, mints `P`). Stateless seed (Jarzynski/Olano
//! hash), so it reproduces bit-for-bit. `Effect::Pure` (no clock — the animation
//! arrives through the `relax` input, which lerps raw seed → relaxed CVT).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `relax` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Target grid samples per point — the Lloyd grid resolution scales so every cell
/// gets roughly this many samples (accurate centroids without over-sampling).
const SAMPLES_PER_POINT: usize = 16;
/// Clamp on the sampling grid side (samples = res²). The lower bound keeps a small
/// count's centroids sane; the upper bound caps the per-eval cost.
const MIN_RES: usize = 20;
const MAX_RES: usize = 96;
/// Cap on the point count (cost is O(iterations · res² · count)).
const MAX_POINTS: usize = 600;

/// The Lloyd sampling grid side for `count` points: `√(count · SAMPLES_PER_POINT)`,
/// clamped. This keeps the grid cost proportional to the work (≈ `count·16` samples)
/// instead of a fixed `64²` — the fix for the per-frame recompute when `relax` is
/// animated (each animated frame re-runs the whole relaxation; see the module docs).
fn resolution(count: usize) -> usize {
    let ideal = ((count * SAMPLES_PER_POINT) as f32).sqrt().ceil() as usize;
    ideal.clamp(MIN_RES, MAX_RES)
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.voronoi"),
    name: "motion.voronoi",
    inputs: &[
        // How far to play Lloyd's relaxation: 0 = the raw random seed, 1 = the fully
        // relaxed CVT (animatable). Optional: unconnected reads as 1 (relaxed).
        PortSpec {
            name: "relax",
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
        ParamSpec {
            name: "count",
            default: 96.0,
        },
        ParamSpec {
            name: "width",
            default: 5.0,
        },
        ParamSpec {
            name: "height",
            default: 5.0,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
        // Lloyd iterations — more = closer to the ideal CVT (it converges fast, so
        // 8 is near-converged; kept modest since it re-runs when `relax` is animated).
        ParamSpec {
            name: "iterations",
            default: 8.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The raw random seed: `count` white-noise points in the `w×h` rectangle (centred).
fn seed_points(count: usize, w: f32, h: f32, seed: u32) -> Vec<[f32; 2]> {
    (0..count as u32)
        .map(|i| [(hash3(seed, i, 0) - 0.5) * w, (hash3(seed, i, 1) - 0.5) * h])
        .collect()
}

/// One Lloyd iteration: assign every grid sample to its nearest point, then move each
/// point to the mean of its assigned samples. Points with no samples hold still.
fn lloyd_step(points: &[[f32; 2]], w: f32, h: f32, res: usize) -> Vec<[f32; 2]> {
    let n = points.len();
    let mut sum = vec![[0.0f32; 2]; n];
    let mut count = vec![0u32; n];
    for gy in 0..res {
        let sy = ((gy as f32 + 0.5) / res as f32 - 0.5) * h;
        for gx in 0..res {
            let sx = ((gx as f32 + 0.5) / res as f32 - 0.5) * w;
            // Nearest point (squared distance — no sqrt).
            let mut best = 0usize;
            let mut best_d = f32::MAX;
            for (j, p) in points.iter().enumerate() {
                let (dx, dy) = (sx - p[0], sy - p[1]);
                let d = dx * dx + dy * dy;
                if d < best_d {
                    best_d = d;
                    best = j;
                }
            }
            sum[best][0] += sx;
            sum[best][1] += sy;
            count[best] += 1;
        }
    }
    points
        .iter()
        .enumerate()
        .map(|(j, p)| {
            if count[j] == 0 {
                *p
            } else {
                [sum[j][0] / count[j] as f32, sum[j][1] / count[j] as f32]
            }
        })
        .collect()
}

/// Relax `count` points to a CVT over `iterations` Lloyd steps, then lerp the raw seed
/// toward the relaxed result by `relax ∈ [0,1]` (so the caller can animate the
/// organise/dissolve). Returns the emitted positions.
fn relaxed_points(
    count: usize,
    w: f32,
    h: f32,
    seed: u32,
    iterations: usize,
    relax: f32,
) -> Vec<[f32; 2]> {
    let raw = seed_points(count, w, h, seed);
    let res = resolution(count);
    let mut relaxed = raw.clone();
    for _ in 0..iterations {
        relaxed = lloyd_step(&relaxed, w, h, res);
    }
    let t = relax.clamp(0.0, 1.0);
    raw.iter()
        .zip(&relaxed)
        .map(|(r, c)| [r[0] + (c[0] - r[0]) * t, r[1] + (c[1] - r[1]) * t])
        .collect()
}

struct MotionVoronoi;

impl NodeOp for MotionVoronoi {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let count = param_as_count(ctx.param("count"), MAX_POINTS);
        let w = ctx.param("width").max(1e-3);
        let h = ctx.param("height").max(1e-3);
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let iterations = (ctx.param("iterations").round() as i64).clamp(0, 64) as usize;
        // Unconnected relax → 1.0 (fully relaxed).
        let relax = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(1.0),
            _ => 1.0,
        };
        let positions = relaxed_points(count, w, h, seed, iterations, relax);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionVoronoi))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Voronoi",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 600.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "width",
        label: "Width",
        min: 0.5,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "height",
        label: "Height",
        min: 0.5,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "iterations",
        label: "Relax Steps",
        min: 0.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The smallest pairwise distance in the set — the packing floor.
    fn min_pair(pts: &[[f32; 2]]) -> f32 {
        let mut m = f32::MAX;
        for (i, a) in pts.iter().enumerate() {
            for b in &pts[i + 1..] {
                let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
                m = m.min((dx * dx + dy * dy).sqrt());
            }
        }
        m
    }

    /// Lloyd's relaxation EVENS the cloud: the fully-relaxed CVT spreads its points so
    /// the smallest pairwise gap is much larger than the raw white-noise seed (which
    /// clumps). FALSIFIED against the raw seed (`relax = 0`), which keeps the clumps.
    #[test]
    fn lloyd_relaxation_evens_the_cloud() {
        let raw = relaxed_points(80, 5.0, 5.0, 1, 12, 0.0);
        let cvt = relaxed_points(80, 5.0, 5.0, 1, 12, 1.0);
        assert!(
            min_pair(&cvt) > min_pair(&raw) * 1.5,
            "the CVT spaces points far better than white noise (cvt {} vs raw {})",
            min_pair(&cvt),
            min_pair(&raw)
        );
    }

    /// `relax` plays the relaxation: 0 is exactly the raw seed, 1 the full CVT, and a
    /// mid value sits strictly between (each point partway along its migration).
    #[test]
    fn relax_lerps_from_raw_seed_to_the_cvt() {
        let raw = relaxed_points(40, 5.0, 5.0, 2, 12, 0.0);
        let cvt = relaxed_points(40, 5.0, 5.0, 2, 12, 1.0);
        let half = relaxed_points(40, 5.0, 5.0, 2, 12, 0.5);
        // The seed at relax 0 is the plain white-noise draw.
        assert_eq!(raw, seed_points(40, 5.0, 5.0, 2), "relax 0 = raw seed");
        // Halfway is the midpoint of each point's raw→cvt migration.
        for i in 0..40 {
            let mid = [(raw[i][0] + cvt[i][0]) * 0.5, (raw[i][1] + cvt[i][1]) * 0.5];
            assert!((half[i][0] - mid[0]).abs() < 1e-4 && (half[i][1] - mid[1]).abs() < 1e-4);
        }
    }

    /// Every point stays inside the domain (the Lloyd centroids never leave it).
    #[test]
    fn all_points_stay_in_the_domain() {
        let pts = relaxed_points(120, 6.0, 4.0, 5, 15, 1.0);
        assert_eq!(pts.len(), 120);
        for p in &pts {
            assert!(
                p[0].abs() <= 3.0 + 1e-3 && p[1].abs() <= 2.0 + 1e-3,
                "outside the 6×4 domain: {p:?}"
            );
        }
    }

    /// Deterministic (scrub-stable), and a new seed re-rolls the whole cloud.
    #[test]
    fn same_seed_reproduces_and_a_new_seed_re_rolls() {
        assert_eq!(
            relaxed_points(50, 5.0, 5.0, 1, 10, 1.0),
            relaxed_points(50, 5.0, 5.0, 1, 10, 1.0),
        );
        assert_ne!(
            relaxed_points(50, 5.0, 5.0, 1, 10, 1.0),
            relaxed_points(50, 5.0, 5.0, 2, 10, 1.0),
        );
    }

    /// Cooks through the registry and emits `P` at the requested count, `relax`
    /// unconnected (→ fully relaxed).
    #[test]
    fn registers_and_cooks() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionVoronoi as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let n = g.add_node("motion.voronoi");
        g.set_param(n, "count", 24.0);
        g.set_param(n, "iterations", 5.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 24),
            _ => panic!("P"),
        }
    }
}
