//! ⭐⭐⭐ **A CRUZA `SourceReadWriteExisting` no CODEGEN** — a peça que a W1 do ciclo 10 pediu
//! (doc 116 §5.3), medida onde ela decide: o plano de bindings e o módulo gerado.
//!
//! ⛔⛔ **O gate do enum (`ph2d-nodegraph`) não prova isto.** Ele afirma o que os predicados
//! respondem; este afirma que o codegen os HONRA — e as duas coisas já divergiram nesta casa
//! (o `ReadBroadcast` nasceu com os predicados certos e um `match` que o tratava como escritor,
//! e o que o apanhou foi o naga a recusar `redefinition of out_v`).
//!
//! ⚠️ **Sem dispositivo, de propósito:** é geração de TEXTO, e um gate que precisasse de placa
//! seria `#[ignore]` ⇒ o CI nunca o correria.

use ph2d_gpu_cook::codegen::{BindingPlan, plan_bindings};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding};
use ph2d_nodegraph::port::Dim;

fn liga(col: &'static str, access: ColumnAccess) -> ColumnBinding {
    ColumnBinding {
        column: col,
        dim: Dim::Scalar,
        access,
        identity: [0.0; 4],
        port: 0,
    }
}

/// ⭐⭐⭐ **A CRUZA ESCREVE UM BUFFER QUANDO A COLUNA EXISTE E UM NO-OP QUANDO ELA FALTA** — as
/// duas metades, e a segunda é a razão de ela existir.
///
/// ⚠️ **O CONTROLO é o `SourceRead` ao lado**, na mesma corrida: ele lê na fonte igual e **nunca**
/// escreve, que é o que o tornava insuficiente para renumerar. *Sem ele este gate passaria com uma
/// cruza que fosse só um `SourceRead` com outro nome.*
#[test]
fn a_cruza_escreve_so_quando_a_coluna_existe() {
    let bindings = [
        liga("Index", ColumnAccess::SourceReadWriteExisting),
        liga("P", ColumnAccess::SourceRead),
    ];
    // Presente: buffer de leitura + buffer de escrita.
    let com = plan_bindings(&bindings, |_| true);
    assert_eq!(
        com[0],
        (
            Some(BindingPlan::ReadBuffer),
            Some(BindingPlan::WriteBuffer)
        ),
        "com a coluna presente a cruza le e ESCREVE"
    );
    // Ausente: lê a identidade e a escrita é DESCARTADA — a coluna fica ausente na saída.
    let sem = plan_bindings(&bindings, |_| false);
    assert_eq!(
        sem[0],
        (
            Some(BindingPlan::ReadIdentity),
            Some(BindingPlan::WriteDropped)
        ),
        "com a coluna ausente a cruza NAO a cunha"
    );
    // ⭐ O CONTROLO: o irmão que só lê nunca escreve, presente ou ausente.
    assert_eq!(
        com[1].1, None,
        "o `SourceRead` nao escreve com a coluna la'"
    );
    assert_eq!(sem[1].1, None, "nem sem ela");
}

/// ⭐⭐ **E o `write_` EXISTE nos dois casos** — é isso que faz um corpo escrito contra ela
/// compilar quer a cadeia traga a coluna quer não.
///
/// ⛔ *É a forma que um booleano não carrega*, e o doc do [`BindingPlan::WriteDropped`] di-lo: a
/// coluna está ausente, logo não há buffer, **mas o corpo ainda chama `write_Index`** — sem o
/// acessor no-op o módulo não compilaria em metade das cadeias do produto (medido: `38` de `78`
/// portas de multiplicador não trazem `Index`/`Count`).
#[test]
fn o_acessor_de_escrita_existe_mesmo_quando_a_coluna_falta() {
    let bindings = [liga("Index", ColumnAccess::SourceReadWriteExisting)];
    for presente in [true, false] {
        let plano = plan_bindings(&bindings, |_| presente);
        assert!(
            plano[0].1.is_some(),
            "presente={presente}: o corpo tem sempre um `write_Index` para chamar"
        );
    }
}
