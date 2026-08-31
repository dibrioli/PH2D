//! ⭐⭐⭐ **A REGRA MALFORMADA CHEGA AO PAINEL** — a costura, e não só a lei.
//!
//! # Por que este ficheiro existe
//!
//! Item aberto desde 2026-08-29: *o feedback ao vivo de uma regra malformada — hoje ela cai em
//! silêncio*. A metade do NÓ (a queixa nascer no mesmo `return Err` que descarta a regra) tem
//! gate próprio na crate
//! ([`a_dropped_rule_says_why`](../../../../crates/ph2d-node-source-lsystem/tests/a_dropped_rule_says_why.rs)).
//!
//! ⚠️⚠️ **Esta é a OUTRA metade, e ela é independente:** uma lei perfeita que nenhuma row
//! carrega é exactamente um controlo morto ao contrário — o app *sabe* o que está mal e o
//! artista não vê. As duas metades falham em sítios diferentes, então gateiam-se em sítios
//! diferentes; e a régua desta é a **`ParamsSnapshot` real**, construída pela porta do produto
//! (`build_params_snapshot`), nunca uma `TextRow` montada à mão.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_node_source_lsystem as ls;
use ph2d_panel_motion_params::ParamRow;

/// A queixa que a row de `Rules` daquele nó carrega, pela porta do produto.
fn problem_of(motion: &MotionState, nid: ph2d_nodegraph::graph::NodeId) -> Option<String> {
    ph2d_panel_motion_graph::set_graph_selection(vec![nid.0]);
    build_params_snapshot(motion, ProjectSettings::default())?
        .rows
        .into_iter()
        .find_map(|r| match r {
            ParamRow::Text(t) if t.name == ls::RULES_PARAM => t.problem,
            _ => None,
        })
}

/// Põe o nó em `Grammar` (onde a caixa de regras é visível) e escreve `rules`.
fn lsystem_with(motion: &mut MotionState, rules: &str) -> ph2d_nodegraph::graph::NodeId {
    let n = motion.doc.graph.add_node("source.lsystem");
    // ⚠️ **O `Mode` importa**: a caixa de `Rules` é gateada por ele, e em `Guided` a row nem
    // existe. Um teste que não o pusesse mediria a ausência da row e leria-a como «sem queixa».
    motion
        .doc
        .graph
        .set_param(n, ls::param::MODE, ls::MODE_GRAMMAR as f32);
    motion.doc.graph.set_text_param(n, ls::RULES_PARAM, rules);
    n
}

#[test]
fn a_rule_the_parser_throws_away_is_named_in_the_panel() {
    let mut motion = MotionState::new();
    let n = lsystem_with(&mut motion, "A(s) -> (40%) F(s)");
    let msg = problem_of(&motion, n).expect("a row de Rules tem de trazer a queixa");
    // ⚠️ **A regra é CITADA** — numa gramática de várias regras separadas por `;`, dizer só
    // *"há um erro"* manda o artista procurar. É o mesmo que separa um aviso útil de um alarme.
    assert!(
        msg.contains("(40%)"),
        "a queixa tem de citar a regra que o artista escreveu: {msg}"
    );
    assert!(
        msg.contains(ls::RuleProblem::BadWeight.say()),
        "e dizer a cura: {msg}"
    );
}

#[test]
fn a_grammar_that_is_right_shows_no_warning_in_the_panel() {
    // ⚠️ **Os OITO moldes + a de fábrica + o vazio.** Um aviso que aparece sobre produto
    // correcto ensina a ignorar avisos, e depois disso ele não existe — por isso a população
    // deste gate é o corpus inteiro, não um exemplo.
    let mut motion = MotionState::new();
    for p in ls::PRESETS {
        let n = lsystem_with(&mut motion, p.rules);
        assert_eq!(
            problem_of(&motion, n),
            None,
            "o molde `{}` faz o painel queixar-se de produto correcto",
            p.label
        );
    }
    let n = lsystem_with(&mut motion, ls::DEFAULT_RULES);
    assert_eq!(problem_of(&motion, n), None, "a gramática de fábrica");
    // ⚠️ Vazio quer dizer *«usa a de fábrica»*, e não *«escreveste nada»*.
    let n = lsystem_with(&mut motion, "");
    assert_eq!(problem_of(&motion, n), None, "a caixa vazia");
}

#[test]
fn only_the_lsystem_rules_box_can_carry_a_complaint() {
    // ⛔ A porta é dirigida por (tipo de nó, param), e não por «qualquer texto que pareça uma
    // gramática». Uma fórmula de `motion.expression` com o mesmo texto não pode herdar a
    // acusação — ela não é uma gramática, e o parser dela é outro.
    let mut motion = MotionState::new();
    let e = motion.doc.graph.add_node("motion.expression");
    ph2d_panel_motion_graph::set_graph_selection(vec![e.0]);
    let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) else {
        panic!("o motion.expression tem snapshot");
    };
    let textos: Vec<_> = snap
        .rows
        .iter()
        .filter_map(|r| match r {
            ParamRow::Text(t) => Some(t),
            _ => None,
        })
        .collect();
    assert!(
        !textos.is_empty(),
        "o motion.expression tem de ter uma row de texto — senão este gate não mede nada"
    );
    for t in textos {
        assert_eq!(
            t.problem, None,
            "`{}` de outro nó não pode herdar a queixa do L-System",
            t.name
        );
    }
}
