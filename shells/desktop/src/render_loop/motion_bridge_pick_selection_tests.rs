//! ⭐⭐ **O BOTÃO «USE SELECTED PATH» CHEGA AO DOCUMENTO** (ordem do dono, 2026-09-08) — pelo
//! funil REAL (`apply_graph_intents`), nunca chamando a função por dentro.
//!
//! ⚠️ **É a metade que nenhum gate de registo apanha.** O `click_does` já não pode esquecer a
//! variante (o `_ => Nothing` morreu, e esquecê-la é erro de compilação), e o censo do painel já
//! lê a row como alcançável — mas *«o clique chega ao barramento»* e *«a escrita chega ao
//! documento»* são duas perguntas, e é a segunda que este ficheiro mede. É a lei que os quatro
//! bugs de fiação do Vector cobraram: três rotas painel→barramento mortas **com o gate de registo
//! VERDE**.

use super::apply_graph_intents;
use crate::motion::motion_state::{FormaEscolhida, MotionState};
use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

/// Empurra o clique do botão pelo funil real.
fn clicar(m: &mut MotionState, node: u32) -> ph2d_editor::ToastQueue {
    let _ = drain_intents();
    push_intent(GraphIntent::PickSelection {
        node,
        param: "path",
    });
    let mut toasts = ph2d_editor::ToastQueue::default();
    apply_graph_intents(
        m,
        &mut ph2d_core::Playhead::default(),
        &mut toasts,
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    toasts
}

/// **Com uma forma seleccionada, o botão escreve o nome dela no nó.** E sem nenhuma, ele **não
/// inventa** — deixa o param como estava.
///
/// ⚠️ **As duas metades num gate só, de propósito:** um botão que escreve e um botão que escreve
/// *qualquer coisa* passam a primeira asserção do mesmo jeito. A segunda é o que separa *«liga o
/// que está seleccionado»* de *«liga alguma coisa»*.
///
/// FALSIFICADO por apagar o braço `GraphIntent::PickSelection` do `apply_graph_intents` — o
/// clique passa a ser engolido e o param fica vazio nas duas metades.
#[test]
fn the_button_writes_the_selected_shape_and_invents_nothing_without_one() {
    let mut m = MotionState::new();
    let sw = m.doc.graph.add_node("motion.spline_wrap".to_string());

    // Nada seleccionado: o botão fala (o toast) e o documento fica intacto.
    m.selected_shape = FormaEscolhida::Nada;
    let _ = clicar(&mut m, sw.0);
    assert!(
        m.doc
            .graph
            .node_text_param_overrides(sw)
            .and_then(|t| t.get("path"))
            .is_none(),
        "sem nada seleccionado o botao nao pode INVENTAR um caminho"
    );

    // Com uma forma seleccionada: o nome dela chega ao documento.
    m.selected_shape = FormaEscolhida::Nome("Curva do Enio".to_string());
    let _ = clicar(&mut m, sw.0);
    assert_eq!(
        m.doc
            .graph
            .node_text_param_overrides(sw)
            .and_then(|t| t.get("path"))
            .map(String::as_str),
        Some("Curva do Enio"),
        "o botao tem de escrever o nome da forma seleccionada no param do no'"
    );
}

/// ⭐ **CADA RECUSA DIZ A SUA RAZÃO** — quatro estados, quatro frases, todas diferentes.
///
/// ⛔ **O defeito que este gate fecha:** a 1.ª redacção dizia sempre *«escolha um desenho: ele
/// precisa de um NOME e de pelo menos dois pontos»* — e **nenhum desenho deste app pode falhar a
/// condição do nome** (todos nascem `Path {id}`, `vec_entities::initial_name`). A frase descrevia
/// uma população vazia e calava a condição que de facto mordia. É a família do
/// [[feedback_the_note_beside_a_count_describes_the_population_it_had_when_it_was_written]].
///
/// ⚠️ **A régua é a frase ser DISTINTA, não o texto exacto** — um gate que fixasse a redacção
/// obrigaria toda melhoria de linguagem a editar um teste, e a i18n muda-a por desenho.
///
/// FALSIFICADO por colapsar dois braços do `match` na mesma frase: o conjunto encolhe e o
/// `assert_eq` de contagem reprova, nomeando o par.
#[test]
fn each_refusal_of_the_button_names_its_own_reason() {
    let recusas = [
        FormaEscolhida::Nada,
        FormaEscolhida::NaoEDesenho,
        FormaEscolhida::SemNome,
        FormaEscolhida::SemArco,
    ];
    let mut ditas: Vec<String> = Vec::new();
    for r in recusas.clone() {
        let mut m = MotionState::new();
        let sw = m.doc.graph.add_node("motion.spline_wrap".to_string());
        m.selected_shape = r.clone();
        let toasts = clicar(&mut m, sw.0);
        let frases: Vec<String> = toasts.iter().map(|t| t.message.clone()).collect();
        assert_eq!(
            frases.len(),
            1,
            "{r:?} tem de dizer exactamente UMA coisa — disse {frases:?}"
        );
        assert!(
            m.doc
                .graph
                .node_text_param_overrides(sw)
                .and_then(|t| t.get("path"))
                .is_none(),
            "{r:?} e' uma recusa: o documento nao pode mudar"
        );
        ditas.push(frases[0].clone());
    }
    let unicas: std::collections::BTreeSet<&String> = ditas.iter().collect();
    assert_eq!(
        unicas.len(),
        recusas.len(),
        "duas recusas com a MESMA frase — o artista nao consegue distinguir: {ditas:?}"
    );
}
