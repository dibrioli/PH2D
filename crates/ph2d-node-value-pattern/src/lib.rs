#![forbid(unsafe_code)]
//! `value.pattern` — the value-domain STEP SEQUENCER: author an explicit list of
//! values that repeats across the instances by index (Motion Nodes M2, the value
//! domain — doc 12/78). Where `instance_field` Random/Ramp and `value.noise`
//! GENERATE a field procedurally, this one lets you TYPE the field — `[v₀, v₁,
//! …]` assigned to instance `0, 1, …` and cycled — the explicit-pattern producer
//! the procedural ones never gave. The step sequencer of a drum machine, the
//! Array of Cavalry, a Houdini detail array.
//!
//! **A PRODUCER** (like `instance_field`): its input is read for its COUNT only
//! (never passed through), and it writes a fresh `v` where `out[i] = pattern[i mod
//! len]`.
//!
//! **Há DUAS formas de autorar o padrão, e a tabela VENCE quando existe:**
//!
//! - **`table`** — um **TEXT PARAM** (doc 32, o canal aditivo de string): a lista
//!   inteira digitada, `"0.1 0.5 0.9 0.2"`, **sem o teto de oito**. Ausente ou
//!   malformada ⇒ o caminho abaixo, byte a byte.
//! - **`steps` + `v0…v7`** — os oito slides, o nó que sempre shipou. `steps = 4`
//!   repete `v0,v1,v2,v3`.
//!
//! ⚠️ **O teto de OITO não era de recurso** (folha 15, `P1`): ele vinha de *"params
//! são f32"* — o `NodeManifest` é f32-only por contrato congelado (ADR-0039), então
//! uma lista arbitrária não cabia num `ParamSpec`. A cura é a mesma que o
//! `motion.expression` e o `ramp` do `motion.color_ramp` já usam, e **não custou
//! canal nenhum**: o `LutSpec` (construído em 2026-07-25 para a curva) já lê um
//! text param e leva um vetor ao DEVICE. Ver [`table`] para o teto que sobra, que
//! é o do buffer.
//!
//! **The value type** is the continuous per-instance scalar field `(Instances,
//! Scalar, Frame)` on the `v` column (doc 12). `Pure` (no clock, no state); the
//! output length matches the input's count.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, LutSpec};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

pub mod table;

/// A chave do text param que carrega a tabela (lida por `EvalCtx::text_param`).
pub const TABLE_KEY: &str = "table";

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of the sibling value nodes; kept local so this stays a leaf drop-crate
/// — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The INPUT type — the instance geometry stream (`Vec2`), read for its COUNT only
/// (the same port `instance_field` reads the grid on). A producer keys on the
/// count, so it takes the geometry, not a value.
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// The value column, out (this is a producer — it writes, never reads `v`).
const VALUE_COL: &str = "v";

/// Quantos slots o caminho LEGADO tem. O comprimento comum de um sequenciador,
/// e o teto vem de o `NodeManifest` ser f32-only (contrato congelado, ADR-0039).
///
/// ⚠️ **Este doc dizia *"um padrão arbitrário mais longo quer um canal de ARRAY
/// no device (uma lista autorada em texto), deferido"* — e é exatamente esta
/// wave.** O canal existe desde 2026-07-25 (o `LutSpec` da curva) e a lista mora
/// no [`TABLE_KEY`]; o que sobra aqui é o caminho que já shipava, vivo enquanto
/// nenhuma tabela for autorada.
const SLOTS: usize = 8;

/// The value for instance `i` given the authored `steps` (clamped to `[1, SLOTS]`,
/// so the cycle never indexes past the slots) and the `SLOTS` values. `out[i] =
/// vals[i mod steps]`.
fn pattern_value(i: usize, steps: usize, vals: &[f32; SLOTS]) -> f32 {
    let steps = steps.clamp(1, SLOTS);
    vals[i % steps]
}

/// Como uma FASE fracionária se resolve num ciclo de valores discretos.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Interp {
    /// A fase encosta no slot mais próximo — o padrão salta de degrau em degrau,
    /// que é o que um padrão de índices sempre foi.
    Step,
    /// A fase mistura os dois slots vizinhos — o padrão **desliza**.
    Linear,
}

impl Interp {
    fn from_param(p: f32) -> Self {
        if p.round() as i32 == 1 {
            Interp::Linear
        } else {
            Interp::Step
        }
    }
}

/// **O comprimento do ciclo** — a tabela autorada, se houver, senão os `steps`
/// slots. Uma resposta só, para que a fase e o valor não possam discordar sobre
/// quantos degraus existem.
fn cycle_len(steps: usize, authored: &[f32]) -> usize {
    if authored.is_empty() {
        steps.clamp(1, SLOTS)
    } else {
        authored.len()
    }
}

/// O valor na POSIÇÃO INTEIRA `k` do ciclo, com `k` a dar a volta nos dois
/// sentidos (`rem_euclid`, nunca `%`: o `%` de Rust devolve negativo para `k`
/// negativo, e um offset negativo é o caso normal deste knob).
fn cycle_at(k: i64, steps: usize, vals: &[f32; SLOTS], authored: &[f32]) -> f32 {
    let n = cycle_len(steps, authored);
    let idx = k.rem_euclid(n as i64) as usize;
    // As duas leis do ciclo continuam a ser as que sempre foram — esta função
    // apenas as alcança numa posição que já deu a volta.
    table::value_at(idx, authored).unwrap_or_else(|| pattern_value(idx, steps, vals))
}

/// **O valor do elemento `i` sob uma FASE.**
///
/// `offset` desloca o padrão em unidades de índice e **pode ser fracionário** —
/// e é isso que dá sentido ao `interp`: num índice inteiro não há nada entre dois
/// slots para interpolar, e é exatamente a fase que cria o entre. `Step` encosta
/// no slot mais próximo (o padrão salta de posição inteira em posição inteira),
/// `Linear` mistura o par (o padrão desliza continuamente, e um `offset` animado
/// vira uma onda a percorrer a fileira em vez de um estroboscópio).
///
/// A volta é **periódica nos dois sentidos**: em `Linear`, o par no encaixe é o
/// último com o primeiro, que é a única leitura coerente com um padrão que já se
/// repetia.
///
/// ⚠️ **`offset = 0` é o nó que sempre shipou, bit-a-bit, nos DOIS modos.**
/// `round(i)` e `floor(i)` são ambos `i` para um índice inteiro, e em `Linear` a
/// fração é `0`, cujo ramo devolve o slot **verbatim** em vez de `a + 0·(b − a)`.
fn phased_value(
    i: usize,
    offset: f32,
    interp: Interp,
    steps: usize,
    vals: &[f32; SLOTS],
    authored: &[f32],
) -> f32 {
    let p = i as f32 + offset;
    match interp {
        Interp::Step => cycle_at(p.round() as i64, steps, vals, authored),
        Interp::Linear => {
            let k0 = p.floor();
            let t = p - k0;
            let a = cycle_at(k0 as i64, steps, vals, authored);
            if t == 0.0 {
                return a;
            }
            let b = cycle_at(k0 as i64 + 1, steps, vals, authored);
            a + t * (b - a)
        }
    }
}

/// The static contract of this node type (ADR-0031). The kernel is side-metadata
/// (ADR-0126); `NodeManifest` stays the frozen 8 fields.
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.pattern"),
    name: "value.pattern",
    // Read for its COUNT only (never passed through), like `instance_field` — the
    // instance geometry stream, so the grid connects straight in.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "steps",
            default: 2.0,
        },
        ParamSpec {
            name: "v0",
            default: 0.0,
        },
        ParamSpec {
            name: "v1",
            default: 1.0,
        },
        ParamSpec {
            name: "v2",
            default: 0.0,
        },
        ParamSpec {
            name: "v3",
            default: 0.0,
        },
        ParamSpec {
            name: "v4",
            default: 0.0,
        },
        ParamSpec {
            name: "v5",
            default: 0.0,
        },
        ParamSpec {
            name: "v6",
            default: 0.0,
        },
        ParamSpec {
            name: "v7",
            default: 0.0,
        },
        // ⚠️ **Apendados**: a FASE do padrão (em unidades de índice, fracionária)
        // e como ela se resolve. `0`/`Step` = o nó que sempre shipou, bit-a-bit.
        // Ver [`phased_value`].
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        ParamSpec {
            name: "interp",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O canal de LUT que este nó registra: a TABELA autorada, levada ao device
/// como um `storage` de floats cujo **slot 0 é a CONTAGEM** (ver
/// [`table::fill_lut`]). Chamá-la `vp_table` faz o binding ser `lut_vp_table`,
/// que é o nome que o corpo do kernel lê.
///
/// ⚠️ **O corpo lê o BUFFER direto, nunca o `vp_table_sample(t)` que o gerador
/// também emite.** Aquele acessor LERPA entre vizinhos — certo para uma curva,
/// errado para um sequenciador, cujos valores são autorados um a um e têm de
/// sair **exatos**. Um `sample` num `t` derivado do índice passaria por
/// `(i/last)*last`, que em `f32` não devolve `i` para todo par, e um passo do
/// padrão sairia misturado com o vizinho por ~1e-7 sem ninguém pedir.
static LUTS: &[LutSpec] = &[LutSpec {
    name: "vp_table",
    text_key: TABLE_KEY,
    resolution: table::LUT_LEN,
    fill: table::fill_lut,
}];

/// GPU compute kernel (ADR-0126) — the WGSL port of [`pattern_value`], **fully
/// device-resident**. No `applicable` gate — the sequencer never falls back to
/// the CPU. VALUE out; the binding is `Write` (a producer — it never reads the
/// input `v`, only its count), so no `in_v` is declared for naga to strip.
///
/// **Dois ramos, e o cabeçalho da LUT escolhe** — exatamente como a CPU escolhe
/// pela lista vazia: `len > 0` lê a tabela, `len == 0` cai nos oito slots, onde o
/// `steps` clamp mantém `vp_idx` dentro deles e o `switch` seleciona.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        // A FASE: o gemeo de `phased_value`. `interp` decide se ela encosta no\n\
        // slot mais proximo ou mistura o par que ela atravessa.\n\
        let vp_p = f32(i) + params.offset;\n\
        var vp_v: f32;\n\
        if (params.interp >= 0.5) {\n\
        \x20   let vp_k0 = floor(vp_p);\n\
        \x20   let vp_t = vp_p - vp_k0;\n\
        \x20   let vp_a = vp_at(i32(vp_k0));\n\
        \x20   if (vp_t == 0.0) {\n\
        \x20       // Verbatim: uma fase inteira le' o slot, nao `a + 0*(b-a)`.\n\
        \x20       vp_v = vp_a;\n\
        \x20   } else {\n\
        \x20       vp_v = vp_a + vp_t * (vp_at(i32(vp_k0) + 1) - vp_a);\n\
        \x20   }\n\
        } else {\n\
        \x20   vp_v = vp_at(i32(vp_round(vp_p)));\n\
        }\n\
        write_v(i, vp_v);\n",
    wgsl_lib: "\
        fn vp_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        // O cabecalho da LUT: quantos valores a tabela autorada carrega (0 =\n\
        // nenhuma, e o ciclo sao os `steps` slots). O gemeo de `cycle_len`.\n\
        fn vp_cycle_len() -> i32 {\n\
            let authored = i32(max(lut_vp_table[0], 0.0));\n\
            if (authored > 0) { return authored; }\n\
            return clamp(i32(vp_round(params.steps)), 1, 8);\n\
        }\n\
        // O valor na posicao INTEIRA `k`, com `k` a dar a volta nos dois sentidos\n\
        // (o gemeo do `rem_euclid`: o `%` do WGSL tambem devolve negativo, e um\n\
        // offset negativo e' o caso normal deste knob).\n\
        fn vp_at(k: i32) -> f32 {\n\
            let n = vp_cycle_len();\n\
            var idx = k % n;\n\
            if (idx < 0) { idx = idx + n; }\n\
            if (i32(max(lut_vp_table[0], 0.0)) > 0) {\n\
                return lut_vp_table[1u + u32(idx)];\n\
            }\n\
            var v: f32;\n\
            switch (idx) {\n\
                case 1: { v = params.v1; }\n\
                case 2: { v = params.v2; }\n\
                case 3: { v = params.v3; }\n\
                case 4: { v = params.v4; }\n\
                case 5: { v = params.v5; }\n\
                case 6: { v = params.v6; }\n\
                case 7: { v = params.v7; }\n\
                default: { v = params.v0; }\n\
            }\n\
            return v;\n\
        }\n",
    bindings: &[ColumnBinding {
        column: VALUE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    }],
    // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
    // CPU, e o device recusa o shader (`invalid field accessor`).
    params: &[
        "steps", "v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7", "offset", "interp",
    ],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

struct ValuePattern;

impl NodeOp for ValuePattern {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let steps = ctx.param("steps").round().max(1.0) as usize;
        let vals: [f32; SLOTS] = [
            ctx.param("v0"),
            ctx.param("v1"),
            ctx.param("v2"),
            ctx.param("v3"),
            ctx.param("v4"),
            ctx.param("v5"),
            ctx.param("v6"),
            ctx.param("v7"),
        ];
        // A tabela autorada, UMA vez por cook. Vazia ⇒ o caminho dos oito slots,
        // literalmente o que shipava.
        let authored = table::parse(ctx.text_param(TABLE_KEY).unwrap_or(""));
        // The input is read for its COUNT only (like `instance_field`); the output
        // is a fresh field of that length.
        let offset = ctx.param("offset");
        let interp = Interp::from_param(ctx.param("interp"));
        let n = ctx.input(0).count();
        let out: Vec<f32> = (0..n)
            .map(|i| phased_value(i, offset, interp, steps, &vals, &authored))
            .collect();
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValuePattern))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Pattern",
            // Source green: a value producer (authors a field), not a transformer.
            category: ph2d_node_registry::NodeUiCategory::Source,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // ⚠️ Autorar a TABELA torna os nove controles legados INERTES — o `eval` nem
    // os lê. *Um controle que não faz nada não é pintado*, e é o mesmo gate que o
    // `motion.spline_wrap` usa para os oito pontos dele quando há forma.
    reg.register_param_gates_text(MANIFEST.id, PARAM_GATES_TEXT);
    Ok(())
}

use ph2d_node_registry::{ParamGateText, ParamGroup, ParamUiHint, ParamWidget};

/// Os nove controles legados aparecem **só sem tabela**.
static PARAM_GATES_TEXT: &[ParamGateText] = &[
    gate_legacy("steps"),
    gate_legacy("v0"),
    gate_legacy("v1"),
    gate_legacy("v2"),
    gate_legacy("v3"),
    gate_legacy("v4"),
    gate_legacy("v5"),
    gate_legacy("v6"),
    gate_legacy("v7"),
];

const fn gate_legacy(param: &'static str) -> ParamGateText {
    ParamGateText {
        param,
        when_text: TABLE_KEY,
        when_present: false,
    }
}

/// A value slider for one slot, `0..1` (the value-domain normalised convention;
/// compose a `value.map_range` for another range).
const fn slot(param: &'static str, label: &'static str) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    }
}

/// As SEÇÕES deste nó (doc 88 B3). Oito valores e um contador não são nove controles do mesmo
/// peso: `steps` diz QUANTOS slots ciclam, e os oito são o conteúdo.
///
/// ⚠️ Uma seção nasce ABERTA — agrupar não é esconder. O que ela compra é o gesto de dobrar
/// depois que o padrão está posto, e a fronteira visível entre *quantos* e *quais*.
static PARAM_GROUPS: &[ParamGroup] = &[
    ParamGroup::new("v0", "Values"),
    ParamGroup::new("v1", "Values"),
    ParamGroup::new("v2", "Values"),
    ParamGroup::new("v3", "Values"),
    ParamGroup::new("v4", "Values"),
    ParamGroup::new("v5", "Values"),
    ParamGroup::new("v6", "Values"),
    ParamGroup::new("v7", "Values"),
];

static PARAM_HINTS: &[ParamUiHint] = &[
    // A TABELA — um text param, então não é um `ParamSpec` do manifesto (que é
    // f32-only por contrato congelado); o painel a desenha como campo de texto,
    // exatamente como desenha a fórmula do `motion.expression`.
    ParamUiHint {
        param: TABLE_KEY,
        label: "Table",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    // How many slots cycle. The extra slots stay on the panel but are ignored
    // above `steps`; the doc says so.
    ParamUiHint {
        param: "steps",
        label: "Steps",
        min: 1.0,
        max: SLOTS as f32,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    slot("v0", "V0"),
    slot("v1", "V1"),
    slot("v2", "V2"),
    slot("v3", "V3"),
    slot("v4", "V4"),
    slot("v5", "V5"),
    slot("v6", "V6"),
    slot("v7", "V7"),
    // A FASE do padrão, em unidades de índice. `0` = o nó de sempre. Fracionária
    // de propósito: é ela que cria o "entre dois slots" que o `interp` resolve.
    ParamUiHint {
        param: "offset",
        label: "Offset",
        min: -8.0,
        max: 8.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "interp",
        label: "Interp",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Step", "Linear"],
        },
    },
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
