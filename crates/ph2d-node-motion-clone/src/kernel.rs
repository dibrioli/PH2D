//! ⭐⭐⭐ **O KERNEL WGSL do `motion.clone`** — a W1(a) do ciclo 10 (doc 116 §5.3/§5.4).
//!
//! O corte do `lib.rs` é por RESPONSABILIDADE e não por tamanho, a mesma costura do
//! `motion.kaleidoscope`: o `lib.rs` responde *o que um cloner É* (o manifesto, o `eval`, o
//! registo) e este responde *o que o dispositivo corre*.
//!
//! ## A forma: o molde do `motion.kaleidoscope`, com uma TRANSLAÇÃO no lugar da rotação
//!
//! Um cloner é um kernel [`StreamOp::SourceRows`] que **muda a contagem** (`n · k`), *slice-major*
//! como o irmão: a saída `i` é a cópia `i / n` na linha-fonte `i % n`. Ele lê a fonte nesse índice
//! ([`ColumnAccess::SourceRead`], a length-decouple que torna a leitura da porta template
//! presente), escreve o `P` deslocado e o [`ROWS_COL`]; o sequenciador colhe **todas as outras**
//! colunas do template nesses índices, que é exactamente a réplica que o `replicate` da CPU faz.
//!
//! ## O que ele NÃO re-deriva em WGSL — e é a razão de a paridade ser barata
//!
//! ⚠️ **Três números saem do hospedeiro por [`DerivedUniform`]**, calculados pelas MESMAS funções
//! que o `eval` chama:
//!
//! | uniform | a lei | quem mais a lê |
//! |---|---|---|
//! | [`K`] | [`super::copies_within_budget`] sobre o [`param_as_count`] | a `count_law` deste kernel |
//! | [`STEP_X`]/[`STEP_Y`] | [`radial::linear_step`] | o `Placement::of` da CPU |
//!
//! ⛔ **O passo NÃO podia ser trig no dispositivo.** A direcção da fila é `distance · (cos θ,
//! sin θ)` com a seno parabólica da casa (HR-5, [`super::trig`]) — portá-la seria a **segunda
//! cópia** de uma lei, e a alternativa (o `sin` do WGSL) é uma curva de outro fabricante, não um ε
//! mais apertado. ⭐ E ela é um número **por NÓ** e não por elemento, logo derivá-la no hospedeiro
//! não custa nem um ciclo ao dispositivo.
//!
//! ⛔ **E o `k` também não.** Ele passa pelo clamp de orçamento (`copies_within_budget` contra o
//! [`RECOMMENDED_MAX_ELEMENTS`], a cerca §6.3 do doc 116): reescrito em WGSL ele seria uma segunda
//! resposta à pergunta *«quantas cópias?»*, e duas respostas que discordem não falham — **desenham
//! um número diferente de coisas**. Derivado, a lei de contagem e o corpo leem o mesmo `f32`.
//!
//! ## A RENUMERAÇÃO, que é a peça que este kernel existe para provar
//!
//! `Index`/`Count` são [`ColumnAccess::SourceReadWriteExisting`] — a cruza que a §5.4 construiu:
//! **lê na FONTE** (`i % n`, porque o template tem `n` linhas e o dispatch tem `n · k`) e **escreve
//! só quando a entrada carrega a coluna**, que é o braço de `match` do `clone_stream`. Medido no
//! produto, `38` de `78` portas de multiplicador não trazem pelo menos uma delas — *nem «escrever
//! sempre» (cunha em 38) nem «recuar sempre» (perde tudo)*.
//!
//! ⛔⛔ **E a `Count` NÃO é a mesma variante, porque um buffer LIDO POR NINGUÉM é um crash.** Ela
//! vale `total` em toda a linha — *não depende do que entrou* —, logo o corpo escreve-a e nunca a
//! lê; ligada como a cruza, o módulo declarava `in_Count`, a naga apagava esse buffer do layout
//! derivado e o bind group do sequenciador ficava com uma entrada a mais. ⇒ ela é a
//! [`ColumnAccess::SourceWriteExisting`], que tem as duas metades de que precisa (porta template ·
//! escreve só se presente) e **larga a leitura**. *Quem o apanhou foi o
//! `every_registered_kernel_validates_across_the_whole_presence_space`, sem placa nenhuma.*
//!
//! ## As RECUSAS, com a população ao lado (doc 116 §5.2)
//!
//! Dos `6` cartões de `motion.clone` no produto, este kernel alcança **`2`**. As outras quatro são
//! [`GpuKernel::applicable`] a devolver `false`, e cada uma tem mecanismo:
//!
//! - **o leque de relógios** (`time_offset ≠ 0`, `1` cartão) é **COZIMENTO e não kernel** — ele
//!   re-cozinha a entrada em N instantes (`TimeFans`, ADR-0163). Não há kernel que o exprima
//!   porque o que muda não é a conta, é quantas vezes o grafo a montante corre;
//! - **o modo `Radial`** (`1` cartão) é uma **variante** por escrever (o `wgsl_lib` do
//!   kaleidoscope já tem a seno parabólica portada), não um bloqueio;
//! - **o taper** (`3` cartões) é o único que *CUNHA*: com o knob ligado e a coluna ausente o `eval`
//!   cria `size`/`rot`, e uma escrita `…Existing` é, por definição, a que **não** cunha. ⚠️ O
//!   `applicable` só vê PARAMS, nunca a forma do stream, logo a recusa é pelo knob — inclusive nas
//!   cadeias em que a coluna existe e a cruza bastaria.
//!
//! *A recusa é pelo lado seguro por construção: a CPU é o caminho canónico, e um `false` aqui
//! entrega o nó inteiro a ela.*

use super::{MODE, ROT_TAPER, SCALE_TAPER, copies_within_budget, fan, radial};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, DerivedUniform, GpuKernel, ROWS_COL, SourceWindow,
    StreamOp,
};
use ph2d_nodegraph::node::{RECOMMENDED_MAX_ELEMENTS, param_as_count};
use ph2d_nodegraph::port::Dim;

/// O slot do uniform que carrega as cópias **já cortadas pelo orçamento** — um nome que só a
/// derivação conhece (o manifesto não tem um param para lhe emprestar, e o `count` cru continua a
/// ser o que a lei de contagem e o `applicable` leem).
const K: &str = "cl_k";
/// O passo da fila em X, `distance · cos θ` — ver [`radial::linear_step`].
const STEP_X: &str = "cl_step_x";
/// O passo em Y, `distance · sin θ`.
const STEP_Y: &str = "cl_step_y";

/// **Quantas cópias este nó emite**, sobre as contagens e os params CRUS — a mesma expressão do
/// `eval`, chamada pela lei de contagem **e** pelo uniform derivado.
///
/// ⚠️ *Duas leis de contagem que discordam não falham: desenham um número diferente de coisas.*
fn copias(c: &CountLawCtx<'_>) -> usize {
    let n = c.inputs.first().copied().unwrap_or(0) as usize;
    let pedidas = param_as_count((c.param)("count"), RECOMMENDED_MAX_ELEMENTS);
    copies_within_budget(pedidas, n, RECOMMENDED_MAX_ELEMENTS)
}

/// O passo da fila, pela porta que a CPU usa.
fn passo(c: &CountLawCtx<'_>) -> (f32, f32) {
    radial::linear_step((c.param)("angle"), (c.param)("distance"))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "contagem de elementos, cortada pelo orçamento ⇒ ≤ 2^24"
)]
fn como_f32(n: usize) -> f32 {
    n as f32
}

/// Os três números que o hospedeiro resolve — ver o cabeçalho.
static DERIVADOS: &[DerivedUniform] = &[
    DerivedUniform {
        param: K,
        derive: |c| como_f32(copias(c)),
    },
    DerivedUniform {
        param: STEP_X,
        derive: |c| passo(c).0,
    },
    DerivedUniform {
        param: STEP_Y,
        derive: |c| passo(c).1,
    },
];

/// O kernel do dispositivo (ADR-0136 `StreamOp::SourceRows`).
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let cl_srcn = max(params.window_src_n, 1u);\n\
        let cl_copy = i / cl_srcn;\n\
        let cl_row = i % cl_srcn;\n\
        write_cp_rows(i, f32(cl_row));\n\
        var cl_rank = f32(cl_copy);\n\
        if (params.center >= 0.5) { cl_rank = cl_rank - (params.cl_k - 1.0) * 0.5; }\n\
        let cl_src = read_P(cl_row);\n\
        write_P(i, vec2<f32>(\n\
        \x20   cl_src.x + cl_rank * params.cl_step_x,\n\
        \x20   cl_src.y + cl_rank * params.cl_step_y));\n\
        write_Index(i, read_Index(cl_row) + f32(cl_copy * cl_srcn));\n\
        write_Count(i, f32(params.count));\n",
    wgsl_lib: "",
    bindings: &[
        // A posição da FONTE, lida na linha `i % n` — a length-decouple: o template tem `n`
        // linhas e o dispatch tem `n · k`.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        // A posição da SAÍDA — um buffer diferente do de cima.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // A linha do template de que cada elemento nasce: o sequenciador colhe aqui todas as
        // outras colunas e deita esta fora.
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // A RENUMERAÇÃO — contínua ao longo das cópias, e só quando a entrada as traz.
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::SourceReadWriteExisting,
            identity: [0.0; 4],
            port: 0,
        },
        // A `Count` é `total` em toda a linha: escreve-se, nunca se lê (ver o cabeçalho).
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::SourceWriteExisting,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["center", K, STEP_X, STEP_Y],
    // `n · k`, com o `k` do orçamento — a `positions.len()` da CPU.
    count_law: Some(|c| {
        let n = c.inputs.first().copied().unwrap_or(0) as usize;
        SourceWindow::of_count(n * copias(c))
    }),
    variant_by_param: None,
    // As quatro recusas do cabeçalho, pela ordem em que a §5.2 as contou.
    applicable: Some(|p| {
        p(fan::TIME_OFFSET).abs() < fan::MIN_OFFSET
            && (p(MODE).round() as i32) != radial::MODE_RADIAL
            && p(SCALE_TAPER) == 1.0
            && p(ROT_TAPER) == 0.0
    }),
};

/// **Regista o caminho do dispositivo.** Side-metadata do ADR-0126/0136 — o contrato congelado do
/// `CLAUDE.md` §6 fica intacto.
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(super::MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(super::MANIFEST.id, StreamOp::SourceRows { port: 0 });
    reg.register_derived_uniforms(super::MANIFEST.id, DERIVADOS);
}

#[cfg(test)]
#[path = "kernel_tests.rs"]
mod tests;
