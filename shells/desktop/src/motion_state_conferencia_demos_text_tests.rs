//! Gates da cena `=39` — **o TEXTO**.
//!
//! ⚠️ Um A/B tem dois modos de falhar em silêncio: os dois lados iguais (o canal
//! não chegou) **e** os dois lados fanados (o "controle" não é controle). Os
//! gates afirmam os dois.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O stream que um sink deixa.
///
/// ⚠️ **Coze pelo `pump.cook` do próprio estado, e isso é load-bearing.** A
/// geometria dos glifos vem do shell por CANAL EXTERNO, e o `publish` o escreve
/// naquele cook — um `Cook::new()` local não tem externo nenhum e devolve **zero
/// instâncias**, que é a assinatura exata da feature quebrada. Foi o que a 1ª
/// versão deste harness fez, e ele acusou o produto de um defeito da fixture.
fn cook(state: &mut MotionState, reg: &NodeRegistry, sink: NodeId) -> Stream {
    crate::render_loop::motion_text_gen::publish(state, 0.0);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, reg, sink, 0.0)
        .expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    s.clone()
}

/// **A cena constrói os DOIS lados.** Se um `?` engolisse uma aresta o roteador
/// devolveria `unwrap_or_default()` — uma tela VAZIA, que num smoke lê como *"a
/// feature não foi construída"* em vez de *"a cena está partida"*.
#[test]
fn the_text_scene_builds_both_words() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_text_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), 2, "duas palavras: reta e em leque");
}

/// **A palavra é uma LETRA POR INSTÂNCIA, e só a de baixo abre em leque.**
///
/// ⚠️ As duas metades são precisas: sem a contagem, um bloco emitido como UMA
/// instância passaria (ele também gira); sem o CONTROLE, uma cena cujas duas
/// palavras girassem passaria, e o A/B que o olho julga teria deixado de existir.
#[test]
fn only_the_lower_word_fans_and_it_fans_letter_by_letter() {
    let reg = registry();
    let mut state = MotionState::new();
    let sinks = build_text_demo_document(&mut state.doc, &reg).expect("a cena constroi");

    let plain = cook(&mut state, &reg, sinks[0]);
    let fanned = cook(&mut state, &reg, sinks[1]);

    let n = WORD.chars().count();
    assert_eq!(plain.count(), n, "uma instancia por LETRA");
    assert_eq!(fanned.count(), n);

    // O CONTROLE, escrito como AUSÊNCIA: sem o canal ninguém escreve `rot`, e
    // `all()` sobre um vetor vazio seria verdade por VÁCUO.
    assert!(
        !matches!(Stream::get(&plain, "rot"), Some(Column::Scalar(v)) if !v.is_empty()),
        "a palavra de cima NAO gira"
    );

    let Some(Column::Scalar(rot)) = Stream::get(&fanned, "rot") else {
        panic!("o lado em leque escreve `rot`")
    };
    assert_eq!(rot.len(), n);
    // O leque é PROGRESSIVO: cada letra roda mais que a anterior, e a última
    // chega ao ângulo autorado. Um bloco rodado rigidamente daria N valores
    // IGUAIS — a distinção inteira da wave numa asserção.
    for w in rot.windows(2) {
        assert!(w[1] > w[0], "cada letra roda mais que a anterior: {rot:?}");
    }
    let span = rot[n - 1] - rot[0];
    assert!(
        (span - FAN_DEG).abs() < 0.5,
        "o leque abre o angulo autorado: {span} contra {FAN_DEG}"
    );
    eprintln!(
        "[text] {n} letras, leque {:.1}..{:.1} graus",
        rot[0],
        rot[n - 1]
    );
}
