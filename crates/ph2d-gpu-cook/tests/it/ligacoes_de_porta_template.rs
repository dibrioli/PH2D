//! ⭐⭐⭐ **AS LIGAÇÕES DE PORTA TEMPLATE no CODEGEN** — a família que a W1 do ciclo 10 construiu
//! (doc 116 §5.3/§5.4), medida onde ela decide: o plano de bindings e a chave da cache.
//!
//! São **três** verbos (`SourceRead` · `SourceReadWriteExisting` · `SourceWriteExisting`) e o que
//! os junta é a PORTA — length-decoupled do dispatch —, não o que cada um faz com ela.
//!
//! ⛔⛔ **O gate do enum (`ph2d-nodegraph`) não prova isto.** Ele afirma o que os predicados
//! respondem; este afirma que o codegen os HONRA — e as duas coisas já divergiram nesta casa
//! (o `ReadBroadcast` nasceu com os predicados certos e um `match` que o tratava como escritor,
//! e o que o apanhou foi o naga a recusar `redefinition of out_v`).
//!
//! ⚠️ **Sem dispositivo, de propósito:** é geração de TEXTO, e um gate que precisasse de placa
//! seria `#[ignore]` ⇒ o CI nunca o correria.

use ph2d_gpu_cook::codegen::{BindingPlan, plan_bindings, presence_signature};
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

/// ⭐⭐⭐ **A ESCRITA SEM LEITURA NÃO LIGA BUFFER DE LEITURA** — e é essa ausência que a faz
/// existir.
///
/// ⛔⛔ **O defeito que ela impede não é teórico e não é um número errado:** com a coluna ligada
/// pela CRUZA, o módulo declara `in_Count`, o corpo nunca chama `read_Count`, **a naga apaga esse
/// buffer do layout derivado** e o bind group do sequenciador fica com uma entrada a mais —
/// `create_bind_group` estoura, e só no tique em que a coluna nasce.
///
/// ⚠️ **O CONTROLO é a cruza na mesma corrida:** ela tem as MESMAS duas metades de escrita e liga
/// um buffer de leitura. *Sem ele este gate passaria com um alias da cruza.*
#[test]
fn a_escrita_sem_leitura_nao_liga_buffer_de_leitura() {
    let bindings = [
        liga("Count", ColumnAccess::SourceWriteExisting),
        liga("Index", ColumnAccess::SourceReadWriteExisting),
    ];
    let com = plan_bindings(&bindings, |_| true);
    assert_eq!(
        com[0],
        (None, Some(BindingPlan::WriteBuffer)),
        "presente: escreve um buffer e NAO liga leitura nenhuma"
    );
    let sem = plan_bindings(&bindings, |_| false);
    assert_eq!(
        sem[0],
        (None, Some(BindingPlan::WriteDropped)),
        "ausente: a escrita e' DESCARTADA (a coluna nao e' cunhada) e continua sem leitura"
    );
    // ⭐ O CONTROLO: a cruza escreve igual E liga o buffer de leitura nos dois casos.
    assert_eq!(com[1].0, Some(BindingPlan::ReadBuffer));
    assert_eq!(sem[1].0, Some(BindingPlan::ReadIdentity));
    assert_eq!(com[1].1, Some(BindingPlan::WriteBuffer));
    assert_eq!(sem[1].1, Some(BindingPlan::WriteDropped));
}

/// ⭐⭐⭐ **E A PRESENÇA DELA ENTRA NA CHAVE DA CACHE** — a metade que o plano de bindings não
/// prova e cuja falha é um crash de `create_bind_group` num tique tardio.
///
/// ⛔ Os dois módulos diferem por uma ligação inteira (`WriteBuffer` contra `WriteDropped`) com o
/// MESMO conjunto de ligações declaradas; presos à mesma entrada da cache, o wgpu valida um bind
/// group contra o layout errado. ⚠️ **A assinatura de presença tinha o bit preso a `reads()`**, e
/// um verbo que escreve sem ler caía sempre no mesmo bit.
#[test]
fn a_presenca_de_uma_escrita_sem_leitura_entra_na_chave() {
    let so_escrita = [liga("Count", ColumnAccess::SourceWriteExisting)];
    assert_ne!(
        presence_signature(&so_escrita, |_| true),
        presence_signature(&so_escrita, |_| false),
        "dois modulos diferentes nao podem partilhar uma entrada da cache"
    );
    // ⭐ O CONTROLO: um verbo cujo plano NÃO depende da presença tem de dar a mesma chave — senão
    // este gate passaria com uma assinatura que muda por tudo e a cache deixava de servir.
    let incondicional = [liga("P", ColumnAccess::Write)];
    assert_eq!(
        presence_signature(&incondicional, |_| true),
        presence_signature(&incondicional, |_| false),
        "uma escrita INCONDICIONAL gera o mesmo modulo com e sem a coluna"
    );
}
