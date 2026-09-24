//! ⭐⭐⭐ **O KERNEL WGSL do `motion.duplicator`** — a W1(b) do ciclo 10, reaberta pelo ciclo 12
//! ([doc 120 §8](../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md)).
//!
//! ## Porque existe — a medição, não a pureza
//!
//! A escada dos tectos mediu que, numa cadeia `emissor → integrador → carimbo ← objecto`, o carimbo
//! custa **~20 %** do cozimento e a simulação **~80 %** — e a simulação só corre na CPU **porque o
//! carimbo é a fronteira do planeador**. Sem ele a mesma cadeia é reclamada inteira pelo
//! dispositivo. *Um nó CPU-only no meio de uma cadeia não custa o que ele custa: custa o dispositivo
//! inteiro* (doc 103 §5.1).
//!
//! ## A forma: o molde do `motion.clone`, com DUAS portas
//!
//! Um carimbo no modo `Off` é o produto cartesiano *shape-major*: a saída `i` é a forma `i / np` no
//! ponto `i % np` — a mesma aritmética do `pairs_for` da CPU. É um kernel
//! [`StreamOp::SourceRows`] sobre a porta `shape` (a 0): ele escreve [`ROWS_COL`] `= i / np`, e o
//! sequenciador colhe **toda outra coluna da forma** nesse índice — que é exactamente o `spread` da
//! CPU (*«every OTHER shape column is appearance — replicate it»*). As colunas dos PONTOS não
//! viajam (a transferência de fábrica é `Shape Wins`: o ponto só dá o `P`).
//!
//! As leituras são duas [`ColumnAccess::SourceRead`] — `read_shape_P(i / np)` e
//! `read_points_P(i % np)`; o codegen qualifica o nome pela porta porque o nó tem duas, e a presença
//! é julgada por PORTA (uma porta de pontos vazia lê a identidade `[0, 0]`, que é o `p_at` da CPU).
//!
//! ## O que ele NÃO re-deriva em WGSL
//!
//! O `np` passa pelo clamp de orçamento ([`super::points_within_budget`] contra o
//! [`MAX_INSTANCIAS_POR_NO`]) e chega por [`DerivedUniform`], calculado pela MESMA função que o
//! `eval` chama — a lei de contagem e o corpo leem o mesmo número. *Duas leis de contagem que
//! discordam não falham: desenham um número diferente de coisas.*
//!
//! ## A lei, coluna a coluna, contra a CPU
//!
//! | coluna | CPU (`duplicate`) | aqui |
//! |---|---|---|
//! | `P` | `shape.P[si] + points.P[pi]` | a mesma soma em `f32` |
//! | `rot` | existe se QUALQUER lado a tem; soma | só a forma a pode trazer: os pontos com `rot` são RECUSADOS ([`ColumnAccess::RefuseIfPresent`]) e a da forma é [`ColumnAccess::SourceReadWriteExisting`] ⇒ existe iff a forma a tem, com `+ 0` exacto |
//! | `Index` | `i` | `f32(i)` — escrita SEMPRE, como a CPU (que a CUNHA) |
//! | `Count` | `total` | o `total` derivado |
//! | o resto da forma | `spread` em `si` | o gather do sequenciador em `cp_rows = si` |
//! | o resto dos pontos | deitado fora (`Shape Wins`) | não viaja |
//!
//! ⚠️ **O desvio DECLARADO, e o único:** com a porta de pontos VAZIA a CPU devolve a forma **tal
//! qual** (`shape.clone()`), e aqui o `Index`/`Count` são escritos na mesma (`0..ns`, `ns`). O
//! desenho é o mesmo — a posição, a textura, o tamanho —, e a diferença é só a presença de duas
//! colunas num quadro sem pontos (o 1.º instante de um emissor). *Um `Write` não sabe não escrever,
//! e a alternativa condicional (`…Existing`) partiria o caso comum, em que a CPU CUNHA as duas.*
//!
//! ## As RECUSAS (`applicable`)
//!
//! Tudo o que o modo de fábrica não é vai à CPU, que é o caminho canónico — a recusa é pelo lado
//! seguro por construção:
//!
//! - **`pick ≠ Off`** (`Cycle` / `Random`) — outra aritmética de pares e outra contagem (`np` e não
//!   `ns · np`); um kernel por escrever, não um bloqueio;
//! - **`transfer ≠ Shape Wins`** — as colunas dos pontos passam a viajar, e são dinâmicas (os nomes
//!   dependem do stream), coisa que uma lista estática de ligações não exprime;
//! - **`point_scale > 0`** — a escala do ponto compõe-se com a da forma, e o ramo da CPU só corre
//!   quando o ponto TRAZ `size`. ⚠️ O predicado é `p <= 0` **à letra do `apply_point_scale`**, e é
//!   isso que manda um `NaN` para a CPU (onde ele aplica a escala inteira) em vez de o engolir.

use super::{MANIFEST, POINT_SCALE, Pick, points_within_budget, transfer};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, DerivedUniform, GpuKernel, ROWS_COL, SourceWindow,
    StreamOp,
};
use ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO;
use ph2d_nodegraph::port::Dim;

/// Os pontos que o carimbo usa, **já cortados pelo orçamento** — `0` quer dizer *«sem pontos:
/// passa a forma»*.
const NP: &str = "dp_np";
/// Quantos elementos saem — a `Count` que se escreve em toda a linha.
const TOTAL: &str = "dp_total";

/// **`(ns, np)` depois do orçamento** — a MESMA chamada do `eval`, e a única resposta a *«quantos
/// pontos?»* que o hospedeiro dá à lei de contagem e ao uniforme.
fn formas_e_pontos(c: &CountLawCtx<'_>) -> (usize, usize) {
    let ns = c.inputs.first().copied().unwrap_or(0) as usize;
    let np_pedidos = c.inputs.get(1).copied().unwrap_or(0) as usize;
    (
        ns,
        points_within_budget(Pick::Off, ns, np_pedidos, MAX_INSTANCIAS_POR_NO),
    )
}

/// **Quantos elementos saem** — `ns · np`, ou a forma inteira quando não há pontos (o
/// `shape.clone()` da CPU).
fn total(c: &CountLawCtx<'_>) -> usize {
    match formas_e_pontos(c) {
        (ns, 0) => ns,
        (ns, np) => ns * np,
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "contagem de elementos, cortada pelo orçamento ⇒ muito abaixo de 2^24"
)]
fn como_f32(n: usize) -> f32 {
    n as f32
}

/// Os dois números que o hospedeiro resolve — ver o cabeçalho.
static DERIVADOS: &[DerivedUniform] = &[
    DerivedUniform {
        param: NP,
        derive: |c| como_f32(formas_e_pontos(c).1),
    },
    DerivedUniform {
        param: TOTAL,
        derive: |c| como_f32(total(c)),
    },
];

/// O kernel do dispositivo (ADR-0136 `StreamOp::SourceRows` sobre a porta `shape`).
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let dp_np = u32(params.dp_np);\n\
        var dp_s = i;\n\
        var dp_p = 0u;\n\
        if (dp_np > 0u) {\n\
        \x20   dp_s = i / dp_np;\n\
        \x20   dp_p = i % dp_np;\n\
        }\n\
        write_cp_rows(i, f32(dp_s));\n\
        let dp_sp = read_shape_P(dp_s);\n\
        let dp_pp = read_points_P(dp_p);\n\
        write_P(i, vec2<f32>(dp_sp.x + dp_pp.x, dp_sp.y + dp_pp.y));\n\
        write_rot(i, read_shape_rot(dp_s));\n\
        write_Index(i, f32(i));\n\
        write_Count(i, params.dp_total);\n",
    wgsl_lib: "",
    bindings: &[
        // A posição da FORMA, lida na forma `i / np` — a length-decouple da porta template.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        // A posição do PONTO, lida no ponto `i % np`. Ausente (porta vazia) ⇒ `[0, 0]`, o `p_at`.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 1,
        },
        // A posição da SAÍDA — um buffer diferente dos de cima.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // A rotação da FORMA: escrita só quando a forma a traz — que é, com a recusa de baixo, o
        // `has_rot` da CPU inteiro.
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::SourceReadWriteExisting,
            identity: [0.0; 4],
            port: 0,
        },
        // Pontos com rotação própria somariam uma coluna que a forma pode não ter — o `has_rot`
        // é um OU das duas portas, e nenhum verbo exprime «escreve se QUALQUER porta a traz».
        // ⇒ recusa ao planear (a CPU é canónica), e só neste caso.
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::RefuseIfPresent,
            identity: [0.0; 4],
            port: 1,
        },
        // A linha da forma de que cada elemento nasce: o sequenciador colhe aqui todas as outras
        // colunas da forma e deita esta fora.
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // A RENUMERAÇÃO — a CPU CUNHA as duas sempre que há pontos.
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[NP, TOTAL],
    count_law: Some(|c| SourceWindow::of_count(total(c))),
    variant_by_param: None,
    // As três recusas do cabeçalho.
    applicable: Some(|p| {
        Pick::of(p("pick")) == Pick::Off
            && transfer::Transfer::of(p(transfer::TRANSFER)).is_inert()
            && p(POINT_SCALE) <= 0.0
    }),
};

/// **Regista o caminho do dispositivo.** Side-metadata do ADR-0126/0136 — o contrato congelado do
/// `CLAUDE.md` §6 fica intacto.
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(MANIFEST.id, StreamOp::SourceRows { port: 0 });
    reg.register_derived_uniforms(MANIFEST.id, DERIVADOS);
}

#[cfg(test)]
#[path = "kernel_tests.rs"]
mod tests;
