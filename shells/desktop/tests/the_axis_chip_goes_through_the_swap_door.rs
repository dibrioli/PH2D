//! ⛔⛔ **O chip de um EIXO de propriedade religa pela porta do `swap`, e por mais nenhuma.**
//!
//! Este é o gémeo geral do `the_variant_chip_goes_through_the_swap_door` do sistema vetorial, e
//! existe pela mesma razão: o `instance_variant::swap` é quem faz o **re-key determinístico** das
//! excepções, quem **sepulta** os órfãos e quem **esquece o eco**. Escrever o vínculo por outra via
//! deixa a cópia a guardar diferenças que apontam peças do mestre ANTIGO — e o sintoma é uma
//! excepção que reaparece no sítio errado, meses depois, sem ninguém saber ligar as duas coisas.
//!
//! ⚠️ **Textual de propósito.** A alternativa é um teste de integração que monte uma família e
//! clique — e ele prova que ESTE caminho funciona, não que **não existe um segundo**. A ausência de
//! uma segunda escrita não se mede correndo o caminho certo.
//!
//! ⛔ Ele descasca comentários antes de varrer: documentar a cura não pode reprovar o portão.

use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Tira comentários de linha — a lição da caça de 2026-08-30.
fn strip_comments(s: &str) -> String {
    s.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐ **O chip resolve o alvo pelo modelo que o PINTOU.**
///
/// ⚠️ O `axes` do `InspectorInstanceInfo` é a lista que o cartão desenhou **neste quadro**; uma
/// segunda travessia (recalcular a família aqui) daria uma ordem que pode divergir, e o chip `Big`
/// escolheria `Medium`. É a lição literal do `addressed_pieces` do vetor.
#[test]
fn the_axis_chip_reads_the_model_that_painted_it() {
    let body = strip_comments(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("raiz do repo")
                .join("crates/ph2d-panel-inspector/src/event.rs"),
        )
        .expect("event.rs do inspector"),
    );
    assert!(
        body.contains("ids::instance_axis_option(id)"),
        "o braço do chip não usa a PORTA da escada de ids — uma varredura à mão aqui seria a \
         quarta cópia da mesma lei"
    );
    assert!(
        body.contains("info.axes.get(a).and_then(|ax| ax.options.get(v))"),
        "o braço não resolve o chip no modelo que o pintou"
    );
    assert!(
        body.contains("EditorAction::InspectorSwapVariant"),
        "o chip não levanta a acção da troca — ele ficaria pintado e morto"
    );
}

/// ⛔ **E o painel NÃO escreve o vínculo por outra via.**
///
/// ⚠️ A metade que o gate acima não cobre: a acção podia ser levantada **e** uma segunda linha
/// escrever o mestre à mão, o que é o defeito com todos os gates verdes.
#[test]
fn the_inspector_never_writes_the_master_link_by_hand() {
    let body = strip_comments(
        &std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .and_then(Path::parent)
                .expect("raiz do repo")
                .join("crates/ph2d-panel-inspector/src/event.rs"),
        )
        .expect("event.rs do inspector"),
    );
    for forbidden in ["InstanceOf {", "InstanceOf::new(", ".master ="] {
        assert!(
            !body.contains(forbidden),
            "o painel escreve o vínculo por `{forbidden}` — o re-key das excepções vive dentro do \
             `instance_variant::swap`, e uma segunda escrita pula-o em silêncio"
        );
    }
}

/// ⭐⭐ **E o shell dreno passa pela porta do `swap`.**
#[test]
fn the_shell_drains_the_swap_through_its_door() {
    let body = strip_comments(&src("render_loop/mod.rs"));
    assert!(
        body.contains("instance_variant::swap("),
        "o dreno da troca não chama a porta — sem ela não há re-key, nem sepultamento de órfãos"
    );
}
