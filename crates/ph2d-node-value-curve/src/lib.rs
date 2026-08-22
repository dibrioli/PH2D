#![forbid(unsafe_code)]
//! `value.curve` — the value-domain **shaper**: pass a value through an artist-
//! DRAWN transfer curve, the arbitrary-shape twin of `value.map_range`'s linear
//! remap (Motion Nodes, the value domain — doc 12). Where map_range fits
//! `[in_lo, in_hi] → [out_lo, out_hi]` with a STRAIGHT line, this fits it through
//! a curve the artist edits by dragging control points — the industry's canonical
//! "shape a value" node: Blender's **Float Curve** (`ShaderNodeFloatCurve`),
//! Houdini's **ramp** parameter, C4D's **Spline** mapper, Cavalry's Value Graph.
//!
//! **Semantics:** normalize `t = clamp((v − in_lo) / (in_hi − in_lo), 0, 1)` (the
//! curve's domain is `[0,1]`), shape `s = curve.eval(t)`, then scale
//! `out = out_lo + s · (out_hi − out_lo)`. An UNSET curve is the identity
//! (`eval(t) = t`), so a fresh node with nothing drawn is exactly `value.map_range`
//! with clamp on — a straight line you then bend. Reuses the `ph2d-curve` engine
//! and the A1 draggable editor (`ParamWidget::Curve`), the SAME transfer the
//! `field.remap` **Curve** contour runs — here on the value column `v` instead of
//! the falloff mask.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on `v` (doc 12). A **unary** map — length preserved, no
//! broadcast decision (that lives in the consumer, `motion.drive`). `Pure` (no
//! clock, no state). Transcendental-free (HR-5): the curve `eval` is `+ − × ÷` +
//! `clamp` (smoothstep is a polynomial), and the GPU path samples a **baked LUT**
//! (A1-gpu, the `luts` channel), so the whole node is **device-resident** — every
//! contour cooks on the GPU, no CPU fallback.

use ph2d_curve::Curve;
use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_map_range::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// Below this the input span is treated as degenerate (`t` collapses to 0), so
/// the node never divides by (near-)zero.
const MIN_SPAN: f32 = 1e-9;
/// The text-param key carrying the transfer curve's shape (a `ph2d-curve`
/// serialized string, authored by the A1 `ParamWidget::Curve` editor). NOT a
/// `ParamSpec` — a curve is not one number (the `motion.expression` /
/// `field.remap` precedent).
const CURVE_KEY: &str = "curve";

/// The static contract of this node type (ADR-0031). The kernel + LUT are
/// side-metadata (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.curve"),
    name: "value.curve",
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
            name: "in_lo",
            default: 0.0,
        },
        ParamSpec {
            name: "in_hi",
            default: 1.0,
        },
        ParamSpec {
            name: "out_lo",
            default: 0.0,
        },
        ParamSpec {
            name: "out_hi",
            default: 1.0,
        },
        // ⚠️ **Apendado**: quanto da resposta MOLDADA fica. `1` = o nó que sempre
        // shipou, verbatim (ver [`blend_toward`]).
        ParamSpec {
            name: "factor",
            default: 1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Shape one value through the curve. `t = clamp((v − in_lo)/span, 0, 1)` (the
/// curve's `[0,1]` domain), `s = curve.eval(t)` (identity when the curve is
/// unset), then `out = out_lo + s·(out_hi − out_lo)`. A degenerate input span
/// (`in_hi == in_lo`) collapses `t` to 0 — `out_lo + curve.eval(0)·span_out`,
/// which for the identity curve is `out_lo` (the `map_range` guard) — never a
/// divide-by-zero NaN.
fn shape_one(
    v: f32,
    in_lo: f32,
    in_hi: f32,
    out_lo: f32,
    out_hi: f32,
    curve: Option<&Curve>,
) -> f32 {
    let span = in_hi - in_lo;
    let t = if span.abs() < MIN_SPAN {
        0.0
    } else {
        ((v - in_lo) / span).clamp(0.0, 1.0)
    };
    let s = curve.map_or(t, |c| c.eval(t));
    out_lo + s * (out_hi - out_lo)
}

/// **How much of the shaped answer to keep** — Blender's Float Curve ships this as
/// its FIRST socket (`Factor`), and it is the difference between a curve node an
/// artist can dial and one they can only switch on.
///
/// `1` is the curve, `0` is the input untouched, and in between it is the straight
/// line from one to the other — literally `value.mix(a = input, b = shaped, t)`
/// with the bifurcation of the wire already done for you.
///
/// ⚠️ **The two anchors are returned VERBATIM, and the branch is the reason.**
/// `input + 1.0·(shaped − input)` is not `shaped` in `f32` — it rounds twice — so
/// an algebraic-only blend would make the default of this node stop being the node
/// that shipped, by a ulp, on every value. A ulp is not visible; a *changed
/// default* is exactly the kind of drift that a byte-identity gate exists to
/// forbid. `factor` is deliberately NOT clamped: past `1` it EXTRAPOLATES away
/// from the input (the caricature of the curve), which is meaningful and which
/// clamping would silently forbid.
fn blend_toward(input: f32, shaped: f32, factor: f32) -> f32 {
    if factor == 1.0 {
        shaped
    } else if factor == 0.0 {
        input
    } else {
        input + factor * (shaped - input)
    }
}

/// The Curve's LUT resolution (A1-gpu) — samples of the authored curve over
/// `t ∈ [0,1]`. 256 keeps a smooth transfer within a few thousandths of the CPU
/// `eval` while costing 1 KiB the sequencer uploads per cook (measured on the
/// `field.remap` twin: worst `|Δ|` at a tent's peak corner ≈ 4e-3).
const LUT_RESOLUTION: u32 = 256;

/// GPU compute kernel (ADR-0126) — the WGSL port of [`shape_one`], **fully
/// device-resident**. The curve's shape is a text param the uniform cannot carry
/// (a curve is not a scalar), so A1-gpu bakes it to a LUT (see [`LUTS`]) and the
/// body samples `vc_curve_sample(t)`. No `applicable` gate — the sequencer never
/// falls back to the CPU for this node (the "maximize GPU" north).
///
/// VALUE in, VALUE out: the base DOES ride here, and that is correct — a VALUE
/// stream carries only `v`, so "base + written `v`" and "a fresh stream with `v`"
/// are the same stream. The `MIN_SPAN` guard is ported verbatim (a zero span is a
/// divide; both devices answer `t = 0` there).
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vc_in = read_v(i);\n\
        let vc_span = params.in_hi - params.in_lo;\n\
        var vc_t = 0.0;\n\
        if (abs(vc_span) >= 1e-9) {\n\
        \x20   vc_t = clamp((vc_in - params.in_lo) / vc_span, 0.0, 1.0);\n\
        }\n\
        let vc_s = vc_curve_sample(vc_t);\n\
        let vc_shaped = params.out_lo + vc_s * (params.out_hi - params.out_lo);\n\
        // As duas ancoras sao VERBATIM -- o gemeo do ramo de `blend_toward`, e a\n\
        // razao e' a mesma dos dois lados: `a + 1.0*(b-a)` nao e' `b` em f32.\n\
        var vc_o = vc_shaped;\n\
        if (params.factor == 0.0) { vc_o = vc_in; }\n\
        else if (params.factor != 1.0) {\n\
        \x20   vc_o = vc_in + params.factor * (vc_shaped - vc_in);\n\
        }\n\
        write_v(i, vc_o);\n",
    wgsl_lib: "",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    }],
    // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
    // CPU, e o device recusa o shader (`invalid field accessor`).
    params: &["in_lo", "in_hi", "out_lo", "out_hi", "factor"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The LUT channel this node registers (A1-gpu): the sequencer samples the
/// authored curve (the `CURVE_KEY` text param) to [`LUT_RESOLUTION`] floats and
/// binds them, so the WGSL body reads `vc_curve_sample(t)` on the DEVICE. Naming
/// it `vc_curve` makes the accessor `vc_curve_sample`, matching the kernel call.
static LUTS: &[LutSpec] = &[LutSpec {
    name: "vc_curve",
    text_key: CURVE_KEY,
    resolution: LUT_RESOLUTION,
    fill: fill_curve_lut,
}];

/// Sample the authored curve into `out` at `t = k/(n−1)` — the NODE-side half of
/// the LUT channel (`ph2d-nodegraph` stays curve-library-agnostic; the sampling
/// lives here). Mirrors the CPU `shape_one`'s `curve.eval(t)`: an unset or
/// malformed string is the identity [`Curve`] (`eval(t) = t`), the passthrough the
/// CPU takes on `None`.
fn fill_curve_lut(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(t);
    }
}

struct ValueCurve;

impl NodeOp for ValueCurve {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let in_lo = ctx.param("in_lo");
        let in_hi = ctx.param("in_hi");
        let out_lo = ctx.param("out_lo");
        let out_hi = ctx.param("out_hi");
        let factor = ctx.param("factor");
        // Parse the curve ONCE per cook — an unset/malformed string is `None`,
        // which `shape_one` treats as the identity (a straight remap).
        let curve: Option<Curve> = ctx.text_param(CURVE_KEY).and_then(ph2d_curve::parse);
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| {
                let shaped = shape_one(v, in_lo, in_hi, out_lo, out_hi, curve.as_ref());
                blend_toward(v, shaped, factor)
            })
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueCurve))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Curve",
            // Utility grey: a value→value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamWidget};

/// O teto que a MÁQUINA (ou o bom senso) impõe, alcançável por DIGITAÇÃO — o slider fica
/// onde a MÃO trabalha (soft/hard do Blender; doc 88 §11). O curso de antes é este número:
/// nada ficou inalcançável, só deixou de ser o que o dedo percorre.
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "in_hi",
        max: 100.0,
    },
    ParamHardMax {
        param: "out_hi",
        max: 100.0,
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "in_lo",
        label: "In Low",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "in_hi",
        label: "In High",
        min: -100.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // The transfer shape — a TEXT param (`CURVE_KEY`), not a `ParamSpec`: the A1
    // draggable curve editor. Unset = identity (a straight remap).
    ParamUiHint {
        param: CURVE_KEY,
        label: "Curve",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Curve,
    },
    ParamUiHint {
        param: "out_lo",
        label: "Out Low",
        min: -100.0,
        max: 100.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "out_hi",
        label: "Out High",
        min: -100.0,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // Quanto da curva fica. `1` = a curva (o nó de sempre); `0` = a entrada
    // intacta. É o primeiro socket do *Float Curve* do Blender.
    ParamUiHint {
        param: "factor",
        label: "Factor",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// A tent: `0 → 1 → 0`, the shape no scalar remap can make (the point of the
    /// node). `eval(0) = 0`, `eval(0.5) = 1`, `eval(1) = 0`.
    const TENT: &str = "c1 0:0:L 0.5:1:L 1:0:L";

    /// **An unset curve is the identity** — a fresh node is exactly `value.map_range`
    /// with clamp on. The whole reason the node is safe to drop into a graph before
    /// drawing anything.
    #[test]
    fn an_unset_curve_is_a_straight_remap() {
        // 0..1 → 0..100, no curve: the midpoint lands at 50 (the linear answer).
        assert_eq!(shape_one(0.5, 0.0, 1.0, 0.0, 100.0, None), 50.0);
        assert_eq!(shape_one(0.0, 0.0, 1.0, 0.0, 100.0, None), 0.0);
        assert_eq!(shape_one(1.0, 0.0, 1.0, 0.0, 100.0, None), 100.0);
    }

    /// FALSIFICATION: a drawn curve bends the transfer. The tent sends the input
    /// MIDPOINT to the output HIGH (its peak) — the opposite of the linear
    /// midpoint. A straight-remap regression (ignoring the curve) would land 50.
    #[test]
    fn a_tent_curve_sends_the_midpoint_to_the_peak() {
        let tent = ph2d_curve::parse(TENT).expect("valid tent");
        // 0..1 → 0..100 through the tent: input 0.5 → peak → 100 (not 50).
        assert_eq!(shape_one(0.5, 0.0, 1.0, 0.0, 100.0, Some(&tent)), 100.0);
        // the ends of the tent are 0 → both extremes map to out_lo.
        assert_eq!(shape_one(0.0, 0.0, 1.0, 0.0, 100.0, Some(&tent)), 0.0);
        assert_eq!(shape_one(1.0, 0.0, 1.0, 0.0, 100.0, Some(&tent)), 0.0);
    }

    /// The input range normalizes into the curve's `[0,1]` domain, and an
    /// over-range input is CLAMPED (the curve is only defined on `[0,1]`): input
    /// 2.0 past `in_hi = 1` reads the curve at `t = 1`, not extrapolated.
    #[test]
    fn the_input_normalizes_and_clamps_into_the_curve_domain() {
        let tent = ph2d_curve::parse(TENT).expect("valid tent");
        // -1..1 → 0..10 (an LFO into a swing): input 0 is t=0.5 → tent peak → 10.
        assert_eq!(shape_one(0.0, -1.0, 1.0, 0.0, 10.0, Some(&tent)), 10.0);
        // input 5 (way past in_hi) clamps to t=1 → tent end 0 → out_lo 0.
        assert_eq!(shape_one(5.0, -1.0, 1.0, 0.0, 10.0, Some(&tent)), 0.0);
    }

    /// A degenerate input span (`in_lo == in_hi`) collapses `t` to 0 instead of
    /// dividing by zero into `NaN` — `out_lo + curve.eval(0)·span`.
    #[test]
    fn a_degenerate_input_span_never_produces_nan() {
        let v = shape_one(5.0, 3.0, 3.0, -2.0, 7.0, None);
        assert!(v.is_finite());
        assert_eq!(v, -2.0, "identity curve at t=0 collapses to out_lo");
    }

    /// The node-side LUT fill (A1-gpu) — the CI-runnable proof of the GPU half,
    /// since a device parity test needs an adapter. An unset string is the
    /// identity ramp; the tent's samples equal `Curve::eval` and its mid peaks.
    #[test]
    fn fill_curve_lut_samples_the_curve_and_falls_back_to_identity() {
        let mut buf = [0.0f32; 5];
        for bad in ["", "not a curve"] {
            fill_curve_lut(bad, &mut buf);
            for (k, v) in buf.iter().enumerate() {
                let t = k as f32 / 4.0;
                assert!((v - t).abs() < 1e-6, "identity ramp for {bad:?} at {k}");
            }
        }
        let tent = ph2d_curve::parse(TENT).expect("valid tent");
        fill_curve_lut(TENT, &mut buf);
        for (k, v) in buf.iter().enumerate() {
            assert!(
                (v - tent.eval(k as f32 / 4.0)).abs() < 1e-6,
                "tent sample {k}"
            );
        }
        assert!((buf[2] - 1.0).abs() < 1e-6, "the tent peaks at t = 0.5");
    }

    /// A value source emitting a fixed field, so `value.curve` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.curve.test.src"),
        name: "value.curve.test.src",
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

    /// End-to-end through the cook: a length-3 field `[0, 0.5, 1]` shaped by the
    /// tent (`0..1 → 0..1`) becomes `[0, 1, 0]` — the shape reaches the output and
    /// the length is preserved (the unary contract). The tent is authored via the
    /// text-param channel, exactly as the panel's Curve editor writes it.
    #[test]
    fn shapes_a_field_through_the_cook_via_the_text_param() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueCurve),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 0.5, 1.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.curve.test.src");
        let vc = g.add_node("value.curve");
        g.connect(Edge {
            from: (src, 0),
            to: (vc, 0),
            delayed: false,
        })
        .unwrap();
        g.set_text_param(vc, CURVE_KEY, TENT.to_string());
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vc, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 3, "unary: length 3 preserved");
                assert_eq!(v, &vec![0.0, 1.0, 0.0], "the tent shaped the field");
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

    /// **AS DUAS ANCORAS SAO VERBATIM** -- `factor = 1` devolve a curva bit-a-bit e
    /// `factor = 0` devolve a entrada bit-a-bit.
    ///
    /// ⚠️ **A mutacao que este gate mata e' escrever so' a algebra** (`a + f·(b−a)`
    /// sem ramo): ela erra as DUAS pontas por um ulp, e a ponta que importa e' a de
    /// cima -- e' o DEFAULT do no'. Um no' cujo default mudou por um ulp passa em
    /// todo teste de aparencia e reprova no unico que conta, o de identidade.
    #[test]
    fn the_two_anchors_are_verbatim() {
        let curve = ph2d_curve::parse(TENT);
        assert!(curve.is_some(), "a fixture tem de conter o fenomeno");
        for k in -40..=140 {
            let v = k as f32 * 0.01;
            let shaped = shape_one(v, 0.0, 1.0, -3.5, 7.25, curve.as_ref());
            assert_eq!(blend_toward(v, shaped, 1.0), shaped, "v={v}: factor 1");
            assert_eq!(blend_toward(v, shaped, 0.0), v, "v={v}: factor 0");
        }
    }

    /// **NO MEIO, E' A RETA ENTRE AS DUAS** -- e o gate mede onde a diferenca de
    /// facto existe (o pico da tenda), nao num ponto onde a curva ja' vale a
    /// entrada.
    #[test]
    fn half_a_factor_lands_halfway_between_the_input_and_the_curve() {
        let curve = ph2d_curve::parse(TENT);
        // No pico da tenda (t = 0.5) a saida moldada e' `out_hi` e a entrada e' 0.5:
        // as duas divergem ao maximo, que e' onde um factor se ve.
        let shaped = shape_one(0.5, 0.0, 1.0, 0.0, 1.0, curve.as_ref());
        assert_eq!(shaped, 1.0, "a fixture contem o fenomeno: pico em 1.0");
        let half = blend_toward(0.5, shaped, 0.5);
        assert!((half - 0.75).abs() < 1e-6, "meio caminho: {half}");
        // E e' MONOTONO de uma ponta a outra -- a propriedade que um artista dial.
        let mut prev = f32::NEG_INFINITY;
        for k in 0..=20 {
            let f = k as f32 / 20.0;
            let o = blend_toward(0.5, shaped, f);
            assert!(o >= prev, "factor={f}: {o} < {prev}");
            prev = o;
        }
    }

    /// **Acima de `1` ele EXTRAPOLA** -- a caricatura da curva, que um clamp
    /// proibiria em silencio. Declarado de proposito no doc de [`blend_toward`].
    #[test]
    fn past_one_the_factor_extrapolates_away_from_the_input() {
        let curve = ph2d_curve::parse(TENT);
        let shaped = shape_one(0.5, 0.0, 1.0, 0.0, 1.0, curve.as_ref());
        let over = blend_toward(0.5, shaped, 2.0);
        assert!((over - 1.5).abs() < 1e-6, "extrapolou para {over}");
    }
}
