//! Os gates da PORTA de autoria do **Resize Box**.
//!
//! O que só se pode afirmar aqui: que o checkbox mostra a resposta EFETIVA (default composto com
//! override), que o clique é um TOGGLE do que se vê, que voltar ao default **DESTACA** o
//! componente, e que uma seleção múltipla não tem linha nenhuma.

use super::*;
use ph2d_ecs::VecFrame;
use ph2d_vec_scene::{VecScene, rectangle};

/// Uma moldura com um filho e uma forma solta, já sincronizadas.
/// Devolve `(sim, map, [moldura, filho, solta])`.
fn scene() -> (SimWorld, VecEntityMap, [VecPathId; 3]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kid_id = scene.push_path(rectangle([10.0, 10.0], [20.0, 20.0]));
    let loose_id = scene.push_path(rectangle([500.0, 0.0], [5.0, 5.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut().entity_mut(frame).insert(VecFrame);
    let kid = Entity::from_bits(map[&kid_id]);
    sim.world_mut()
        .entity_mut(kid)
        .insert(ph2d_ecs::ChildOf(frame));
    (sim, map, [frame_id, kid_id, loose_id])
}

fn has_override(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> bool {
    sim.world()
        .get::<VecResizeBox>(Entity::from_bits(map[&id]))
        .is_some()
}

/// **O checkbox mostra a resposta EFETIVA** — moldura e filho marcados, forma solta desmarcada,
/// sem que nenhum deles tenha componente nenhum.
#[test]
fn the_checkbox_shows_the_derived_default_with_no_component_written() {
    let (sim, map, ids) = scene();
    assert_eq!(selected_resize_box(&sim, &map, &[ids[0]]), Some(true));
    assert_eq!(selected_resize_box(&sim, &map, &[ids[1]]), Some(true));
    assert_eq!(selected_resize_box(&sim, &map, &[ids[2]]), Some(false));
    assert!(!has_override(&sim, &map, ids[0]), "nada foi escrito");
}

/// **O clique inverte o que se vê**, e o override nasce onde ele discorda do default.
#[test]
fn a_click_toggles_what_the_artist_sees() {
    let (mut sim, map, ids) = scene();
    assert!(toggle_resize_box(&mut sim, &map, &[ids[0]]));
    assert_eq!(
        selected_resize_box(&sim, &map, &[ids[0]]),
        Some(false),
        "a moldura desmarcada volta a escalar tudo"
    );
    assert!(
        has_override(&sim, &map, ids[0]),
        "a discordancia foi gravada"
    );
}

/// **Voltar ao default DESTACA o componente.**
///
/// ⚠️ É o gate que impede o `true` de viajar: uma forma marcada por já ser filha de moldura
/// levaria o valor consigo ao sair dela, e passaria a redimensionar onde o default diz escalar.
#[test]
fn returning_to_the_default_detaches_the_component() {
    let (mut sim, map, ids) = scene();
    assert!(toggle_resize_box(&mut sim, &map, &[ids[1]]));
    assert!(has_override(&sim, &map, ids[1]));
    assert!(toggle_resize_box(&mut sim, &map, &[ids[1]]));
    assert!(
        !has_override(&sim, &map, ids[1]),
        "de volta ao default, o componente sai"
    );
    assert_eq!(selected_resize_box(&sim, &map, &[ids[1]]), Some(true));
}

/// **Uma seleção múltipla não tem linha** — nem para mostrar, nem para clicar.
#[test]
fn a_multi_selection_has_no_row_and_no_click() {
    let (mut sim, map, ids) = scene();
    assert_eq!(selected_resize_box(&sim, &map, &[ids[0], ids[1]]), None);
    assert!(!toggle_resize_box(&mut sim, &map, &[ids[0], ids[1]]));
    assert_eq!(selected_resize_box(&sim, &map, &[]), None);
}

/// **A forma solta marcada mantém o override** — a outra direcção, e é ela que dá a uma forma
/// dentro de um fluxo uma caixa que o passe de layout pode medir.
#[test]
fn marking_a_loose_shape_keeps_the_override() {
    let (mut sim, map, ids) = scene();
    assert!(toggle_resize_box(&mut sim, &map, &[ids[2]]));
    assert_eq!(selected_resize_box(&sim, &map, &[ids[2]]), Some(true));
    assert!(has_override(&sim, &map, ids[2]));
}
