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
            push_intent(GraphIntent::StepChoice {
                node: id.0,
                param: "attr",
                delta: 1,
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

/// ⭐⭐⭐ **A LINHA QUE A LISTA MOSTRA É A LINHA QUE A ESCOLHA ESCREVE** (report do Enio,
/// 2026-09-07: *«se clicar no centro (nome) abre-se um dropdown»*).
///
/// ⚠️ **Um dropdown tem DUAS metades que podem discordar em silêncio:** a lista desenhada e a
/// resolução do índice. Se elas saíssem de duas derivações, o artista clicaria em `Speed` e o nó
/// passaria a ler `Size` — e nada na tela diria porquê, porque a linha certa ficou realçada.
/// Aqui as duas são amarradas contra a MESMA `channel_walk`, opção a opção.
///
/// FALSIFICADO por o `pick_choice` resolver o índice contra outra lista (por exemplo só os
/// canais curados, ignorando as colunas vivas).
#[test]
fn the_line_the_list_shows_is_the_line_the_pick_writes() {
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    use ph2d_panel_motion_params::MotionParamIntent as I;

    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("value.attribute");
    let (lista, _) = crate::render_loop::motion_bridge::params::channel_walk_for_tests(
        &m,
        id,
        "attr",
        "mode",
        ph2d_node_value_attribute::READ_CHANNELS,
    );
    assert!(lista.len() >= 2, "ha' lista por onde escolher");

    for (k, esperado) in lista.iter().enumerate() {
        let _ = drain_intents();
        let _ = ph2d_panel_motion_params::drain_param_intents();
        push_intent(GraphIntent::PickChoice {
            node: id.0,
            param: "attr",
            index: u16::try_from(k).expect("a lista cabe num u16"),
        });
        crate::render_loop::motion_bridge::apply_graph_intents(
            &mut m,
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
        assert_eq!(
            (coluna.as_deref(), modo),
            (Some(esperado.0.as_str()), Some(f64::from(esperado.1))),
            "a linha {k} da lista tem de escrever a opcao {k}"
        );
    }
}

/// ⭐⭐ **UM ÍNDICE FORA DA LISTA NÃO ESCREVE NADA.**
///
/// ⚠️ A lista é **viva** — as colunas que a corrente de cima cozinhou entram e saem —, então
/// entre desenhar o popup e clicar nele ela pode ter encolhido. Escrever «a última» seria pôr o
/// nó a ler uma coisa que o artista não apontou.
#[test]
fn an_index_past_the_end_of_a_live_list_writes_nothing() {
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("value.attribute");
    let _ = drain_intents();
    let _ = ph2d_panel_motion_params::drain_param_intents();
    push_intent(GraphIntent::PickChoice {
        node: id.0,
        param: "attr",
        index: u16::MAX,
    });
    crate::render_loop::motion_bridge::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    let saiu = ph2d_panel_motion_params::drain_param_intents();
    assert!(saiu.is_empty(), "um indice fora da lista e' mudo: {saiu:?}");
}

/// ⭐⭐ **OS RÓTULOS QUE A LISTA MOSTRA SÃO OS NOMES DO PAINEL, E A MARCA ESTÁ NA OPÇÃO CERTA.**
///
/// ⚠️ **Um `current` fora do fim é a resposta CERTA** (nenhuma das opções: uma coluna escrita à
/// mão), e é o que faz a lista abrir sem nada marcado. Inventar uma marca diria que o nó está
/// numa opção em que ele não está.
#[test]
fn the_list_shows_the_panels_names_and_marks_where_the_node_is() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("value.attribute");
    let curados = ph2d_node_value_attribute::READ_CHANNELS;

    let (rotulos, atual) = crate::render_loop::motion_bridge::params::channel_labels_for_tests(
        &m, id, "attr", "mode", curados,
    );
    assert!(
        atual as usize >= rotulos.len(),
        "sem coluna escolhida, nenhuma opcao esta' marcada (atual {atual} de {})",
        rotulos.len()
    );
    assert_eq!(
        rotulos.first().map(String::as_str),
        Some(curados[0].label),
        "o primeiro rotulo e' o NOME do primeiro canal curado"
    );

    m.doc
        .graph
        .set_text_param(id, "attr", curados[1].column.to_string());
    m.doc.graph.set_param(id, "mode", curados[1].mode as f32);
    let (_, atual) = crate::render_loop::motion_bridge::params::channel_labels_for_tests(
        &m, id, "attr", "mode", curados,
    );
    assert_eq!(atual, 1, "com o 2.o canal escolhido, a marca esta' nele");
}

/// ⭐⭐⭐ **A LISTA INCLUI O QUE A CORRENTE DE CIMA COZINHOU** — e é isso que a torna VIVA.
///
/// ⚠️⚠️ **Sem esta fixtura os outros gates desta wave passavam sobre metade da lei:** um
/// `value.attribute` **desligado** não tem colunas a montante, logo a lista viva coincide com a
/// lista curada, e uma resolução que ignorasse as vivas ficaria verde. *Uma fixtura em que as
/// duas metades coincidem não testa nenhuma das duas.*
///
/// Aqui uma tabela real entra na porta 0, e as colunas dela — que nenhum canal curado cobre —
/// têm de aparecer no fim da lista, escolhíveis pelo índice.
///
/// FALSIFICADO por o `channel_walk` deixar de concatenar as colunas vivas, ou por o
/// `pick_choice` resolver o índice só contra os canais curados.
#[test]
fn the_live_columns_the_stream_cooked_are_in_the_list_and_pickable() {
    use ph2d_nodegraph::graph::Edge;
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    use ph2d_panel_motion_params::MotionParamIntent as I;

    // Um CSV próprio deste gate (nome único: o vizinho partilhado já custou uma reprovação).
    let csv = std::env::temp_dir().join(format!(
        "ph2d_canal_vivo_{}_{}.csv",
        std::process::id(),
        line!()
    ));
    std::fs::write(&csv, "mes,nivel\njan,0.2\nfev,0.8\n").expect("o CSV de teste escreve-se");

    let mut m = MotionState::new();
    let src = m.doc.graph.add_node("source.table");
    m.doc
        .graph
        .set_text_param(src, "file", csv.to_string_lossy().into_owned());
    let attr = m.doc.graph.add_node("value.attribute");
    m.doc
        .graph
        .connect(Edge {
            from: (src, 0),
            to: (attr, 0),
            delayed: false,
        })
        .expect("a tabela liga-se ao atributo");
    // ⚠️ A tabela chega ao cook por um canal EXTERNO que só a shell escreve — sem este passo o
    // stream vem vazio, que é a assinatura exacta da feature partida.
    crate::render_loop::motion_table_gen::publish(&mut m);
    let reg = std::mem::take(&mut m.registry);
    let cozeu = m.pump.cook.cook(&m.doc.graph, &reg, attr, 0.0).is_ok();
    m.registry = reg;
    assert!(cozeu, "a fixtura tem de cozer, senao nao ha' colunas vivas");

    let (lista, _) = crate::render_loop::motion_bridge::params::channel_walk_for_tests(
        &m,
        attr,
        "attr",
        "mode",
        ph2d_node_value_attribute::READ_CHANNELS,
    );
    let curados = ph2d_node_value_attribute::READ_CHANNELS.len();
    assert!(
        lista.len() > curados,
        "a lista tem de crescer com o que a corrente cozinhou: {lista:?}"
    );
    let vivas: Vec<&str> = lista[curados..].iter().map(|(c, _)| c.as_str()).collect();
    assert!(
        vivas.contains(&"nivel"),
        "a coluna `nivel` da tabela tem de ser escolhivel: {vivas:?}"
    );

    // E o índice dela escreve-a — a metade que o resto do gate não alcança.
    let k = lista
        .iter()
        .position(|(c, _)| c == "nivel")
        .expect("acabou de ser encontrada");
    let _ = drain_intents();
    let _ = ph2d_panel_motion_params::drain_param_intents();
    push_intent(GraphIntent::PickChoice {
        node: attr.0,
        param: "attr",
        index: u16::try_from(k).expect("cabe"),
    });
    crate::render_loop::motion_bridge::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor::ToastQueue::default(),
        &mut ph2d_editor::screens::layout::CenterSplit::None,
    );
    let saiu = ph2d_panel_motion_params::drain_param_intents();
    assert!(
        saiu.iter().any(|i| matches!(
            i,
            I::SetTextParam { param, value, .. } if *param == "attr" && value == "nivel"
        )),
        "escolher a linha da coluna viva tem de a escrever: {saiu:?}"
    );
    let _ = std::fs::remove_file(&csv);
}

/// ⭐⭐⭐ **COM O PAINEL FORA, O CARTÃO CONTINUA A ESCREVER** — a metade load-bearing da saída.
///
/// ⛔⛔ **O `publish` do painel é quem DRENA as intenções de param**, e desde o ciclo 1 a maioria
/// delas vem do CARTÃO (o `apply_graph_intents` traduz-as). Saltá-lo com o painel desligado —
/// que é a optimização óbvia, já que ninguém lê mais o snapshot — pararia **o cartão inteiro**:
/// todo arrasto, toda caixa, todo selector ficariam mudos, e nada no ecrã diria porquê.
///
/// As duas metades: o snapshot **não** se publica (não há quem o leia) e a edição **passa**.
///
/// FALSIFICADO por o `publish` sair antes do `apply_param_edits` quando o painel está fora.
#[test]
fn with_the_side_panel_out_the_card_still_writes() {
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.oscillator");
    let antes = crate::render_loop::motion_bridge::params::param_value(&m, id, "frequency");
    let _ = ph2d_panel_motion_params::drain_param_intents();
    ph2d_panel_motion_params::push_param_intent(
        ph2d_panel_motion_params::MotionParamIntent::SetParam {
            node: id.0,
            param: "frequency",
            value: f64::from(antes) + 3.0,
        },
    );
    let mut store = ph2d_editor::interaction::WidgetStore::default();
    crate::render_loop::motion_bridge::params::publish_for_tests(
        &mut m,
        &mut store,
        true,
        ph2d_editor::ProjectSettings::default(),
        &mut ph2d_editor::ToastQueue::default(),
    );
    assert!(
        ph2d_panel_motion_params::current_params().is_none(),
        "com o painel fora, o snapshot nao se constroi para ninguem"
    );
    let depois = crate::render_loop::motion_bridge::params::param_value(&m, id, "frequency");
    assert!(
        (depois - antes - 3.0).abs() < 1e-4,
        "a edicao do cartao tem de passar mesmo com o painel fora: {antes} -> {depois}"
    );
}
