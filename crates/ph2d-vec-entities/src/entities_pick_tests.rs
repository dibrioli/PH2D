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
        editing: vec![a],
        ..Default::default()
    };
    let v = view_state_for_pick(&SimWorld::new(), &VecEntityMap::new(), &drawn);
    assert_eq!(v.editing, vec![a]);
    assert!(
        v.is_pickable(a) && !v.is_pickable(b),
        "a outra forma agarra-se em Edit"
    );
}
