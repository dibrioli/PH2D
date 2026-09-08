//! **O KERNEL WGSL do `motion.mirror`** — o nó que MULTIPLICA a contagem e que era o único
//! da família TRANSFORM sem rota de dispositivo (ciclo 3, W2 — doc 106 §2.2).
//!
//! ## ⛔⛔ Não era lacuna de param: era lacuna de COBERTURA, e a folha 05 §0 escreveu-a em 2026-08-09
//!
//! > *«`motion.mirror` é o ÚNICO dos seis sem rota de GPU — e é justamente o que MULTIPLICA a
//! > contagem, a família que o ADR-0136 desenhou e que o `motion.kaleidoscope` (mesma forma:
//! > `count → k·n`) já percorre com `count_law: Some(..)`.»*
//!
//! A medição do ciclo 3 confirmou-o com número: o `motion.kaleidoscope` devolve **614 400**
//! objectos de 102 400 (`6×`) e a cadeia continua reivindicada pelo dispositivo; o
//! `motion.mirror` faz `2n` — a **mesma forma** — e derrubava a cadeia inteira para a CPU.
//! ⚠️ *E o que se perde não são os milissegundos deste nó: um nó CPU-only no meio de uma cadeia
//! custa o DISPOSITIVO INTEIRO* — `50,9×` medidos na [auditoria 98].
//!
//! [auditoria 98]: ../../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md

use super::{FLIP_ROT, KEEP, KEEP_REFLECTION, REINDEX};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ROWS_COL, SourceWindow};
use ph2d_nodegraph::port::Dim;

/// A saída é `2n` (o par) ou `n` (só o reflexo) — a MESMA escada que o `eval` percorre, lida do
/// mesmo param e com o mesmo arredondamento, senão os dois lados cunham contagens diferentes.
pub(crate) const fn keeps_only_the_reflection(keep: f32) -> bool {
    // `round` do Rust: metade para longe do zero. Um `Enum` só produz inteiros, mas a
    // convenção fica escrita porque o param é alcançável por um fio.
    (if keep >= 0.0 {
        (keep + 0.5) as i32
    } else {
        -((-keep + 0.5) as i32)
    }) == KEEP_REFLECTION
}

/// O device do `motion.mirror` (ADR-0136 `StreamOp::SourceRows`), na forma do irmão
/// `motion.kaleidoscope`: o corpo escreve `cp_rows` e o `P` de saída, e o sequenciador junta
/// as outras colunas por um GATHER do template.
///
/// ⚠️ **A linha de espelho é o CENTROIDE do que entrou, mais o `offset`** — duas reduções
/// `Sum` da porta [`ph2d_nodegraph::pivot`], divididas pela contagem da **ENTRADA**
/// (`window_src_n`) e **não** por `params.count`, que aqui é a da saída (`2n`). *Esse foi o
/// defeito que o gate red-first do `motion.kaleidoscope` mediu a `3300×` a barra.*
///
/// ⚠️ **A aritmética é a da CPU, na mesma ORDEM** (`2·(c + offset) − q`): trocar para
/// `2c + 2·offset − q` é a mesma álgebra e outro número.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let mr_srcn = max(params.window_src_n, 1u);\n\
        let mr_row = i % mr_srcn;\n\
        write_cp_rows(i, f32(mr_row));\n\
        // ⚠️ `read_P` e nao `read_in_P`: o prefixo da porta so' aparece num no' com MAIS DE\n\
        // UMA entrada (o irmao `motion.kaleidoscope` tem a `spin`, este nao tem nenhuma).\n\
        let mr_src = read_P(mr_row);\n\
        let mr_c = vec2<f32>(reduce_cx(), reduce_cy()) / f32(mr_srcn);\n\
        // O GEMEO e' a segunda metade -- ou TODA a saida, quando so' o reflexo fica.\n\
        let mr_twin = (round(params.keep) == 1.0) || (i >= mr_srcn);\n\
        var mr_out = mr_src;\n\
        if (mr_twin) {\n\
        \x20   if (round(params.axis) == 0.0) {\n\
        \x20       mr_out.x = 2.0 * (mr_c.x + params.offset) - mr_src.x;\n\
        \x20   } else {\n\
        \x20       mr_out.y = 2.0 * (mr_c.y + params.offset) - mr_src.y;\n\
        \x20   }\n\
        }\n\
        write_P(i, mr_out);\n",
    wgsl_lib: "",
    bindings: &[
        // A posição de origem, lida na linha mapeada — comprimento desacoplado do dispatch
        // (o template é `n`, a saída `2n`).
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        // A posição de SAÍDA — um buffer separado da leitura acima.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        // A linha do template de que cada elemento nasce — a maquinaria do `SourceRows`.
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &["axis", "offset", KEEP],
    count_law: Some(|c| {
        let n = c.inputs.first().copied().unwrap_or(0) as usize;
        SourceWindow::of_count(if keeps_only_the_reflection((c.param)(KEEP)) {
            n
        } else {
            n * 2
        })
    }),
    variant_by_param: None,
    // ⛔ **DOIS knobs recuam para a CPU, e as duas razões são a mesma coisa vista de dois
    // lados: as colunas que NÃO são `P` chegam à saída por um GATHER do template.**
    //
    // 1. `reindex` escreve `Index`/`Count` **novos** — outra operação, e não a que este
    //    kernel acelera. É o mesmo `applicable`, pelo mesmo motivo, que o
    //    `motion.kaleidoscope` e o `motion.combine` já declaram.
    // 2. `flip_rot` **reflecte** o `rot` e o `vel` do gémeo, e o gather copia-os. ⚠️ A saída
    //    existe (o gather salta a coluna que o corpo escreveu, `!out.cols.contains_key`), mas
    //    o preço é uma **binding de escrita condicional à PRESENÇA da coluna no template**: um
    //    `rot` ausente é ausente na CPU e seria **cunhado** pelo device, que é divergência de
    //    SHAPE e não um ε — a armadilha que o `motion.orbit` já documentou no `carry_rotation`.
    //    ⇒ nomeado com o mecanismo, medido como o caso raro que é (`rot`/`vel` são colunas de
    //    sim, e o default do knob é `0`).
    applicable: Some(|p| p(REINDEX) < 0.5 && p(FLIP_ROT) < 0.5),
};
