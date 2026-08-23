//! **Os gates do REALCE DE PROVENIÊNCIA** (estudo de UI viva, C2).
//!
//! O que só se pode afirmar aqui é a metade que não precisa de janela: *que geometria o contorno
//! desenha, dado um objecto*. A precedência (painel vs canvas) vive no
//! [`crate::App::pick_hovered_object`], que precisa de uma `HeroScreen` com store — há gate de
//! FONTE sobre ela em `tests/the_highlight_has_one_source.rs`.

use super::hover_outline_world;
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Entity, SimWorld, VecBoolGroup};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecScene, VecXforms, rectangle};

/// A cena da booleana viva: o de fora e o de dentro, agrupados em `op`, já cozidos.
fn cooked_donut(op: u8) -> (SimWorld, VecScene, VecEntityMap, LiveGeometry, [u64; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let outer = scene.push_path(rectangle([0.0, 0.0], [20.0, 20.0]));
    let inner = scene.push_path(rectangle([6.0, 6.0], [14.0, 14.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let g = Entity::from_bits(
        crate::vec_entities::group_entities(&mut sim, &[map[&outer], map[&inner]], "Bool".into())
            .unwrap(),
    );
    sim.world_mut().entity_mut(g).insert(VecBoolGroup { op });
    let mut live = LiveGeometry::new();
    crate::bool_live::BoolLive::default().recook(
        &scene,
        &sim,
        &map,
        &VecXforms::default(),
        &[],
        &mut live,
    );
    let ops = [map[&outer], map[&inner]];
    (sim, scene, map, live, ops)
}

fn area(items: &[VecPath]) -> f64 {
    items.iter().map(|p| ph2d_vec_boolean::area(p).abs()).sum()
}

/// ⭐ **UM OPERANDO ABSORVIDO CONTORNA A PEGADA PRÓPRIA, e não o vazio.**
///
/// ⚠️ É o caso que a feature existe para servir, e o que a implementação ingénua erra: numa
/// booleana viva o resultado pousa na BASE e cada outro operando recebe uma entrada **VAZIA** no
/// mapa vivo. Contornar essa entrada desenha **nada** — justamente na cena em que o artista mais
/// precisa de saber qual das cinco formas é aquela linha da Hierarquia.
#[test]
fn an_absorbed_operand_outlines_its_own_footprint_not_the_void() {
    let (sim, scene, map, live, ops) = cooked_donut(0);
    // Pré-condição: o mapa vivo do operando absorvido está de facto VAZIO — sem isto o gate
    // mediria uma cena onde o fenómeno não existe.
    let inner_id = scene.paths()[1].id;
    assert!(
        live.get(&inner_id).is_some_and(|v| v.is_empty()),
        "a fixture não absorveu nada: o gate não prova o caso que ele existe para provar"
    );

    let world = hover_outline_world(&sim, &scene, &map, &live, ops[1]);
    assert!(
        !world.is_empty(),
        "o operando absorvido não contornou nada — a linha da Hierarquia acenderia e o canvas \
         ficaria mudo"
    );
    assert!(
        (area(&world) - 64.0).abs() < 1.0,
        "o contorno não é a pegada PRÓPRIA da forma (64,0): mediu {:.2}",
        area(&world)
    );
}

/// **E a BASE contorna o que ela DESENHA** — o resultado do grupo, não o retângulo dela.
///
/// ⚠️ O par com o gate acima é a lei inteira: *o que o mapa vivo diz, ou — se ele nada diz — a
/// pegada própria*. Sem esta metade, a regra podia ser *"sempre a pegada própria"*, e o contorno
/// de uma forma com offset vivo mentiria sobre o que está na tela.
///
/// ⚠️⚠️ **O grupo está em `Subtract`, e a escolha é a metade do gate.** Em `Union` a pegada
/// própria da base (o retângulo de fora, 400) e o que o grupo desenha (a união, 400) medem **o
/// mesmo** — e a mutação que troca a regra por *"sempre a pegada própria"* **sobreviveu** a este
/// gate escrito assim. Com `Subtract` o desenho tem um buraco de 64 e as duas respostas separam-se.
/// *Uma fixture que não separa os dois casos é um gate que não decide nada.*
#[test]
fn the_base_outlines_what_it_draws() {
    const SUBTRACT: u8 = 1;
    let (sim, scene, map, live, ops) = cooked_donut(SUBTRACT);
    let world = hover_outline_world(&sim, &scene, &map, &live, ops[0]);
    assert!(
        (area(&world) - 336.0).abs() < 1.0,
        "a base tinha de contornar o que o GRUPO desenha — a subtração, 336,0 (a pegada própria \
         dela mede 400,0): mediu {:.2}",
        area(&world)
    );
}

/// **Um objecto sem forma nenhuma contorna nada** — e não entra em pânico.
#[test]
fn an_object_with_no_shape_outlines_nothing() {
    let (sim, scene, map, live, _ops) = cooked_donut(0);
    let ghost = sim.world().entities().len() + 1000;
    let world = hover_outline_world(
        &sim,
        &scene,
        &map,
        &live,
        Entity::from_bits(u64::from(ghost)).to_bits(),
    );
    assert!(world.is_empty(), "um objecto que não existe contornou algo");
}
