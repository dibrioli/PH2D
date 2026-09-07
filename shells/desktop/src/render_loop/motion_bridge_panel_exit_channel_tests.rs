//! ⭐⭐ **O CANAL ESCOLHE-SE NO CARTÃO** — os gates da wave que fechou a espécie *selector de
//! CANAL* da conta do [`super::panel_exit_probe`].
//!
//! ⚠️ **Irmão por RESPONSABILIDADE, não por contagem de linhas** (HR-18): aquele ficheiro
//! responde *«o que o cartão ainda NÃO alcança»* — um censo sobre o catálogo inteiro —, e este
//! *«o canal, alcança-o assim»*. Um leitor que procure a lei do clique num censo não a acha.

use crate::motion_state::MotionState;

/// ⭐⭐⭐ **O CLIQUE ANDA PELA LISTA DE CANAIS, E ESCREVE AS DUAS METADES.**
///
/// Um canal não é um enum: o enum guarda o ÍNDICE da opção no próprio param, e um canal guarda
/// o **nome da coluna** num param de texto MAIS um **`mode`** noutro. Escrever só a coluna
/// deixaria o nó a ler o sítio certo no modo errado — que é ler **zeros em silêncio**, o defeito
/// que a própria lista de canais do `value.attribute` existe para não ter (o doc dela nomeia-o).
///
/// As três metades: a **primeira** escolha quando nada está escolhido, o **par** (as duas
/// escritas juntas) e a **volta ao princípio** no fim.
///
/// FALSIFICADO por apagar o braço `GraphIntent::CycleChannel` do `apply_graph_intents` (nada
/// sai), ou por ele emitir só o `SetTextParam` (o par quebra).
#[test]
fn the_card_walks_the_channel_list_and_writes_both_halves() {
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    use ph2d_panel_motion_params::MotionParamIntent as I;

    let clique =
        |m: &mut MotionState, id: ph2d_nodegraph::graph::NodeId| -> Option<(String, f64)> {
            let _ = drain_intents();
            let _ = ph2d_panel_motion_params::drain_param_intents();
            push_intent(GraphIntent::CycleChannel {
                node: id.0,
                param: "attr",
            });
            crate::render_loop::motion_bridge::apply_graph_intents(
                m,
                &mut ph2d_core::Playhead::default(),
                &mut ph2d_editor::ToastQueue::default(),
                &mut ph2d_editor::screens::layout::CenterSplit::None,
            );
            let saiu = ph2d_panel_motion_params::drain_param_intents();
            let coluna = saiu.iter().find_map(|i| match i {
                I::SetTextParam { param, value, .. } if *param == "attr" => Some(value.clone()),
                _ => None,
            });
            let modo = saiu.iter().find_map(|i| match i {
                I::SetParam { param, value, .. } if *param == "mode" => Some(*value),
                _ => None,
            });
            // ⚠️ O par, ou nada: um `zip` aqui é o que torna «escreveu meia escolha» inexprimível
            // como verde.
            coluna.zip(modo)
        };

    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("value.attribute");
    let curados = ph2d_node_value_attribute::READ_CHANNELS;
    assert!(curados.len() >= 2, "a lista curada tem por onde andar");

    let (c0, m0) = clique(&mut m, id).expect("sem escolha feita, o clique escolhe a primeira");
    assert_eq!(
        (c0.as_str(), m0),
        (curados[0].column, f64::from(curados[0].mode)),
        "a primeira escolha e' a primeira da lista curada"
    );

    // Andar exige aplicar o que saiu — o «seguinte» é função de onde o nó ESTÁ.
    for k in 0..curados.len() {
        m.doc
            .graph
            .set_text_param(id, "attr", curados[k].column.to_string());
        m.doc.graph.set_param(id, "mode", curados[k].mode as f32);
        let esperado = &curados[(k + 1) % curados.len()];
        let (c, md) = clique(&mut m, id).expect("com lista, o clique escolhe");
        assert_eq!(
            (c.as_str(), md),
            (esperado.column, f64::from(esperado.mode)),
            "de {} a lista tem de avancar para {} (e dar a volta no fim)",
            curados[k].label,
            esperado.label
        );
    }
}

/// ⭐⭐ **O CARTÃO ANDA PELA MESMA ORDEM QUE O PAINEL PINTA.**
///
/// ⚠️ *Uma lei escrita em dois sítios ainda não é uma lei.* O painel desenha o selector dos
/// canais **curados** e, por baixo, as chips das colunas **vivas**; o cartão anda por uma lista.
/// Se as duas ordens divergissem, o mesmo clique escolheria coisas diferentes conforme a
/// superfície — e nada no ecrã diria porquê.
///
/// FALSIFICADO por o `channel_walk` pôr as colunas vivas à frente das curadas.
#[test]
fn the_card_walks_the_same_channel_order_the_panel_paints() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("value.attribute");
    ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
    let painel = crate::render_loop::motion_bridge::params::build_params_snapshot(
        &m,
        ph2d_editor::ProjectSettings::default(),
    )
    .expect("o no' selecionado tem painel");
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    let row = painel
        .rows
        .iter()
        .find_map(|r| match r {
            ph2d_panel_motion_params::ParamRow::Channels(c) => Some(c),
            _ => None,
        })
        .expect("o `value.attribute` tem a row de canais");

    let (lista, _) = crate::render_loop::motion_bridge::params::channel_walk_for_tests(
        &m,
        id,
        row.text_param,
        row.mode_param,
        ph2d_node_value_attribute::READ_CHANNELS,
    );
    let pintada: Vec<(String, i32)> = row
        .channels
        .iter()
        .map(|(_, col, mode)| ((*col).to_string(), *mode))
        .chain(row.extra.iter().map(|c| (c.clone(), 0)))
        .collect();
    assert_eq!(
        lista, pintada,
        "a ordem por onde o cartao anda tem de ser a que o painel desenha"
    );
}

/// ⭐ **O CARTÃO DIZ O NOME QUE O PAINEL DIZ** — `Speed`, não `vel`.
///
/// ⚠️ Sem isto o clique funciona, a tela responde, e a row mostra uma palavra que não aparece em
/// mais lado nenhum do app: a mesma escolha com dois nomes lê-se como duas escolhas.
/// Fora da lista curada o nome É a coluna, e é essa que fica.
///
/// FALSIFICADO por o `card_text` voltar a devolver a coluna crua.
#[test]
fn the_card_names_a_channel_the_way_the_panel_names_it() {
    let texto_de = |coluna: &str, modo: f32| -> String {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node("value.attribute");
        m.doc.graph.set_text_param(id, "attr", coluna.to_string());
        m.doc.graph.set_param(id, "mode", modo);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        snap.nodes
            .iter()
            .find(|v| v.id == id.0)
            .and_then(|v| v.params.iter().find(|p| p.hint.param == "attr"))
            .map(|p| p.text.as_str().to_string())
            .expect("`value.attribute::attr` esta' no cartao")
    };
    let primeiro = &ph2d_node_value_attribute::READ_CHANNELS[0];
    assert_eq!(
        texto_de(primeiro.column, primeiro.mode as f32),
        primeiro.label,
        "um canal curado le^-se pelo ROTULO"
    );
    assert_eq!(
        texto_de("uma_coluna_minha", 0.0),
        "uma_coluna_minha",
        "fora da lista curada o nome E' a coluna"
    );
}
