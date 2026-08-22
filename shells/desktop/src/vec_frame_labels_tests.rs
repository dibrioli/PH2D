//! Os gates de *quem é nomeado, e onde*.

use super::*;
use crate::vec_entities::{VecEntityMap, sync};
use ph2d_ecs::{Entity, VecClipContent, VecFrame};
use ph2d_vec_scene::rectangle;

/// Uma cena com uma moldura, uma forma comum, e o mapa.
fn scene() -> (SimWorld, VecScene, VecEntityMap, VecPathId, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame = scene.push_path(rectangle([1.0, 2.0], [5.0, 6.0]));
    let plain = scene.push_path(rectangle([20.0, 0.0], [21.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let fe = Entity::from_bits(map[&frame]);
    sim.world_mut()
        .entity_mut(fe)
        .insert((VecFrame, VecClipContent));
    sim.world_mut().entity_mut(fe).insert(Name::new("Capa"));
    (sim, scene, map, frame, plain)
}

/// **Só molduras são nomeadas**, e o canto é o TOPO-esquerdo do mundo (`min_x`, `max_y`).
#[test]
fn only_frames_get_a_label_and_it_sits_at_the_world_top_left() {
    let (sim, scene, map, _frame, _plain) = scene();
    let xf = VecXforms::default();
    let ls = frame_labels(&sim, &scene, &map, &xf, &[]);

    assert_eq!(ls.len(), 1, "a forma comum não é uma moldura");
    assert_eq!(ls[0].name, "Capa", "o nome vem do `Name` da entidade");
    assert!(
        (ls[0].world_top_left[0] - 1.0).abs() < 1e-9,
        "x é o MENOR ({:?})",
        ls[0].world_top_left
    );
    assert!(
        (ls[0].world_top_left[1] - 6.0).abs() < 1e-9,
        "y é o MAIOR — o mundo é Y-up, então o topo é o máximo ({:?})",
        ls[0].world_top_left
    );
    assert!(!ls[0].selected);
}

/// A moldura selecionada é MARCADA — é o que dá à etiqueta a cor de destaque.
#[test]
fn the_selected_frame_is_flagged() {
    let (sim, scene, map, frame, plain) = scene();
    let xf = VecXforms::default();
    assert!(frame_labels(&sim, &scene, &map, &xf, &[frame])[0].selected);
    assert!(
        !frame_labels(&sim, &scene, &map, &xf, &[plain])[0].selected,
        "selecionar OUTRA coisa não pode acender a etiqueta da moldura"
    );
}

/// Uma moldura sem nome ainda é identificável — a etiqueta existe **para** dizer que aquilo é uma
/// moldura, e some-la sobre uma sem `Name` esconderia justamente a que o artista não reconhece.
#[test]
fn an_unnamed_frame_still_gets_a_label() {
    let (mut sim, scene, map, frame, _plain) = scene();
    let fe = Entity::from_bits(map[&frame]);
    sim.world_mut().entity_mut(fe).remove::<Name>();
    let ls = frame_labels(&sim, &scene, &map, &VecXforms::default(), &[]);
    assert_eq!(ls.len(), 1);
    assert!(!ls[0].name.is_empty(), "um nome vazio desenharia nada");
}
