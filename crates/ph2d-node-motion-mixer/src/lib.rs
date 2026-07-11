#![forbid(unsafe_code)]
//! `motion.mixer` — **blend several instance streams element-wise**: the Houdini
//! "Attribute Interpolate" / "Sequence Blend" (Motion Nodes M1, streams — doc 01 §1.7 /
//! doc 30). Where `motion.combine` stacks streams end to end, this fuses them per
//! element: element `i` of the output is the average / sum / lerp of element `i` across
//! the inputs. **Blend** two layouts and a `value.lfo` morphs one into the other (a grid
//! into a ring); **Avg** blends up to four at once.
//!
//! **Algorithm — element-wise reduction over the common columns.** The count is the
//! **minimum** across the contributing inputs (the extra tail of a longer input is
//! dropped — the Sequence-Blend convention). Every column present in **all** contributing
//! inputs is reduced: **Avg** = mean, **Add** = sum (both over all non-empty inputs);
//! **Blend** = `lerp(in0, in1, blend)` with the `blend` value input (0..1, unconnected →
//! 0.5). Transcendental-free (HR-5): component arithmetic. `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `blend` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Mix modes (the `mode` param). Avg is `0` — the default arm of the reduce match, so it
/// needs no named constant in production (the test module names it for readability).
const MODE_ADD: i64 = 1;
/// Blend mode: `lerp(in0, in1, blend)`.
const MODE_BLEND: i64 = 2;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mixer"),
    name: "motion.mixer",
    inputs: &[
        PortSpec {
            name: "in0",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in1",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in2",
            ty: INST_VEC2,
        },
        PortSpec {
            name: "in3",
            ty: INST_VEC2,
        },
        // Blend weight for the Blend mode (animatable). Optional: unconnected → 0.5.
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
    params: &[
        // 0 Avg · 1 Add · 2 Blend (in0→in1 by the blend input).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// A cloned snapshot of one input.
struct Snap {
    count: usize,
    cols: Vec<(String, Column)>,
}

impl Snap {
    fn column(&self, name: &str) -> Option<&Column> {
        self.cols.iter().find(|(n, _)| n == name).map(|(_, c)| c)
    }
}

fn snapshot(s: &Stream) -> Snap {
    Snap {
        count: s.count(),
        cols: s.columns().map(|(n, c)| (n.clone(), c.clone())).collect(),
    }
}

/// A column truncated to the first `n` rows.
fn trunc(c: &Column, n: usize) -> Column {
    match c {
        Column::Scalar(v) => Column::Scalar(v[..n].to_vec()),
        Column::Vec2(v) => Column::Vec2(v[..n].to_vec()),
        Column::Vec3(v) => Column::Vec3(v[..n].to_vec()),
        Column::Vec4(v) => Column::Vec4(v[..n].to_vec()),
    }
}

/// Component-wise `a + b·k` (same variant, same length).
fn add_scaled(a: &Column, b: &Column, k: f32) -> Column {
    macro_rules! z {
        ($va:expr, $vb:expr, $w:literal) => {{
            $va.iter()
                .zip($vb.iter())
                .map(|(x, y)| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] += y[c] * k;
                    }
                    r
                })
                .collect()
        }};
    }
    match (a, b) {
        (Column::Scalar(x), Column::Scalar(y)) => {
            Column::Scalar(x.iter().zip(y).map(|(a, b)| a + b * k).collect())
        }
        (Column::Vec2(x), Column::Vec2(y)) => Column::Vec2(z!(x, y, 2)),
        (Column::Vec3(x), Column::Vec3(y)) => Column::Vec3(z!(x, y, 3)),
        (Column::Vec4(x), Column::Vec4(y)) => Column::Vec4(z!(x, y, 4)),
        _ => a.clone(),
    }
}

/// Component-wise scale.
fn scale(c: &Column, k: f32) -> Column {
    macro_rules! s {
        ($v:expr, $w:literal) => {{
            $v.iter()
                .map(|x| {
                    let mut r = *x;
                    for c in 0..$w {
                        r[c] *= k;
                    }
                    r
                })
                .collect()
        }};
    }
    match c {
        Column::Scalar(v) => Column::Scalar(v.iter().map(|x| x * k).collect()),
        Column::Vec2(v) => Column::Vec2(s!(v, 2)),
        Column::Vec3(v) => Column::Vec3(s!(v, 3)),
        Column::Vec4(v) => Column::Vec4(s!(v, 4)),
    }
}

/// Column names present in every contributing snapshot, in the first input's order.
fn common_columns(snaps: &[&Snap]) -> Vec<String> {
    let Some(first) = snaps.first() else {
        return Vec::new();
    };
    first
        .cols
        .iter()
        .map(|(n, _)| n.clone())
        .filter(|n| snaps.iter().all(|s| s.column(n).is_some()))
        .collect()
}

/// Reduce the contributing inputs into one stream. `blend` is only used in Blend mode.
fn mix(mode: i64, contributing: &[&Snap], blend: f32) -> Stream {
    if contributing.is_empty() {
        return Stream::new(0);
    }
    let count = contributing.iter().map(|s| s.count).min().unwrap_or(0);
    let mut out = Stream::new(count);
    if count == 0 {
        return out;
    }
    for name in common_columns(contributing) {
        let cols: Vec<Column> = contributing
            .iter()
            .map(|s| trunc(s.column(&name).unwrap(), count))
            .collect();
        let mixed = match mode {
            MODE_BLEND if cols.len() >= 2 => {
                add_scaled(&scale(&cols[0], 1.0 - blend), &cols[1], blend)
            }
            MODE_ADD => cols
                .iter()
                .skip(1)
                .fold(cols[0].clone(), |acc, c| add_scaled(&acc, c, 1.0)),
            _ => {
                // Avg (and Blend with a single input): mean over the inputs.
                let sum = cols
                    .iter()
                    .skip(1)
                    .fold(cols[0].clone(), |acc, c| add_scaled(&acc, c, 1.0));
                scale(&sum, 1.0 / cols.len() as f32)
            }
        };
        out.set(name, mixed);
    }
    out
}

struct MotionMixer;

impl NodeOp for MotionMixer {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i64;
        let blend = match ctx.input(4).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.5),
            _ => 0.5,
        };
        // Snapshot the four stream inputs, one at a time.
        let snaps: Vec<Snap> = (0..4u16)
            .map(|k| snapshot(ctx.input(k as usize)))
            .filter(|s| s.count > 0)
            .collect();
        // Blend uses only the first two inputs; Avg/Add use all non-empty.
        let contributing: Vec<&Snap> = if mode == MODE_BLEND {
            snaps.iter().take(2).collect()
        } else {
            snaps.iter().collect()
        };
        ctx.emit(mix(mode, &contributing, blend));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMixer))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mixer",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "mode",
    label: "Mode",
    min: 0.0,
    max: 2.0,
    step: 1.0,
    widget: ParamWidget::Enum {
        labels: &["Avg", "Add", "Blend"],
    },
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// Avg mode (the production default arm), named here for the tests' readability.
    const MODE_AVG: i64 = 0;

    fn snap_p(p: Vec<[f32; 2]>) -> Snap {
        Snap {
            count: p.len(),
            cols: vec![("P".to_string(), Column::Vec2(p))],
        }
    }

    fn p_of(s: &Stream) -> Vec<[f32; 2]> {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v.clone(),
            _ => panic!("P"),
        }
    }

    /// Avg is the midpoint of the inputs: two points averaged land halfway between.
    #[test]
    fn avg_is_the_midpoint() {
        let a = snap_p(vec![[0.0, 0.0], [2.0, 0.0]]);
        let b = snap_p(vec![[4.0, 0.0], [2.0, 4.0]]);
        let out = mix(MODE_AVG, &[&a, &b], 0.5);
        assert_eq!(p_of(&out), vec![[2.0, 0.0], [2.0, 2.0]]);
    }

    /// Add sums the inputs component-wise.
    #[test]
    fn add_sums_the_inputs() {
        let a = snap_p(vec![[1.0, 1.0]]);
        let b = snap_p(vec![[2.0, 3.0]]);
        let out = mix(MODE_ADD, &[&a, &b], 0.5);
        assert_eq!(p_of(&out), vec![[3.0, 4.0]]);
    }

    /// Blend lerps in0→in1: weight 0 is in0, 1 is in1, 0.25 is a quarter across.
    /// FALSIFIED by an averaging that ignores the weight.
    #[test]
    fn blend_lerps_in0_to_in1() {
        let a = snap_p(vec![[0.0, 0.0]]);
        let b = snap_p(vec![[4.0, 8.0]]);
        assert_eq!(p_of(&mix(MODE_BLEND, &[&a, &b], 0.0)), vec![[0.0, 0.0]]);
        assert_eq!(p_of(&mix(MODE_BLEND, &[&a, &b], 1.0)), vec![[4.0, 8.0]]);
        assert_eq!(p_of(&mix(MODE_BLEND, &[&a, &b], 0.25)), vec![[1.0, 2.0]]);
    }

    /// Mismatched counts blend the common prefix (the minimum count).
    #[test]
    fn count_is_the_minimum() {
        let a = snap_p(vec![[0.0, 0.0], [0.0, 0.0], [0.0, 0.0]]);
        let b = snap_p(vec![[2.0, 2.0]]);
        let out = mix(MODE_AVG, &[&a, &b], 0.5);
        assert_eq!(out.count(), 1, "truncated to the shorter input");
    }

    /// Deterministic + cooks through the registry: two sources blend to their midpoint at
    /// blend 0.5.
    #[test]
    fn registers_and_mixes_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        const fn src(id: &'static str) -> NodeManifest {
            NodeManifest {
                id: NodeTypeId::of(id),
                name: id,
                inputs: &[],
                outputs: &[PortSpec {
                    name: "out",
                    ty: INST_VEC2,
                }],
                effect: Effect::Pure,
                clock: Clock::Frame,
                params: &[] as &[ParamSpec],
                lowerings: &[LoweringKind::Cpu],
            }
        }
        static SA: NodeManifest = src("motion.mixer.test.a");
        static SB: NodeManifest = src("motion.mixer.test.b");
        struct A;
        impl NodeOp for A {
            fn manifest(&self) -> &'static NodeManifest {
                &SA
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]])));
            }
        }
        struct B;
        impl NodeOp for B {
            fn manifest(&self) -> &'static NodeManifest {
                &SB
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[4.0, 0.0], [4.0, 0.0]])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SA.id => Some(&A),
                    t if t == SB.id => Some(&B),
                    t if t == MANIFEST.id => Some(&MotionMixer),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let a = g.add_node("motion.mixer.test.a");
        let b = g.add_node("motion.mixer.test.b");
        let m = g.add_node("motion.mixer");
        g.connect(Edge {
            from: (a, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (b, 0),
            to: (m, 1),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [2.0, 0.0], "midpoint of the two sources"),
            _ => panic!("P"),
        }
    }
}
