//! Os gates do CONJUNTO de estados (plano 32 W8) — a **lei** do grafo, e a **costura** que a
//! aplica ao mundo.

use super::{create, eligible, graph_of, upkeep};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, VecMorph, VecMorphMachine};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::{VecEntityMap, sync};

/// Três formas soltas, com nome, já sincronizadas com o mundo.
fn world(n: usize) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(VecPath::default()))
        .collect();
    sync(&mut sim, &mut scene, &mut map);
    for (i, id) in ids.iter().enumerate() {
        let e = Entity::from_bits(map[id]);
        sim.world_mut()
            .entity_mut(e)
            .insert(Name::new(format!("S{i}")));
    }
    (sim, scene, map, ids)
}

/// ⭐⭐⭐ **A LISTA SÃO OS FILHOS, e uma forma arrastada para dentro ENTRA** — a lei da W11.
///
/// Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se for
/// arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente passa a
/// fazer parte do sistema."*
///
/// ⚠️ **Este gate é a feature INTEIRA, e repare no que ele NÃO faz:** não chama função nenhuma de
/// «entrar». Ele só reparenta — que é o que a Hierarquia faz — e volta a perguntar. *Arrastar para
/// dentro é entrar porque a lista É a hierarquia.*
///
/// **Mutação que deve sangrar:** o `graph_of` voltar a ler uma lista guardada no componente.
#[test]
fn a_shape_dragged_into_the_set_joins_it_with_no_code_reacting() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids[..2], 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    assert_eq!(
        graph_of(&sim, &map, host).shapes(),
        ids[..2].to_vec(),
        "o CONTROLE: o conjunto nasce com as DUAS que foram escolhidas"
    );

    // ⭐ O GESTO: a terceira forma vira filha. Nada mais.
    let third = Entity::from_bits(map[&ids[2]]);
    crate::vec_transform::reparent_keeping_world(&mut sim, third, host);

    assert_eq!(
        graph_of(&sim, &map, host).shapes(),
        ids,
        "arrastar para dentro TEM de fazer entrar -- a lista sao os filhos"
    );
    // ⭐ E ela entra **sem tecla**, com os valores de partida: ninguém escreveu nada por ela.
    let g = graph_of(&sim, &map, host);
    assert!(
        g.states.last().unwrap().when.is_empty(),
        "a forma nova entra MUDA -- uma tecla de fabrica dispararia sem ninguem pedir"
    );
    assert!(
        (g.states.last().unwrap().duration_s - ph2d_morph_machine::DEFAULT_DURATION_S).abs() < 1e-9,
        "e com o ritmo de partida"
    );
}

/// ⭐ **E arrastar para FORA sai** — a outra metade, e é ela que dá o *Desconectar* de graça.
#[test]
fn a_shape_dragged_out_of_the_set_leaves_it() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    assert_eq!(graph_of(&sim, &map, host).shapes().len(), 3);

    // O GESTO inverso: o filho do meio deixa de ser filho.
    let mid = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(mid).remove::<ChildOf>();

    assert_eq!(
        graph_of(&sim, &map, host).shapes(),
        vec![ids[0], ids[2]],
        "sair da hierarquia TEM de sair da lista"
    );
}

/// **A primeira forma é onde a máquina nasce — e é o primeiro FILHO.**
///
/// **Mutação que deve sangrar:** o `start()` devolver `states.last()`.
#[test]
fn the_first_child_is_the_start() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    assert_eq!(graph_of(&sim, &map, host).start(), Some(ids[0]));
}

/// ⭐ **A TECLA SOBREVIVE a sair e voltar a entrar.**
///
/// ⚠️ As chaves são indexadas por forma e **não** são varridas quando um filho sai: perder o
/// trabalho do artista por um gesto reversível seria a pior leitura possível de *"desconectar"*.
///
/// **Mutação que deve sangrar:** o `graph_of` (ou um futuro *Disconnect*) apagar a chave ao sair.
#[test]
fn the_key_survives_leaving_and_coming_back() {
    let (mut sim, mut scene, mut map, ids) = world(2);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    // O artista dá uma tecla à segunda forma.
    sim.world_mut()
        .get_mut::<VecMorphMachine>(host)
        .unwrap()
        .keys
        .insert(
            ids[1],
            ph2d_morph_machine::MorphKey {
                when: "jump".into(),
                ..Default::default()
            },
        );
    let kid = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(kid).remove::<ChildOf>();
    assert_eq!(graph_of(&sim, &map, host).shapes(), vec![ids[0]]);

    crate::vec_transform::reparent_keeping_world(&mut sim, kid, host);
    let g = graph_of(&sim, &map, host);
    assert_eq!(
        g.states.iter().find(|st| st.shape == ids[1]).unwrap().when,
        "jump",
        "a tecla tem de VOLTAR com a forma -- desconectar nao pode destruir autoria"
    );
}

/// ⛔ **Uma forma que JÁ é um Morph nunca vira estado.**
///
/// ⚠️ Um conjunto sobre um conjunto daria uma máquina cujos estados são re-escritos por baixo dela
/// a cada quadro (o `recook` do morph interior).
#[test]
fn a_morph_is_never_a_state_of_another_set() {
    let (mut sim, _scene, map, ids) = world(3);
    let e = Entity::from_bits(map[&ids[1]]);
    sim.world_mut().entity_mut(e).insert(VecMorph::new(1, 2));
    let ok = eligible(&sim, &map, &ids);
    assert_eq!(ok, vec![ids[0], ids[2]], "o morph do meio tem de sair");
    // O CONTROLE: sem morph nenhum, as três passam.
    let (sim2, _s2, map2, ids2) = world(3);
    assert_eq!(eligible(&sim2, &map2, &ids2), ids2);
}

/// **Uma forma só, ou formas a mais, RECUSAM** — e a recusa não põe lixo na cena.
///
/// ⚠️ A metade da cena é a que importa: um `push_path` antes da checagem deixaria um path órfão por
/// cada clique recusado, e eles acumulam sem nada na tela a dizê-lo.
#[test]
fn one_shape_or_too_many_refuses_without_littering_the_scene() {
    let (sim, mut scene, map, ids) = world(3);
    let before = scene.paths().len();
    assert!(create(&sim, &mut scene, &map, &ids[..1], 9).is_none());
    assert!(
        create(&sim, &mut scene, &map, &ids, 2).is_none(),
        "tres > 2"
    );
    assert_eq!(
        scene.paths().len(),
        before,
        "uma recusa nao pode deixar um path orfao na cena"
    );
    // O CONTROLE POSITIVO: dentro do tecto, ela aceita e o path nasce.
    assert!(create(&sim, &mut scene, &map, &ids, 3).is_some());
    assert_eq!(scene.paths().len(), before + 1);
}

/// ⭐ **O que o conjunto FAZ AO MUNDO** — irmão por assunto, cortado pelo teto de 600 LOC.
#[path = "morph_set_world_tests.rs"]
mod world_tests;
