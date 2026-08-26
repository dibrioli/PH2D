//! Os gates do CONJUNTO de estados (plano 32 W8) — a **lei** do grafo, e a **costura** que a
//! aplica ao mundo.

use super::{complete_digraph, create, eligible, upkeep};
use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, VecMorph, VecMorphMachine, Visibility};
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

/// ⭐ **TODAS as morphs possíveis, de ida E de volta** — `n(n-1)`, sem laço e sem repetida.
///
/// **Mutação que deve sangrar:** trocar `from != to` por `true` — nasceriam `n` laços (uma forma
/// que transita para si própria), e a lista mostraria linhas `S0 -> S0` que nunca fazem nada.
///
/// **Segunda mutação:** emitir só `from < to` — metade das setas, e o artista que fosse de `S0`
/// para `S1` **nunca mais voltaria**.
#[test]
fn the_graph_covers_every_ordered_pair_in_both_directions() {
    for n in 2..=6usize {
        let shapes: Vec<VecPathId> = (1..=n as u64).collect();
        let g = complete_digraph(&shapes);
        assert_eq!(
            g.edges.len(),
            n * (n - 1),
            "com {n} formas o grafo completo tem n(n-1) arestas"
        );
        for &a in &shapes {
            for &b in &shapes {
                if a == b {
                    assert!(
                        !g.edges.iter().any(|e| e.from == a && e.to == b),
                        "um laco {a}->{a} e' uma transicao que nunca faz nada"
                    );
                } else {
                    assert_eq!(
                        g.edges.iter().filter(|e| e.from == a && e.to == b).count(),
                        1,
                        "a passagem {a}->{b} tem de existir EXACTAMENTE uma vez"
                    );
                }
            }
        }
    }
}

/// **A primeira forma da seleção é o estado inicial.**
///
/// **Mutação que deve sangrar:** `shapes.last()` — o conjunto nasceria a mostrar a última forma
/// escolhida, e o artista que escolheu da esquerda para a direita veria a da direita.
#[test]
fn the_first_shape_chosen_is_the_start() {
    let g = complete_digraph(&[7, 3, 9]);
    assert_eq!(g.start, 7);
    // O CONTROLE: sem formas nenhumas não há `start` inventado nem pânico.
    assert_eq!(complete_digraph(&[]).edges.len(), 0);
}

/// ⭐ **A ORDEM das arestas é determinística** — a lista do painel indexa por POSIÇÃO.
///
/// ⚠️ Sem isto o menu «When» da linha 3 escreveria a condição noutra transição depois de um undo, e
/// o artista não teria como saber. É a mesma razão do `BTreeMap` da física.
#[test]
fn the_edge_order_is_stable_so_a_row_index_means_one_thing() {
    let a = complete_digraph(&[10, 20, 30]);
    let b = complete_digraph(&[10, 20, 30]);
    let key = |g: &ph2d_morph_machine::MorphGraph| -> Vec<(u64, u64)> {
        g.edges.iter().map(|e| (e.from, e.to)).collect()
    };
    assert_eq!(key(&a), key(&b));
    // E a ordem é a dos MEMBROS, não a dos ids: começar por 30 muda a lista.
    assert_ne!(key(&a), key(&complete_digraph(&[30, 20, 10])));
}

/// ⭐ **Toda transição nasce SEM condição** — existe e nunca acontece.
///
/// ⚠️ É a metade que torna o grafo completo seguro: se cada aresta nascesse com uma acção, um
/// conjunto de 9 formas nasceria com 72 regras a disparar todas na primeira tecla.
#[test]
fn every_transition_is_born_silent() {
    let g = complete_digraph(&[1, 2, 3]);
    assert!(
        g.edges.iter().all(|e| e.when.is_empty()),
        "uma seta com condicao de fabrica dispara sem ninguem a ter pedido"
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

/// ⭐⭐ **A COSTURA: o botão faz o objecto, ele fica com os filhos, e SÓ ELE aparece.**
///
/// **Mutação que deve sangrar (1):** tirar o `Visibility::hidden()` — as nove formas ficariam
/// empilhadas por cima do conjunto, que é a foto que o Enio nunca quer ver.
///
/// **Mutação que deve sangrar (2):** esconder o HOST em vez dos membros — o `visible_chain` lê os
/// ancestrais, então o conjunto inteiro sumiria do canvas.
///
/// **Mutação que deve sangrar (3):** `sources: [start, start]` virar `VecMorph::new(a, b)` (que
/// nasce em `t = 0,5`) — a primeira coisa na tela seria uma forma a meio caminho, que ninguém
/// desenhou.
#[test]
fn the_set_owns_the_shapes_hides_them_and_shows_the_start() {
    let (mut sim, mut scene, mut map, ids) = world(3);
    let mut pending = create(&sim, &mut scene, &map, &ids, 9);
    assert!(pending.is_some(), "tres formas dao um conjunto");
    // O `sync` do quadro seguinte é que faz nascer a entidade do path novo.
    sync(&mut sim, &mut scene, &mut map);
    upkeep(&mut sim, &scene, &map, &mut pending);
    assert!(pending.is_none(), "o pendente tem de ser consumido");

    let host = Entity::from_bits(map[&scene.paths().last().unwrap().id]);
    let m = sim
        .world()
        .get::<VecMorph>(host)
        .expect("o conjunto DESENHA");
    assert_eq!(
        (m.sources, m.t),
        ([ids[0], ids[0]], 0.0),
        "ele tem de mostrar EXACTAMENTE o estado inicial"
    );
    assert_eq!(
        sim.world()
            .get::<VecMorphMachine>(host)
            .expect("e tem maquina")
            .graph
            .edges
            .len(),
        6,
        "tres formas => 3x2 transicoes"
    );
    assert!(
        sim.world().get::<Visibility>(host).is_none(),
        "o CONTROLE: o conjunto NAO se esconde -- e' ele que se ve'"
    );
    for id in &ids {
        let e = Entity::from_bits(map[id]);
        assert_eq!(
            sim.world().get::<ChildOf>(e).map(ChildOf::parent),
            Some(host),
            "a forma {id} tem de ser FILHA do conjunto"
        );
        assert!(
            sim.world().get::<Visibility>(e).is_some_and(|v| v.hidden),
            "a forma {id} tem de ficar oculta -- so' o estado actual aparece"
        );
    }
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
