//! Os gates da costura das setas — **a lei**, que não precisa de janela, e o **fio**, que só se lê.

use super::{ArrowCmd, apply, arrow_cmd_for_id, morph_of_selection, publish};
use ph2d_ecs::{Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_morph_machine::{MorphEdge, MorphGraph};

const A: u64 = 10;
const B: u64 = 20;

fn actions() -> Vec<String> {
    vec!["jump".to_string(), "dash".to_string()]
}

fn world() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let mut m = VecMorphMachine::new(A);
    let mut jump = MorphEdge::new(A, B);
    jump.when = "jump".to_string();
    m.graph = MorphGraph {
        start: A,
        edges: vec![jump, MorphEdge::new(B, A)],
    };
    let e = sim.world_mut().spawn((VecMorph::new(A, B), m)).id();
    (sim, e)
}

/// **Cada id de linha resolve para o comando dela** — a tabela, e não uma cadeia de `if`.
///
/// **Mutação que deve sangrar:** o laço parar na primeira linha (`0..1`) — as setas a partir da
/// segunda ficariam **mortas sob o ponteiro**, e só a de cima funcionaria.
#[test]
fn every_arrow_row_resolves_to_its_own_command() {
    use ph2d_editor::ids as i;
    for row in 0..i::MAX_MORPH_ARROWS {
        assert_eq!(
            arrow_cmd_for_id(i::morph_arrow_delete_id(row)),
            Some(ArrowCmd::Delete { row }),
            "a lixeira da linha {row} nao resolve"
        );
        assert_eq!(
            arrow_cmd_for_id(i::morph_arrow_when_option_id(row, 3)),
            Some(ArrowCmd::SetWhen { row, action: 3 }),
            "a opcao 3 da linha {row} nao resolve"
        );
    }
    // O CONTROLE: um id que não é de seta tem de devolver `None`, senão a tabela engoliria
    // cliques alheios.
    assert_eq!(arrow_cmd_for_id(ph2d_editor::ids::VECTOR_BOOL_UNION), None);
}

/// ⭐ **A CONDIÇÃO é escolhida pelo ÍNDICE do menu, e o `0` é o «—».**
///
/// **Mutação que deve sangrar:** tratar o `0` como a primeira acção — tirar a condição passaria a
/// pôr `jump`, e o artista não teria gesto nenhum para a limpar.
#[test]
fn the_first_option_clears_the_condition_and_the_rest_pick_an_action() {
    let (mut sim, e) = world();
    assert!(apply(
        &mut sim,
        e,
        ArrowCmd::SetWhen { row: 1, action: 2 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[1].when,
        "dash"
    );
    assert!(apply(
        &mut sim,
        e,
        ArrowCmd::SetWhen { row: 1, action: 0 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[1].when,
        "",
        "o «—» tem de LIMPAR a condicao"
    );
}

/// ⛔ **Um índice fora da lista publicada RECUSA**, em vez de escrever um nome inventado.
///
/// ⚠️ Ele é alcançável: o mapa pode mudar entre o menu abrir e o clique chegar.
#[test]
fn an_index_beyond_the_published_list_refuses() {
    let (mut sim, e) = world();
    assert!(!apply(
        &mut sim,
        e,
        ArrowCmd::SetWhen { row: 0, action: 99 },
        &actions()
    ));
    assert_eq!(
        sim.world().get::<VecMorphMachine>(e).unwrap().graph.edges[0].when,
        "jump",
        "a condicao antiga tem de ficar intacta"
    );
}

/// **Apagar tira a linha certa, e uma linha que não existe recusa.**
#[test]
fn deleting_removes_that_row_and_only_that_row() {
    let (mut sim, e) = world();
    assert!(apply(&mut sim, e, ArrowCmd::Delete { row: 0 }, &actions()));
    let g = &sim.world().get::<VecMorphMachine>(e).unwrap().graph;
    assert_eq!(g.edges.len(), 1);
    assert_eq!(
        (g.edges[0].from, g.edges[0].to),
        (B, A),
        "sobrou a OUTRA seta"
    );
    assert!(!apply(&mut sim, e, ArrowCmd::Delete { row: 9 }, &actions()));
}

/// ⭐ **O Morph é achado na seleção INTEIRA, nunca no primeiro operando.**
///
/// **Mutação que deve sangrar:** usar `sel.first()` — tocar num morph traz o grupo, e a seção
/// mostraria as setas de um objecto enquanto o clique escreveria noutro.
#[test]
fn the_morph_is_found_anywhere_in_the_selection() {
    let (mut sim, e) = world();
    let other = sim.world_mut().spawn(Name("um grupo".to_string())).id();
    let sel = vec![other.to_bits(), e.to_bits()];
    assert_eq!(morph_of_selection(&sim, &sel), Some(e));
    // O CONTROLE: sem morph nenhum, `None`.
    assert_eq!(morph_of_selection(&sim, &[other.to_bits()]), None);
}

/// ⭐⭐ **Um Morph SEM máquina publica a face VAZIA — e nunca `None`.**
///
/// ⚠️ As duas coisas pintam faces diferentes: `None` = *"a seleção não é um Morph"* (a seção nem
/// fala de setas); vazio = *"é um Morph e ainda não tem setas"*, e é essa face que diz **como**
/// desenhar a primeira. Sem ela o artista vê um cabeçalho e nada por baixo.
///
/// **Mutação que deve sangrar:** o `publish` devolver `None` quando não há máquina.
#[test]
fn a_morph_without_a_machine_publishes_the_empty_face() {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecMorph::new(A, B)).id();
    let scene = ph2d_vec_scene::VecScene::new();
    let map = crate::vec_entities::VecEntityMap::default();
    let s = publish(&sim, &scene, &map, &[e.to_bits()], actions())
        .expect("um Morph SEM maquina ainda publica");
    assert!(s.rows.is_empty(), "e a lista de setas vem vazia");
    assert_eq!(
        s.actions,
        actions(),
        "as accoes vem sempre -- o menu precisa delas"
    );
}

/// **A COSTURA: o clique da seta atravessa o barramento e chega ao mundo.**
///
/// ⚠️ Um gate de TEXTO, e a razão é a mesma do gesto: o caminho real precisa de um `AppGfx`. A
/// **lei** está gateada acima; o que sobra é provar que alguém a chama — e isso só se pode ler.
///
/// ⛔ Sem esta metade, os gates acima ficariam verdes sobre controlos que **pintam, acendem sob o
/// rato e cujo clique morre no painel** — que é o defeito que a décima lista do modo custou uma
/// wave atrás.
#[test]
fn the_arrow_click_reaches_the_world() {
    let shell = include_str!("render_loop/mod.rs");
    for (needle, what) in [
        (
            "crate::vec_morph_edit::arrow_cmd_for_id(*id)",
            "RESOLVER o id do clique",
        ),
        (
            "crate::vec_morph_edit::apply(sim, e, cmd, &actions)",
            "APLICAR ao mundo",
        ),
        (
            "set_morph_states_state(crate::vec_morph_edit::publish(",
            "PUBLICAR a projeccao",
        ),
    ] {
        assert!(
            shell.contains(needle),
            "a shell perdeu o `{needle}` -- {what}. A lei continua gateada e INALCANCAVEL."
        );
    }
    // E o painel tem de FORWARDAR o clique, senão ele morre antes de chegar aqui.
    let panel = include_str!("../../../crates/ph2d-panel-vector/src/event_clicks.rs");
    assert!(
        panel.contains("morph_arrow_delete_id(r)"),
        "o painel deixou de encaminhar o clique da seta: ele acende sob o rato e morre ali"
    );
}
