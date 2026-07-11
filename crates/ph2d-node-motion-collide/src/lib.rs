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
//! ends up touching with their midpoint preserved. The projection is swept over every
//! pair, `iterations` times (Gauss–Seidel), which lets a crowded cloud settle into a
//! packing. A **pure relaxation of the input each cook** (no state, like the Voronoi's
//! Lloyd), so a `radius` value input that breathes makes the packing expand and
//! contract — deterministic and replay-safe (HR-5: arithmetic + `sqrt`, no trig).
//! `Effect::Pure`.
//!
//! O(n²·iterations): fine at the node's element budget; the scale path (spatial hash /
//! GPU) is `docs/plans/2026-07-gpu-resident-node-pipeline.md`.

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
        // Gauss–Seidel sweeps over all pairs (more = tighter packing).
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
/// `iterations` times. Returns the relaxed positions; the midpoint of each corrected
/// pair is preserved. A pure function — the whole node.
fn push_apart(p: &[[f32; 2]], radius: f32, iterations: usize, strength: f32) -> Vec<[f32; 2]> {
    let n = p.len();
    let mut q = p.to_vec();
    let min_dist = 2.0 * radius;
    if n < 2 || min_dist <= 0.0 || strength <= 0.0 {
        return q;
    }
    let min_d2 = min_dist * min_dist;
    for _ in 0..iterations {
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = q[j][0] - q[i][0];
                let dy = q[j][1] - q[i][1];
                let d2 = dx * dx + dy * dy;
                if d2 >= min_d2 {
                    continue;
                }
                let (nx, ny, push) = if d2 > EPS {
                    let d = d2.sqrt();
                    ((dx / d), (dy / d), (min_dist - d) * 0.5 * strength)
                } else {
                    // Coincident: split along a deterministic axis (x for even i+j,
                    // y otherwise) so the relaxation stays replay-stable (HR-5, no rng).
                    let half = min_dist * 0.5 * strength;
                    if (i + j) % 2 == 0 {
                        (1.0, 0.0, half)
                    } else {
                        (0.0, 1.0, half)
                    }
                };
                q[i][0] -= nx * push;
                q[i][1] -= ny * push;
                q[j][0] += nx * push;
                q[j][1] += ny * push;
            }
        }
    }
    q
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
        let out_p = push_apart(&p, radius, iterations, strength);
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

    /// Two overlapping discs are pushed apart until they merely touch (distance =
    /// 2·radius). FALSIFIED if separation were skipped (they stay overlapping).
    #[test]
    fn overlapping_discs_separate_to_touching() {
        let p = vec![[0.0, 0.0], [0.1, 0.0]]; // 0.1 apart, radius 0.3 → must reach 0.6
        let out = push_apart(&p, 0.3, 8, 1.0);
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
        let out = push_apart(&p, 0.3, 8, 1.0);
        assert_eq!(out, p, "no overlap -> identity");
    }

    /// The correction is symmetric: the pair's midpoint is preserved (each disc moves
    /// half the penetration).
    #[test]
    fn separation_preserves_the_midpoint() {
        let p = vec![[1.0, 2.0], [1.2, 2.0]];
        let mid0 = [(p[0][0] + p[1][0]) * 0.5, (p[0][1] + p[1][1]) * 0.5];
        let out = push_apart(&p, 0.4, 8, 1.0);
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
        let out = push_apart(&p, 0.25, 32, 1.0);
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
        let a = push_apart(&p, 0.3, 8, 1.0);
        let b = push_apart(&p, 0.3, 8, 1.0);
        assert_eq!(a, b, "deterministic");
        assert!(
            dist(a[0], a[1]) > 0.5,
            "no longer coincident: {}",
            dist(a[0], a[1])
        );
    }

    /// `strength` 0 (or radius 0) is the identity.
    #[test]
    fn zero_strength_is_the_identity() {
        let p = vec![[0.0, 0.0], [0.1, 0.0]];
        assert_eq!(push_apart(&p, 0.3, 8, 0.0), p);
        assert_eq!(push_apart(&p, 0.0, 8, 1.0), p);
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
