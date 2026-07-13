//! Gate da política das alças de raio: **uma forma viva não tem quina arrastável.**
//!
//! É o gate que me faltava e que quase deixou passar um bug de "funciona e depois
//! esquece": eu havia gateado o MODO (a alça é do Node) e achado que estava coberto. Mas
//! uma forma viva **selecionada dentro do modo Node** é outra coisa — e ali a alça
//! aparecia, funcionava, e o próximo arrasto de slider do painel varria o raio, porque o
//! `recook_into` reescreve `verts` inteiro. Funcionar e depois desfazer sozinho é pior que
//! não funcionar.

use super::*;
use ph2d_ecs::{Transform, VecShape};
use ph2d_vec_scene::{VecPath, VecVertex};

/// Uma cena com um quadrado, a entidade dele, e o mapa path↔entidade.
fn square(live: bool) -> (SimWorld, VecScene, VecEntityMap, ph2d_vec_scene::VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    let mut sim = SimWorld::default();
    let e = sim.world_mut().spawn(Transform::IDENTITY).id();
    if live {
        // A RECEITA: é isto que faz o `recook_into` reescrever a geometria toda vez.
        sim.world_mut().entity_mut(e).insert(VecShape::Param {
            kind: 0,
            w: 10.0,
            h: 10.0,
            values: [0.0; ph2d_ecs::MAX_SHAPE_VALUES],
        });
    }
    let mut map = VecEntityMap::default();
    map.insert(id, e.to_bits());
    (sim, scene, map, id)
}

/// Um caminho DESENHADO (sem receita) tem alça em cada quina — quatro, no quadrado.
#[test]
fn a_drawn_path_gets_a_radius_handle_on_every_corner() {
    let (sim, scene, map, id) = square(false);
    assert!(!is_live_shape(&sim, &map, id));
    let handles = view(
        &sim,
        &scene,
        &map,
        Some(id),
        &ph2d_vec_scene::VecXforms::default(),
        0.01,
    );
    assert_eq!(handles.len(), 4, "as 4 quinas do quadrado");
}

/// **Uma FORMA VIVA não tem nenhuma.**
///
/// Não é conservadorismo: o `vec_shape_live::recook_into` substitui `path.verts` INTEIRO a
/// cada mudança de parâmetro, e o `corner_radius` mora dentro do vértice. Um raio autorado
/// aqui sobreviveria até o usuário encostar num slider — e sumiria sem erro nenhum. O raio
/// de uma forma viva é um campo DELA (o painel); o por-vértice é para caminho desenhado.
#[test]
fn a_live_shape_has_no_radius_handles_because_the_recook_would_erase_them() {
    let (sim, scene, map, id) = square(true);
    assert!(is_live_shape(&sim, &map, id), "a receita está pendurada");
    let handles = view(
        &sim,
        &scene,
        &map,
        Some(id),
        &ph2d_vec_scene::VecXforms::default(),
        0.01,
    );
    assert!(
        handles.is_empty(),
        "a forma viva NÃO pode oferecer uma alça que o recook dela vai varrer"
    );
}

/// Sem seleção, nenhuma alça (e sem varrer a cena inteira atrás delas).
#[test]
fn no_selection_no_handles() {
    let (sim, scene, map, _) = square(false);
    assert!(
        view(
            &sim,
            &scene,
            &map,
            None,
            &ph2d_vec_scene::VecXforms::default(),
            0.01
        )
        .is_empty()
    );
}
