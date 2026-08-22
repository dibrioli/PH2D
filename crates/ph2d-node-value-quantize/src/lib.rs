#![forbid(unsafe_code)]
//! `value.quantize` — the value-domain STAIRCASE: snap a value to a grid of
//! spacing `step`, the "stepped / posterized" look (Motion Nodes M2, the value
//! domain — doc 12/71). It is the third value SHAPER, alongside `value.map_range`
//! (linear) and `value.curve` (freeform): where those move a value smoothly, this
//! collapses a continuous field onto discrete levels — the signature motion-
//! graphics move (stop-motion cadence, chunky drives, retro quantization). Every
//! mature tool ships it: TouchDesigner's **Limit CHOP** (Quantize), Blender's
//! **Snap**/increment, After Effects' **Posterize**, Cavalry's **Quantize**.
//!
//! **Grid by SIZE, not by count** (doc 71): `q = round(v / step) · step` snaps `v`
//! to the nearest multiple of `step` — range-agnostic, so it composes (a
//! `value.map_range`/`value.curve` before or after sets the range). `step = 0.25`
//! turns a `[0,1]` ramp into `{0, 0.25, 0.5, 0.75, 1}`; `step = 0` is a passthrough
//! (the identity — safe to drop into a graph before choosing a grid).
//!
//! **`mode`** picks the rounding: **Round** (nearest — a symmetric staircase),
//! **Floor** (snaps DOWN — the sample-and-hold staircase, values never exceed the
//! input), **Ceil** (snaps UP). Round matches Rust's half-away-from-zero via the
//! `vq_round` helper (WGSL `round` is half-to-even), so CPU and GPU agree.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). A **unary** map — length preserved,
//! no broadcast decision (that lives in the consumer). `Pure` (no clock, no state).
//! Transcendental-free (HR-5): `/ × floor ceil` and one guarded compare. The GPU
//! kernel is the WGSL port of the same snap, so the node is **device-resident** —
//! it cooks on the GPU, no CPU fallback.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_value_curve::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, in and out (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// Below this the grid spacing is treated as zero — the node is a passthrough
/// (the identity), never a divide by (near-)zero.
const MIN_STEP: f32 = 1e-9;

/// The rounding mode: which multiple of `step` a value snaps to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Nearest multiple (a symmetric staircase). Half away from zero, matching
    /// Rust's `f32::round` and the WGSL `vq_round` helper.
    Round,
    /// The multiple at or below `v` (the sample-and-hold staircase — snaps down).
    Floor,
    /// The multiple at or above `v` (snaps up).
    Ceil,
    /// The multiple TOWARD ZERO — floor above zero, ceil below it (Blender
    /// *Float to Integer > Truncate*, TD Math CHOP *Integer*). It is the one
    /// mode that is **not** expressible as a single other mode, because it
    /// CHANGES which one it is at the origin; the chain that fakes it is
    /// `sign · floor(abs)` = three nodes.
    Truncate,
}

impl Mode {
    fn from_param(p: f32) -> Self {
        match p.round() as i32 {
            1 => Mode::Floor,
            2 => Mode::Ceil,
            3 => Mode::Truncate,
            _ => Mode::Round,
        }
    }
}

/// Snap one value to the grid: `q = mode(v / step) · step`. A (near-)zero `step`
/// is a passthrough — the identity, never a divide-by-zero `NaN`.
fn snap(v: f32, step: f32, mode: Mode) -> f32 {
    if step.abs() < MIN_STEP {
        return v;
    }
    let n = v / step;
    // `f32::round` is half-away-from-zero (== the WGSL `vq_round` helper); floor /
    // ceil agree between Rust and WGSL exactly.
    let k = match mode {
        Mode::Round => n.round(),
        Mode::Floor => n.floor(),
        Mode::Ceil => n.ceil(),
        Mode::Truncate => n.trunc(),
    };
    k * step
}

/// The node as the artist sees it: the staircase, on a grid whose **PHASE** the
/// `offset` moves. Quantise in the offset's frame and come back —
/// `snap(v − offset) + offset`.
///
/// **The offset is what makes a grid a grid the artist chose.** Houdini's `snap`
/// and TouchDesigner's Limit CHOP both expose it, and the reason is that a
/// staircase pinned to the origin can only ever land on multiples of `step`: an
/// artist quantising a rotation to 30° steps who wants the rest positions at 15°,
/// 45°, 75° has no way to say so. The composition that fakes it —
/// `math(Sub) → quantize → math(Add)` — is three nodes and the offset written
/// twice, which drifts the day one of the two is tuned.
///
/// ⚠️ **The degenerate guard is repeated here, and that is load-bearing, not
/// copy-paste.** `(v − offset) + offset` is **not** `v` in `f32`, so a passthrough
/// routed through the shift would move the value the node promised not to touch —
/// the identity would stop being the identity for exactly the artists who set an
/// offset. Both spellings read the SAME `MIN_STEP`, so there is one number here,
/// not two.
///
/// `offset = 0` reduces to [`snap`] **bit-for-bit**: `v − 0.0` and `k·step + 0.0`
/// are exact in IEEE-754 for every finite `v`.
fn quantize_one(v: f32, step: f32, mode: Mode, offset: f32) -> f32 {
    if step.abs() < MIN_STEP {
        return v;
    }
    snap(v - offset, step, mode) + offset
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.quantize"),
    name: "value.quantize",
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
            name: "step",
            default: 0.25,
        },
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // ⚠️ **Apendado**: a FASE da grade. `0` = a grade pinada na origem, o nó
        // que sempre shipou, bit-a-bit. Ver [`quantize_one`].
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — the WGSL port of [`quantize_one`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU (the "maximize GPU" north). VALUE in, VALUE out: the base rides here
/// (a VALUE stream carries only `v`, so "base + written `v`" and "a fresh stream
/// with `v`" are the same stream). The `MIN_STEP` guard is ported verbatim.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vq_step = params.step;\n\
        var vq = read_v(i);\n\
        if (abs(vq_step) >= 1e-9) {\n\
        \x20   // A FASE: quantiza no referencial do offset e volta. O ramo\n\
        \x20   // degenerado nunca chega aqui, e por isso o passe e' verbatim.\n\
        \x20   let vq_n = (vq - params.offset) / vq_step;\n\
        \x20   let vq_mode = i32(vq_round(params.mode));\n\
        \x20   var vq_k = vq_round(vq_n);\n\
        \x20   if (vq_mode == 1) { vq_k = floor(vq_n); }\n\
        \x20   else if (vq_mode == 2) { vq_k = ceil(vq_n); }\n\
        \x20   else if (vq_mode == 3) { vq_k = trunc(vq_n); }\n\
        \x20   vq = vq_k * vq_step + params.offset;\n\
        }\n\
        write_v(i, vq);\n",
    wgsl_lib: "\
        fn vq_round(x: f32) -> f32 {\n\
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
    // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
    // CPU, e o device recusa o shader (`invalid field accessor`).
    params: &["step", "mode", "offset"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValueQuantize;

impl NodeOp for ValueQuantize {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let step = ctx.param("step");
        let mode = Mode::from_param(ctx.param("mode"));
        let offset = ctx.param("offset");
        let input: Vec<f32> = match ctx.input(0).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let n = input.len();
        // Unary map — the field's length is preserved exactly.
        let out: Vec<f32> = input
            .iter()
            .map(|&v| quantize_one(v, step, mode, offset))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueQuantize))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Quantize",
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
    // The grid spacing. `0` is a passthrough (the identity); raise it for a
    // coarser staircase. In the input's units — a `[0,1]` ramp with 0.25 gives 5
    // levels.
    ParamUiHint {
        param: "step",
        label: "Step",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 3.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Round", "Floor", "Ceil", "Truncate"],
        },
    },
    // A FASE da grade, nas unidades da entrada. `0` = pinada na origem (o nó de
    // sempre). Meio degrau move os patamares para o meio entre eles.
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -1.0,
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

    /// **A `[0,1]` field snaps to a `0.25` grid** — the core behaviour. Round sends
    /// each value to its NEAREST multiple, so a smooth ramp becomes a staircase.
    #[test]
    fn round_snaps_to_the_nearest_multiple() {
        let q = |v| snap(v, 0.25, Mode::Round);
        assert_eq!(q(0.0), 0.0);
        assert_eq!(q(0.1), 0.0, "0.1 rounds down to 0");
        assert_eq!(q(0.2), 0.25, "0.2 rounds up to 0.25");
        assert_eq!(q(0.6), 0.5, "0.6 rounds down to 0.5");
        assert_eq!(q(0.9), 1.0, "0.9 rounds up to 1.0");
    }

    /// **Floor vs Ceil vs Round differ** — the whole point of the mode. At `v = 0.6`
    /// on a `0.25` grid: floor holds at 0.5, ceil jumps to 0.75, round nears 0.5.
    #[test]
    fn floor_and_ceil_bracket_the_value() {
        assert_eq!(snap(0.6, 0.25, Mode::Floor), 0.5, "floor snaps down");
        assert_eq!(snap(0.6, 0.25, Mode::Ceil), 0.75, "ceil snaps up");
        assert_eq!(snap(0.6, 0.25, Mode::Round), 0.5, "round nears 0.5");
        // Floor never exceeds the input; ceil is never below it.
        for k in 0..100 {
            let v = k as f32 * 0.037;
            assert!(snap(v, 0.25, Mode::Floor) <= v + 1e-6, "floor ≤ v at {v}");
            assert!(snap(v, 0.25, Mode::Ceil) >= v - 1e-6, "ceil ≥ v at {v}");
        }
    }

    /// **`step = 0` is a passthrough** — the identity, never a divide-by-zero NaN.
    /// The reason the node is safe to drop into a graph before choosing a grid.
    #[test]
    fn a_zero_step_is_a_passthrough() {
        for k in 0..50 {
            let v = k as f32 * 0.13 - 3.0;
            let out = snap(v, 0.0, Mode::Round);
            assert!(out.is_finite(), "finite at {v}");
            assert_eq!(out, v, "step 0 passes the value through unchanged");
        }
    }

    /// **Negative values snap symmetrically** (round is half-away-from-zero, the
    /// GPU `vq_round` match): `-0.6` on a `0.25` grid rounds to `-0.5`, not `-0.75`.
    #[test]
    fn negatives_round_half_away_from_zero() {
        assert_eq!(snap(-0.6, 0.25, Mode::Round), -0.5);
        assert_eq!(
            snap(-0.125, 0.25, Mode::Round),
            -0.25,
            "0.5 case away from 0"
        );
        assert_eq!(
            snap(-0.6, 0.25, Mode::Floor),
            -0.75,
            "floor snaps toward -inf"
        );
        assert_eq!(snap(-0.6, 0.25, Mode::Ceil), -0.5, "ceil snaps toward +inf");
    }

    /// A value source emitting a fixed field, so `value.quantize` can be driven
    /// through a real cook (the whole-chain proof, not just the math).
    static SRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.quantize.test.src"),
        name: "value.quantize.test.src",
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

    /// End-to-end through the cook: a length-5 ramp `[0, 0.25, 0.5, 0.75, 1]` is
    /// already on the `0.25` grid, so it survives; the OFF-grid `[0.1, 0.4, 0.9]`
    /// snaps to `[0, 0.5, 1]`. The length is preserved (the unary contract).
    #[test]
    fn quantizes_a_field_through_the_cook() {
        struct Ops(Vec<f32>);
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC_MAN.id => {
                        Some(Box::leak(Box::new(Src(self.0.clone()))) as &dyn NodeOp)
                    }
                    t if t == MANIFEST.id => Some(&ValueQuantize),
                    _ => None,
                }
            }
        }
        let ops = Ops(vec![0.1, 0.4, 0.9]);
        let mut g = Graph::new();
        let src = g.add_node("value.quantize.test.src");
        let vq = g.add_node("value.quantize");
        g.set_param(vq, "step", 0.5); // a coarse grid: {0, 0.5, 1}
        g.connect(Edge {
            from: (src, 0),
            to: (vq, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &ops, vq, 0.0).unwrap();
        match out[0].as_stream().get(VALUE_COL).unwrap() {
            Column::Scalar(v) => {
                assert_eq!(v.len(), 3, "unary: length 3 preserved");
                assert_eq!(v, &vec![0.0, 0.5, 1.0], "snapped to the 0.5 grid");
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

    /// **Truncate e' Floor acima de zero e Ceil abaixo** -- a unica modalidade
    /// que TROCA de identidade na origem, e por isso a unica que nenhuma das
    /// outras exprime. Uma fixture so' com positivos nao a distinguiria do
    /// Floor: e' por isso que ela atravessa o eixo.
    #[test]
    fn truncate_is_floor_above_zero_and_ceil_below() {
        for k in 1..40 {
            let v = k as f32 * 0.07;
            assert_eq!(
                snap(v, 0.25, Mode::Truncate),
                snap(v, 0.25, Mode::Floor),
                "acima de zero: v={v}"
            );
            assert_eq!(
                snap(-v, 0.25, Mode::Truncate),
                snap(-v, 0.25, Mode::Ceil),
                "abaixo de zero: v={}",
                -v
            );
        }
        // E onde importa, ele DIFERE do Floor -- o par que o gate existe para
        // separar.
        assert_eq!(snap(-0.6, 0.25, Mode::Truncate), -0.5);
        assert_eq!(snap(-0.6, 0.25, Mode::Floor), -0.75);
    }

    /// **Truncate nunca AUMENTA a magnitude** -- o que "snap para zero" promete,
    /// afirmado como propriedade dos dois lados do eixo em vez de num ponto.
    #[test]
    fn truncate_never_grows_the_magnitude() {
        for k in -60..60 {
            let v = k as f32 * 0.037;
            let q = snap(v, 0.25, Mode::Truncate);
            assert!(
                q.abs() <= v.abs() + 1e-6,
                "v={v} -> {q}: a magnitude cresceu"
            );
            assert!(
                q == 0.0 || q.signum() == v.signum(),
                "v={v} -> {q}: trocou de lado do eixo"
            );
        }
    }

    /// **`offset = 0` e' o no' que sempre shipou -- BIT-A-BIT**, nos quatro modos
    /// e numa faixa de passos. `v − 0.0` e `k·step + 0.0` sao exactos em IEEE-754,
    /// entao isto e' `assert_eq!` de f32 e nao um epsilon.
    #[test]
    fn a_zero_offset_is_the_node_that_shipped_bit_for_bit() {
        for &m in &[Mode::Round, Mode::Floor, Mode::Ceil, Mode::Truncate] {
            for &s in &[0.0, 0.1, 0.25, 1.0, 3.7] {
                for k in -80..80 {
                    let v = k as f32 * 0.041;
                    assert_eq!(quantize_one(v, s, m, 0.0), snap(v, s, m), "v={v} s={s}");
                }
            }
        }
    }

    /// **O OFFSET MOVE OS PATAMARES, e o resultado continua NA grade deslocada.**
    ///
    /// O oraculo e' a definicao, nao um valor a olho: toda saida tem de ser
    /// `offset + k·step` para um inteiro `k`. Uma implementacao que somasse o
    /// offset **sem** o subtrair antes passaria um teste de "mudou alguma coisa" e
    /// reprovaria este -- ela deslocaria a grade E o valor, e a saida cairia entre
    /// os degraus.
    #[test]
    fn the_offset_moves_the_treads_and_the_result_stays_on_the_shifted_grid() {
        let (step, off) = (0.25f32, 0.125f32);
        for k in -80..80 {
            let v = k as f32 * 0.041;
            let q = quantize_one(v, step, Mode::Round, off);
            let n = ((q - off) / step).round();
            assert!(
                ((q - off) - n * step).abs() < 1e-5,
                "v={v} -> {q}: fora da grade deslocada"
            );
            // E o erro nunca passa de meio degrau (a lei do Round, preservada).
            assert!((q - v).abs() <= 0.5 * step + 1e-5, "v={v} -> {q}: saltou");
        }
        // Meio degrau poe os patamares no MEIO entre os antigos -- a razao de ser
        // do knob, num ponto que qualquer um confere de cabeca.
        assert_eq!(quantize_one(0.125, 0.25, Mode::Round, 0.125), 0.125);
        assert_eq!(quantize_one(0.25, 0.25, Mode::Round, 0.125), 0.375);
        // ⚠️ E o EMPATE cai onde a lei do `Round` manda (metade PARA LONGE do
        // zero), medido no referencial DESLOCADO: `v = 0` fica a meio caminho de
        // `−0,125` e `+0,125`, e a regra o manda para `−0,125`. Nao e' um capricho
        // do teste -- e' a mesma `f32::round` que o `vq_round` do WGSL espelha, e
        // um empate resolvido de outra forma seria uma divergencia CPU/GPU.
        assert_eq!(quantize_one(0.0, 0.25, Mode::Round, 0.125), -0.125);
    }

    /// **Um passo degenerado e' passe VERBATIM mesmo com offset** -- o gate da
    /// guarda repetida.
    ///
    /// ⚠️ **A mutacao que este gate mata e' delegar a guarda:** `snap(v−o) + o`
    /// com passo zero devolve `(v−o)+o`, que em `f32` **nao e'** `v` (ex.: `v =
    /// 1e-7`, `o = 1000` perde toda a mantissa). A identidade deixaria de ser a
    /// identidade exactamente para quem pos um offset.
    #[test]
    fn a_degenerate_step_is_a_verbatim_passthrough_even_with_an_offset() {
        for &off in &[0.0, 0.125, -3.0, 1000.0] {
            for k in -40..40 {
                let v = k as f32 * 0.037;
                assert_eq!(quantize_one(v, 0.0, Mode::Round, off), v, "v={v} off={off}");
            }
            // O caso que a guarda delegada perderia: mantissa engolida pelo offset.
            assert_eq!(quantize_one(1e-7, 0.0, Mode::Round, 1000.0), 1e-7);
        }
    }
}
