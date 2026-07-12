#![forbid(unsafe_code)]
//! `motion.scale` — a Motion **modifier**: multiplies the `size` (Vec2)
//! attribute by `amount`, eased per-instance by the multiplicative `falloff`
//! column (§1.2). The effective factor is `1 + (amount - 1) * falloff_i`, so an
//! instance with `falloff = 0` keeps its size and `falloff = 1` gets the full
//! `amount`. A stream without a `size` column starts from
//! [`SIZE_IDENTITY`](ph2d_nodegraph::attr::SIZE_IDENTITY) — the SAME unit scale
//! the lowering falls back to, which is what makes `amount = 1` a true no-op on
//! the render (doc 39). Every other column passes through unchanged (count
//! preserved). Pure.
//!
//! Params (read via `ctx.param`): `amount` (1.0).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.scale"),
    name: "motion.scale",
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
    params: &[ParamSpec {
        name: "amount",
        default: 1.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The falloff-eased scale factor: `1` at `falloff = 0`, `amount` at `1`.
fn eff_factor(amount: f32, falloff: f32) -> f32 {
    1.0 + (amount - 1.0) * falloff
}

struct MotionScale;

impl NodeOp for MotionScale {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let amount = ctx.param("amount");
        let out = {
            let input = ctx.input(0);
            let n = input.count();
            // Base per-instance size (absent column → the identity the lowering
            // itself falls back to — see SIZE_IDENTITY: any other base would make
            // `amount = 1` resize the scene).
            let base: Vec<[f32; 2]> = match input.get("size") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => vec![SIZE_IDENTITY; n],
            };
            let scaled: Vec<[f32; 2]> = (0..n)
                .map(|i| {
                    let f = eff_factor(amount, falloff_at(input, i));
                    let s = base.get(i).copied().unwrap_or(SIZE_IDENTITY);
                    [s[0] * f, s[1] * f]
                })
                .collect();
            let mut out = Stream::new(n);
            for (name, col) in input.columns() {
                if name != "size" {
                    out.set(name.clone(), col.clone());
                }
            }
            out.set("size", Column::Vec2(scaled));
            out
        };
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionScale))?;
    // M1.R1 — UI metadata (a spatial modifier → blue transform, rounded-rect).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Scale",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// Param UI hints (M1.P1): a positive scale multiplier.
static PARAM_HINTS: &[ParamUiHint] = &[ParamUiHint {
    param: "amount",
    label: "Scale",
    min: 0.0,
    max: 5.0,
    step: 0.05,
    widget: ParamWidget::Slider,
}];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, EvalCtx, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    // Source: 2 instances with size [2,2] and falloff [1, 0.5].
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.scale.test.src"),
        name: "motion.scale.test.src",
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
                    .with("size", Column::Vec2(vec![[2.0, 2.0], [2.0, 2.0]]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5])),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC_MAN.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionScale),
                _ => None,
            }
        }
    }

    #[test]
    fn scale_is_eased_by_falloff() {
        let mut g = Graph::new();
        let src = g.add_node("motion.scale.test.src");
        let sc = g.add_node("motion.scale");
        g.connect(Edge {
            from: (src, 0),
            to: (sc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_param(sc, "amount", 3.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, sc, 0.0).unwrap();
        match out[0].as_stream().get("size").unwrap() {
            // i0 f=1: factor 3 → 2*3=6 ; i1 f=0.5: factor 1+2*0.5=2 → 2*2=4
            Column::Vec2(v) => assert_eq!(v, &vec![[6.0, 6.0], [4.0, 4.0]]),
            _ => panic!("size"),
        }
    }

    #[test]
    fn eff_factor_interpolates_one_to_amount() {
        assert_eq!(eff_factor(3.0, 0.0), 1.0); // no falloff → identity
        assert_eq!(eff_factor(3.0, 1.0), 3.0); // full falloff → amount
        assert_eq!(eff_factor(3.0, 0.5), 2.0); // half
    }
}
