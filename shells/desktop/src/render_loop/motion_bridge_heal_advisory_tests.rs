//! **O TEXTO DO AVISO** — os gates da frase que o artista lê, cortados do irmão no teto de
//! LOC do shell (600).
//!
//! O corte é por RESPONSABILIDADE: o arquivo pai dirige o **funil de intents** (o clique no
//! badge, a inserção, o undo) e este verifica **o que a frase diz**. São perguntas
//! diferentes — uma cura pode ser aplicada correctamente e ser explicada errado.

use super::*;

/// **CADA DÉFICIT TEM MENSAGEM PRÓPRIA — o `_ =>` não pode absorver um variante novo.**
///
/// ⚠️ **O `match` do [`explain`] termina num catch-all**, então um `Deficit` acrescentado
/// compila em silêncio e o artista recebe *"This node produces data nothing downstream
/// consumes"* — que para um passe de tela SOMBREADO é simplesmente falso (ele não produz
/// coluna nenhuma; ele foi ignorado). É a armadilha nomeada na memória do repo: *um `match`
/// exaustivo não guarda a lista que um catch-all cobre.*
///
/// O gate compara cada mensagem com a do catch-all e exige que difira.
#[test]
fn every_deficit_has_its_own_advisory_and_none_falls_into_the_catch_all() {
    let generic = explain(&Diagnostic {
        node: NodeId(0),
        deficit: Deficit::InertProducer("uma_coluna_que_nao_existe"),
        fix: Fix::Offer,
    });
    for deficit in [
        Deficit::InertProducer("accel"),
        Deficit::InertProducer("inv_mass"),
        Deficit::InertProducer("falloff"),
        Deficit::MissingSource("P"),
        Deficit::MissingInput("shape"),
        Deficit::Shadowed("fx.glow"),
    ] {
        let fix = if matches!(deficit, Deficit::InertProducer("accel")) {
            Fix::Reorder
        } else {
            Fix::Offer
        };
        let msg = explain(&Diagnostic {
            node: NodeId(0),
            deficit,
            fix,
        });
        assert_ne!(
            msg, generic,
            "{deficit:?} caiu no catch-all -- o artista recebe uma frase que nao descreve o \
             defeito dele"
        );
        assert!(!msg.is_empty());
    }
    // ⚠️ CONTROLE POSITIVO: o catch-all EXISTE e é alcançável — sem isto o gate acima
    // passaria numa versão em que todo braço fosse explícito e a comparação fosse vazia.
    assert!(
        generic.contains("nothing downstream consumes"),
        "o catch-all tem de ser o que se pensa que ele e': {generic}"
    );
    // E o do passe sombreado NOMEIA o tipo, que é o que o artista procura no grafo.
    let shadowed = explain(&Diagnostic {
        node: NodeId(0),
        deficit: Deficit::Shadowed("fx.glow"),
        fix: Fix::Offer,
    });
    assert!(
        shadowed.contains("fx.glow"),
        "a mensagem tem de nomear o tipo: {shadowed}"
    );
}
