#![forbid(unsafe_code)]
//! `motion.grid` — a Motion **generator**: emits a `rows × cols` grid of
//! instances on the `P` (Vec2) attribute, spaced by `spacing` meters. No
//! inputs. Pure (combinational). The instance stream convention is
//! `ph2d-eval-motion`'s (P → world_pos).
//!
//! Params (read via `ctx.param` — per-instance override else the manifest
//! default shown): `rows` (3), `cols` (3), `spacing` (1.0). `rows`/`cols` are read as element
//! counts via [`param_as_count`] (non-finite/negative → 0) and the `rows × cols`
//! product is capped at [`RECOMMENDED_MAX_ELEMENTS`], so no param value can
//! overflow the allocation.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.grid"),
    name: "motion.grid",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "rows",
            default: 3.0,
        },
        ParamSpec {
            name: "cols",
            default: 3.0,
        },
        ParamSpec {
            name: "spacing",
            default: 1.0,
        },
    ],
    // CPU-only by design: there is no Instances-domain WGSL runtime yet
    // (handoff §9), and the grid is a structural *generator* (it produces an
    // element count), not a per-element `ph2d-expr` map that an `eval_column`
    // could lower. Declaring `Wgsl` here would claim parity nothing can run.
    lowerings: &[LoweringKind::Cpu],
};

/// Build the `rows × cols` position grid (row-major, spaced by `spacing`),
/// capping the element count at `max` so a pathological `rows × cols` can never
/// overflow the allocation. Pure and `max`-parameterized so the cap is testable
/// without allocating the full budget. The emitted count is `positions.len()`.
fn build_grid(rows: usize, cols: usize, spacing: f32, max: usize) -> Vec<[f32; 2]> {
    let count = rows.saturating_mul(cols).min(max);
    let mut positions = Vec::with_capacity(count);
    'outer: for r in 0..rows {
        for c in 0..cols {
            if positions.len() == count {
                break 'outer;
            }
            positions.push([c as f32 * spacing, r as f32 * spacing]);
        }
    }
    positions
}

struct MotionGrid;

impl NodeOp for MotionGrid {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // `rows`/`cols` come from `f32` params; convert *totally* (a non-finite
        // or negative override yields 0, huge values clamp) and cap the product
        // so a corrupt scene value can never overflow the allocation.
        let rows = param_as_count(ctx.param("rows"), RECOMMENDED_MAX_ELEMENTS);
        let cols = param_as_count(ctx.param("cols"), RECOMMENDED_MAX_ELEMENTS);
        let positions = build_grid(rows, cols, ctx.param("spacing"), RECOMMENDED_MAX_ELEMENTS);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionGrid))?;
    // M1.R1 — UI metadata for the card (a generator → green source, widening
    // trapezoid silhouette).
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Grid",
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::Graph;

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            (ty == MANIFEST.id).then_some(&MotionGrid as &dyn NodeOp)
        }
    }

    #[test]
    fn emits_default_3x3_grid() {
        let mut g = Graph::new();
        let n = g.add_node("motion.grid");
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        let p = out[0].as_stream().get("P").unwrap();
        assert_eq!(out[0].as_stream().count(), 9); // 3×3
        match p {
            Column::Vec2(v) => {
                assert_eq!(v[0], [0.0, 0.0]);
                assert_eq!(v[1], [1.0, 0.0]); // col 1, spacing 1.0
                assert_eq!(v[3], [0.0, 1.0]); // row 1
                assert_eq!(v[8], [2.0, 2.0]); // last
            }
            _ => panic!("P must be Vec2"),
        }
    }

    #[test]
    fn per_instance_override_changes_the_grid() {
        // The headline of per-instance params: override `rows` to 2 → 2×3 = 6
        // points (vs the 3×3 = 9 default), proven through the real cook path.
        let mut g = Graph::new();
        let n = g.add_node("motion.grid");
        g.set_param(n, "rows", 2.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        assert_eq!(out[0].as_stream().count(), 6);
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v.len(), 6);
                assert_eq!(v[5], [2.0, 1.0]); // last of row 1
            }
            _ => panic!("P must be Vec2"),
        }
    }

    #[test]
    fn build_grid_caps_pathological_product_at_max() {
        // 100 × 100 = 10_000 requested, but max is 4 → exactly 4 emitted, and
        // the count matches the Vec len (the emit invariant). Row-major order is
        // preserved up to the cut.
        let g = build_grid(100, 100, 1.0, 4);
        assert_eq!(g.len(), 4);
        assert_eq!(g, vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]);
    }

    #[test]
    fn build_grid_zero_dim_is_empty() {
        assert!(build_grid(0, 5, 1.0, 64).is_empty());
        assert!(build_grid(5, 0, 1.0, 64).is_empty());
    }
}
