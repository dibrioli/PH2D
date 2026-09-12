//! ⭐⭐⭐ **O QUE O PAINEL LATERAL FAZ QUE O CARTÃO AINDA NÃO FAZ** — a medição que tem de vir
//! ANTES de o painel sair ([doc 103 §4](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md):
//! *«o painel lateral de params SAI»*, ordem do Enio de 2026-09-05).
//!
//! ⛔⛔ **Apagar uma superfície e levar junto um controlo sem substituto é a forma exacta do
//! knob INALCANÇÁVEL** (`CLAUDE.md §5.0`), e é o defeito que nenhuma sonda de registo apanha:
//! o controlo continua declarado, continua pintado, e simplesmente não abre. O gate do ciclo 1
//! (`no_param_the_panel_offers_falls_off_the_card`) responde *«o cartão MOSTRA todos?»*; esta
//! sonda responde a outra pergunta, que é a que decide a wave: ***o cartão ALCANÇA todos?***
//!
//! ⚠️ **A régua não é uma lista de espécies escrita aqui** — é a porta única do próprio gesto,
//! [`ph2d_panel_motion_graph::click_does`]. Uma espécie nova entra no produto e entra na conta
//! no mesmo dia; uma lista à mão deixaria a espécie nova invisível, que é o defeito que esta
//! sonda existe para não ter.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_the_card_still_cannot_reach
//! ```

use crate::motion_state::MotionState;

/// Os gates da wave que fechou o **selector de canal** — irmão por responsabilidade (o censo
/// aqui, a lei do clique lá).
#[cfg(test)]
#[path = "motion_bridge_panel_exit_channel_tests.rs"]
mod channel_tests;
use ph2d_panel_motion_graph::{ClickDoes, click_does};
use std::collections::BTreeMap;

/// Como o PAINEL desenha este param — o que se perderia ao fechá-lo.
fn especie_no_painel(row: &ph2d_panel_motion_params::ParamRow) -> &'static str {
    use ph2d_panel_motion_params::ParamRow as R;
    match row {
        R::Scalar(_) => "número",
        R::Color(_) => "AMOSTRA + selector OKLCH",
        R::Toggle(_) => "interruptor",
        R::Enum(_) => "selector segmentado",
        R::Angle(_) => "ângulo",
        R::Seed(_) => "semente + re-sortear",
        R::Text(_) => "CAMPO DE TEXTO",
        R::Curve(_) => "EDITOR DE CURVA",
        R::Gradient(_) => "EDITOR DE GRADIENTE",
        R::Palette(_) => "EDITOR DE PALETA",
        R::Channels(_) => "selector de CANAL (+ texto)",
        R::Source(_) => "selector de FONTE publicada",
        R::File(_) => "caminho + diálogo de FICHEIRO",
    }
}

/// `(total de rows, por veredito, os inalcançáveis por espécie)`.
type Censo = (
    usize,
    BTreeMap<&'static str, usize>,
    BTreeMap<String, Vec<String>>,
);

fn censo() -> Censo {
    let todos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    let (mut total, mut veredito, mut presos): Censo = (0, BTreeMap::new(), BTreeMap::new());
    for nome in todos.iter().map(String::as_str) {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor_core::ProjectSettings::default(),
            &mut snap,
        );
        // O MESMO nó no painel — para a linha dizer o que se perde, e não só que se perde.
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = crate::motion_bridge::params::build_params_snapshot(
            &m,
            ph2d_editor_core::ProjectSettings::default(),
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
        let Some(cartao) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        for p in &cartao.params {
            total += 1;
            let v = match click_does(p) {
                ClickDoes::Type => "escreve-se",
                ClickDoes::Toggle => "vira",
                ClickDoes::Cycle(_) => "avança",
                ClickDoes::PickFile => "abre ficheiro",
                ClickDoes::PickSelection => "liga o seleccionado",
                ClickDoes::CycleSource => "avança a fonte",
                ClickDoes::CycleChannel => "avança o canal",
                ClickDoes::OpensPicker => "abre o selector",
                ClickDoes::OpensEditor => "abre o editor",
                ClickDoes::TypeText => "escreve texto",
                ClickDoes::Nothing => "NADA",
            };
            *veredito.entry(v).or_default() += 1;
            if v != "NADA" {
                continue;
            }
            // O que o painel faz com ele — a metade que diz o que há para construir.
            let no_painel = painel
                .as_ref()
                .and_then(|s| {
                    s.rows
                        .iter()
                        .find(|r| r.params().contains(&p.hint.param))
                        .map(especie_no_painel)
                })
                .unwrap_or("(o painel também não o mostra)");
            presos
                .entry(no_painel.to_string())
                .or_default()
                .push(format!("{nome}::{}", p.hint.param));
        }
    }
    (total, veredito, presos)
}

#[test]
#[ignore = "sonda de censo — corra à mão"]
fn what_the_card_still_cannot_reach() {
    let (total, veredito, presos) = censo();
    eprintln!("\n  {total} rows de cartão em todo o catálogo:");
    for (v, n) in &veredito {
        eprintln!("    {v:<12} {n:>4}");
    }
    let inalcancaveis: usize = presos.values().map(Vec::len).sum();
    eprintln!(
        "\n  ⛔ {inalcancaveis} controlos que o cartão PINTA e não abre — o que o painel faria:\n"
    );
    for (especie, quais) in &presos {
        eprintln!("  ── {especie} — {} controlo(s)", quais.len());
        for q in quais {
            eprintln!("       {q}");
        }
    }
    eprintln!();
}

/// ⛔⛔ **O PAINEL LATERAL NÃO PODE SAIR ENQUANTO ISTO NÃO FOR ZERO.**
///
/// A ordem do Enio (2026-09-05) é retirá-lo; esta é a conta que diz **quando**. Enquanto houver
/// um controlo que só o painel abre, fechá-lo torna-o inalcançável — e um controlo que se vê e
/// não se toca é pior que um ausente, porque o artista conclui que o app está avariado.
///
/// ⚠️ **Este gate falha de PROPÓSITO hoje?** Não: ele afirma o que é verdade agora — que a
/// contagem é conhecida e está ANOTADA. Ele quebra quando alguém acrescenta um editor rico novo
/// sem o alcançar no cartão (a conta sobe) **ou** quando a wave o cura (a conta desce e a
/// anotação fica a mentir). Nos dois casos o número aqui tem de ser reconciliado por MEDIÇÃO.
#[test]
fn the_side_panel_cannot_leave_while_the_card_cannot_open_these() {
    let (total, _, presos) = censo();
    let inalcancaveis: usize = presos.values().map(Vec::len).sum();
    assert!(total > 600, "controle: a varredura viu {total} rows");
    assert_eq!(
        inalcancaveis, TRANCADOS_NO_PAINEL,
        "a conta dos controlos que só o painel abre mudou ({inalcancaveis} contra \
         {TRANCADOS_NO_PAINEL}) — corra `what_the_card_still_cannot_reach` e reconcilie o \
         número com a MEDIÇÃO, nunca ao contrário"
    );
}

/// ⭐⭐ **O TEXTO ESCRITO NO CARTÃO CHEGA À PORTA DE TEXTO DO PAINEL** — a segunda metade do
/// gesto, e a que o painel não pode provar sozinho.
///
/// FALSIFICADO por apagar o braço `GraphIntent::SetTextParam` do `apply_graph_intents`: o
/// artista escreve, carrega `Enter`, e a edição evapora sem uma palavra.
#[test]
fn the_text_typed_on_a_card_reaches_the_panels_text_door() {
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.expression");
    let _ = drain_intents();
    let _ = ph2d_panel_motion_params::drain_param_intents();
    push_intent(GraphIntent::SetTextParam {
        node: id.0,
        param: "expr",
        value: "sin(t)".to_string(),
    });
    crate::motion_bridge::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
    );
    let saiu = ph2d_panel_motion_params::drain_param_intents();
    assert!(
        saiu.iter().any(|i| matches!(
            i,
            ph2d_panel_motion_params::MotionParamIntent::SetTextParam { node, param, value }
                if *node == id.0 && *param == "expr" && value == "sin(t)"
        )),
        "o texto do cartao nao chegou a` porta do painel: {saiu:?}"
    );
}

/// ⭐⭐ **O PEDIDO DO CARTÃO CHEGA À PORTA DO PAINEL** — a metade que o gate do painel não
/// prova. Lá mede-se que o clique **emite** a intenção; aqui, que a shell a **traduz** para a
/// mesma `MotionParamIntent::PickFile` que a row do painel usa há meses.
///
/// ⚠️ **São dois defeitos diferentes e nenhum gate via os dois:** um clique que não emite nada
/// (o braço `_ => {}` de antes) e uma emissão que a shell ignora. O segundo é o pior — a fila
/// enche e nada acontece, e nenhuma superfície diz porquê.
///
/// ⛔ **E ele não abre diálogo nenhum:** o teste pára na tradução, de propósito. Abrir uma
/// janela do sistema dentro de um teste é o que a porta `modal::pick_file` existe para
/// cronometrar, e um gate que a chamasse ficaria pendurado à espera de um humano.
///
/// FALSIFICADO por apagar o braço `GraphIntent::PickFile` do `apply_graph_intents`.
#[test]
fn the_cards_file_click_reaches_the_same_door_the_panel_row_uses() {
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("source.table");
    let _ = drain_intents();
    let _ = ph2d_panel_motion_params::drain_param_intents();
    push_intent(GraphIntent::PickFile {
        node: id.0,
        param: "file",
    });
    crate::motion_bridge::apply_graph_intents(
        &mut m,
        &mut ph2d_core::Playhead::default(),
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
    );
    let saiu = ph2d_panel_motion_params::drain_param_intents();
    assert!(
        saiu.iter().any(|i| matches!(
            i,
            ph2d_panel_motion_params::MotionParamIntent::PickFile { node, param }
                if *node == id.0 && *param == "file"
        )),
        "o pedido do cartao nao chegou a` porta do painel: {saiu:?}"
    );
}

/// ⭐⭐⭐ **DOIS CARTÕES DO MESMO TIPO NÃO PEDEM O MESMO SELECTOR** — e o nó sai do ID, não da
/// selecção.
///
/// ⛔⛔ **É o defeito que esta wave existe para não ter.** O id da amostra do painel é função
/// **só do nome do param âncora** (*«unique within a node»*, diz o doc dele) — verdade ali,
/// onde há **um** nó selecionado. No canvas há vinte cartões: com esse id, escolher a cor de um
/// `motion.tint` escreveria no outro, **em silêncio**.
///
/// As duas metades:
/// 1. os ids de dois nós do mesmo tipo **diferem**;
/// 2. com o selector apontado ao cartão de **B**, a porta devolve **B** — mesmo com **A**
///    selecionado. *Uma amostra de cartão não precisa de selecionar o nó para o editar.*
///
/// FALSIFICADO por o `card_swatch_id` ignorar o nó (1 falha) ou por a porta voltar a presumir o
/// nó selecionado (2 falha).
#[test]
fn two_cards_of_the_same_type_never_ask_for_the_same_colour_picker() {
    use super::super::color::{card_swatch_id, picker_target_of};
    let mut m = MotionState::new();
    let a = m.doc.graph.add_node("motion.tint");
    let b = m.doc.graph.add_node("motion.tint");
    let (ida, idb) = (card_swatch_id(a.0, "r"), card_swatch_id(b.0, "r"));
    assert_ne!(
        ida, idb,
        "dois `motion.tint` pediriam o MESMO selector — escolher a cor de um escreveria no outro"
    );

    let tipo = m.doc.graph.node(a).expect("o no' existe").type_id();
    let grupos = super::super::color::color_groups(&m.registry, tipo);
    assert!(!grupos.is_empty(), "o `motion.tint` tem um grupo de cor");
    let mut store = ph2d_editor_core::interaction::WidgetStore::default();
    store.set_picker_target(Some(idb));
    let alvo = picker_target_of(&m, Some(a), &grupos, &store);
    assert_eq!(
        alvo.map(|(n, _)| n),
        Some(b),
        "o selector aponta ao cartao de B: a cor tem de ir a B, com A seleccionado"
    );
}

/// ⭐⭐⭐ **O CARTÃO DIZ O QUE ESTÁ ESCOLHIDO** — a metade que faltava às duas waves de hoje.
///
/// Depois delas o artista **muda** a forma e o ficheiro a partir do cartão; até agora não os
/// **lia** — a row voltava ao selo, e mudar uma coisa que não se consegue ler é meio controlo.
///
/// As três afirmações, e a terceira é a que impede o ruído:
/// 1. de uma **fonte publicada** mostra-se o nome, tal e qual;
/// 2. de um **ficheiro** mostra-se o NOME, nunca o caminho (o elidor corta pelo fim, e um
///    caminho absoluto mostraria a metade que não identifica ficheiro nenhum);
/// 3. de uma **curva** não se mostra nada — o valor dela é uma serialização, texto de máquina.
///
/// FALSIFICADO por o `card_text` devolver o caminho inteiro (1 falha), ou por deixar passar a
/// serialização de uma curva (3 falha).
#[test]
fn the_card_reads_back_the_name_the_file_and_never_a_machine_string() {
    let texto_de = |tipo: &str, param: &'static str, valor: &str| -> String {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(tipo.to_string());
        m.doc.graph.set_text_param(id, param, valor.to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor_core::ProjectSettings::default(),
            &mut snap,
        );
        snap.nodes
            .iter()
            .find(|v| v.id == id.0)
            .and_then(|v| v.params.iter().find(|p| p.hint.param == param))
            .map(|p| p.text.as_str().to_string())
            .unwrap_or_else(|| panic!("`{tipo}::{param}` nao esta' no cartao"))
    };

    assert_eq!(
        texto_de("motion.path", "path", "Estrela"),
        "Estrela",
        "a forma escolhida le^-se na row"
    );
    assert_eq!(
        texto_de("source.table", "file", "/home/enio/Documentos/dados.csv"),
        "dados.csv",
        "de um caminho fica o NOME — o elidor corta pelo fim"
    );
    assert_eq!(
        texto_de("value.curve", "curve", "0.0,0.0;0.5,1.0;1.0,1.0"),
        "",
        "uma curva e' texto de MAQUINA: a row mostra o selo, nao a serializacao"
    );
}

/// ⭐⭐ **O CLIQUE ANDA PELA LISTA VIVA, E PÁRA QUANDO NÃO HÁ LISTA.**
///
/// As três metades que só juntas fazem o gesto: a **primeira** escolha quando nada está
/// escolhido, a **volta ao princípio** no fim, e — a que interessa — **nada escrito** quando o
/// artista ainda não desenhou nada. ⚠️ *Um clique que inventasse um nome escreveria uma
/// referência a uma forma que não existe, e o nó ficaria mudo sem dizer porquê.*
///
/// FALSIFICADO por `next_source` devolver `Some` sobre uma lista vazia, ou por não dar a volta.
#[test]
fn the_card_walks_the_live_source_list_and_writes_nothing_when_it_is_empty() {
    use ph2d_nodegraph::attr::{Column, Stream};
    use ph2d_panel_motion_graph::{GraphIntent, drain_intents, push_intent};

    let clique = |m: &mut MotionState, id: ph2d_nodegraph::graph::NodeId| -> Option<String> {
        let _ = drain_intents();
        let _ = ph2d_panel_motion_params::drain_param_intents();
        push_intent(GraphIntent::StepChoice {
            node: id.0,
            param: "path",
            delta: 1,
        });
        crate::motion_bridge::apply_graph_intents(
            m,
            &mut ph2d_core::Playhead::default(),
            &mut ph2d_editor_core::ToastQueue::default(),
            &mut ph2d_editor_core::screens::layout::CenterSplit::None,
        );
        ph2d_panel_motion_params::drain_param_intents()
            .into_iter()
            .find_map(|i| match i {
                ph2d_panel_motion_params::MotionParamIntent::SetTextParam { value, .. } => {
                    Some(value)
                }
                _ => None,
            })
    };

    // (a) NADA publicado — o clique não escreve.
    let mut m = MotionState::new();
    let id = m.doc.graph.add_node("motion.path");
    assert_eq!(
        clique(&mut m, id),
        None,
        "sem nada desenhado nao ha' o que escolher, e o clique nao pode inventar um nome"
    );

    // (b) duas formas publicadas — a primeira, a seguinte, e a volta.
    let at = |x: f32| Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]));
    m.pump.cook.set_external("Estrela".to_string(), at(1.0));
    m.pump.cook.set_external("Lua".to_string(), at(2.0));
    let lista = super::super::params::choices::source_options_live(&m);
    assert_eq!(lista.len(), 2, "as duas formas sao pickaveis: {lista:?}");

    let primeira = clique(&mut m, id).expect("com lista, o clique escolhe");
    assert_eq!(primeira, lista[0], "sem escolha feita, escolhe a primeira");
    m.doc.graph.set_text_param(id, "path", primeira.clone());
    let segunda = clique(&mut m, id).expect("e avanca");
    assert_eq!(segunda, lista[1], "de {primeira} para a seguinte");
    m.doc.graph.set_text_param(id, "path", segunda);
    assert_eq!(
        clique(&mut m, id).as_deref(),
        Some(lista[0].as_str()),
        "e da ultima volta ao principio, como o enum do cartao"
    );
}

/// **Medido em 2026-09-07: `0` de `683` rows** (eram `26`) — reconciliado pela sonda, nunca
/// escrito de memória.
///
/// | espécie | quantos | como fechou |
/// |---|---:|---|
/// | campo de TEXTO | 9 | a caixa abre com o valor INTEIRO |
/// | amostra + selector de COR | 4 | o id da amostra passou a carregar o NÓ |
/// | selector de FONTE publicada | 4 | setas + lista, pela lista viva |
/// | caminho + diálogo de FICHEIRO | 3 | o cartão pede, a shell abre |
/// | selector de CANAL | 1 | o clique anda pelos canais e escreve o PAR |
/// | editor de CURVA | 2 | **janela flutuante** sobre o cartão |
/// | editor de GRADIENTE | 2 | a mesma janela |
/// | editor de PALETA | 1 | a mesma janela |
///
/// ⛔⛔ **ZERO AQUI NÃO É AUTORIZAÇÃO PARA RETIRAR O PAINEL, e a razão é um PONTO CEGO desta
/// sonda.** Ela pergunta *o que um clique nesta row FAZ* — e por isso conta um gradiente como
/// alcançável no dia em que a janela ABRE, mesmo que a cor escolhida lá dentro não chegue a
/// lado nenhum. *Um editor que abre, aceita o gesto e não guarda é a espécie de controlo morto
/// que uma sonda de alcance lê como vivo.*
///
/// ⇒ A outra metade tem gates PRÓPRIOS, e eles vivem do lado da shell porque é lá que a leitura
/// de volta do selector acontece: [`super::super::color`] (`card_tests`) mede que a cor
/// escolhida numa janela de cartão chega à string, e que dois cartões do mesmo tipo não
/// partilham a amostra. **Antes de retirar o painel, corra as duas famílias.**
///
/// ⏳ **E fica um item de PRODUTO, não de alcance:** o painel lateral ainda mostra as rows, e
/// retirá-lo é uma decisão do Enio com um smoke pelo meio — o cartão passou a alcançar tudo,
/// que era a condição, não a ordem.
const TRANCADOS_NO_PAINEL: usize = 0;
