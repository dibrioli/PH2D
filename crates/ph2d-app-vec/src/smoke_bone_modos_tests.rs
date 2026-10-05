//! O gate da cena dos três modos (A14).

use super::*;

/// ⭐⭐ A cena monta UM esqueleto-OBJECTO com os ossos dele e uma barra SOLTA (prender é o gesto que
/// ela ensina). Controlo: depois do 2.º tempo continua nada preso.
#[test]
fn the_three_modes_scene_has_one_skeleton_object_and_a_loose_bar() {
    let mut scene = VecScene::new();
    let mut sim = SimWorld::default();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    let esq = sim
        .world()
        .iter_entities()
        .find(|er| er.contains::<ph2d_skeleton_ecs::Skeleton>())
        .map(|er| er.id())
        .expect("a cena monta um esqueleto");
    assert_eq!(scene.paths().len(), 1, "a barra");
    let ossos = ph2d_skeleton_ecs::bones_of(sim.world(), esq);
    assert_eq!(ossos.len(), OSSOS as usize, "os ossos são do objecto");
    bind(&mut st);
    let presas = sim
        .world()
        .iter_entities()
        .filter(|er| er.contains::<ph2d_skeleton_ecs::SkinBind>())
        .count();
    assert_eq!(
        presas, 0,
        "a cena prendeu a barra — o gesto que ela ensina já vinha feito"
    );
    assert_eq!(st.bone_smoke_step, 2);
}
