//! Gates da vista do CLIQUE (`view_state_for_pick`) — a fusão do mundo deste instante com o que o
//! quadro desenhou.

use super::*;

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector) — **o Edit chega ao clique**: as formas do modo são do quadro (o
/// mundo não as sabe), e a vista do gesto tem de as levar — sem elas, em Edit o clique agarraria
/// outra forma (todas as ferramentas do Edit perguntam ao `is_pickable` desta vista).
#[test]
fn the_pick_view_carries_the_shapes_of_the_edit() {
    let mut scene = ph2d_vec_scene::VecScene::new();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(ph2d_vec_scene::rectangle([2.0, 0.0], [3.0, 1.0]));
    let drawn = VecViewState {
        editing: Some(vec![a]),
        ..Default::default()
    };
    let v = view_state_for_pick(&SimWorld::new(), &VecEntityMap::new(), &drawn);
    assert_eq!(v.editing, Some(vec![a]));
    assert!(
        v.is_pickable(a) && !v.is_pickable(b),
        "a outra forma agarra-se em Edit"
    );
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector) — **o Edit de um objecto VAZIO não é Object**: o objecto nasce
/// sem formas e em Edit (*Add ▸ Vector Object*), e nesse instante nenhuma forma de FORA se agarra.
/// Com a lista vazia a ler-se como «Object», todas as formas da cena se agarravam e mostravam nós.
#[test]
fn an_edit_with_no_shapes_yet_grabs_nothing_from_outside() {
    let mut scene = ph2d_vec_scene::VecScene::new();
    let a = scene.push_path(ph2d_vec_scene::rectangle([0.0, 0.0], [1.0, 1.0]));
    let object = VecViewState::default();
    assert!(
        object.is_pickable(a),
        "controlo: em Object a forma agarra-se"
    );
    let empty_edit = VecViewState {
        editing: Some(Vec::new()),
        ..Default::default()
    };
    assert!(!empty_edit.is_pickable(a) && !empty_edit.in_edit(a));
}
