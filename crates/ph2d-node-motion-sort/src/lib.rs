#![forbid(unsafe_code)]
//! `motion.sort` — **reorder the instance stream by a key**: the Houdini / TouchDesigner
//! "Sort SOP" (Motion Nodes M3, structure — doc 01 §3 / doc 27). Every other node
//! transforms values in place or grows the count; this is the first that **reorders**.
//! On its own it looks like nothing changed — its purpose is to set the *order* a
//! downstream index-based effector reveals in: `sort` (radial) → `cull` (a growing
//! fraction) is a wipe that fills from the centre out; `sort` (random) → `cull` is a
//! dissolve.
//!
//! **Algorithm — a stable sort of the element permutation by a computed key.** Each
//! element gets a key from `key` — **Radial** (squared distance from `(center_x,
//! center_y)`; squared, so no `sqrt` and the same order), **X**, **Y**, **Random** (a
//! splitmix hash of the index → a deterministic shuffle), or **Index** (identity) — and
//! the permutation is stably sorted ascending (`descending` reverses it). Every column
//! (`P`, `size`, `tint`, `id`, …) is reordered by that permutation, so the whole
//! instance travels together. The count is unchanged. Transcendental-free (HR-5):
//! comparison keys are arithmetic (Radial uses squared distance) + the integer hash.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::rand01;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Sort keys (the `key` param).
const KEY_RADIAL: i64 = 0;
const KEY_X: i64 = 1;
const KEY_Y: i64 = 2;
const KEY_RANDOM: i64 = 3;
// KEY_INDEX (4) = identity — any other value falls through to it.

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sort"),
    name: "motion.sort",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Radial · 1 X · 2 Y · 3 Random · 4 Index (identity).
        ParamSpec {
            name: "key",
            default: 0.0,
        },
        // 0 ascending · 1 descending.
        ParamSpec {
            name: "descending",
            default: 0.0,
        },
        // Centre for the Radial key (world units).
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        // Seed for the Random key.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The sort key for each element, given the positions and params.
fn keys(p: &[[f32; 2]], key: i64, center: [f32; 2], seed: u32) -> Vec<f32> {
    (0..p.len())
        .map(|i| match key {
            KEY_RADIAL => {
                let (dx, dy) = (p[i][0] - center[0], p[i][1] - center[1]);
                dx * dx + dy * dy
            }
            KEY_X => p[i][0],
            KEY_Y => p[i][1],
            KEY_RANDOM => rand01(seed, i as u32, 0),
            _ => i as f32, // Index (identity)
        })
        .collect()
}

/// The stable-sorted permutation of `0..n` by `k` (ascending; reversed if `descending`).
fn permutation(k: &[f32], descending: bool) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..k.len()).collect();
    perm.sort_by(|&a, &b| k[a].total_cmp(&k[b]));
    if descending {
        perm.reverse();
    }
    perm
}

/// Reorder a column by the permutation (`out[i] = col[perm[i]]`).
fn permute(col: &Column, perm: &[usize]) -> Column {
    fn gather<T: Clone>(v: &[T], perm: &[usize]) -> Vec<T> {
        perm.iter().map(|&i| v[i].clone()).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(gather(v, perm)),
        Column::Vec2(v) => Column::Vec2(gather(v, perm)),
        Column::Vec3(v) => Column::Vec3(gather(v, perm)),
        Column::Vec4(v) => Column::Vec4(gather(v, perm)),
    }
}

struct MotionSort;

impl NodeOp for MotionSort {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let key = ctx.param("key").round() as i64;
        let descending = ctx.param("descending").round() as i64 != 0;
        let center = [ctx.param("center_x"), ctx.param("center_y")];
        let seed = ctx.param("seed").round().max(0.0) as u32;
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let perm = permutation(&keys(&p, key, center, seed), descending);
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            out.set(name.clone(), permute(col, &perm));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSort))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Sort",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "key",
        label: "Key",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Radial", "X", "Y", "Random", "Index"],
        },
    },
    ParamUiHint {
        param: "descending",
        label: "Descending",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Ascending", "Descending"],
        },
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -20.0,
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
];

#[cfg(test)]
mod tests {
    use super::*;

    const O: [f32; 2] = [0.0, 0.0];

    fn r2(p: [f32; 2]) -> f32 {
        p[0] * p[0] + p[1] * p[1]
    }

    /// Radial sort orders elements by distance from the centre (ascending), and the
    /// result is a permutation of the input (same multiset). FALSIFIED if the output
    /// radii weren't monotone.
    #[test]
    fn radial_sort_orders_by_distance() {
        let p = vec![[3.0, 0.0], [0.0, 1.0], [-2.0, 0.0], [0.5, 0.5]];
        let perm = permutation(&keys(&p, KEY_RADIAL, O, 0), false);
        let sorted: Vec<[f32; 2]> = perm.iter().map(|&i| p[i]).collect();
        for w in sorted.windows(2) {
            assert!(r2(w[0]) <= r2(w[1]), "radii non-decreasing: {sorted:?}");
        }
        // Nearest to the centre (0.5,0.5) is first; farthest (3,0) is last.
        assert_eq!(sorted[0], [0.5, 0.5]);
        assert_eq!(*sorted.last().unwrap(), [3.0, 0.0]);
    }

    /// X sort orders by x; descending reverses it.
    #[test]
    fn x_sort_and_descending() {
        let p = vec![[2.0, 0.0], [-1.0, 0.0], [0.5, 0.0]];
        let asc = permutation(&keys(&p, KEY_X, O, 0), false);
        assert_eq!(asc, vec![1, 2, 0], "ascending by x");
        let desc = permutation(&keys(&p, KEY_X, O, 0), true);
        assert_eq!(desc, vec![0, 2, 1], "descending reverses");
    }

    /// The Random key is a deterministic shuffle: a permutation of the input, the same
    /// every run, and (for a non-trivial set) not the identity order.
    #[test]
    fn random_is_a_deterministic_shuffle() {
        let p: Vec<[f32; 2]> = (0..20).map(|i| [i as f32, 0.0]).collect();
        let a = permutation(&keys(&p, KEY_RANDOM, O, 42), false);
        let b = permutation(&keys(&p, KEY_RANDOM, O, 42), false);
        assert_eq!(a, b, "deterministic");
        assert_ne!(a, (0..20).collect::<Vec<_>>(), "actually shuffled");
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>(), "a permutation");
    }

    /// Deterministic + cooks through the registry: every column is reordered by the same
    /// permutation (P and size travel together).
    #[test]
    fn registers_and_reorders_all_columns() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.sort.test.src"),
            name: "motion.sort.test.src",
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
                // Two elements, far one first; each tagged by a distinct size.
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[5.0, 0.0], [1.0, 0.0]]))
                        .with("size", Column::Scalar(vec![9.0, 1.0])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionSort),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.sort.test.src");
        let s = g.add_node("motion.sort");
        g.set_param(s, "key", KEY_RADIAL as f32);
        g.connect(Edge {
            from: (src, 0),
            to: (s, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, s, 0.0).unwrap();
        let st = out[0].as_stream();
        // The near element (P=(1,0), size 1) sorts first, dragging its size with it.
        match (st.get("P").unwrap(), st.get("size").unwrap()) {
            (Column::Vec2(pv), Column::Scalar(sv)) => {
                assert_eq!(pv[0], [1.0, 0.0], "near element first");
                assert_eq!(sv[0], 1.0, "its size travelled with it");
            }
            _ => panic!("columns"),
        }
    }
}
