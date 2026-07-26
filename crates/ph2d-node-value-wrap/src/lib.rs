#![forbid(unsafe_code)]
//! `value.wrap` — the value-domain ADDRESS mode: fold a field's out-of-range
//! values back into `[min, max]` by **Clamp**, **Repeat**, or **Mirror** (Motion
//! Nodes M2, the value domain — doc 12/79). Where `value.map_range` LINEARLY
//! rescales one interval onto another (and clamps at the ends), this one takes a
//! field that already runs off the ends of a range and decides *what happens past
//! the edge* — the same choice a sampler makes at a texture boundary, or an
//! animation makes past its last key.
//!
//! **The gold standard is the texture-address trio** every renderer ships —
//! `ClampToEdge` / `Repeat` / `MirroredRepeat` (Vulkan/GL/WebGPU), After Effects'
//! loopOut `Continue`/`Cycle`/`PingPong`, Houdini VEX `clamp`/`(x%w)`/triangle.
//! All three are **transcendental-free** (HR-5): a `clamp`, or a `floor`-based
//! fold — so the GPU port is bit-comparable to the CPU and the node is
//! **device-resident** (it cooks on the GPU, no CPU fallback).
//!
//! **`mode`** picks the edge behaviour over the range `[min, max]` (width
//! `w = max − min`):
//! - **Clamp** — hold at the edges: below `min` reads `min`, above `max` reads
//!   `max`. A plateau either side (`ClampToEdge` / loopOut Continue).
//! - **Repeat** — tile the range: the value wraps every `w`, a **sawtooth** that
//!   jumps from `max` back to `min` (`Repeat` / loopOut Cycle). `max` itself maps
//!   to `min` (a half-open `[min, max)` tile, the sampler convention).
//! - **Mirror** — fold back and forth: a **triangle** that rises to `max` then
//!   falls to `min` and back, period `2w` (`MirroredRepeat` / loopOut PingPong).
//!
//! **A composer, not a producer.** Feed it a `value.instance_field` Ramp scaled
//! past `1.0` (or any field with reach beyond the range) and Repeat tiles the
//! grid into `N` copies of the ramp, Mirror into a zig-zag — an authored spatial
//! period that no single producer gives. `value.wrap` after `value.map_range`
//! places the range where you want it; `value.quantize` after it stairs the tile.
//!
//! **The output is NOT a `[0,1]` mask** — it lands in `[min, max]`, on whatever
//! scale the range names (a comparison-free fold is meaningful on any scale). A
//! **degenerate range** (`max ≤ min`, `w ≤ 0`) has nothing to fold into, so the
//! whole field pins to `min` — finite, never a division by zero. `Pure` (no
//! clock, no state); a **unary** map, length preserved.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12).

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";

/// How the fold treats values past the ends of `[lo, hi]`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Hold at the edges — a plateau either side (`ClampToEdge`).
    Clamp,
    /// Tile the range — a sawtooth, `max` wraps to `min` (`Repeat`).
    Repeat,
    /// Fold back and forth — a triangle, period `2w` (`MirroredRepeat`).
    Mirror,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Repeat,
            2 => Mode::Mirror,
            _ => Mode::Clamp,
        }
    }
}

/// Fold one value into `[lo, hi]` by `mode`. A degenerate range (`hi ≤ lo`) pins
/// to `lo` — there is no interval to fold into, and this is what keeps every path
/// finite (no division by a zero width). Transcendental-free: a `clamp`, or a
/// `floor`-based positive modulo.
fn wrap_one(v: f32, lo: f32, hi: f32, mode: Mode) -> f32 {
    let w = hi - lo;
    if w <= 0.0 {
        return lo; // degenerate range: nothing to fold into
    }
    match mode {
        Mode::Clamp => v.clamp(lo, hi),
        Mode::Repeat => {
            // Positive modulo into [0, w): r − w·⌊r/w⌋. `max` (r = w) maps to `min`.
            let r = v - lo;
            lo + (r - w * (r / w).floor())
        }
        Mode::Mirror => {
            // Fold over a period 2w: rise on [0, w], fall on [w, 2w].
            let p = 2.0 * w;
            let r = v - lo;
            let m = r - p * (r / p).floor(); // m in [0, 2w)
            lo + if m > w { p - m } else { m }
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.wrap"),
    name: "value.wrap",
    inputs: &[PortSpec {
        name: "in",
        ty: VALUE,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    // `lo`/`hi`, not `min`/`max`: WGSL `min`/`max` are builtin functions, and the
    // param names become the uniform's struct-field names — `params.min` is a
    // field access, safe, but the sidestep costs nothing and reads clean.
    params: &[
        ParamSpec {
            name: "lo",
            default: 0.0,
        },
        ParamSpec {
            name: "hi",
            default: 1.0,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`wrap_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out; the fold is ported
/// verbatim, `floor` and `clamp` matching the CPU's arithmetic. The only
/// device-divergence is the FMA the driver may fuse in `lo + (…)`, ε below the
/// parity budget; a value landing EXACTLY on a cell edge is the one place `floor`
/// could disagree by a whole period, so the parity fixture uses un-round params
/// (the `value.quantize`/`field.remap` precedent — a boundary is measure-zero and
/// the artist never sees it).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vw_mode = i32(vw_round(params.mode));\n\
        let vw_lo = params.lo;\n\
        let vw_hi = params.hi;\n\
        let vw_v = read_v(i);\n\
        let vw_w = vw_hi - vw_lo;\n\
        var vw_o: f32;\n\
        if (vw_w <= 0.0) {\n\
            vw_o = vw_lo;\n\
        } else if (vw_mode == 1) {\n\
            let vw_r = vw_v - vw_lo;\n\
            vw_o = vw_lo + (vw_r - vw_w * floor(vw_r / vw_w));\n\
        } else if (vw_mode == 2) {\n\
            let vw_p = 2.0 * vw_w;\n\
            let vw_r = vw_v - vw_lo;\n\
            let vw_m = vw_r - vw_p * floor(vw_r / vw_p);\n\
            vw_o = vw_lo + select(vw_m, vw_p - vw_m, vw_m > vw_w);\n\
        } else {\n\
            vw_o = clamp(vw_v, vw_lo, vw_hi);\n\
        }\n\
        write_v(i, vw_o);\n",
    wgsl_lib: "\
        fn vw_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["lo", "hi", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueWrap;

impl NodeOp for ValueWrap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let lo = ctx.param("lo");
        let hi = ctx.param("hi");
        let mode = Mode::from_param(ctx.param("mode"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input.iter().map(|&v| wrap_one(v, lo, hi, mode)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueWrap))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Wrap",
            // Utility grey: a value->value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The range the field folds into. `0..1` is the natural home (the normalised
    // convention), but the fold is scale-free: any `[min, max]` is a valid tile.
    ParamUiHint {
        param: "lo",
        label: "Min",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "hi",
        label: "Max",
        min: -10.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Clamp", "Repeat", "Mirror"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **Clamp holds at the edges** — inside the range is identity, below `min`
    /// reads `min`, above `max` reads `max`. The plateau either side.
    #[test]
    fn clamp_holds_at_the_edges() {
        let (lo, hi) = (0.0, 1.0);
        assert_eq!(wrap_one(0.3, lo, hi, Mode::Clamp), 0.3, "inside is identity");
        assert_eq!(wrap_one(-2.0, lo, hi, Mode::Clamp), 0.0, "below pins to min");
        assert_eq!(wrap_one(5.0, lo, hi, Mode::Clamp), 1.0, "above pins to max");
        // A non-`[0,1]` range clamps just the same.
        assert_eq!(wrap_one(9.0, -2.0, 3.0, Mode::Clamp), 3.0, "clamps to max=3");
    }

    /// **Repeat tiles the range into a sawtooth** — a value `w` past `min` reads
    /// the same as `min`, and `max` itself wraps back to `min` (the half-open
    /// `[min, max)` tile). Falsifiable: a Clamp implementation would pin the tail
    /// to `max` instead of wrapping it.
    #[test]
    fn repeat_tiles_into_a_sawtooth() {
        let (lo, hi) = (0.0, 1.0); // width 1
        assert_eq!(wrap_one(0.3, lo, hi, Mode::Repeat), 0.3, "inside is identity");
        assert!((wrap_one(1.3, lo, hi, Mode::Repeat) - 0.3).abs() < 1e-6, "1.3 wraps to 0.3");
        assert!((wrap_one(2.3, lo, hi, Mode::Repeat) - 0.3).abs() < 1e-6, "2.3 wraps to 0.3");
        assert!((wrap_one(-0.3, lo, hi, Mode::Repeat) - 0.7).abs() < 1e-6, "-0.3 wraps to 0.7");
        assert_eq!(wrap_one(1.0, lo, hi, Mode::Repeat), 0.0, "max wraps to min");
        // A shifted range: [2, 5], width 3. 8 → 2, 6.5 → 3.5.
        assert!((wrap_one(8.0, 2.0, 5.0, Mode::Repeat) - 2.0).abs() < 1e-6, "8 wraps into [2,5] as 2");
        assert!((wrap_one(6.5, 2.0, 5.0, Mode::Repeat) - 3.5).abs() < 1e-6, "6.5 -> 3.5");
    }

    /// **Mirror folds back and forth into a triangle** — it rises to `max`, then
    /// FALLS back to `min` over the next `w`, period `2w`. The distinguishing case
    /// is `1.5·w` past `min`: Repeat reads `0.5·w` (rising), Mirror reads the
    /// mirror `0.5·w` down from the top. Falsifiable against Repeat.
    #[test]
    fn mirror_folds_into_a_triangle() {
        let (lo, hi) = (0.0, 1.0); // width 1, period 2
        assert_eq!(wrap_one(0.0, lo, hi, Mode::Mirror), 0.0, "min");
        assert_eq!(wrap_one(1.0, lo, hi, Mode::Mirror), 1.0, "peak at max");
        // 1.3 is 0.3 into the falling half → 1 − 0.3 = 0.7 (Repeat would give 0.3).
        assert!((wrap_one(1.3, lo, hi, Mode::Mirror) - 0.7).abs() < 1e-6, "1.3 folds to 0.7");
        assert!((wrap_one(1.7, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6, "1.7 folds to 0.3");
        assert!((wrap_one(2.0, lo, hi, Mode::Mirror)).abs() < 1e-6, "2.0 back to min (period 2)");
        assert!((wrap_one(2.3, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6, "2.3 rises again to 0.3");
        // Negative side mirrors symmetrically: -0.3 folds up to 0.3.
        assert!((wrap_one(-0.3, lo, hi, Mode::Mirror) - 0.3).abs() < 1e-6, "-0.3 folds to 0.3");
    }

    /// **A degenerate range pins to `min`** — `max ≤ min` has no interval to fold
    /// into, so every value collapses to `min`, finite, never a division by a
    /// zero width. Falsifiable: an unguarded `r / w` would be `inf`/`NaN`.
    #[test]
    fn a_degenerate_range_pins_to_min_and_stays_finite() {
        for &m in &[Mode::Clamp, Mode::Repeat, Mode::Mirror] {
            for &v in &[-3.0f32, 0.0, 2.5, 100.0] {
                // Zero-width and inverted ranges both degenerate.
                assert_eq!(wrap_one(v, 0.5, 0.5, m), 0.5, "zero width pins to lo ({m:?})");
                assert_eq!(wrap_one(v, 1.0, 0.0, m), 1.0, "inverted pins to lo ({m:?})");
            }
        }
    }

    /// **The fold always lands in `[min, max]` and stays finite** for any value,
    /// range and mode — the whole point of an address mode (a sampler never reads
    /// off the texture). Repeat's half-open top is the only exclusion.
    #[test]
    fn the_result_is_finite_and_inside_the_range() {
        for &m in &[Mode::Clamp, Mode::Repeat, Mode::Mirror] {
            for &(lo, hi) in &[(0.0f32, 1.0), (-2.0, 3.0), (1.5, 1.6)] {
                for k in -200..200 {
                    let v = k as f32 * 0.13;
                    let o = wrap_one(v, lo, hi, m);
                    assert!(o.is_finite(), "finite at v={v} [{lo},{hi}] {m:?}");
                    // A tiny ε for the floor's fp reconstruction at the seam.
                    assert!(o >= lo - 1e-4 && o <= hi + 1e-4, "in range at v={v} [{lo},{hi}] {m:?}: {o}");
                }
            }
        }
    }

    /// A value source emitting a fixed field, so `value.wrap` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.wrap.test.src"),
        name: "value.wrap.test.src",
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
    struct Src(Vec<f32>);
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(self.0.len()).with(VALUE_COL, Column::Scalar(self.0.clone())));
        }
    }

    /// End-to-end through the cook: a ramp that runs `[0, 2]` folded by Repeat
    /// into `[0, 1]` becomes two copies of the tile, length preserved (the unary
    /// contract).
    #[test]
    fn tiles_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueWrap),
                    _ => None,
                }
            }
        }
        // 0, 0.5, 1.0, 1.5 over [0,1] Repeat → 0, 0.5, 0, 0.5 (the tile repeats).
        let ops = Ops(vec![0.0, 0.5, 1.0, 1.5]);
        let mut g = Graph::new();
        let src = g.add_node("value.wrap.test.src");
        let w = g.add_node("value.wrap");
        g.set_param(w, "lo", 0.0);
        g.set_param(w, "hi", 1.0);
        g.set_param(w, "mode", 1.0); // Repeat
        g.connect(Edge {
            from: (src, 0),
            to: (w, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, w, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 4, "length preserved");
                assert!((v[0] - 0.0).abs() < 1e-6, "0 -> 0");
                assert!((v[1] - 0.5).abs() < 1e-6, "0.5 -> 0.5");
                assert!((v[2] - 0.0).abs() < 1e-6, "1.0 wraps to 0");
                assert!((v[3] - 0.5).abs() < 1e-6, "1.5 wraps to 0.5");
            }
            _ => panic!("v"),
        }
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }
}
