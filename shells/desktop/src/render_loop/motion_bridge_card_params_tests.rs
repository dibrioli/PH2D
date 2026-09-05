//! **OS PARAMS DO CARTÃO** — os gates do ciclo 1 da dinâmica
//! ([doc 103](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)): decisão do Enio de
//! 2026-09-05, *os params dos nós são desenhados nos nós*.
//!
//! ⚠️ Irmão de `motion_bridge_params_visible_tests` por RESPONSABILIDADE: aquele pergunta
//! *«quando é que uma row aparece?»* (as três famílias de gate) e este *«o que é que o CARTÃO
//! recebe?»* — a mesma porta de visibilidade, um segundo consumidor.

use super::*;
use crate::motion_state::MotionState;

/// Um nó solto, cozido até à lista de params que o CARTÃO dele recebe. Devolve também o estado
/// e o id, porque os gates precisam de perguntar à mesma árvore (dois `MotionState` seriam duas
/// árvores, e o gate mediria a coincidência entre elas).
fn one_node(type_name: &str) -> (MotionState, ph2d_nodegraph::graph::NodeId, Vec<ph2d_panel_motion_graph::CardParam>) {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node(type_name.to_string());
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    stamp_card_params(&m, &mut snap);
    let params = snap
        .nodes
        .iter()
        .find(|n| n.id == id.0)
        .map(|n| n.params.clone())
        .unwrap_or_default();
    (m, id, params)
}

/// **O CARTÃO RECEBE OS PARAMS DO NÓ** — e não uma lista vazia. FALSIFICADO por
/// `stamp_card_params` não ser chamada (a cena fica com cartões nus e o painel lateral volta a
/// ser o único sítio onde um param existe, que é o que esta wave apaga).
#[test]
fn a_card_carries_the_params_of_its_node() {
    let (_, _, p) = one_node("motion.grid");
    assert!(
        p.len() >= 4,
        "o Grid tem rows/cols/gap_x/gap_y no cartao, e nao {}",
        p.len()
    );
    assert!(
        p.iter().any(|c| c.hint.param == "rows"),
        "o param `rows` esta' na lista do cartao"
    );
}

/// ⭐⭐ **UMA COR É UMA AMOSTRA, NÃO UM NÚMERO** — e o resto do gate é um **CENSO**, porque a
/// mutação me corrigiu.
///
/// Eu escrevera um filtro que suprimia os canais crus de um hint [`ParamWidget::Color`], e a
/// mutação que o removia **SOBREVIVEU**. O censo do registry diz porquê: das **5** cores
/// declaradas, **0** têm um canal não-âncora com `ParamUiHint` próprio — os canais nunca
/// entram na lista do cartão porque nunca são declarados. O filtro foi apagado; o que sobra é
/// esta contagem, que **fala** no dia em que o mundo passar a conter o caso.
///
/// FALSIFICADO por: a amostra deixar de ser preenchida (a cor passa a ler-se `0.50`), ou por
/// alguém declarar um hint para um canal cru sem repor a supressão.
#[test]
fn a_colour_is_a_swatch_and_no_raw_channel_declares_a_row() {
    let (m, id, p) = one_node("motion.tint");
    let hints = m
        .registry
        .param_ui(m.doc.graph.node(id).expect("no'").type_id())
        .expect("o tint declara hints");
    let visivel = params_visible::shown_params(&m, id);
    // ⚠️ Só as âncoras VISÍVEIS: um modo pode esconder a segunda cor, e exigir a row dela
    // mediria o gate de modo em vez desta lei.
    let anchors: Vec<&str> = hints
        .iter()
        .filter(|h| matches!(h.widget, ph2d_node_registry::ParamWidget::Color { .. }))
        .map(|h| h.param)
        .filter(|a| visivel(a))
        .collect();
    assert!(
        !anchors.is_empty(),
        "o tint mostra pelo menos uma cor no modo de omissao"
    );
    for a in &anchors {
        let row = p
            .iter()
            .find(|c| c.hint.param == *a)
            .unwrap_or_else(|| panic!("a ancora de cor `{a}` tem row no cartao"));
        assert!(
            row.swatch.is_some(),
            "a row da ancora `{a}` leva a AMOSTRA, nao um numero"
        );
    }

    // O CENSO, sobre o registry inteiro — a população que a supressão apagada serviria.
    let mut cores = 0usize;
    let mut canais_com_hint = Vec::new();
    for (_, hints) in todos_os_hints(&m) {
        let declarados: Vec<&str> = hints.iter().map(|h| h.param).collect();
        for h in hints {
            let ph2d_node_registry::ParamWidget::Color { channels } = h.widget else {
                continue;
            };
            cores += 1;
            for c in channels {
                if c != h.param && declarados.contains(&c) {
                    canais_com_hint.push(c);
                }
            }
        }
    }
    assert!(cores >= 5, "o registry declara pelo menos as 5 cores medidas em 2026-09-05, e nao {cores}");
    assert!(
        canais_com_hint.is_empty(),
        "de {cores} cores, {} canais crus passaram a declarar hint proprio ({canais_com_hint:?}) \
         -- o cartao vai mostrar um NUMERO onde pertence uma amostra; repor a supressao",
        canais_com_hint.len()
    );
}

/// Os hints de todo tipo de nó do registry — a população dos censos deste ficheiro.
fn todos_os_hints(
    m: &MotionState,
) -> Vec<(&'static str, &'static [ph2d_node_registry::ParamUiHint])> {
    m.registry
        .manifests()
        .filter_map(|man| m.registry.param_ui(man.id).map(|h| (man.name, h)))
        .collect()
}

/// **O CARTÃO E O PAINEL MOSTRAM A MESMA LISTA** — a porta de visibilidade é uma só. Um param
/// escondido por modo (`ParamGate`) não pode aparecer num sítio do app e não noutro.
/// FALSIFICADO por o cartão deixar de filtrar por `shown_params`.
#[test]
fn the_card_and_the_panel_agree_on_which_params_are_visible() {
    let mut m = MotionState::new();
    // O `motion.noise` tem gates de modo (`range_mode` revela `min`/`max`), que é a fixtura
    // que contém o fenómeno — um nó sem gates concordaria por vazio.
    let id = m.doc.graph.add_node("motion.noise".to_string());
    for modo in [0.0_f32, 1.0] {
        m.doc.graph.set_param(id, "range_mode", modo);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, &mut snap);
        let no_cartao: Vec<&str> = snap
            .nodes
            .iter()
            .find(|n| n.id == id.0)
            .map(|n| n.params.iter().map(|c| c.hint.param).collect())
            .unwrap_or_default();
        let visivel = params_visible::shown_params(&m, id);
        let hints = m
            .registry
            .param_ui(m.doc.graph.node(id).expect("no'").type_id())
            .expect("o noise declara hints");
        for h in hints {
            assert_eq!(
                no_cartao.contains(&h.param),
                visivel(h.param),
                "`{}` com range_mode={modo}: o cartao e o painel discordam",
                h.param
            );
        }
    }
}
