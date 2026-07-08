#![forbid(unsafe_code)]
//! `motion.tint` — a Motion **colour modifier**: sets the `tint` (Vec4 RGBA,
//! linear straight — §1.2) attribute to the target colour `(r, g, b, a)`,
//! masked per-instance by the multiplicative `falloff` column. The per-instance
//! result is `lerp(existing, (r,g,b,a), falloff_i)`, so at `falloff = 1` the
//! instance takes the target colour **exactly** (any RGBA, alpha included) and
//! at `falloff = 0` it keeps its existing tint (absent → opaque white). Every
//! other column passes through unchanged (count preserved). Pure.
//!
//! Params (read via `ctx.param`): `r` (1.0), `g` (0.3), `b` (0.1), `a` (1.0) —
//! a warm opaque default so the colour reads immediately once wired.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.tint"),
    name: "motion.tint",
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
        ParamSpec {
            name: "r",
            default: 1.0,
        },
        ParamSpec {
            name: "g",
            default: 0.3,
        },
        ParamSpec {
            name: "b",
            default: 0.1,
        },
        ParamSpec {
            name: "a",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The falloff-masked colour for one instance: `lerp(existing, target, f)` per
/// RGBA channel via the endpoint-exact form `existing·(1-f) + target·f` — so it
/// returns exactly `existing` at `f = 0` and exactly `target` at `f = 1` (any
/// colour + alpha, no float drift).
fn mixed_tint(existing: [f32; 4], target: [f32; 4], f: f32) -> [f32; 4] {
    let lerp = |e: f32, t: f32| e * (1.0 - f) + t * f;
    [
        lerp(existing[0], target[0]),
        lerp(existing[1], target[1]),
        lerp(existing[2], target[2]),
        lerp(existing[3], target[3]),
    ]
}

struct MotionTint;

impl NodeOp for MotionTint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let target = [
            ctx.param("r"),
            ctx.param("g"),
            ctx.param("b"),
            ctx.param("a"),
        ];
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance tint (absent column → opaque white).
            let base: Vec<[f32; 4]> = match input.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => vec![[1.0, 1.0, 1.0, 1.0]; n],
            };
            let tinted: Vec<[f32; 4]> = (0..n)
                .map(|i| {
                    let e = base.get(i).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
                    mixed_tint(e, target, falloff_at(input, i))
                })
                .collect();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "tint" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("tint", Column::Vec4(tinted));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionTint))?;
    // M1.R1 — UI metadata (a colour effect → magenta Fx, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Tint",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): four linear RGBA channels in `0..1`.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "r",
        label: "Red",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "g",
        label: "Green",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "b",
        label: "Blue",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "a",
        label: "Alpha",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 white instances with falloff [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.tint.test.src"),
        name: "motion.tint.test.src",
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
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 1.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionTint),
                _ => None,
            }
        }
    }

    #[test]
    fn tint_sets_target_masked_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.tint.test.src");
        let tn = g.add_node("motion.tint");
        g.connect(Edge {
            from: (src, 0),
            to: (tn, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(tn, "r", 1.0);
        g.set_param(tn, "g", 0.0);
        g.set_param(tn, "b", 0.0);
        g.set_param(tn, "a", 0.4);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, tn, 0.0).unwrap();
        match out[0].as_stream().get("tint").unwrap() {
            // existing = white; target = (1,0,0,0.4).
            // i0 f=1: exactly the target ; i1 f=0.5: lerp(white,target,0.5).
            Column::Vec4(v) => {
                assert_eq!(v, &vec![[1.0, 0.0, 0.0, 0.4], [1.0, 0.5, 0.5, 0.7]]);
            }
            _ => panic!("tint"),
        }
    }

    #[test]
    fn mixed_tint_reaches_any_rgba_at_full_falloff() {
        // f=0 → exactly existing (identity); f=1 → exactly the target (all RGBA).
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 0.0),
            [1.0, 1.0, 1.0, 1.0]
        );
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 1.0], [0.2, 0.4, 0.6, 0.3], 1.0),
            [0.2, 0.4, 0.6, 0.3]
        );
    }
}
