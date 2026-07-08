#![forbid(unsafe_code)]
//! `motion.tint` — a Motion **colour modifier**: multiplies the `tint` (Vec4
//! RGBA, linear straight — §1.2) attribute by a colour that fades from white to
//! `(r, g, b)` with the multiplicative `falloff` column. The per-instance mix is
//! `lerp(white, (r,g,b), falloff_i)`, multiplied into any existing `tint`
//! (absent → white). Alpha is preserved (absent → `1`). Every other column
//! passes through unchanged (count preserved). Pure.
//!
//! Params (read via `ctx.param`): `r` (1.0), `g` (0.3), `b` (0.1) — a warm
//! default so the wash reads immediately when a falloff field is attached.

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

/// The falloff-mixed RGB multiplier for one instance: `lerp(1, channel, f)` per
/// channel (white at `f = 0`, the full colour at `f = 1`), then multiplied into
/// the existing RGB. Alpha is carried straight through.
fn mixed_tint(existing: [f32; 4], color: [f32; 3], f: f32) -> [f32; 4] {
    // `(1-f)·white + f·c` — the endpoint-exact lerp form (returns exactly white
    // at f=0 and exactly `c` at f=1, no float drift), then × existing.
    let mix = |c: f32| (1.0 - f) + c * f;
    [
        existing[0] * mix(color[0]),
        existing[1] * mix(color[1]),
        existing[2] * mix(color[2]),
        existing[3],
    ]
}

struct MotionTint;

impl NodeOp for MotionTint {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let color = [ctx.param("r"), ctx.param("g"), ctx.param("b")];
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
                    mixed_tint(e, color, falloff_at(input, i))
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

/// Param UI hints (M1.P1): three linear RGB channels in `0..1`.
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
    fn tint_fades_from_white_by_falloff() {
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
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, tn, 0.0).unwrap();
        match out[0].as_stream().get("tint").unwrap() {
            // i0 f=1: full red (1,0,0,1) ; i1 f=0.5: lerp(white,red,0.5)=(1,0.5,0.5,1)
            Column::Vec4(v) => assert_eq!(v, &vec![[1.0, 0.0, 0.0, 1.0], [1.0, 0.5, 0.5, 1.0]]),
            _ => panic!("tint"),
        }
    }

    #[test]
    fn mixed_tint_preserves_alpha_and_lerps_rgb() {
        // f=0 → white multiplier (identity on existing); f=1 → the full colour.
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 0.5], [0.2, 0.4, 0.6], 0.0),
            [1.0, 1.0, 1.0, 0.5]
        );
        assert_eq!(
            mixed_tint([1.0, 1.0, 1.0, 0.5], [0.2, 0.4, 0.6], 1.0),
            [0.2, 0.4, 0.6, 0.5]
        );
    }
}
