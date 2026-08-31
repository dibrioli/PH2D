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
    // ⛔⛔ **A CAIXA TEM DE TER TEXTO, e a 1.ª redacção deixava-a VAZIA** (doc 96 §4.1). Com
    // texto vazio a porta sai em `queixas.first()?` → `None` **independentemente das guardas**,
    // e as três remoções (só o tipo · só o param · ambas) ficavam **verdes**. *Um controlo de
    // filtro sobre a população certa não garante que a população carregue o fenómeno.*
    //
    // ⚠️ O fenómeno é real e medido: `grammar_complaints("sin(t)*2")` devolve **1** queixa
    // (`NoArrow`) — uma fórmula qualquer lê-se como uma gramática partida. E os **axiomas dos
    // oito moldes** são acusados 8/8, o que torna a metade `param != RULES_PARAM` load-bearing:
    // sem ela, uma linha vermelha FALSA aparece debaixo da caixa *Axiom* em todo estado normal.
    for t in ["sin(t)*2", "A(step)", "x", "0"] {
        motion.doc.graph.set_text_param(e, "expr", t);
        assert_eq!(
            ph2d_node_source_lsystem::grammar_complaints(t).len(),
            1,
            "a fixtura tem de conter o FENÓMENO: `{t}` tem de ser acusado por quem lê gramáticas"
        );
        ph2d_panel_motion_graph::set_graph_selection(vec![e.0]);
        let snap = build_params_snapshot(&motion, ProjectSettings::default())
            .expect("o motion.expression tem snapshot");
        for row in &snap.rows {
            if let ParamRow::Text(tr) = row {
                assert_eq!(
                    tr.problem, None,
                    "`{}` de outro nó herdou a queixa do L-System com o texto `{t}`",
                    tr.name
                );
            }
        }
    }
    // ⭐⭐ **A CAIXA `axiom` DO PRÓPRIO L-SYSTEM** — a metade que faltava, e a que mata a
    // mutação do NOME DO PARAM. Os axiomas dos oito moldes lêem-se como gramáticas partidas
    // (`8/8` acusados por `grammar_complaints`), então sem a guarda `param != RULES_PARAM` uma
    // linha vermelha **FALSA** aparecia debaixo do *Axiom* em todo estado normal — e o
    // `apply_lsystem_preset` e o `bake_lsystem_grammar` escrevem sempre um axioma.
    let n = lsystem_with(&mut motion, ls::PRESETS[0].rules);
    motion
        .doc
        .graph
        .set_text_param(n, ls::AXIOM_PARAM, ls::PRESETS[0].axiom);
    assert_eq!(
        ph2d_node_source_lsystem::grammar_complaints(ls::PRESETS[0].axiom).len(),
        1,
        "a fixtura tem de conter o fenómeno: um AXIOMA lê-se como gramática partida"
    );
    ph2d_panel_motion_graph::set_graph_selection(vec![n.0]);
    let snap = build_params_snapshot(&motion, ProjectSettings::default()).expect("snapshot");
    for row in &snap.rows {
        if let ParamRow::Text(t) = row
            && t.name == ls::AXIOM_PARAM
        {
            assert_eq!(
                t.problem, None,
                "a caixa `Axiom` do L-System ganhou uma queixa — ela não é uma lista de regras"
            );
        }
    }

    // ⚠️ **E a guarda do TIPO DE NÓ é DEFENSIVA, não falsificável hoje** — e isso diz-se em vez
    // de se fingir: **nenhum outro nó da casa tem um param chamado `rules`** (conferido por
    // varredura), então removê-la não muda nada enquanto isso for verdade. O que este laço faz é
    // tornar a afirmação tão forte quanto a população permite — e não-vazia no dia em que outro
    // nó ganhar aquele nome.
    let tipos: Vec<&'static str> = motion
        .registry
        .manifests()
        .map(|m| m.name)
        .filter(|n| *n != ls::MANIFEST.name)
        .collect();
    let mut com_texto = 0usize;
    for ty in tipos {
        let id = motion.doc.graph.add_node(ty);
        motion
            .doc
            .graph
            .set_text_param(id, ls::RULES_PARAM, "A(s) -> (40%) F(s)");
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let Some(snap) = build_params_snapshot(&motion, ProjectSettings::default()) else {
            continue;
        };
        for row in &snap.rows {
            if let ParamRow::Text(t) = row {
                com_texto += 1;
                assert_eq!(
                    t.problem, None,
                    "`{ty}::{}` herdou a queixa do L-System",
                    t.name
                );
            }
        }
    }
    assert!(
        com_texto > 0,
        "nenhum outro nó da casa tem row de texto — o laço acima não mediu nada"
    );
}
