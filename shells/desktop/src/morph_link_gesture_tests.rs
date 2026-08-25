//! Os gates da seta — a **lei**, que não precisa de janela, e a **costura**, que só se pode ler.

use super::link_shapes;
use ph2d_ecs::{SimWorld, VecMorph, VecMorphMachine};

const A: u64 = 10;
const B: u64 = 20;
const C: u64 = 30;

/// Um mundo com um objecto de Morph entre `A` e `B`, e ainda **sem máquina**.
fn morph_world() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(VecMorph::new(A, B)).id();
    (sim, e)
}

/// ⭐ **A PRIMEIRA seta faz nascer a máquina, com o `start` na forma de onde ela parte.**
///
/// É por isso que o `VecMorphMachine` **não tem `Default`**: o `start` é um facto do gesto, nunca
/// um zero que alguém teria de corrigir depois.
///
/// **Mutação que deve sangrar:** a máquina nascer com `start: 0`.
#[test]
fn the_first_arrow_gives_birth_to_the_machine_starting_where_it_left() {
    let (mut sim, e) = morph_world();
    assert!(
        sim.world().get::<VecMorphMachine>(e).is_none(),
        "o CONTROLE: um Morph nasce SEM maquina"
    );
    assert!(link_shapes(&mut sim, e, A, B));
    let m = sim.world().get::<VecMorphMachine>(e).expect("nasceu");
    assert_eq!(
        m.graph.start, A,
        "a maquina tem de comecar onde a seta partiu"
    );
    assert_eq!(m.graph.edges.len(), 1);
    assert_eq!((m.graph.edges[0].from, m.graph.edges[0].to), (A, B));
}

/// **A segunda seta entra na MESMA máquina** — e não faz nascer outra.
#[test]
fn the_second_arrow_joins_the_machine_that_is_already_there() {
    let (mut sim, e) = morph_world();
    link_shapes(&mut sim, e, A, B);
    assert!(link_shapes(&mut sim, e, B, C));
    let m = sim.world().get::<VecMorphMachine>(e).expect("existe");
    assert_eq!(
        m.graph.start, A,
        "a segunda seta nao pode remarcar o comeco"
    );
    assert_eq!(m.graph.edges.len(), 2);
    assert_eq!(
        m.graph.shapes(),
        vec![A, B, C],
        "as tres formas sao estados"
    );
}

/// ⛔ **Uma forma não se liga a si própria.** Um morph de uma forma para ela mesma é a identidade —
/// uma transição que não transita.
///
/// ⚠️ **E isto NÃO é a decisão do conector**, que aceita o laço de propósito: lá o laço é um
/// desenho legítimo; aqui seria uma regra vazia.
///
/// **Mutação que deve sangrar:** largar a guarda `from == to`.
#[test]
fn a_shape_never_links_to_itself() {
    let (mut sim, e) = morph_world();
    assert!(!link_shapes(&mut sim, e, A, A));
    assert!(
        sim.world().get::<VecMorphMachine>(e).is_none(),
        "e a recusa nao pode deixar uma maquina vazia para tras"
    );
}

/// ⚠️ **Uma seta repetida não se duplica.** Duas arestas iguais seriam duas linhas idênticas no
/// painel, uma impossível de distinguir da outra ao apagar.
///
/// **Mutação que deve sangrar:** largar a guarda do `any(...)`.
#[test]
fn drawing_the_same_arrow_twice_never_duplicates_it() {
    let (mut sim, e) = morph_world();
    assert!(link_shapes(&mut sim, e, A, B));
    assert!(!link_shapes(&mut sim, e, A, B), "a segunda vez recusa");
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(e)
            .unwrap()
            .graph
            .edges
            .len(),
        1
    );
    // O CONTROLE: a seta OPOSTA e' outra seta, e essa entra.
    assert!(link_shapes(&mut sim, e, B, A));
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(e)
            .unwrap()
            .graph
            .edges
            .len(),
        2
    );
}

/// **A COSTURA: o gesto está ligado aos dois lados do despacho.**
///
/// ⚠️ **Um gate de TEXTO, e a razão fica escrita:** o `morph_link_down`/`_up` precisam de um
/// `AppGfx` — uma janela real e uma superfície de GPU —, que um teste não alcança (a mesma parede
/// que a linha do sculpt3d registou no undo do filtro). A **lei** está gateada acima; o que sobra
/// é provar que alguém a chama, e isso só se pode **ler**.
///
/// ⛔ Sem esta metade, os quatro gates acima ficariam verdes sobre uma feature que **gesto nenhum
/// alcança** — que é exactamente o defeito que esta linha já pagou três vezes.
#[test]
fn both_halves_of_the_gesture_are_wired_to_the_dispatch() {
    let src = include_str!("input_dispatch.rs");
    for (needle, what) in [
        ("self.morph_link_down(w);", "ARMAR no Primary Down"),
        ("self.morph_link_up(w)", "FECHAR no Primary Up"),
        (
            "self.morph_link_cancel()",
            "LARGAR quando o ponto de mundo nao resolve",
        ),
        (
            "ph2d_tool_vector::DrawMode::MorphLink",
            "e o ramo do Down tem de perguntar pelo MODO",
        ),
    ] {
        assert!(
            src.contains(needle),
            "o despacho perdeu o `{needle}` -- {what}. A lei continua gateada e INALCANCAVEL."
        );
    }
}
