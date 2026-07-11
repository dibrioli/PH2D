#![forbid(unsafe_code)]
//! `motion.lattice` — a **hexagonal (triangular) lattice** distribution: `rows×cols`
//! points on the densest regular packing, every point equidistant from its six
//! neighbours (Motion Nodes M3, distributions — doc 01 §3 / doc 23). The crystalline
//! counterpart to `motion.grid` (square) and the ordered opposite of the blue-noise
//! `motion.scatter`. Honeycombs, bubble rafts, close-packed circles.
//!
//! **Algorithm — the triangular lattice, the 2D densest circle packing.** Even rows
//! sit on a square pitch `spacing`; odd rows are shifted half a cell and the row
//! pitch is `spacing·√3/2`, so every nearest-neighbour distance equals `spacing`
//! exactly (equilateral triangles / regular hexagons). A `jitter` value input melts
//! the lattice toward white noise: each point is displaced by a hashed offset scaled
//! by `jitter` (world units), so a `value.lfo` makes the honeycomb shimmer and reform.
//!
//! A **Source** node (no stream input, mints `P`). Stateless (Jarzynski/Olano): the
//! jitter is a pure hash of `(seed, index)`, so the layout reproduces bit-for-bit.
//! `Effect::Pure` (no clock — animation arrives through the `jitter` input).
//! Transcendental-free (HR-5): the `√3/2` row pitch is a constant, the jitter is the
//! splitmix hash; no calls.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
use hash::hash3;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `jitter` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// `√3/2` — the equilateral row pitch (a constant, not a call; HR-5).
const ROW_PITCH: f32 = 0.866_025_4;
/// Grid side clamp (cost is O(rows·cols)).
const MAX_SIDE: i64 = 400;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.lattice"),
    name: "motion.lattice",
    inputs: &[
        // Displacement scale toward white noise (animatable). Optional: unconnected
        // reads as 0 → a perfect lattice.
        PortSpec {
            name: "jitter",
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
        ParamSpec {
            name: "rows",
            default: 6.0,
        },
        ParamSpec {
            name: "cols",
            default: 7.0,
        },
        ParamSpec {
            name: "spacing",
            default: 0.7,
        },
        ParamSpec {
            name: "seed",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Lay out the hexagonal lattice (centred on the origin), each point displaced by a
/// hashed offset scaled by `jitter`.
fn lattice(rows: usize, cols: usize, spacing: f32, seed: u32, jitter: f32) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(rows * cols);
    // Half-extents for centring: odd rows reach half a cell further in x.
    let half_w = ((cols as f32 - 1.0) * spacing + spacing * 0.5) * 0.5;
    let half_h = (rows as f32 - 1.0) * spacing * ROW_PITCH * 0.5;
    for r in 0..rows {
        let row_shift = if r % 2 == 1 { spacing * 0.5 } else { 0.0 };
        for c in 0..cols {
            let i = (r * cols + c) as u32;
            let jx = (hash3(seed, i, 0) - 0.5) * 2.0 * jitter;
            let jy = (hash3(seed, i, 1) - 0.5) * 2.0 * jitter;
            out.push([
                c as f32 * spacing + row_shift - half_w + jx,
                r as f32 * spacing * ROW_PITCH - half_h + jy,
            ]);
        }
    }
    out
}

struct MotionLattice;

impl NodeOp for MotionLattice {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let side = |name: &str| (ctx.param(name).round() as i64).clamp(1, MAX_SIDE) as usize;
        let rows = side("rows");
        let cols = side("cols");
        let spacing = ctx.param("spacing").max(1e-3);
        let seed = ctx.param("seed").max(0.0).round() as u32;
        let jitter = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.first().copied().unwrap_or(0.0),
            _ => 0.0,
        };
        let positions = lattice(rows, cols, spacing, seed, jitter);
        ctx.emit(Stream::new(positions.len()).with("P", Column::Vec2(positions)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionLattice))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Lattice",
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
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cols",
        label: "Cols",
        min: 1.0,
        max: 60.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "spacing",
        label: "Spacing",
        min: 0.1,
        max: 4.0,
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

    /// The nearest-neighbour distance in the set.
    fn nearest_neighbour(pts: &[[f32; 2]]) -> f32 {
        let mut min = f32::MAX;
        for (i, a) in pts.iter().enumerate() {
            for b in &pts[i + 1..] {
                let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
                min = min.min((dx * dx + dy * dy).sqrt());
            }
        }
        min
    }

    /// The lattice IS hexagonal: with no jitter, the nearest-neighbour distance equals
    /// `spacing` for every point (equilateral packing). FALSIFIED for a square grid,
    /// whose diagonal neighbours would be `spacing·√2` — but the row pitch `√3/2` keeps
    /// the offset rows exactly `spacing` away.
    #[test]
    fn it_is_a_hexagonal_packing() {
        let pts = lattice(5, 6, 0.7, 1, 0.0);
        assert_eq!(pts.len(), 30);
        let nn = nearest_neighbour(&pts);
        assert!(
            (nn - 0.7).abs() < 1e-4,
            "every nearest neighbour is one spacing away (hex): {nn}"
        );
    }

    /// Odd rows are shifted half a cell — the defining hex offset. Row 0 col 0 and
    /// row 1 col 0 differ by half a spacing in x (not zero, as a square grid would).
    #[test]
    fn odd_rows_are_half_shifted() {
        let cols = 6;
        let pts = lattice(2, cols, 1.0, 1, 0.0);
        let dx = pts[cols][0] - pts[0][0]; // (row1,col0) − (row0,col0)
        assert!((dx - 0.5).abs() < 1e-5, "odd row shifted half a cell: {dx}");
    }

    /// The lattice is centred on the origin (mean position ≈ 0).
    #[test]
    fn it_is_centred_on_the_origin() {
        let pts = lattice(7, 7, 0.6, 1, 0.0);
        let mean = pts
            .iter()
            .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
        let n = pts.len() as f32;
        assert!((mean[0] / n).abs() < 0.05 && (mean[1] / n).abs() < 0.05);
    }

    /// `jitter` melts the lattice: a positive jitter breaks the exact packing (the
    /// nearest-neighbour distance drops below `spacing`), and it is deterministic.
    #[test]
    fn jitter_melts_the_lattice_deterministically() {
        let ordered = lattice(6, 6, 0.7, 3, 0.0);
        let melted = lattice(6, 6, 0.7, 3, 0.3);
        assert!(
            nearest_neighbour(&melted) < nearest_neighbour(&ordered),
            "jitter clumps some points closer than the perfect packing"
        );
        assert_eq!(melted, lattice(6, 6, 0.7, 3, 0.3), "reproducible");
        assert_ne!(
            melted,
            lattice(6, 6, 0.7, 4, 0.3),
            "seed re-rolls the jitter"
        );
    }

    /// Cooks through the registry and emits the `P` column, with the `jitter` input
    /// unconnected (→ a perfect lattice).
    #[test]
    fn registers_and_cooks() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::Graph;

        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                (ty == MANIFEST.id).then_some(&MotionLattice as &dyn NodeOp)
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let n = g.add_node("motion.lattice");
        g.set_param(n, "rows", 4.0);
        g.set_param(n, "cols", 5.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, n, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 20),
            _ => panic!("P"),
        }
    }
}
