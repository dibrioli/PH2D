//! Gates da cena `=77` (doc 89, folha 07).
//!
//! ⚠️ **A leitura da cena é uma SOMA de cores na tela, e nenhum gate headless a vê.** O que
//! se pode provar aqui é o que a torna capaz de a mostrar: que as metades autoram operadores
//! DIFERENTES, que o caminho de facto se CRUZA (sem cruzamento não há o que somar, e as duas
//! metades sairiam iguais com o produto correcto), e que a coluna chega ao stream.

use super::{ADD, build_operator_demo_document};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// **AS DUAS METADES DE CADA LINHA AUTORAM OPERADORES DIFERENTES.**
#[test]
fn each_row_authors_two_different_operators() {
    let mut state = MotionState::new();
    let _ = build_operator_demo_document(&mut state.doc, &state.registry).expect("a cena monta");
    for (ty, param) in [
        ("motion.trail", ph2d_node_motion_trail::ECHO_BLEND),
        ("motion.strobe", ph2d_node_motion_strobe::FLASH_BLEND),
    ] {
        let vals: Vec<f32> = state
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == ty)
            .map(|n| {
                state
                    .doc
                    .graph
                    .node_param_overrides(n.id)
                    .and_then(|m| m.get(param).copied())
                    .unwrap_or(0.0)
            })
            .collect();
        assert_eq!(vals.len(), 2, "{ty}: duas bandas");
        assert!(vals.contains(&0.0), "{ty}: uma delas tem de ser o `Sink`");
        assert!(vals.contains(&ADD), "{ty}: e a outra `Add`");
    }
}

/// **A COLUNA CHEGA AO STREAM, E SÓ NA METADE QUE A PEDIU.**
///
/// ⚠️ O oráculo é o STREAM COZIDO, não o param: um param que o `eval` lesse e não usasse
/// deixaria este gate verde se ele olhasse a tabela de params (foi o modo de falha de três
/// mutações desta linha).
#[test]
fn only_the_chosen_half_carries_the_blend_column() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    // sinks: [rastro esquerdo, rastro direito, flash esquerdo, flash direito]
    let carries = |state: &mut MotionState, sink: ph2d_nodegraph::graph::NodeId| -> Option<f32> {
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sink, 0.5)
            .expect("coze");
        match out[0].as_stream().get(ph2d_node_motion_trail::BLEND_COLUMN) {
            Some(Column::Scalar(v)) if !v.is_empty() => Some(v[0]),
            _ => None,
        }
    };
    for pair in [[sinks[0], sinks[1]], [sinks[2], sinks[3]]] {
        assert_eq!(carries(&mut state, pair[0]), None, "a esquerda nao escolhe");
        assert_eq!(carries(&mut state, pair[1]), Some(ADD), "a direita escolhe");
    }
}

/// **O CAMINHO CRUZA-SE A SI PRÓPRIO** — sem isto a cena não tem o que somar.
///
/// ⚠️ **É a metade que separa esta cena de uma que passaria com o produto CERTO e não
/// mostraria nada.** Uma órbita circular nunca se atravessa: os ecos ficariam lado a lado,
/// `Add` e `Normal` desenhariam o mesmo, e o smoke reprovaria a feature por causa da cena.
/// O oito é medido pelo X voltar ao mesmo sítio com o Y noutro.
#[test]
fn the_path_crosses_itself_so_the_echoes_have_something_to_add() {
    let mut state = MotionState::new();
    let sinks = build_operator_demo_document(&mut state.doc, &state.registry).expect("monta");
    let at = |state: &mut MotionState, t: f64| -> [f32; 2] {
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, sinks[0], t)
            .expect("coze");
        let Some(Column::Vec2(p)) = out[0].as_stream().get("P") else {
            panic!("P");
        };
        p[0]
    };
    // Varre um período inteiro e procura DOIS instantes com o mesmo x e y diferente — a
    // assinatura de um oito, e a de nada mais que este caminho pudesse ser.
    const STEPS: usize = 240;
    let period = 1.0 / f64::from(super::LOOP_HZ_FOR_TEST);
    let pts: Vec<[f32; 2]> = (0..STEPS)
        .map(|k| at(&mut state, period * k as f64 / STEPS as f64))
        .collect();
    let crossed = pts.iter().enumerate().any(|(i, a)| {
        pts.iter()
            .skip(i + STEPS / 8)
            .any(|b| (a[0] - b[0]).abs() < 0.02 && (a[1] - b[1]).abs() > 0.15)
    });
    assert!(
        crossed,
        "o caminho tem de se cruzar — senao `Add` e `Normal` desenham o mesmo"
    );
}
