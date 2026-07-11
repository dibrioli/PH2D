//! `motion.fibonacci` — a **phyllotaxis** generator: lay out `count` points on a
//! Vogel spiral, the sunflower/pinecone packing (Motion Nodes M3, distributions —
//! doc 01 §3 / doc 18). This is the canonical "even-yet-organic" layout of
//! generative art: Vogel's model (1979) places seed `i` at angle `i · θ` and
//! radius `c · √i`, where `θ` is the **golden angle** (~137.5°). The `√i` radius
//! gives constant area per seed (even packing) and the golden angle is the "most
//! irrational" turn, so no two seeds ever line up into spokes — the deep reason a
//! sunflower looks both regular and never gridded.
//!
//! A **Source** node (like `motion.grid`): no input, emits the `P` position
//! stream. Params: `count`, `spacing` (the `c`, world units), and `angle` (the
//! per-seed turn in DEGREES, default the golden angle — nudge it and the spiral
//! re-packs into visible spokes, a fun knob). The count is capped like the grid so
//! a corrupt scene value can never overflow the allocation.
//!
//! Transcendental-free (HR-5): the seed angle uses the corrected-parabolic
//! `cos/sin` (`trig`, in cycles), and `√i` is IEEE-deterministic. `Pure` — a pure
//! function of the params, no clock or state.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{
    LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec, RECOMMENDED_MAX_ELEMENTS,
    param_as_count,
};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod trig;
use trig::cos_sin_cycles;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Degrees per full turn — the exact divisor into the cycle-based trig's unit.
/// IEEE division is correctly rounded, so this is deterministic (HR-5).
const DEG_PER_TURN: f32 = 360.0;
/// The golden angle in degrees (`360 · (1 − 1/φ)` = `360 / φ²`) — the default
/// per-seed turn, the "most irrational" angle that packs a sunflower.
const GOLDEN_ANGLE_DEG: f32 = 137.507_77;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.fibonacci"),
    name: "motion.fibonacci",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // Number of seeds. Clamped ≥0 and capped like the grid.
        ParamSpec {
            name: "count",
            default: 180.0,
        },
        // The `c` in `r = c·√i` (world units) — the spiral's overall scale.
        ParamSpec {
            name: "spacing",
            default: 0.15,
        },
        // Per-seed turn in degrees (default the golden angle). A different angle
        // re-packs the spiral into visible spokes.
        ParamSpec {
            name: "angle",
            default: GOLDEN_ANGLE_DEG,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Build the `count`-seed Vogel spiral: seed `i` at angle `i·angle` and radius
/// `spacing·√i`. Centered on the origin.
fn build_spiral(count: usize, spacing: f32, angle_deg: f32) -> Vec<[f32; 2]> {
    (0..count)
        .map(|i| {
            let cycles = angle_deg * i as f32 / DEG_PER_TURN;
            let (c, s) = cos_sin_cycles(cycles);
            let r = spacing * (i as f32).sqrt();
            [r * c, r * s]
        })
        .collect()
}

struct MotionFibonacci;

impl NodeOp for MotionFibonacci {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Total conversion (non-finite/negative → 0) + cap, like the grid.
        let count = param_as_count(ctx.param("count"), RECOMMENDED_MAX_ELEMENTS);
        let spacing = ctx.param("spacing");
        let angle = ctx.param("angle");
        let positions = build_spiral(count, spacing, angle);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionFibonacci))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Fibonacci Spiral",
            // Source: a generator that mints the stream (like Grid).
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
        max: 2000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.01,
        max: 1.0,
        step: 0.005,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "angle",
        label: "Angle",
        min: 0.0,
        max: 360.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::OpResolver;

    fn radius(p: [f32; 2]) -> f32 {
        (p[0] * p[0] + p[1] * p[1]).sqrt()
    }

    /// The Vogel signature: the radius grows as `spacing·√i`, so seed radii are
    /// monotonic non-decreasing by index and the outermost sits at `c·√(N−1)`.
    #[test]
    fn the_radius_follows_the_square_root_of_the_index() {
        let s = build_spiral(100, 0.1, GOLDEN_ANGLE_DEG);
        assert_eq!(s.len(), 100);
        assert_eq!(radius(s[0]), 0.0, "seed 0 sits at the centre");
        // Monotonic by index (√i is increasing).
        for i in 1..s.len() {
            assert!(
                radius(s[i]) >= radius(s[i - 1]) - 1e-6,
                "radius grows with i at {i}"
            );
        }
        // The outer radius is c·√(N−1) = 0.1·√99 ≈ 0.995.
        let expected = 0.1 * 99.0_f32.sqrt();
        assert!(
            (radius(s[99]) - expected).abs() < 1e-3,
            "outer radius {}",
            radius(s[99])
        );
    }

    /// FALSIFICATION of "it's really a phyllotaxis, not a ring/line": consecutive
    /// seeds are separated by the golden angle, so the angular STEP between seed 0
    /// and seed 1 is ~137.5° — not 0 (a line) and not 360/N (a ring). We check the
    /// unit turn on the first non-degenerate pair using the dot/‖ of the arms.
    #[test]
    fn consecutive_seeds_turn_by_the_golden_angle() {
        // Use seeds 1 and 2 (seed 0 is at the origin, no direction). Their angular
        // difference is exactly `angle` (137.50777°). cos of that ≈ -0.7374.
        let s = build_spiral(8, 0.2, GOLDEN_ANGLE_DEG);
        let (a, b) = (s[1], s[2]);
        let dot = a[0] * b[0] + a[1] * b[1];
        let cos_between = dot / (radius(a) * radius(b));
        // cos(137.50777°) = -0.73736...
        assert!(
            (cos_between - (-0.7373688)).abs() < 0.01,
            "the turn between seeds is the golden angle (cos {cos_between})"
        );
    }

    /// A degenerate `count = 0` emits an empty stream (no seeds), never a panic.
    #[test]
    fn zero_count_is_empty() {
        assert!(build_spiral(0, 0.15, GOLDEN_ANGLE_DEG).is_empty());
    }

    /// Cooks through the registry and emits the `P` column at the requested count.
    #[test]
    fn registers_and_cooks_the_position_stream() {
        use ph2d_nodegraph::cook::Cook;
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionFibonacci as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let f = g.add_node("motion.fibonacci");
        g.set_param(f, "count", 12.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, f, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 12, "12 seeds emitted"),
            _ => panic!("P"),
        }
    }
}
