//! **O RASTRO NO DISPOSITIVO** (ciclo 7, W1d — doc 112) — o modo `Remembered` do `step`, como um
//! [`StreamOp::Carry`]. Cortado do `lib.rs` por RESPONSABILIDADE: o `lib.rs` responde *quem
//! sobrevive a este tique e quanto desbota*, este responde *o que o dispositivo corre para isso*.
//!
//! ## A forma
//!
//! Cada tique a CPU faz `carregados = filtrar(estado)`, envelhece-os e desbota-os, e devolve
//! `carregados ++ vivo` (com a idade `0` e as colunas de render materializadas no vivo). No
//! dispositivo isso são três passos do sequenciador e um corpo:
//!
//! 1. o **predicado** ([`PREDICADO`]) decide, linha a linha do estado CRU, quem sobrevive — com a
//!    **redução** [`BANDA`] a responder, sobre o mesmo estado, a porta única do espaçamento (*há
//!    algum eco na faixa `1..s`?*, o `promotes_head` da CPU);
//! 2. os sobreviventes e o vivo são **juntados** com as identidades da CPU ([`IDENTIDADES`] — o
//!    `default_for`: um `size`/`tint` a zero apagava o eco);
//! 3. o **corpo** corre sobre a junção: as linhas `i < partição` são as carregadas (envelhecem e
//!    desbotam), as outras são a cabeça (idade `0`; o `size`/`tint` que ela não tinha já chega
//!    materializado pela junção ou pela identidade da ligação).
//!
//! ⚠️⚠️ **As taxas são as MESMAS funções da CPU** ([`DerivedUniform`] sobre as contagens CRUAS): a
//! janela sai da contagem VIVA (`generations` com o tecto de instâncias), e o `Decay::per_tick`
//! dela dá as taxas por tique e a matriz de cor — que tem trigonometria (`libm::sincosf`) e por
//! isso NUNCA vai para o WGSL: viaja como nove números. Os slots do `fade`, do `shrink` e do
//! `spin` levam as taxas; os `tr_*` são nomes que só a derivação conhece.
//!
//! ⚠️ **O caso de UM eco** (`generations ≤ 1`) é a entrada tal e qual na CPU — com a coluna do
//! modo por cima quando o artista o escolheu. O `Carry` responde-o por [`identidade`] e corre
//! então o [`SO_MODO`] ou um passa-tudo sobre a entrada viva.
//!
//! ⚠️ **Divergência DECLARADA (um tique, uma borda):** quando NADA sobrevive a CPU ainda junta as
//! COLUNAS do estado (um `gather` de zero linhas continua a tê-las), e o dispositivo não — a
//! compactação de zero linhas não tem buffers. Só se vê se a montante uma coluna deixar de
//! existir exactamente num tique em que o rastro não carrega nada; no tique seguinte as duas rotas
//! voltam a coincidir.
//!
//! ⚠️ **E um `spin` sub-normal** (`total / vão` a dar `0` num `f32`) liga a variante do `rot` sem
//! que a CPU o materialize: a coluna existe com os valores que tinha (ou `0`), o que desenha igual.
//!
//! ⚠️ O modo `Resampled` re-cozinha a própria entrada em N instantes (ADR-0163) e fica CPU **por
//! desenho** (`applicable`).

use super::{
    AGE, ALPHA_MAX, BLEND_COLUMN, Decay, ECHO_BLEND, MANIFEST, SOURCE, colour, echo_blend_tag,
    generations, spacing_of,
};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, ConcatFill, CountLawCtx, DerivedUniform, GpuKernel, KEEP_FLAG_COL,
    StreamOp,
};
use ph2d_nodegraph::port::Dim;
use ph2d_nodegraph::reduce_meta::{ReduceOp, ReduceSpec};

/// A contagem VIVA (porta 0), crua.
fn vivos(c: &CountLawCtx<'_>) -> usize {
    c.inputs.first().copied().unwrap_or(0) as usize
}

/// As gerações — o `generations` do `step`, com o tecto de instâncias sobre a contagem viva.
fn geracoes(c: &CountLawCtx<'_>) -> usize {
    generations((c.param)("length"), vivos(c))
}

/// A janela de idade — `(k − 1)·s + 1`, a expressão do `step`.
fn janela(c: &CountLawCtx<'_>) -> usize {
    (geracoes(c) - 1).saturating_mul(spacing_of((c.param)("spacing"))) + 1
}

/// As taxas por tique — o `decay.per_tick(span)` do `step`, com o vão da janela.
fn taxas(c: &CountLawCtx<'_>) -> Decay {
    let p = c.param;
    let autorado = Decay {
        alpha_max: p(ALPHA_MAX),
        fade: p("fade"),
        shrink: p("shrink"),
        hue_shift: p("hue_shift"),
        saturation: p("saturation"),
        spin: p("spin"),
    };
    autorado.per_tick(u32::try_from(janela(c) - 1).unwrap_or(u32::MAX))
}

/// A matriz de cor por tique — a MESMA composição do `Decay::apply`.
fn matriz(c: &CountLawCtx<'_>) -> colour::Mat3 {
    let d = taxas(c);
    colour::compose(
        colour::hue_rotation(d.hue_shift),
        colour::saturation(d.saturation),
    )
}

/// Uma contagem como `f32` — exacta abaixo do `ID_WRAP` (`2²⁴`), que é o tecto do modelo de ids.
#[expect(clippy::cast_precision_loss, reason = "uma contagem abaixo do ID_WRAP")]
fn como_f32(n: usize) -> f32 {
    n as f32
}

/// `true` quando a CPU devolve a entrada tal e qual (o `k <= 1` do `step`).
fn identidade(c: &CountLawCtx<'_>) -> bool {
    geracoes(c) <= 1
}

macro_rules! m {
    ($r:literal, $c:literal) => {
        |c| matriz(c)[$r][$c]
    };
}

/// Os slots derivados — o predicado lê o `tr_window`; o corpo lê o resto.
static DERIVADOS: &[DerivedUniform] = &[
    DerivedUniform {
        param: "fade",
        derive: |c| taxas(c).fade,
    },
    DerivedUniform {
        param: "shrink",
        derive: |c| taxas(c).shrink,
    },
    DerivedUniform {
        param: "spin",
        derive: |c| taxas(c).spin,
    },
    DerivedUniform {
        param: "tr_live",
        derive: |c| como_f32(vivos(c)),
    },
    DerivedUniform {
        param: "tr_window",
        derive: |c| como_f32(janela(c)),
    },
    DerivedUniform {
        param: "tr_colour",
        derive: |c| f32::from(u8::from(matriz(c) != colour::IDENTITY)),
    },
    DerivedUniform {
        param: "tr_m00",
        derive: m!(0, 0),
    },
    DerivedUniform {
        param: "tr_m01",
        derive: m!(0, 1),
    },
    DerivedUniform {
        param: "tr_m02",
        derive: m!(0, 2),
    },
    DerivedUniform {
        param: "tr_m10",
        derive: m!(1, 0),
    },
    DerivedUniform {
        param: "tr_m11",
        derive: m!(1, 1),
    },
    DerivedUniform {
        param: "tr_m12",
        derive: m!(1, 2),
    },
    DerivedUniform {
        param: "tr_m20",
        derive: m!(2, 0),
    },
    DerivedUniform {
        param: "tr_m21",
        derive: m!(2, 1),
    },
    DerivedUniform {
        param: "tr_m22",
        derive: m!(2, 2),
    },
];

/// O `spacing_of` da CPU, em WGSL — partilhado pela redução (que não vê derivados: o uniform
/// dela é o dos params crus que ela declara). `round` para longe do zero é `floor(x + ½)` para
/// `x ≥ 1`; o não-finito e o abaixo de `1` são `1`.
const PARTILHADO: &str = "\
fn tr_spacing_of(x: f32) -> f32 {\n\
\x20   if (x >= 1.0 && x <= 3.4028235e38) { return min(floor(x + 0.5), 16.0); }\n\
\x20   return 1.0;\n\
}\n";

/// **Há algum eco já promovido na faixa `1..s`?** — o `promotes_head` da CPU, dobrado sobre o
/// estado CRU (`a >= 1 && (a as usize) < s`). `Max` é exacto em qualquer ordem.
static BANDA: &[ReduceSpec] = &[ReduceSpec {
    name: "tr_band",
    column: AGE,
    dim: Dim::Scalar,
    port: 1,
    identity: [0.0; 4],
    op: ReduceOp::Max,
    value: "select(0.0, 1.0, v >= 1.0 && floor(v) < tr_spacing_of(params.spacing))",
    params: &["spacing"],
}];

/// **Quem sobrevive** — o filtro do `step`, linha a linha do estado cru: a idade envelhecida
/// cabe na janela DESTA linha (`masked_window`, com o `falloff` que o eco herdou) e a linha é
/// um eco já promovido OU a cabeça do tique anterior num tique de promoção.
///
/// ⚠️ O arredondamento do `masked_window` é o do Rust (meio para LONGE do zero), escrito sem o
/// `x + ½` — que a `0,49999997` arredonda PARA CIMA num `f32` e daria uma linha a mais.
const PREDICADO: GpuKernel = GpuKernel {
    wgsl: "\
let pr_a = read_state_trail_age(i);\n\
let pr_bumped = floor(max(pr_a, 0.0)) + 1.0;\n\
let pr_fr = read_state_falloff(i);\n\
var pr_f = 0.0;\n\
if (pr_fr >= 0.0) { pr_f = min(pr_fr, 1.0); }\n\
let pr_x = (params.tr_window - 1.0) * pr_f;\n\
var pr_r = floor(pr_x);\n\
if (pr_x - pr_r >= 0.5) { pr_r = pr_r + 1.0; }\n\
let pr_promote = reduce_tr_band() < 0.5;\n\
let pr_keep = pr_bumped < 1.0 + pr_r && (pr_a >= 1.0 || pr_promote);\n\
write_cp_keep(i, select(0.0, 1.0, pr_keep));\n",
    wgsl_lib: "",
    bindings: &[
        ColumnBinding {
            column: AGE,
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 1,
        },
        // Ausente ⇒ `1` (o `falloff_at` da CPU).
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0, 0.0, 0.0, 0.0],
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
    params: &["tr_window"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// A identidade de cada coluna na junção — o `default_for` da CPU.
static IDENTIDADES: &[ConcatFill] = &[
    ConcatFill {
        column: "size",
        value: [1.0; 4],
        only_dim: None,
    },
    ConcatFill {
        column: "tint",
        value: [1.0; 4],
        only_dim: None,
    },
    ConcatFill {
        column: "uv_rect",
        value: [0.0, 0.0, 1.0, 1.0],
        only_dim: Some(Dim::Vec4),
    },
];

/// O corpo. Os três `$rot_*` lêem, giram as carregadas e escrevem o `rot` (vazios sem `spin`);
/// `$blend` escreve o modo em todas as linhas (vazio no `Sink`).
///
/// ⚠️ A ORDEM é a do `Decay::apply` seguida do teto: alfa, tamanho, cor, giro, teto de estreia.
///
/// ⚠️ **Toda coluna escrita é escrita UMA vez, depois do ramo, para TODAS as linhas** — uma
/// escrita dentro do `else` (a cabeça) é a forma de erro que uma paridade só apanha às vezes: a
/// linha que ninguém escreve fica com o que o buffer reciclado tinha, e com a cena parada isso é o
/// valor certo (medido: a mutação «a cabeça não escreve o `rot`» sobrevivia de forma intermitente).
macro_rules! corpo {
    ($rot_le:literal, $rot_gira:literal, $rot_escreve:literal, $blend:literal) => {
        concat!(
            "let tr_split = params.count - u32(params.tr_live);\n",
            "var tr_t = read_in_tint(i);\n",
            "var tr_s = read_in_size(i);\n",
            $rot_le,
            "var tr_age = 0.0;\n",
            "if (i < tr_split) {\n",
            "    let tr_bumped = read_in_trail_age(i) + 1.0;\n",
            "    tr_age = tr_bumped;\n",
            "    tr_t.a = tr_t.a * params.fade;\n",
            "    tr_s = vec2<f32>(tr_s.x * params.shrink, tr_s.y * params.shrink);\n",
            "    if (params.tr_colour > 0.5) {\n",
            "        let tr_r = tr_t.r;\n",
            "        let tr_g = tr_t.g;\n",
            "        let tr_b = tr_t.b;\n",
            "        tr_t.r = params.tr_m00 * tr_r + params.tr_m01 * tr_g + params.tr_m02 * tr_b;\n",
            "        tr_t.g = params.tr_m10 * tr_r + params.tr_m11 * tr_g + params.tr_m12 * tr_b;\n",
            "        tr_t.b = params.tr_m20 * tr_r + params.tr_m21 * tr_g + params.tr_m22 * tr_b;\n",
            "    }\n",
            $rot_gira,
            "    if (params.alpha_max != 1.0 && abs(tr_bumped - 1.0) < 1e-6) {\n",
            "        tr_t.a = tr_t.a * params.alpha_max;\n",
            "    }\n",
            "}\n",
            "write_trail_age(i, tr_age);\n",
            "write_tint(i, tr_t);\n",
            "write_size(i, tr_s);\n",
            $rot_escreve,
            $blend,
        )
    };
}

const LIGACOES: [ColumnBinding; 3] = [
    ColumnBinding {
        column: AGE,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "tint",
        dim: Dim::Vec4,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "size",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [1.0; 4],
        port: 0,
    },
];
const ROT: ColumnBinding = ColumnBinding {
    column: "rot",
    dim: Dim::Scalar,
    access: ColumnAccess::ReadWrite,
    identity: [0.0; 4],
    port: 0,
};
const BLEND: ColumnBinding = ColumnBinding {
    column: BLEND_COLUMN,
    dim: Dim::Scalar,
    access: ColumnAccess::Write,
    identity: [0.0; 4],
    port: 0,
};

const PARAMS: &[&str] = &[
    "fade",
    "shrink",
    "spin",
    ALPHA_MAX,
    ECHO_BLEND,
    "tr_live",
    "tr_colour",
    "tr_m00",
    "tr_m01",
    "tr_m02",
    "tr_m10",
    "tr_m11",
    "tr_m12",
    "tr_m20",
    "tr_m21",
    "tr_m22",
];

macro_rules! variante {
    ($corpo:expr, $bindings:expr) => {
        GpuKernel {
            wgsl: $corpo,
            wgsl_lib: "",
            bindings: $bindings,
            params: PARAMS,
            count_law: None,
            variant_by_param: None,
            applicable: None,
        }
    };
}

static SIMPLES: GpuKernel = variante!(corpo!("", "", "", ""), &LIGACOES);
static COM_GIRO: GpuKernel = variante!(
    corpo!(
        "var tr_rot = read_in_rot(i);\n",
        "    tr_rot = tr_rot + params.spin;\n",
        "write_rot(i, tr_rot);\n",
        ""
    ),
    &[LIGACOES[0], LIGACOES[1], LIGACOES[2], ROT]
);
static COM_MODO: GpuKernel = variante!(
    corpo!(
        "",
        "",
        "",
        "write_blend(i, clamp(floor(params.echo_blend + 0.5), 1.0, 6.0));\n"
    ),
    &[LIGACOES[0], LIGACOES[1], LIGACOES[2], BLEND]
);
static COM_GIRO_E_MODO: GpuKernel = variante!(
    corpo!(
        "var tr_rot = read_in_rot(i);\n",
        "    tr_rot = tr_rot + params.spin;\n",
        "write_rot(i, tr_rot);\n",
        "write_blend(i, clamp(floor(params.echo_blend + 0.5), 1.0, 6.0));\n"
    ),
    &[LIGACOES[0], LIGACOES[1], LIGACOES[2], ROT, BLEND]
);

/// A variante do `rot` — o `spin` pedido (a CPU materializa o `rot` quando a TAXA não é zero).
fn gira(spin: f32) -> bool {
    spin.is_finite() && spin != 0.0
}

/// O kernel registado: o despachante. A forma de topo **é** a [`SIMPLES`].
static GPU_KERNEL: GpuKernel = GpuKernel {
    variant_by_param: Some(
        |p| match (gira(p("spin")), echo_blend_tag(p(ECHO_BLEND)).is_some()) {
            (false, false) => &SIMPLES,
            (true, false) => &COM_GIRO,
            (false, true) => &COM_MODO,
            (true, true) => &COM_GIRO_E_MODO,
        },
    ),
    // ⚠️ O `Resampled` é CPU por desenho (o leque de tempo, ADR-0163).
    applicable: Some(|p| p(SOURCE) < 0.5),
    ..SIMPLES
};

/// O caso de UM eco: a entrada viva tal e qual — com o modo por cima quando o artista o
/// escolheu (o `eval` escreve-o depois do `step`, mesmo aí).
const SO_MODO: GpuKernel = GpuKernel {
    wgsl: "write_blend(i, clamp(floor(params.echo_blend + 0.5), 1.0, 6.0));\n",
    wgsl_lib: "",
    bindings: &[BLEND],
    params: &[ECHO_BLEND],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};
static PASSA: GpuKernel = GpuKernel::PASSTHROUGH;
const IDENTIDADE_KERNEL: GpuKernel = GpuKernel {
    variant_by_param: Some(|p| {
        if echo_blend_tag(p(ECHO_BLEND)).is_some() {
            &SO_MODO
        } else {
            &PASSA
        }
    }),
    ..SO_MODO
};

/// **Regista o caminho do dispositivo.**
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(
        MANIFEST.id,
        StreamOp::Carry {
            state_port: 1,
            live_port: 0,
            predicate: PREDICADO,
            predicate_reduces: BANDA,
            fills: IDENTIDADES,
            identity: identidade,
            identity_kernel: IDENTIDADE_KERNEL,
        },
    );
    reg.register_wgsl_shared(MANIFEST.id, PARTILHADO);
    reg.register_derived_uniforms(MANIFEST.id, DERIVADOS);
}

#[cfg(test)]
#[path = "kernel_tests.rs"]
mod tests;
