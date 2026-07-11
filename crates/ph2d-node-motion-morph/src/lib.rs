//! `motion.morph` — a **vertex crossfade** between two instance streams (Motion
//! Nodes M3, deformers — doc 01 §3 / doc 19). It linearly interpolates each
//! element's position from stream `a` toward stream `b` by a `blend` value, so one
//! layout melts into another: a `motion.grid` into a `motion.fibonacci` spiral, a
//! spiral into a `motion.scatter` cloud. This is the classic morph target /
//! blend-shape reduced to points — the deformer that reshapes by INTERPOLATION
//! rather than by a spatial function (`motion.twist`) or a random field.
//!
//! **The blend is a value input** (the value domain — doc 12), so it can be
//! ANIMATED: wire a `value.lfo` to `blend` and the two shapes trade back and forth
//! (the boot scene does exactly this). `blend = 0` is all `a`, `1` all `b`, and it
//! is per-element (a length-N `blend` morphs each element on its own schedule — a
//! staggered dissolve); unconnected it reads as `0` (all `a`). The output length
//! is the `min` of the two streams (only the PAIRED elements morph, by row order),
//! and the interpolated `P` is emitted (downstream nodes add size/tint).
//!
//! `Pure` — no clock, no state; the animation comes from the `blend` input.
//! Transcendental-free (HR-5): a lerp.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `blend` input — the per-instance scalar field on the `v`
/// column (mirror of `ph2d_node_pulse_counter::VALUE`; kept local, leaf crate).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value stream's column.
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.morph"),
    name: "motion.morph",
    inputs: &[
        PortSpec {
            name: "a",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "b",
            ty: INST_VEC2,
        },
        // The 0..1 crossfade — a value, so it can be animated. Optional:
        // unconnected reads as 0 (all `a`).
        PortSpec {
            name: "blend",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The `blend` for element `i`: **unconnected (empty) → 0.0** (all `a`); length-1
/// broadcasts; length-N is per-element. Clamped to `[0, 1]`.
fn blend_at(vals: &[f32], i: usize) -> f32 {
    let t = match vals.len() {
        0 => 0.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(0.0),
    };
    t.clamp(0.0, 1.0)
}

/// Crossfade the paired positions: element `i` (for `i < min(len)`) is
/// `lerp(a[i], b[i], blend_i)`.
fn morph(a: &[[f32; 2]], b: &[[f32; 2]], blend: &[f32]) -> Vec<[f32; 2]> {
    let n = a.len().min(b.len());
    (0..n)
        .map(|i| {
            let t = blend_at(blend, i);
            [
                a[i][0] + (b[i][0] - a[i][0]) * t,
                a[i][1] + (b[i][1] - a[i][1]) * t,
            ]
        })
        .collect()
}

struct MotionMorph;

impl NodeOp for MotionMorph {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let a = positions(ctx.input(0));
        let b = positions(ctx.input(1));
        let blend: Vec<f32> = match ctx.input(2).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let out = morph(&a, &b, &blend);
        ctx.emit(Stream::new(out.len()).with("P", Column::Vec2(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMorph))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Morph",
            // Transform blue: a spatial deformer (crossfade of two layouts).
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // No params — the crossfade is a value input, so it can be animated.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    /// `blend = 0` is all `a`, `1` all `b`, `0.5` the midpoint — the crossfade.
    #[test]
    fn it_crossfades_from_a_to_b() {
        let a = [[0.0, 0.0], [10.0, 0.0]];
        let b = [[0.0, 10.0], [10.0, 10.0]];
        assert_eq!(
            morph(&a, &b, &[0.0]),
            vec![[0.0, 0.0], [10.0, 0.0]],
            "0 = a"
        );
        assert_eq!(
            morph(&a, &b, &[1.0]),
            vec![[0.0, 10.0], [10.0, 10.0]],
            "1 = b"
        );
        assert_eq!(
            morph(&a, &b, &[0.5]),
            vec![[0.0, 5.0], [10.0, 5.0]],
            "0.5 = midpoint"
        );
    }

    /// Unconnected `blend` (empty) reads as 0 → the output is `a` untouched, so a
    /// bare morph is not a surprise. Values clamp to `[0, 1]`.
    #[test]
    fn an_empty_blend_is_a_and_values_clamp() {
        let a = [[1.0, 2.0]];
        let b = [[9.0, 9.0]];
        assert_eq!(morph(&a, &b, &[]), vec![[1.0, 2.0]], "empty → a");
        assert_eq!(
            morph(&a, &b, &[2.0]),
            vec![[9.0, 9.0]],
            "over-1 clamps to b"
        );
        assert_eq!(
            morph(&a, &b, &[-1.0]),
            vec![[1.0, 2.0]],
            "under-0 clamps to a"
        );
    }

    /// FALSIFICATION of the per-element blend: a length-N `blend` morphs each
    /// element on its OWN schedule (a staggered dissolve), not the whole set at one
    /// rate. Element 0 stays at `a`, element 1 reaches `b`.
    #[test]
    fn a_per_element_blend_staggers_the_morph() {
        let a = [[0.0, 0.0], [0.0, 0.0]];
        let b = [[10.0, 0.0], [0.0, 10.0]];
        assert_eq!(
            morph(&a, &b, &[0.0, 1.0]),
            vec![[0.0, 0.0], [0.0, 10.0]],
            "element 0 held at a, element 1 at b"
        );
    }

    /// Mismatched lengths morph only the PAIRED prefix (`min` of the two) — no
    /// out-of-bounds, no invented points.
    #[test]
    fn mismatched_lengths_morph_the_paired_prefix() {
        let a = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let b = [[0.0, 8.0]]; // only one point in b
        let out = morph(&a, &b, &[1.0]);
        assert_eq!(
            out,
            vec![[0.0, 8.0]],
            "only the first (paired) element morphs"
        );
    }

    /// End to end through the cook: two position sources crossfade at blend 0.5.
    #[test]
    fn morphs_two_sources_through_the_cook() {
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.morph.test.src"),
            name: "motion.morph.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[ph2d_nodegraph::node::ParamSpec {
                name: "y",
                default: 0.0,
            }],
            lowerings: &[LoweringKind::Cpu],
        };
        static VAL: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.morph.test.val"),
            name: "motion.morph.test.val",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: VALUE,
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
                let y = ctx.param("y");
                ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[0.0, y]])));
            }
        }
        struct Val;
        impl NodeOp for Val {
            fn manifest(&self) -> &'static NodeManifest {
                &VAL
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![0.5])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == VAL.id => Some(&Val),
                    t if t == MANIFEST.id => Some(&MotionMorph),
                    _ => None,
                }
            }
        }
        let mut g = Graph::new();
        let a = g.add_node("motion.morph.test.src");
        let b = g.add_node("motion.morph.test.src");
        let blend = g.add_node("motion.morph.test.val");
        let m = g.add_node("motion.morph");
        g.set_param(a, "y", 0.0);
        g.set_param(b, "y", 10.0);
        for (from, port) in [(a, 0), (b, 1), (blend, 2)] {
            g.connect(Edge {
                from: (from, 0),
                to: (m, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 5.0]], "blend 0.5 → midpoint"),
            _ => panic!("P"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
