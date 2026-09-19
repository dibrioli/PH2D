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
    // ⚠️ **A lista vem do CRATE, não daqui** (`Deficit::ALL`, ao lado da definição). Uma cópia
    // local tinha o mesmo buraco um nível acima: quem acrescenta um variante está no `enum`, não
    // neste arquivo, e uma lista que é preciso LEMBRAR de estender não é um censo.
    for &deficit in Deficit::ALL {
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
    // E a da ramificação morta NOMEIA a porta, pela mesma razão: é o que ele tem de ligar.
    let dead = explain(&Diagnostic {
        node: NodeId(0),
        deficit: Deficit::DeadBranch("in1"),
        fix: Fix::Offer,
    });
    assert!(
        dead.contains("in1"),
        "a mensagem tem de nomear a porta: {dead}"
    );
}

/// ⭐⭐⭐ **A FRASE DA ORDEM DO DONO NOMEIA AS TRÊS COISAS** — o que se vê agora, o que falta, e
/// o que o que falta precisa.
///
/// ⚠️ **O irmão acima só exige que ela DIFIRA do catch-all**, e uma frase diferente e inútil
/// passa nele. A ordem era literal (*«coloque um alerta de que se não forem usados com
/// duplicator e um objeto a ser copiado, são invisíveis»*), e sem a 1.ª parte o artista lê
/// *«não funciona»*; sem a 3.ª ele põe o duplicador e continua sem ver nada.
#[test]
fn a_frase_das_posicoes_diz_o_que_se_ve_e_o_que_falta() {
    let msg = explain(&Diagnostic {
        node: NodeId(0),
        deficit: Deficit::SemQuemVista,
        fix: Fix::Offer,
    })
    .to_lowercase();
    for parte in ["positions", "dots", "duplicator", "shape"] {
        assert!(
            msg.contains(parte),
            "a frase tem de nomear {parte:?}, e diz: {msg:?}"
        );
    }
}
