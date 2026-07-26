#![forbid(unsafe_code)]
//! `value.gain` — the value-domain CONTRAST / GAMMA shaper: push a normalised
//! `[0,1]` field toward the ends (an S-curve) or toward one end (a gamma bend),
//! with a single driveable knob (Motion Nodes M2, the value domain — doc 12/72).
//! It is the fourth value SHAPER, alongside `value.map_range` (linear),
//! `value.curve` (freeform) and `value.quantize` (discrete): where `value.curve`
//! is a hand-drawn LUT frozen at authoring time, this is the **parametric** one —
//! `strength` is a scalar you can animate or DRIVE per instance, which a baked
//! curve cannot.
//!
//! **The gold standard is Schlick 1994** ("Fast Alternatives to Perlin's Bias and
//! Gain Functions", Graphics Gems): the two rational functions that give the
//! bias/gain shapes WITHOUT a `pow` — so the node is **transcendental-free**
//! (HR-5) and its GPU port is bit-comparable to the CPU. Every procedural tool
//! ships the pair: Houdini VEX `bias()`/`gain()`, the "Contrast"/"Gamma" of a
//! grade, After Effects' Curves S-shape, Unity Shader Graph's Contrast.
//!
//! **`mode`** picks which shape:
//! - **Gain** — an S-curve about `0.5`: `strength > 0` adds CONTRAST (the ends
//!   spread, the middle steepens), `strength < 0` flattens toward `0.5`. Fixed
//!   points at `0`, `0.5`, `1`.
//! - **Bias** — a gamma bend: `strength > 0` pushes UP (toward `1`), `strength <
//!   0` pushes DOWN (toward `0`). Fixed points at `0` and `1`.
//!
//! **`strength ∈ [-1, 1]`, `0` = neutral** — the sign is intuitive (positive is
//! "more", in the mode's natural direction), mapped onto Schlick's native
//! `a ∈ (0,1)` so the two modes point the same way. At `strength = 0` the map is
//! `a = 0.5`, where `1/a - 2 = 0` exactly (a power of two), so both shapes reduce
//! to the identity BIT-EXACT — the node is neutral by construction.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). A **unary** map — length preserved.
//! It operates on the `[0,1]` band (the domain bias/gain are defined on), so the
//! input is CLAMPED to `[0,1]` first: put a `value.map_range` before it to
//! normalise an arbitrary driver, and after it to place the shaped result. `Pure`
//! (no clock, no state). The GPU kernel is the WGSL port of the same rationals, so
//! the node is **device-resident** — it cooks on the GPU, no CPU fallback.

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
/// Schlick's `a` is clamped away from `0` and `1`: `a = 0` divides by zero
/// (`1/a`), `a = 1` degenerates. This is the far-field guard; the reachable range
/// (`strength = ±1`) lands exactly on `0`/`1`, so the clamp is what makes the
/// extremes finite. `strength = 0` maps to `a = 0.5`, well inside it.
const A_MIN: f32 = 1e-4;

/// Which Schlick shape to apply.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// An S-curve about `0.5` — contrast. `strength > 0` spreads the ends.
    Gain,
    /// A gamma bend — `strength > 0` pushes the whole field toward `1`.
    Bias,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Bias,
            _ => Mode::Gain,
        }
    }
}

/// Schlick's bias: a gamma-like bend of `t ∈ [0,1]` by `a ∈ (0,1)`. `a = 0.5` is
/// the identity (the `1/a - 2` term is exactly `0`); `a > 0.5` pushes up, `a <
/// 0.5` pushes down. Fixed points at `0` and `1`. Transcendental-free.
fn schlick_bias(t: f32, a: f32) -> f32 {
    t / ((1.0 / a - 2.0) * (1.0 - t) + 1.0)
}

/// Schlick's gain: an S-curve about `0.5`, built from two half-biases so the
/// midpoint is a fixed point. `a < 0.5` steepens (more contrast), `a > 0.5`
/// flattens. Fixed points at `0`, `0.5`, `1`.
fn schlick_gain(t: f32, a: f32) -> f32 {
    if t < 0.5 {
        schlick_bias(2.0 * t, a) * 0.5
    } else {
        1.0 - schlick_bias(2.0 - 2.0 * t, a) * 0.5
    }
}

/// Shape one value. `t` is clamped to `[0,1]` (the domain bias/gain live on), and
/// `strength ∈ [-1,1]` maps to Schlick's `a` so positive is "more" in the mode's
/// natural direction. `strength = 0` ⇒ `a = 0.5` ⇒ the identity, bit-exact.
fn gain_one(v: f32, strength: f32, mode: Mode) -> f32 {
    let t = v.clamp(0.0, 1.0);
    // Gain: +strength = MORE contrast (a below 0.5). Bias: +strength = push UP
    // (a above 0.5). The opposite signs are what make the slider point the same
    // way (positive = "more") for both shapes.
    let a = match mode {
        Mode::Gain => 0.5 - 0.5 * strength,
        Mode::Bias => 0.5 + 0.5 * strength,
    }
    .clamp(A_MIN, 1.0 - A_MIN);
    match mode {
        Mode::Gain => schlick_gain(t, a),
        Mode::Bias => schlick_bias(t, a),
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.gain"),
    name: "value.gain",
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
    params: &[
        ParamSpec {
            name: "strength",
            default: 0.0,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`gain_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out: the base rides here
/// (a VALUE stream carries only `v`). The clamp, the sign map and the two Schlick
/// rationals are ported verbatim.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vg_s = params.strength;\n\
        let vg_mode = i32(vg_round(params.mode));\n\
        let vg_t = clamp(read_v(i), 0.0, 1.0);\n\
        var vg_a = select(0.5 - 0.5 * vg_s, 0.5 + 0.5 * vg_s, vg_mode == 1);\n\
        vg_a = clamp(vg_a, 1e-4, 1.0 - 1e-4);\n\
        var vg_o: f32;\n\
        if (vg_mode == 1) { vg_o = vg_bias(vg_t, vg_a); }\n\
        else { vg_o = vg_gain(vg_t, vg_a); }\n\
        write_v(i, vg_o);\n",
    wgsl_lib: "\
        fn vg_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn vg_bias(t: f32, a: f32) -> f32 {\n\
            return t / ((1.0 / a - 2.0) * (1.0 - t) + 1.0);\n\
        }\n\
        fn vg_gain(t: f32, a: f32) -> f32 {\n\
            if (t < 0.5) { return vg_bias(2.0 * t, a) * 0.5; }\n\
            return 1.0 - vg_bias(2.0 - 2.0 * t, a) * 0.5;\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    params: &["strength", "mode"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueGain;

impl NodeOp for ValueGain {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let strength = ctx.param("strength");
        let mode = Mode::from_param(ctx.param("mode"));
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input.iter().map(|&v| gain_one(v, strength, mode)).collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueGain))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Gain",
            // Utility grey: a value→value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    // The shaping amount. `0` is neutral; positive is "more" in the mode's
    // natural direction (Gain: more contrast · Bias: push toward 1). The sign
    // lets one slider drive the effect both ways from a centred rest.
    ParamUiHint {
        param: "strength",
        label: "Strength",
        min: -1.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Gain", "Bias"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// **`strength = 0` is the identity** on `[0,1]` — the `a = 0.5` fixed point,
    /// bit-exact (both modes). The reason the node is neutral dropped into a graph.
    #[test]
    fn neutral_strength_is_the_identity_on_the_unit_band() {
        for k in 0..=20 {
            let v = k as f32 / 20.0;
            assert_eq!(gain_one(v, 0.0, Mode::Gain), v, "gain neutral is identity at {v}");
            assert_eq!(gain_one(v, 0.0, Mode::Bias), v, "bias neutral is identity at {v}");
        }
    }

    /// **Gain adds contrast** — positive strength spreads the ends about `0.5`: a
    /// low value drops, a high value rises, and `0.5` is a fixed point.
    #[test]
    fn gain_spreads_the_ends_and_pins_the_middle() {
        let s = 0.6;
        assert!(gain_one(0.25, s, Mode::Gain) < 0.25, "the low quarter drops");
        assert!(gain_one(0.75, s, Mode::Gain) > 0.75, "the high quarter rises");
        assert!((gain_one(0.5, s, Mode::Gain) - 0.5).abs() < 1e-6, "0.5 is a fixed point");
        // Negative strength does the opposite — flattens toward 0.5.
        assert!(gain_one(0.25, -s, Mode::Gain) > 0.25, "negative flattens the low quarter up");
        assert!(gain_one(0.75, -s, Mode::Gain) < 0.75, "negative flattens the high quarter down");
    }

    /// **Bias bends toward an end** — positive strength pushes the whole field up
    /// (toward 1), negative down; the endpoints `0` and `1` are fixed.
    #[test]
    fn bias_pushes_toward_an_end_with_fixed_endpoints() {
        assert!(gain_one(0.5, 0.6, Mode::Bias) > 0.5, "positive bias pushes 0.5 up");
        assert!(gain_one(0.5, -0.6, Mode::Bias) < 0.5, "negative bias pushes 0.5 down");
        for &s in &[-1.0, -0.3, 0.4, 1.0] {
            assert!(gain_one(0.0, s, Mode::Bias).abs() < 1e-6, "0 stays 0 at s={s}");
            assert!((gain_one(1.0, s, Mode::Bias) - 1.0).abs() < 1e-6, "1 stays 1 at s={s}");
        }
    }

    /// **The input is clamped to `[0,1]`** and the output stays there — bias/gain
    /// are defined on the unit band, so a driver outside it is brought in, never a
    /// blow-up. Finite at every strength, including the extremes.
    #[test]
    fn out_of_range_is_clamped_and_output_stays_finite_in_the_band() {
        for &s in &[-1.0, -0.5, 0.0, 0.5, 1.0] {
            for &m in &[Mode::Gain, Mode::Bias] {
                for k in -30..30 {
                    let v = k as f32 * 0.13;
                    let o = gain_one(v, s, m);
                    assert!(o.is_finite(), "finite at v={v} s={s} {m:?}");
                    assert!((-1e-4..=1.0 + 1e-4).contains(&o), "in [0,1] at v={v} s={s} {m:?}");
                }
            }
        }
    }

    /// A value source emitting a fixed field, so `value.gain` can be driven through
    /// a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.gain.test.src"),
        name: "value.gain.test.src",
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

    /// End-to-end through the cook: a `[0, 0.25, 0.5, 0.75, 1]` ramp through a
    /// Gain S-curve keeps the endpoints and midpoint, spreads the quarters, and
    /// preserves the length (the unary contract).
    #[test]
    fn shapes_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueGain),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.gain.test.src");
        let vg = g.add_node("value.gain");
        g.set_param(vg, "strength", 0.6); // a real S-curve
        g.connect(Edge {
            from: (src, 0),
            to: (vg, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vg, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 5, "unary: length 5 preserved");
                assert_eq!(v[0], 0.0, "0 stays 0");
                assert_eq!(v[4], 1.0, "1 stays 1");
                assert!((v[2] - 0.5).abs() < 1e-6, "0.5 stays 0.5");
                assert!(v[1] < 0.25 && v[3] > 0.75, "the quarters spread");
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
