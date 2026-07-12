#![forbid(unsafe_code)]
//! `motion.make_point` — **build positions from value fields**: the adapter that turns
//! two per-instance scalars into geometry (Motion Nodes M1, adapters — doc 01 §1.7 /
//! doc 31). The bridge FROM the value domain INTO geometry — the distributions mint `P`
//! from *params*; this mints `P` from *data*. Feed it two `value.lfo`s (with a per-
//! instance `phase_stagger`) and it plots a **Lissajous**; feed it a `value.instance_
//! field` and a formula chain and it plots any parametric curve.
//!
//! **Algorithm — `P[i] = (x[i], y[i])`.** The `x` and `y` **value** inputs are read per
//! instance (a length-1 field broadcasts to all, a length-N field maps element-wise, an
//! absent one reads 0). The count is the largest of the `in` stream and the two fields —
//! so a stream on `in` fixes the cardinality (and its columns pass through), or the value
//! fields' own length does. Transcendental-free (HR-5): a coordinate pack, no maths.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `x`/`y` inputs (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.make_point"),
    name: "motion.make_point",
    inputs: &[
        // Optional cardinality carrier: its count and columns pass through. Unconnected →
        // the count comes from the value fields.
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "x",
            ty: VALUE,
        },
        PortSpec {
            name: "y",
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

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Read value field `v` at element `i` of `n`: empty → 0, length-1 → broadcast, else the
/// element (0 past the end).
fn at(v: &[f32], i: usize) -> f32 {
    match v.len() {
        0 => 0.0,
        1 => v[0],
        _ => v.get(i).copied().unwrap_or(0.0),
    }
}

/// Pack `x`/`y` into `n` positions.
fn make_points(x: &[f32], y: &[f32], n: usize) -> Vec<[f32; 2]> {
    (0..n).map(|i| [at(x, i), at(y, i)]).collect()
}

struct MotionMakePoint;

impl NodeOp for MotionMakePoint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let x = scalar_col(ctx.input(1), VALUE_COL);
        let y = scalar_col(ctx.input(2), VALUE_COL);
        let in_stream = ctx.input(0);
        let in_count = in_stream.count();
        let n = in_count.max(x.len()).max(y.len());
        let positions = make_points(&x, &y, n);
        let mut out = Stream::new(n);
        // Carry the `in` columns through only when it set the count (they line up).
        if in_count == n {
            for (name, col) in in_stream.columns() {
                if name != "P" {
                    out.set(name.clone(), col.clone());
                }
            }
        }
        out.set("P", Column::Vec2(positions));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMakePoint))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Make Point",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two length-N fields pack element-wise into `P = (x, y)`. FALSIFIED if it zipped
    /// them wrong (e.g. transposed).
    #[test]
    fn packs_x_and_y_element_wise() {
        let p = make_points(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0], 3);
        assert_eq!(p, vec![[1.0, 4.0], [2.0, 5.0], [3.0, 6.0]]);
    }

    /// A length-1 field broadcasts to every element (a column of constant x).
    #[test]
    fn a_length_one_field_broadcasts() {
        let p = make_points(&[5.0], &[1.0, 2.0, 3.0], 3);
        assert_eq!(p, vec![[5.0, 1.0], [5.0, 2.0], [5.0, 3.0]]);
    }

    /// A missing field reads as 0 (a bare x becomes a horizontal line on y = 0).
    #[test]
    fn a_missing_field_is_zero() {
        let p = make_points(&[1.0, 2.0], &[], 2);
        assert_eq!(p, vec![[1.0, 0.0], [2.0, 0.0]]);
    }

    /// Deterministic + cooks through the registry: the `in` stream fixes the count and
    /// the value fields become `P`.
    #[test]
    fn registers_and_makes_points_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static CARRIER: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.make_point.test.in"),
            name: "motion.make_point.test.in",
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
        static XF: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.make_point.test.x"),
            name: "motion.make_point.test.x",
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
        struct Carrier;
        impl NodeOp for Carrier {
            fn manifest(&self) -> &'static NodeManifest {
                &CARRIER
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
            }
        }
        struct Xf;
        impl NodeOp for Xf {
            fn manifest(&self) -> &'static NodeManifest {
                &XF
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with(VALUE_COL, Column::Scalar(vec![7.0, 8.0])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == CARRIER.id => Some(&Carrier),
                    t if t == XF.id => Some(&Xf),
                    t if t == MANIFEST.id => Some(&MotionMakePoint),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let carrier = g.add_node("motion.make_point.test.in");
        let xf = g.add_node("motion.make_point.test.x");
        let mp = g.add_node("motion.make_point");
        g.connect(Edge {
            from: (carrier, 0),
            to: (mp, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (xf, 0),
            to: (mp, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, mp, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(
                v,
                &vec![[7.0, 0.0], [8.0, 0.0]],
                "x field -> P.x, y absent -> 0"
            ),
            _ => panic!("P"),
        }
    }
}
