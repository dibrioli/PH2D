#![forbid(unsafe_code)]
//! `motion.cull` — **prune the instance stream by a predicate**: the Houdini "Blast" /
//! "Delete SOP" (Motion Nodes M3, structure — doc 01 §3 / doc 27). The first node that
//! *shrinks* the count (mirror/kaleidoscope grow it; sort reorders it). Downstream of a
//! `motion.sort`, an animated cull is a reveal: sort **radial** + cull a growing
//! **fraction** wipes a layout in from the centre; sort **random** + cull dissolves it.
//!
//! **Algorithm — keep the elements passing the predicate, filter every column.** Two
//! modes: **Fraction** keeps the first `amount·n` elements (so it reveals in the
//! upstream order — pair with `sort`); **Falloff** keeps the elements whose `falloff`
//! column is ≥ `amount` (a spatial mask — pair with `motion.falloff`, absent column
//! reads as 1). `invert` keeps the complement. The surviving indices gather every column
//! (`P`, `size`, `tint`, …), so the kept instances stay intact at a smaller count. An
//! `amount` **value** input (unconnected → the param) animates the reveal, so a
//! `value.lfo` sweeps it. Transcendental-free (HR-5): counting and comparison only.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, KEEP_FLAG_COL, StreamOp};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// Prune modes (the `mode` param).
const MODE_FRACTION: i64 = 0;
// MODE_FALLOFF (1) = keep where the `falloff` column ≥ amount.
/// **O TETO DE POPULAÇÃO** (doc 89, folha 13): mantém no máximo `max` elementos, e **os mais
/// NOVOS**. Abaixo do teto é um NO-OP, que é a diferença inteira para o Fraction — uma fração
/// rala a população o tempo todo, um teto só morde quando é atingido.
const MODE_COUNT: i64 = 2;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.cull"),
    name: "motion.cull",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // The keep fraction / threshold (animatable): unconnected reads the `amount`
        // param. A `value.lfo` sweeps the reveal.
        PortSpec {
            name: "amount",
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
        // 0 Fraction (keep first amount·n) · 1 Falloff (keep falloff ≥ amount).
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        // The fraction (Fraction mode) or threshold (Falloff mode).
        ParamSpec {
            name: "amount",
            default: 1.0,
        },
        // 0 keep the passing set · 1 keep the complement.
        ParamSpec {
            name: "invert",
            default: 0.0,
        },
        // **O teto de população** (modo Count). APENDADO; inerte nos dois modos que já existiam,
        // então todo grafo salvo é byte-idêntico. O default é o **512 do `motion.emitter`** — o
        // número do irmão, não um inventado: escolher Count tem de dar um teto que já funciona.
        ParamSpec {
            name: "max",
            default: 512.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// The `amount`: the value input's first element if connected, else the param.
fn amount_of(value: &[f32], param: f32) -> f32 {
    value.first().copied().unwrap_or(param)
}

/// The indices that survive the cull, in their original order.
fn keep_indices(n: usize, mode: i64, amount: f32, invert: bool, falloff: &[f32]) -> Vec<usize> {
    let pass: Box<dyn Fn(usize) -> bool> = match mode {
        MODE_FRACTION => {
            // Round the fraction to a count of leading elements.
            let keep =
                ((amount.clamp(0.0, 1.0) * n as f32).round() as i64).clamp(0, n as i64) as usize;
            Box::new(move |i: usize| i < keep)
        }
        MODE_COUNT => {
            // ⚠️ **O teto mantém a CAUDA, e a razão está escrita no irmão** (`motion.emitter`,
            // que a folha 13 cita como *"o irmão já tem"*): *"the cap keeps the NEWEST
            // particles: an emitter whose rate outruns the cap should look like a dense young
            // jet, not a frozen ancient cloud"*. Numa zona a ordem é o `motion.combine`
            // apendando os recém-nascidos ao estado ⇒ o PREFIXO é o mais velho, então um teto
            // escrito como o Fraction (*"os primeiros N"*) seria exatamente a nuvem congelada
            // que aquele comentário recusa: a população enche, e nada novo aparece nunca mais.
            let cap = (amount.max(0.0).round() as i64).clamp(0, n as i64) as usize;
            let first = n - cap;
            Box::new(move |i: usize| i >= first)
        }
        _ => {
            // Falloff: keep where the mask (absent → 1) is at or above the threshold.
            let f = falloff.to_vec();
            Box::new(move |i: usize| f.get(i).copied().unwrap_or(1.0) >= amount)
        }
    };
    (0..n).filter(|&i| pass(i) != invert).collect()
}

/// Gather a column at the kept indices (`out[k] = col[keep[k]]`).
fn gather_col(col: &Column, keep: &[usize]) -> Column {
    fn take<T: Clone>(v: &[T], keep: &[usize]) -> Vec<T> {
        keep.iter().map(|&i| v[i].clone()).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(take(v, keep)),
        Column::Vec2(v) => Column::Vec2(take(v, keep)),
        Column::Vec3(v) => Column::Vec3(take(v, keep)),
        Column::Vec4(v) => Column::Vec4(take(v, keep)),
    }
}

/// The GPU predicate (ADR-0136): one flag per element, exactly
/// [`keep_indices`]'s `pass(i) != invert`, in the same expressions.
///
/// - **Count** keeps the LAST `round(max)` — a teto, então `i >= n − cap`. A cauda e não o
///   prefixo, pela razão que o `motion.emitter` escreveu: um teto que mantivesse o prefixo
///   congelaria a população numa nuvem antiga.
/// - **Fraction** keeps the first `round(amount·n)` — `floor(x + 0.5)` because
///   WGSL's `round` is half-to-even while Rust's is half-away (the value is
///   non-negative here, where the two agree on `floor(x+0.5)`). The count and
///   the compare both stay under 2²⁴, where `f32` holds integers exactly.
/// - **Falloff** keeps `falloff ≥ amount`; an absent column reads its identity
///   `1.0` (the CPU's `unwrap_or(1.0)`).
/// - `amount` comes from the value input's ROW 0 when connected — the CPU's
///   `value.first()`, which reads element 0 of a field of ANY length — else the
///   param (`HAS_v`).
/// - The mode/invert tests are the CPU casts made explicit: `round(x) as i64 == 0`
///   ⇔ `abs(x) < 0.5`, `round(x) as i64 != 0` ⇔ `abs(x) ≥ 0.5`, for finite x.
///
/// The node's own kernel is [`GpuKernel::PASSTHROUGH`] — the compaction IS the
/// node.
const GPU_PREDICATE: GpuKernel = GpuKernel {
    wgsl: "\
        let cl_is_count = abs(params.mode - 2.0) < 0.5;\n\
        let cl_param = select(params.amount, params.max, cl_is_count);\n\
        let cl_amount = select(cl_param, read_amount_v(0u), HAS_amount_v);\n\
        var cl_pass: bool;\n\
        if (cl_is_count) {\n\
        \x20   let cl_cap = clamp(floor(max(cl_amount, 0.0) + 0.5), 0.0, f32(params.count));\n\
        \x20   cl_pass = f32(i) >= f32(params.count) - cl_cap;\n\
        } else if (abs(params.mode) < 0.5) {\n\
        \x20   let cl_frac = clamp(cl_amount, 0.0, 1.0);\n\
        \x20   let cl_keep = clamp(floor(cl_frac * f32(params.count) + 0.5), 0.0, f32(params.count));\n\
        \x20   cl_pass = f32(i) < cl_keep;\n\
        } else {\n\
        \x20   cl_pass = read_in_falloff(i) >= cl_amount;\n\
        }\n\
        let cl_invert = abs(params.invert) >= 0.5;\n\
        write_cp_keep(i, select(0.0, 1.0, cl_pass != cl_invert));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            // The spatial mask; absent reads 1 (everything passes the threshold
            // at full falloff) — the CPU's `unwrap_or(1.0)`.
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
        ColumnBinding {
            // The animatable amount (port 1). Broadcast: connected at ANY
            // pairable length, only row 0 is read (`amount_of`'s `first()`).
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: KEEP_FLAG_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["mode", "amount", "invert", "max"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct MotionCull;

impl NodeOp for MotionCull {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mode = ctx.param("mode").round() as i64;
        let invert = ctx.param("invert").round() as i64 != 0;
        // ⚠️ **A porta `amount` alimenta o número que o MODO usa** — a fração, o limiar ou o
        // teto. Uma 2ª porta seria um socket morto em dois dos três modos; assim um `value.lfo`
        // anima o teto pelo mesmo fio que já animava a fração.
        let number = if mode == MODE_COUNT {
            ctx.param("max")
        } else {
            ctx.param("amount")
        };
        let amount = amount_of(&scalar_col(ctx.input(1), VALUE_COL), number);
        let input = ctx.input(0);
        let n = input.count();
        let falloff = scalar_col(input, "falloff");
        let keep = keep_indices(n, mode, amount, invert, &falloff);
        let mut out = Stream::new(keep.len());
        for (name, col) in input.columns() {
            out.set(name.clone(), gather_col(col, &keep));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionCull))?;
    // ADR-0136: the passthrough kernel makes the plan claim the node; the
    // compaction (the actual filter) is the stream op's machinery. NOT a
    // `register_dense_window` — a cull breaks the dense id window by definition.
    reg.register_gpu_kernel(MANIFEST.id, GpuKernel::PASSTHROUGH);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Compact {
            port: 0,
            predicate: GPU_PREDICATE,
        },
    );
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Cull",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // ⚠️ **Este nó TEM binding de `falloff` — e ela é INVISÍVEL à derivação.** A
    // leitura mora no `GPU_PREDICATE` do `StreamOp::Compact`, e o kernel que o
    // registry guarda é o `PASSTHROUGH` (ADR-0136: a compactação é maquinaria do
    // stream op, não do kernel). O `consumes` do diagnoser só consulta
    // `gpu_kernel(ty).bindings`, então o canal que resta é a declaração — o mesmo
    // vão de um nó CPU-only, por outro motivo (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

/// **Um número por modo, e só o do modo aparece.** Sem isto o painel ofereceria a `Amount` (uma
/// fração 0..1) ao lado do `Max Count` (uma contagem), e o artista teria de saber qual dos dois
/// o modo escolhido lê — dois controles vivos para uma pergunta é como nasce um knob morto.
static PARAM_GATES: &[ph2d_node_registry::ParamGate] = &[
    ph2d_node_registry::ParamGate {
        param: "amount",
        when: "mode",
        values: &[0, 1],
    },
    ph2d_node_registry::ParamGate {
        param: "max",
        when: "mode",
        values: &[2],
    },
];

/// O que o número É (doc 88): um teto de população é uma CONTAGEM de coisas. A `amount` fica
/// bare — ela é uma fração num modo e um limiar de máscara noutro, e **uma unidade errada é
/// pior que uma ausente**.
static PARAM_UNITS: &[ph2d_node_registry::ParamUnitDecl] = &[ph2d_node_registry::ParamUnitDecl {
    param: "max",
    unit: ph2d_node_registry::ParamUnit::Count,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "mode",
        label: "Mode",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Fraction", "Falloff", "Max Count"],
        },
    },
    ParamUiHint {
        param: "amount",
        label: "Amount",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max",
        label: "Max Count",
        min: 0.0,
        max: 4096.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "invert",
        label: "Invert",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Keep", "Complement"],
        },
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Fraction mode keeps the FIRST `amount·n` elements. FALSIFIED if it kept a
    /// different count or the wrong (non-leading) ones.
    #[test]
    fn fraction_keeps_the_leading_elements() {
        let keep = keep_indices(10, MODE_FRACTION, 0.3, false, &[]);
        assert_eq!(keep, vec![0, 1, 2], "0.3 * 10 -> the first 3");
    }

    /// Fraction 0 empties the stream; 1 keeps all.
    #[test]
    fn fraction_endpoints() {
        assert!(
            keep_indices(8, MODE_FRACTION, 0.0, false, &[]).is_empty(),
            "0 -> none"
        );
        assert_eq!(
            keep_indices(8, MODE_FRACTION, 1.0, false, &[]).len(),
            8,
            "1 -> all"
        );
    }

    /// Invert keeps the complement (the trailing elements under Fraction).
    #[test]
    fn invert_keeps_the_complement() {
        let keep = keep_indices(10, MODE_FRACTION, 0.3, true, &[]);
        assert_eq!(keep, vec![3, 4, 5, 6, 7, 8, 9], "the other 7");
    }

    /// Falloff mode keeps the elements whose mask is ≥ the threshold.
    #[test]
    fn falloff_threshold_keeps_above() {
        let falloff = vec![0.1, 0.9, 0.5, 1.0, 0.2];
        let keep = keep_indices(5, 1, 0.5, false, &falloff);
        assert_eq!(keep, vec![1, 2, 3], "falloff ≥ 0.5");
    }

    /// Deterministic + cooks through the registry: the `amount` value input drives the
    /// keep count and every column is filtered to the survivors.
    #[test]
    fn registers_and_culls_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.cull.test.src"),
            name: "motion.cull.test.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let p: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
                let s: Vec<f32> = (0..10).map(|i| i as f32).collect();
                ctx.emit(
                    Stream::new(10)
                        .with("P", Column::Vec2(p))
                        .with("size", Column::Scalar(s)),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionCull),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.cull.test.src");
        let c = g.add_node("motion.cull");
        g.set_param(c, "amount", 0.4); // keep the first 4
        g.connect(Edge {
            from: (src, 0),
            to: (c, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        let st = out[0].as_stream();
        assert_eq!(st.count(), 4, "0.4 × 10 kept");
        match (st.get("P").unwrap(), st.get("size").unwrap()) {
            (Column::Vec2(pv), Column::Scalar(sv)) => {
                assert_eq!(pv.len(), 4, "P filtered");
                assert_eq!(
                    sv,
                    &vec![0.0, 1.0, 2.0, 3.0],
                    "size filtered to the survivors"
                );
            }
            _ => panic!("columns"),
        }
    }

    /// **O teto mantém os mais NOVOS** — a decisão da wave, e ela NÃO é gosto: o
    /// `motion.emitter`, o irmão que a folha 13 cita, já a tem escrita com o motivo (*"a dense
    /// young jet, not a frozen ancient cloud"*), e numa zona o prefixo é o mais VELHO porque o
    /// `motion.combine` apende os recém-nascidos ao estado. FALSIFICADO se guardasse o prefixo.
    #[test]
    fn the_count_mode_keeps_the_newest_not_the_oldest() {
        let keep = keep_indices(10, MODE_COUNT, 3.0, false, &[]);
        assert_eq!(keep, vec![7, 8, 9], "a CAUDA, nunca o prefixo");
    }

    /// **Abaixo do teto o modo é um NO-OP, e é isso que o separa do Fraction.** Uma fração rala
    /// a população o tempo todo (0,5 de 4 mantém 2); um teto de 100 sobre 4 não toca em nada.
    #[test]
    fn below_the_cap_the_count_mode_touches_nothing() {
        assert_eq!(
            keep_indices(4, MODE_COUNT, 100.0, false, &[]),
            vec![0, 1, 2, 3],
            "um teto que ninguém atingiu não corta"
        );
        // O CONTROLE, ao lado: o mesmo número lido como FRAÇÃO rala metade.
        assert_eq!(
            keep_indices(4, MODE_FRACTION, 0.5, false, &[]).len(),
            2,
            "o controle: uma fração corta mesmo com folga"
        );
    }

    /// O teto é uma contagem, então ele **satura** em vez de estourar: `max` maior que a
    /// população mantém tudo, `0` mantém nada, e nenhum dos dois indexa fora do stream.
    #[test]
    fn the_cap_saturates_at_both_ends() {
        assert!(keep_indices(6, MODE_COUNT, 0.0, false, &[]).is_empty());
        assert_eq!(keep_indices(6, MODE_COUNT, 6.0, false, &[]).len(), 6);
        assert_eq!(keep_indices(0, MODE_COUNT, 9.0, false, &[]).len(), 0);
        assert_eq!(keep_indices(6, MODE_COUNT, -4.0, false, &[]).len(), 0);
    }

    /// **A porta `amount` alimenta o número que o MODO usa.** Sem isto o teto seria o único
    /// número do nó que um `value.*` não alcança — e o socket já está lá, vivo, animando a
    /// fração; deixá-lo mudo num modo é a metade de um controle.
    #[test]
    fn the_value_port_feeds_whichever_number_the_mode_reads() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.cull.test.port.src"),
            name: "motion.cull.test.port.src",
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
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                let p: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
                ctx.emit(Stream::new(10).with("P", Column::Vec2(p)));
            }
        }
        static VAL: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.cull.test.port.value"),
            name: "motion.cull.test.port.value",
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
        struct Val;
        impl NodeOp for Val {
            fn manifest(&self) -> &'static NodeManifest {
                &VAL
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![2.0])));
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == VAL.id => Some(&Val),
                    t if t == MANIFEST.id => Some(&MotionCull),
                    _ => None,
                }
            }
        }

        let mut g = Graph::new();
        let src = g.add_node("motion.cull.test.port.src");
        let val = g.add_node("motion.cull.test.port.value");
        let c = g.add_node("motion.cull");
        g.set_param(c, "mode", MODE_COUNT as f32);
        g.set_param(c, "max", 9.0); // o param diz 9...
        for (from, to, port) in [(src, c, 0u16), (val, c, 1)] {
            g.connect(Edge {
                from: (from, 0),
                to: (to, port),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
        // ...e a porta diz 2, então sobram 2 — o fio vence o param, no modo Count como já
        // vencia no Fraction.
        assert_eq!(out[0].as_stream().count(), 2);

        // ⚠️ E a METADE que o nome deste gate promete e a de cima NÃO testa: **sem** fio, qual
        // param o modo lê. Com a porta ligada os dois ramos leem o mesmo número, então uma
        // mutação que faça o Count ler `amount` passa aqui em cima — medido. Sem fio, `max` e
        // `amount` discordam de propósito, e só um dos dois pode estar certo.
        let mut g2 = Graph::new();
        let src2 = g2.add_node("motion.cull.test.port.src");
        let c2 = g2.add_node("motion.cull");
        g2.set_param(c2, "mode", MODE_COUNT as f32);
        g2.set_param(c2, "max", 2.0);
        g2.set_param(c2, "amount", 9.0);
        g2.connect(Edge {
            from: (src2, 0),
            to: (c2, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook2 = Cook::new();
        let out2 = cook2.cook(&g2, &Ops, c2, 0.0).unwrap();
        assert_eq!(
            out2[0].as_stream().count(),
            2,
            "o modo Count lê `max`; ler `amount` daria 9"
        );
    }
}
