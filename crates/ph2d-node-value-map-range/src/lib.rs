//! `value.map_range` — the value-domain GLUE: linearly remap a **value** field
//! from an input range to an output range (Motion Nodes M2, the value domain —
//! doc 12). This is the single most-used node in every mature value graph:
//! Houdini's `fit()`, TouchDesigner's **Math CHOP** Range, Nuke/Cavalry "Remap",
//! Max's `scale`. It is what turns a raw oscillation or a normalized count into
//! the units a channel actually wants (a `[-1,1]` LFO → a `[0,90]°` swing).
//!
//! **Semantics** (the reference convergence): map `[in_lo, in_hi] → [out_lo,
//! out_hi]` linearly. `t = (v − in_lo) / (in_hi − in_lo)`, then
//! `out = out_lo + t · (out_hi − out_lo)`. Clamping is applied to the *normalized*
//! `t ∈ [0,1]` (so inverted output ranges stay honest) — **on by default**,
//! matching Houdini's canonical `fit()` (turn it off for `efit`-style
//! extrapolation). A degenerate input span (`in_hi == in_lo`) can't divide, so
//! the whole input collapses to `out_lo` (the documented guard) instead of
//! producing `NaN`.
//!
//! **The value type** is the continuous per-instance field `(Instances, Scalar,
//! Frame)` on the `v` column (doc 12). This is a **unary** map — it preserves the
//! field's length exactly (no broadcast decision to make; that rule lives in the
//! *consumer*, `motion.drive`). `Pure` (no clock, no state). Transcendental-free:
//! only `+ − × ÷` and `clamp` (HR-5; division is IEEE-deterministic).

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// Below this the input span is treated as degenerate (collapses to `out_lo`),
/// so `map_range` never divides by (near-)zero.
const MIN_SPAN: f32 = 1e-9;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.map_range"),
    name: "value.map_range",
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
        // 0 off (extrapolate, `efit`) · 1 on (clamp to the output range, `fit`).
        // On by default — the Houdini `fit()` convention.
        ParamSpec {
            name: "clamp",
            default: 1.0,
        },
        // 0 Linear · 1 Stepped · 2 Smooth · 3 Smoother — Blender's Map Range
        // dropdown, verbatim. Linear is the default, so a graph built before this
        // param existed is byte-identical.
        ParamSpec {
            name: "interpolation",
            default: 0.0,
        },
        // How many steps the Stepped ramp visits; `steps + 1` levels, so `4` gives
        // {0, ¼, ½, ¾, 1}. Ignored by every other interpolation.
        ParamSpec {
            name: "steps",
            default: 4.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// How the normalised parameter `t` is shaped on its way from the input range to
/// the output range.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Interp {
    /// `t` unchanged — the straight line this node always drew.
    Linear,
    /// `t` snapped to a staircase of `steps + 1` levels.
    Stepped,
    /// The Hermite cubic `3t² − 2t³` — an ease at both ends.
    Smooth,
    /// Perlin's quintic `6t⁵ − 15t⁴ + 10t³` — the ease whose SECOND derivative
    /// also vanishes at the ends. Inexpressible as a chain of the other nodes
    /// (no node computes a fifth power), which is why it is the item that made
    /// this dropdown worth building.
    Smoother,
}

impl Interp {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Interp::Stepped,
            2 => Interp::Smooth,
            3 => Interp::Smoother,
            _ => Interp::Linear,
        }
    }
}

/// Shape the normalised parameter. ⚠️ **The two easings clamp `t` regardless of
/// the `clamp` toggle**, and that is not an oversight: `3t² − 2t³` outside `[0,1]`
/// runs AWAY from the range (it is a cubic, not a sigmoid), so "extrapolate with
/// an ease" is not a thing anyone means. Blender greys the Clamp checkbox out for
/// exactly these two. Linear and Stepped honour the toggle, which is where
/// extrapolation is meaningful.
fn shape_t(t: f32, interp: Interp, steps: f32, clamp: bool) -> f32 {
    match interp {
        Interp::Linear => {
            if clamp {
                t.clamp(0.0, 1.0)
            } else {
                t
            }
        }
        Interp::Stepped => {
            let t = if clamp { t.clamp(0.0, 1.0) } else { t };
            // `steps` steps means `steps + 1` levels (Blender). Below one step
            // there is no staircase to build, so the ramp collapses to its floor.
            let t = if steps >= 1.0 {
                (t * (steps + 1.0)).floor() / steps
            } else {
                0.0
            };
            // ⚠️ The second clamp is the one that makes `t = 1` land on `1` and
            // not on `(steps+1)/steps`: the top level is a full step wide and its
            // right edge is the only sample that falls into the level above.
            if clamp { t.clamp(0.0, 1.0) } else { t }
        }
        Interp::Smooth => {
            let t = t.clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        }
        Interp::Smoother => {
            let t = t.clamp(0.0, 1.0);
            t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
        }
    }
}

/// Remap one value from `[in_lo, in_hi]` to `[out_lo, out_hi]`. Clamping (when
/// `clamp`) is applied to the normalized parameter `t ∈ [0,1]`, so inverted
/// output ranges stay within bounds. A degenerate input span collapses to
/// `out_lo`.
/// ⚠️ Oito argumentos e não uma struct: eles são os **params do manifesto**, um a
/// um, e é isso que faz o `eval` ler como a lista que o artista vê no painel. Uma
/// struct intermediária seria uma segunda descrição da mesma tabela.
#[allow(clippy::too_many_arguments)]
fn map_one(
    v: f32,
    in_lo: f32,
    in_hi: f32,
    out_lo: f32,
    out_hi: f32,
    clamp: bool,
    interp: Interp,
    steps: f32,
) -> f32 {
    let span = in_hi - in_lo;
    if span.abs() < MIN_SPAN {
        return out_lo;
    }
    let t = shape_t((v - in_lo) / span, interp, steps, clamp);
    out_lo + t * (out_hi - out_lo)
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`map_one`].
///
/// VALUE in, VALUE out: the base DOES ride here, and that is correct — a VALUE
/// stream carries only `v`, so "base + written `v`" and "a fresh stream with `v`"
/// are the same stream. (The distinction only bites when the KINDS differ, as in
/// `motion.luminance`.)
///
/// The `MIN_SPAN` guard is ported verbatim, not approximated: a zero input span
/// is a divide, and the CPU answers `out_lo` there. `clamp` is a boolean-ish
/// param compared at `> 0.5`, exactly as the CPU reads it.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let mr_span = params.in_hi - params.in_lo;\n\
        var mr_out = params.out_lo;\n\
        if (abs(mr_span) >= 1e-9) {\n\
        \x20   var mr_t = (read_v(i) - params.in_lo) / mr_span;\n\
        \x20   let mr_ip = i32(mr_round(params.interpolation));\n\
        \x20   let mr_cl = params.clamp > 0.5;\n\
        \x20   if (mr_ip == 2) {\n\
        \x20       mr_t = clamp(mr_t, 0.0, 1.0);\n\
        \x20       mr_t = mr_t * mr_t * (3.0 - 2.0 * mr_t);\n\
        \x20   } else if (mr_ip == 3) {\n\
        \x20       mr_t = clamp(mr_t, 0.0, 1.0);\n\
        \x20       mr_t = mr_t * mr_t * mr_t * (mr_t * (mr_t * 6.0 - 15.0) + 10.0);\n\
        \x20   } else if (mr_ip == 1) {\n\
        \x20       if (mr_cl) { mr_t = clamp(mr_t, 0.0, 1.0); }\n\
        \x20       if (params.steps >= 1.0) {\n\
        \x20           mr_t = floor(mr_t * (params.steps + 1.0)) / params.steps;\n\
        \x20       } else { mr_t = 0.0; }\n\
        \x20       if (mr_cl) { mr_t = clamp(mr_t, 0.0, 1.0); }\n\
        \x20   } else {\n\
        \x20       if (mr_cl) { mr_t = clamp(mr_t, 0.0, 1.0); }\n\
        \x20   }\n\
        \x20   mr_out = params.out_lo + mr_t * (params.out_hi - params.out_lo);\n\
        }\n\
        write_v(i, mr_out);\n",
    wgsl_lib: "\
        fn mr_round(x: f32) -> f32 {\n\
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
    params: &[
        "in_lo",
        "in_hi",
        "out_lo",
        "out_hi",
        "clamp",
        "interpolation",
        "steps",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueMapRange;

impl NodeOp for ValueMapRange {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let in_lo = ctx.param("in_lo");
        let in_hi = ctx.param("in_hi");
        let out_lo = ctx.param("out_lo");
        let out_hi = ctx.param("out_hi");
        let clamp = ctx.param("clamp") > 0.5;
        let interp = Interp::from_param(ctx.param("interpolation"));
        let steps = ctx.param("steps");
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| map_one(v, in_lo, in_hi, out_lo, out_hi, clamp, interp, steps))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueMapRange))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Map Range",
            // Utility grey: a value→value transformer, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamHardMax, ParamUiHint, ParamWidget};

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
    // ⚠️ O recurso é **PRECISÃO DE REPRESENTAÇÃO**, e o número é derivado, não
    // escolhido: o degrau sai de `floor(t · (steps + 1))`, e `2^24` é o último
    // inteiro que um `f32` resolve — acima dele dois níveis vizinhos caem no
    // MESMO float e o controle deixa de controlar. O slider para em 32 porque é
    // onde a mão trabalha; o resto continua digitável.
    ParamHardMax {
        param: "steps",
        max: 16_777_215.0,
    },
];

/// O `steps` só existe para a rampa em DEGRAUS — nos outros três ele é inerte, e
/// *um controle que não faz nada não é pintado*.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: "steps",
        when: "interpolation",
        values: &[1],
    },
    // ⚠️ **`Smooth` e `Smoother` clampam INCONDICIONALMENTE** — o `shape_t` deles fecha a
    // faixa antes de olhar para o toggle, então ali o `Clamp` é um controle mudo (doc 90 §2).
    // O próprio doc-comment do nó já notava que *"o Blender cinzenta a caixa Clamp para
    // exactamente estes dois"*; faltava executá-lo.
    // `0 = Linear` · `1 = Stepped` · `2 = Smooth` · `3 = Smoother`.
    ParamGate {
        param: "clamp",
        when: "interpolation",
        values: &[0, 1],
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
    ParamUiHint {
        param: "clamp",
        label: "Clamp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
    ParamUiHint {
        param: "interpolation",
        label: "Interpolation",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Stepped", "Smooth", "Smoother"],
        },
    },
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 1.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

    /// A value source emitting a fixed field, so map_range can be driven through
    /// a real cook.
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.map_range.test.src"),
        name: "value.map_range.test.src",
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

    // Direct unit tests of the core (no cook needed for the math).
    #[test]
    fn maps_the_canonical_ranges() {
        // 0..1 → 0..100: the midpoint lands at 50.
        assert_eq!(
            map_one(0.5, 0.0, 1.0, 0.0, 100.0, true, Interp::Linear, 4.0),
            50.0
        );
        // -1..1 → 0..90 (an LFO → a degree swing): 0 → 45.
        assert_eq!(
            map_one(0.0, -1.0, 1.0, 0.0, 90.0, true, Interp::Linear, 4.0),
            45.0
        );
        assert_eq!(
            map_one(-1.0, -1.0, 1.0, 0.0, 90.0, true, Interp::Linear, 4.0),
            0.0
        );
        assert_eq!(
            map_one(1.0, -1.0, 1.0, 0.0, 90.0, true, Interp::Linear, 4.0),
            90.0
        );
    }

    /// FALSIFICATION of the clamp path: with clamp ON an over-range input pins to
    /// the output bound; with clamp OFF it extrapolates past it (`efit`).
    #[test]
    fn clamp_pins_the_output_and_off_extrapolates() {
        // input 2.0 is past in_hi=1 → clamped to out_hi=10.
        assert_eq!(
            map_one(2.0, 0.0, 1.0, 0.0, 10.0, true, Interp::Linear, 4.0),
            10.0
        );
        // same input, clamp off → extrapolates to 20.
        assert_eq!(
            map_one(2.0, 0.0, 1.0, 0.0, 10.0, false, Interp::Linear, 4.0),
            20.0
        );
    }

    /// An INVERTED output range stays within bounds under clamp (the reason to
    /// clamp `t`, not the raw output): 0..1 → 10..0, input 2 pins to 0, not -10.
    #[test]
    fn an_inverted_output_range_stays_in_bounds() {
        assert_eq!(
            map_one(2.0, 0.0, 1.0, 10.0, 0.0, true, Interp::Linear, 4.0),
            0.0
        );
        assert_eq!(
            map_one(0.0, 0.0, 1.0, 10.0, 0.0, true, Interp::Linear, 4.0),
            10.0
        );
    }

    /// A degenerate input span (`in_lo == in_hi`) collapses to `out_lo` instead
    /// of dividing by zero into `NaN`.
    #[test]
    fn a_degenerate_input_span_never_produces_nan() {
        let v = map_one(5.0, 3.0, 3.0, -2.0, 7.0, true, Interp::Linear, 4.0);
        assert!(v.is_finite());
        assert_eq!(v, -2.0, "collapses to out_lo");
    }

    /// End-to-end through the cook: a length-2 value field is remapped
    /// element-wise, length preserved (the unary contract).
    #[test]
    fn remaps_a_field_through_the_cook_preserving_length() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                // Leaked to satisfy the `'static` op borrow in this tiny harness.
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueMapRange),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.0, 1.0]);
        let mut g = Graph::new();
        let src = g.add_node("value.map_range.test.src");
        let mr = g.add_node("value.map_range");
        g.connect(Edge {
            from: (src, 0),
            to: (mr, 0),
            delayed: false,
        })
        .unwrap();
        set_range(&mut g, mr, 0.0, 1.0, 0.0, 10.0);
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, mr, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => assert_eq!(v, &vec![0.0, 10.0], "element-wise, length 2 kept"),
            _ => panic!("v"),
        }
    }

    fn set_range(g: &mut Graph, n: NodeId, il: f32, ih: f32, ol: f32, oh: f32) {
        g.set_param(n, "in_lo", il);
        g.set_param(n, "in_hi", ih);
        g.set_param(n, "out_lo", ol);
        g.set_param(n, "out_hi", oh);
    }

    #[test]
    fn registers_and_resolves() {
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());
    }

    /// **`Linear` e' o default E o mundo anterior a este dropdown, ao BIT.** O
    /// oraculo e' a expressao que shipava, escrita a' mao aqui -- chamar a
    /// funcao sob teste para computar o que se espera e' o gate sempre-verde
    /// que esta casa ja' documentou tres vezes.
    #[test]
    fn linear_is_byte_identical_to_the_world_before_the_dropdown() {
        for k in 0..120 {
            let v = k as f32 * 0.017 - 0.4;
            let now = map_one(v, 0.0, 1.0, 2.0, 5.0, true, Interp::Linear, 4.0);
            let before = {
                let mut t = (v - 0.0) / 1.0;
                t = t.clamp(0.0, 1.0);
                2.0 + t * (5.0 - 2.0)
            };
            assert_eq!(now.to_bits(), before.to_bits(), "v={v}");
        }
    }

    /// **A escada visita `steps + 1` niveis, e o topo e' UM deles.** O segundo
    /// clamp existe exactamente para isto: sem ele `t = 1` aterraria em
    /// `(steps+1)/steps`, um nivel acima do alcance que o artista pediu.
    #[test]
    fn the_stepped_ramp_visits_steps_plus_one_levels() {
        let mut seen: Vec<f32> = Vec::new();
        for k in 0..=400 {
            let t = k as f32 / 400.0;
            let o = map_one(t, 0.0, 1.0, 0.0, 1.0, true, Interp::Stepped, 4.0);
            if !seen.iter().any(|s| (s - o).abs() < 1e-6) {
                seen.push(o);
            }
        }
        seen.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(seen.len(), 5, "niveis: {seen:?}");
        assert!(seen[0].abs() < 1e-6, "o piso e' 0: {seen:?}");
        assert!(
            (seen[4] - 1.0).abs() < 1e-6,
            "o topo e' 1 e nao 1.25: {seen:?}"
        );
    }

    /// **Menos de um degrau nao e' uma escada** -- `steps = 0` colapsa no piso em
    /// vez de dividir por zero.
    #[test]
    fn fewer_than_one_step_collapses_to_the_floor() {
        for k in 0..20 {
            let t = k as f32 / 19.0;
            let o = map_one(t, 0.0, 1.0, 3.0, 9.0, true, Interp::Stepped, 0.0);
            assert!(o.is_finite() && (o - 3.0).abs() < 1e-6, "t={t} -> {o}");
        }
    }

    /// **As duas easings pinam os extremos e o meio; a quintica e' a mais chata
    /// nas pontas.** Sem a segunda metade, uma quintica trocada pela cubica
    /// passaria.
    #[test]
    fn the_two_easings_pin_the_ends_and_the_middle() {
        for &t in &[0.0, 0.5, 1.0] {
            let c = map_one(t, 0.0, 1.0, 0.0, 1.0, true, Interp::Smooth, 4.0);
            let q = map_one(t, 0.0, 1.0, 0.0, 1.0, true, Interp::Smoother, 4.0);
            assert!((c - q).abs() < 1e-6, "t={t}: {c} vs {q}");
            assert!((c - t).abs() < 1e-6, "t={t}: a easing tem de fixar {t}");
        }
        let c = map_one(0.25, 0.0, 1.0, 0.0, 1.0, true, Interp::Smooth, 4.0);
        let q = map_one(0.25, 0.0, 1.0, 0.0, 1.0, true, Interp::Smoother, 4.0);
        assert!(q < c - 0.04, "no quarto: cubica {c}, quintica {q}");
    }

    /// ⚠️ **As easings clampam mesmo com o toggle DESLIGADO, e o Linear nao** --
    /// a decisao esta' escrita no `shape_t`, e este gate e' o que a torna
    /// executavel. O CONTROLE (o Linear a extrapolar de facto) e' a metade sem a
    /// qual "as easings clampam" seria verdade num mundo em que TUDO clampa.
    #[test]
    fn the_easings_clamp_even_with_the_toggle_off() {
        let hi = map_one(3.0, 0.0, 1.0, 0.0, 1.0, false, Interp::Smooth, 4.0);
        assert!((hi - 1.0).abs() < 1e-6, "smooth acima do alcance: {hi}");
        let lo = map_one(-3.0, 0.0, 1.0, 0.0, 1.0, false, Interp::Smoother, 4.0);
        assert!(lo.abs() < 1e-6, "smoother abaixo do alcance: {lo}");
        let ex = map_one(3.0, 0.0, 1.0, 0.0, 1.0, false, Interp::Linear, 4.0);
        assert!(ex > 2.9, "o CONTROLE tem de extrapolar: {ex}");
    }
}
