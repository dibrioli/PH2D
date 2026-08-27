//! `value.switch` — the value-domain ROUTER: pick one of N **value** fields by a
//! `select` value (Motion Nodes M2, the value domain — doc 12/17). This is the
//! multiplexer every mature node graph ships: TouchDesigner's **Switch CHOP**,
//! Houdini's **Switch VOP**, Nuke's **Switch** (`which`), Max's `selector~`. It is
//! what lets a value graph BRANCH — route a different source onto the same wire as
//! a selector animates — the last routing primitive the value vocabulary was
//! missing (docs 12–16 gave produce / combine / sample / compare / remap / drive).
//!
//! **`select` is a value, not a param** — so a `pulse.counter`, a `value.lfo`, or
//! any field can drive the selection and animate it. The chosen input index is
//! `clamp(round(select), 0, N-1)`; `select` unconnected reads as `0` (→ `in0`, the
//! sensible default).
//!
//! **Per-element AND broadcast (doc 12).** Because `select` is itself a field, the
//! switch is per-element by construction: element `i` reads
//! `in[round(select_i)][i]`. A length-1 `select` broadcasts (the whole grid
//! switches together — the common case); a length-N `select` picks a possibly
//! different input for each element (a Houdini-style per-point mux). Every input
//! obeys the `1→N` hold, so a length-1 source is held across the grid. The output
//! length is the `max` of all connected inputs. `Pure` (no clock, no state).
//! Transcendental-free (HR-5): `round` / `clamp` / index only.

#![forbid(unsafe_code)]

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, SourceWindow};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// The value type — the continuous per-instance scalar field on the `v` column
/// (mirror of `ph2d_node_pulse_counter::VALUE`; kept local so this stays a leaf
/// drop-crate — the shared vocabulary is the port, not a shared symbol).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// The value column, inputs and output (the canonical `value`-domain column).
const VALUE_COL: &str = "v";
/// The number of routed data inputs (`in0..in3`). Four covers the common mux
/// without a variable-arity port surface.
const N_INPUTS: usize = 4;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.switch"),
    name: "value.switch",
    inputs: &[
        PortSpec {
            name: "select",
            ty: VALUE,
        },
        PortSpec {
            name: "in0",
            ty: VALUE,
        },
        PortSpec {
            name: "in1",
            ty: VALUE,
        },
        PortSpec {
            name: "in2",
            ty: VALUE,
        },
        PortSpec {
            name: "in3",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // ⚠️ **Apendado**: `0` = o roteador que sempre shipou (arredonda e
        // salta), `1` = o crossfader (índice fracionário mistura o par). Ver
        // [`switch`].
        ParamSpec {
            name: BLEND,
            default: 0.0,
        },
        // ⚠️ **Apendado** (doc 89, folha 15): `0` = o cook puxa as quatro entradas, como sempre;
        // `1` = ele salta as que este `select` não escolhe. Ver [`LAZY`], onde estão a medição,
        // as três condições e a razão de a preguiça ser um MODO e não uma optimização calada.
        ParamSpec {
            name: LAZY,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// **O MODO DO ROTEADOR** — `0` arredonda e salta, `1` mistura o par que ladeia o índice.
///
/// ⚠️ **Ele é uma CONSTANTE porque dois leitores discordarem sobre esta chave produz saída errada
/// em silêncio, não um erro.** Quem a lê: o [`MANIFEST`], o [`GPU_KERNEL`], o `eval`, o
/// [`PARAM_HINTS`] e — desde a auditoria de 2026-08-27 — o construtor do plano em [`lazy`], que a
/// usa para escolher **qual lei** viaja para o cook. Enquanto o nome era um literal em cinco
/// sítios, essa escolha era um `"blend"` digitado a três ficheiros de distância do `eval` que o
/// consome. *Uma lei escrita em cinco sítios ainda não é uma lei — só uma porta é.*
pub const BLEND: &str = "blend";

/// **A AVALIAÇÃO PREGUIÇOSA** — a chave do param que a liga (doc 89, folha 15).
///
/// O Blender documenta-a duas vezes (*"only the input that is passed through the node is
/// computed"*). Medido (`measure_switch_laziness`, ramos de oito oitavas sobre 4096 peças):
/// **3,90×** quando os ramos são caros **e** exclusivos; **1,03×** quando são o mesmo ramo,
/// porque aí o memo já a tinha entregue.
///
/// ⚠️ **É um MODO e não um default, e a razão é uma medição, não prudência.** A contagem de
/// saída deste nó é o **máximo** dos comprimentos de TODAS as entradas (ver [`switch`] e o gate
/// `the_output_count_is_decided_by_branches_nobody_chose`), então um ramo comprido que ninguém
/// escolheu ainda decide quantos elementos saem — e no caminho de CPU um comprimento só existe
/// depois da avaliação, logo não há como o saber sem cozinhar. ⇒ **ligar isto muda o que o nó
/// computa** quando os ramos têm comprimentos diferentes, e é por isso que ele nasce desligado
/// e o diz no rótulo.
///
/// As outras duas condições (o `select` uniforme, a sub-árvore saltada `Pure`) são verificadas
/// pelo cook e pelo construtor do plano — ver `ph2d_nodegraph::cook::LazySelect`.
/// ⚠️⚠️ **E ELE É UMA PROPRIEDADE DO COZIMENTO DE *CPU*. No device não há ramo para saltar.**
///
/// O cook do Motion é **GPU-residente por omissão** (`PH2D_GPU_COOK=0` volta à CPU): quando o
/// plano cobre o grafo, ele inteiro vira **um dispatch** e os quatro ramos são palavras do mesmo
/// kernel. ⇒ **num grafo que corre no device, ligar isto não muda nada.**
///
/// ⚠️ **E a recusa óbvia — «então força a CPU quando o artista liga» — está REFUTADA por
/// medição**, no quadro real e no mesmo grafo: a rota de GPU faz `3,75 ms` com os quatro ramos,
/// contra `13,10 ms` da CPU com a preguiça ligada. Forçar a CPU tornaria o botão **3,5× pior**
/// que não lhe tocar. *Uma cura que troca um botão inerte por um botão nocivo não é uma cura.*
///
/// ⇒ **Onde ele vale é onde a CPU já é o caminho** — e isso não é raro nem contrived: o plano de
/// GPU recusa vector vivo, objeto com geometria viva, escopos de tempo, nós de CPU-only e
/// **mais de um sink** (`motion_bridge_gpu`). Nesses documentos o roteador custa o que TODOS os
/// ramos custam, e é aí que os `3,87×` medidos aparecem.
pub const LAZY: &str = "lazy";

/// **O construtor do PLANO de preguiça** — irmão por assunto: ele conhece o registry e o grafo,
/// e este ficheiro conhece a lei do nó.
pub mod lazy;

/// **QUAIS RAMOS ESTE `select` PRECISA** — a lei que o cook chama para decidir o que saltar, no
/// modo de ROTEAMENTO (`blend = 0`).
///
/// ⚠️ **Ela mora aqui, ao lado de [`switch`], e viaja para o cook como ponteiro.** Reimplementá-la
/// no escalonador seria a segunda porta que diverge no primeiro ajuste — e o par arredondar/
/// grampear é exactamente onde ela divergiria (o clamp é a `N_INPUTS − 1` incondicional, que é
/// a regra da CPU e está documentada em [`GPU_KERNEL`]).
pub fn needed_round(select: f32, out: &mut [bool]) {
    out.fill(false);
    let last = out.len().saturating_sub(1) as i32;
    let idx = (select.round() as i32).clamp(0, last) as usize;
    if let Some(slot) = out.get_mut(idx) {
        *slot = true;
    }
}

/// Idem, no modo de MISTURA (`blend = 1`) — e aqui são **dois** ramos, não um.
///
/// ⚠️ **O par colapsa num só quando `t == 0`**, porque ali o nó devolve `a` verbatim sem tocar em
/// `b` (é o que impede um `select` inteiro autorado de passar por `a + 0,0·(b − a)`, que
/// arredonda). ⇒ *quantos ramos estão vivos depende do VALOR do select, não só do modo* — que é
/// precisamente a razão de esta lei ser um ponteiro para cá e não uma cópia lá.
pub fn needed_blend(select: f32, out: &mut [bool]) {
    out.fill(false);
    let last = out.len().saturating_sub(1) as i32;
    let lo = (select.floor() as i32).clamp(0, last) as usize;
    if let Some(slot) = out.get_mut(lo) {
        *slot = true;
    }
    let t = (select - select.floor()).clamp(0.0, 1.0);
    if t == 0.0 {
        return;
    }
    let hi = (select.floor() as i32 + 1).clamp(0, last) as usize;
    if let Some(slot) = out.get_mut(hi) {
        *slot = true;
    }
}

/// A porta do `select`, e as portas candidatas — o que o plano de preguiça precisa de saber
/// sobre a FORMA deste nó. Derivadas do manifesto, nunca escritas duas vezes.
pub const SELECT_PORT: u16 = 0;
/// As portas candidatas, na ordem em que [`needed_round`] / [`needed_blend`] as indexam.
pub const CHOICE_PORTS: &[u16] = &[1, 2, 3, 4];
/// A coluna escalar em que o valor do `select` viaja.
pub const SELECT_COLUMN: &str = VALUE_COL;

/// The sample of value field `v` at index `i`, applying the `1→N` broadcast rule
/// (mirror of `motion.drive`/`value.math`): a length-1 field is held at every
/// index; a length-N field is read element-wise; a missing field reads as `0.0`.
fn field_at(v: &[f32], i: usize) -> f32 {
    match v.len() {
        0 => 0.0,
        1 => v[0], // broadcast: one value → every index (the 1→N rule)
        _ => v.get(i).copied().unwrap_or(0.0),
    }
}

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// Route the `ins` fields by the `select` field. Output length is the `max` of
/// all inputs; element `i` reads `ins[clamp(round(select_i), 0, N-1)][i]` under
/// the broadcast rule.
///
/// **`blend` turns a router into a CROSSFADER** — TouchDesigner's Switch CHOP
/// ships the pair, and the reason is that a switch driven by an animated `select`
/// POPS: the output jumps a whole input's worth in one frame, at the half-way
/// point of a smooth ramp. With `blend` on, a fractional select reads the two
/// inputs it sits between and lerps by the fraction, so the same ramp reads as a
/// dissolve.
///
/// ⚠️ **The two laws are NOT the same function at a half-integer, and that is the
/// whole point.** Rounding sends `select = 0.5` to `in1` (half away from zero);
/// blending sends it to the MIDPOINT of `in0` and `in1`. Off is the node that
/// shipped, bit-for-bit — an integer select lands on `t = 0`, which returns the
/// lower input verbatim, so the crossfader agrees with the router everywhere the
/// artist authored an integer.
fn switch(select: &[f32], ins: &[Vec<f32>], blend: bool) -> Vec<f32> {
    let n = ins
        .iter()
        .map(|c| c.len())
        .chain(std::iter::once(select.len()))
        .max()
        .unwrap_or(0);
    let last = ins.len().saturating_sub(1) as i32;
    (0..n)
        .map(|i| {
            let s = field_at(select, i);
            if !blend {
                // round-to-nearest, then clamp into the connected input range.
                let idx = (s.round() as i32).clamp(0, last) as usize;
                return field_at(&ins[idx], i);
            }
            // The pair the fractional select sits between, each clamped into range
            // — so the ends SATURATE rather than wrapping, the same edge law the
            // rounding path has.
            let lo = (s.floor() as i32).clamp(0, last) as usize;
            let hi = (s.floor() as i32 + 1).clamp(0, last) as usize;
            let t = (s - s.floor()).clamp(0.0, 1.0);
            let a = field_at(&ins[lo], i);
            // `t == 0` is returned VERBATIM: an authored integer select must read
            // the input itself, not `a + 0.0·(b − a)`, which rounds.
            if t == 0.0 {
                return a;
            }
            let b = field_at(&ins[hi], i);
            a + t * (b - a)
        })
        .collect()
}

/// GPU compute kernel (ADR-0126) — the WGSL port of [`switch`].
///
/// **A dynamic index over a STATIC set of readers.** The generated module names
/// one accessor per bound port (`read_in0_v` … `read_in3_v`), so the routing is
/// a branch over four constants rather than an indexed lookup — which is what a
/// GPU wants anyway, and which is why `N_INPUTS` being a fixed 4 is a
/// convenience here rather than a limitation to route around.
///
/// **The clamp bound is 3, unconditionally, and that is the CPU's rule**: `eval`
/// always builds `N_INPUTS` vectors regardless of what is connected, so `last`
/// is always `3` and a disconnected input is the empty field. Clamping to *the
/// number of connected inputs* instead would look tidier and would be a
/// different function — `select = 2` with only `in0` wired reads `0.0` on the
/// CPU, not `in0`.
///
/// Every port is [`ColumnAccess::ReadBroadcast`], including `select`: a length-1
/// selector switches the whole grid together (the common case) and a length-N
/// one picks per element, and both are the same compiled module — the broadcast
/// is a uniform bit, not a pipeline variant.
///
/// `vs_round` is round-half-AWAY-from-zero to match Rust's `f32::round`.
/// `select` picks a BRANCH, so at `select = 0.5` a half-even disagreement routes
/// a *different input* — the loudest possible way for a rounding convention to
/// be wrong.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let vs_s = read_select_v(i);\n\
        // Os dois indices: o arredondado (roteador) e o par inferior/superior\n\
        // (crossfader). O `blend` escolhe qual deles manda -- a mesma bifurcacao\n\
        // do `switch` da CPU, e nao uma segunda lei.\n\
        let vs_i = clamp(i32(vs_round(vs_s)), 0, 3);\n\
        let vs_lo = clamp(i32(floor(vs_s)), 0, 3);\n\
        let vs_hi = clamp(i32(floor(vs_s)) + 1, 0, 3);\n\
        let vs_ia = select(vs_i, vs_lo, params.blend >= 0.5);\n\
        var vs_a = read_in0_v(i);\n\
        if (vs_ia == 1) {\n\
        \x20   vs_a = read_in1_v(i);\n\
        } else if (vs_ia == 2) {\n\
        \x20   vs_a = read_in2_v(i);\n\
        } else if (vs_ia == 3) {\n\
        \x20   vs_a = read_in3_v(i);\n\
        }\n\
        var vs_r = vs_a;\n\
        if (params.blend >= 0.5) {\n\
        \x20   let vs_t = clamp(vs_s - floor(vs_s), 0.0, 1.0);\n\
        \x20   if (vs_t != 0.0) {\n\
        \x20       var vs_b = read_in0_v(i);\n\
        \x20       if (vs_hi == 1) {\n\
        \x20           vs_b = read_in1_v(i);\n\
        \x20       } else if (vs_hi == 2) {\n\
        \x20           vs_b = read_in2_v(i);\n\
        \x20       } else if (vs_hi == 3) {\n\
        \x20           vs_b = read_in3_v(i);\n\
        \x20       }\n\
        \x20       vs_r = vs_a + vs_t * (vs_b - vs_a);\n\
        \x20   }\n\
        }\n\
        write_v(i, vs_r);\n",
    wgsl_lib: "\
        fn vs_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 2,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 3,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 4,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    // ⚠️ Esta lista não é derivada do manifesto: um param novo compila, coza na
    // CPU, e o device recusa o shader (`invalid field accessor`).
    params: &[BLEND],
    count_law: Some(switch_count),
    variant_by_param: None,
    applicable: None,
};

/// **How wide is the output?** — the `max` over every port, which is the same
/// expression `switch` computes, **`select` included**. Leaving the selector out
/// would be the tempting simplification and it is wrong: an animated length-N
/// selector against length-1 sources is exactly the per-element mux this node
/// advertises, and the output has to be as long as the selection.
fn switch_count(ctx: &CountLawCtx<'_>) -> SourceWindow {
    SourceWindow::of_count(ctx.inputs.iter().copied().max().unwrap_or(0) as usize)
}

struct ValueSwitch;

impl NodeOp for ValueSwitch {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let select = scalar_col(ctx.input(0), VALUE_COL);
        let ins: Vec<Vec<f32>> = (0..N_INPUTS)
            .map(|k| scalar_col(ctx.input(k + 1), VALUE_COL))
            .collect();
        let out = switch(&select, &ins, ctx.param(BLEND) >= 0.5);
        ctx.emit(Stream::new(out.len()).with(VALUE_COL, Column::Scalar(out)));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueSwitch))?;
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Switch",
            // Utility grey: a value→value router, plumbing (not a transform).
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    // ⚠️ A SELEÇÃO continua sem param — ela é uma porta, para poder ser animada.
    // O `blend` não escolhe *qual*, escolhe *como*: saltar ou dissolver.
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: BLEND,
        label: "Blend",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
    // ⚠️ **O rótulo diz o que MUDA, não como funciona** (HR-15: rótulos por resultado). «Skip
    // Unused Inputs» é o que o artista ganha; o preço — os ramos saltados deixam de decidir a
    // contagem — está no doc de [`LAZY`] e no aviso da folha.
    ParamUiHint {
        param: LAZY,
        label: "Skip Unused Inputs",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Off", "On"],
        },
    },
];

/// **AS PROVAS DO ROTEADOR** — irmãs por arquivo, pelo teto de LOC.
#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
