//! **Re-parentear NÃO move a forma** — os gates da porta `reparent_keeping_world`.
//!
//! Arquivo IRMÃO por LOC (HR-18), módulo FILHO por `#[path]`. O gate central carrega o próprio
//! CONTROLE: ele prende as duas formas pelo `ChildOf` cru ANTES de usar a porta, e exige que o
//! cru MOVA a forma — sem essa metade, um gate verde não distingue *a porta funciona* de *não
//! havia nada a corrigir*.

use super::*;
use ph2d_vec_scene::rectangle;

/// Duas formas assentadas, uma dentro da outra, e a caixa de mundo de cada uma.
fn rig() -> (
    SimWorld,
    ph2d_vec_scene::VecScene,
    crate::vec_entities::VecEntityMap,
    Entity,
    Entity,
) {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let body = scene.push_path(rectangle([-9.0, -3.0], [-1.0, 3.0]));
    let bar = scene.push_path(rectangle([-7.5, 0.5], [-2.5, 2.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    // ⚠️ O assentamento é a PREMISSA: é ele que dá a cada raiz uma translação própria, e é a
    // soma dessas duas que desloca um filho pendurado a cru.
    settle_origins(&mut sim, &mut scene, &map, &[]);
    let (b, l) = (Entity::from_bits(map[&body]), Entity::from_bits(map[&bar]));
    (sim, scene, map, b, l)
}

/// A caixa de MUNDO de uma entidade-caminho.
fn world_box(
    sim: &SimWorld,
    scene: &ph2d_vec_scene::VecScene,
    map: &crate::vec_entities::VecEntityMap,
    e: Entity,
) -> ([f64; 2], [f64; 2]) {
    let id = *map.iter().find(|(_, b)| **b == e.to_bits()).unwrap().0;
    let (lo, hi) = scene.path_curve_bbox(id).unwrap();
    let x = xform_of_transform(world_transform(sim, e));
    let (a, b) = (x.apply(lo), x.apply(hi));
    (
        [a[0].min(b[0]), a[1].min(b[1])],
        [a[0].max(b[0]), a[1].max(b[1])],
    )
}

/// **Prender pela porta não move a forma — e um `ChildOf` cru move.**
///
/// ⚠️ O CONTROLE está dentro do próprio gate, e é ele que o torna afiado: sem a segunda
/// metade, um `reparent_keeping_world` que não fizesse nada de especial passaria num mundo
/// onde o pai estivesse na identidade — que é exactamente o mundo em que este defeito não
/// aparece.
#[test]
fn reparenting_through_the_door_does_not_move_the_shape() {
    let (mut sim, scene, map, body, bar) = rig();
    let before = world_box(&sim, &scene, &map, bar);
    assert!(reparent_keeping_world(&mut sim, bar, body));
    let after = world_box(&sim, &scene, &map, bar);
    for a in 0..2 {
        assert!(
            (after.0[a] - before.0[a]).abs() < 1e-4 && (after.1[a] - before.1[a]).abs() < 1e-4,
            "a porta moveu a forma: {before:?} -> {after:?}"
        );
    }

    // O CONTROLE: o mesmo parentesco a cru, no mesmo mundo.
    let (mut sim, scene, map, body, bar) = rig();
    let before = world_box(&sim, &scene, &map, bar);
    sim.world_mut()
        .entity_mut(bar)
        .insert(ph2d_ecs::ChildOf(body));
    let after = world_box(&sim, &scene, &map, bar);
    let moved = (after.0[0] - before.0[0]).abs() + (after.0[1] - before.0[1]).abs();
    assert!(
        moved > 1.0,
        "o `ChildOf` cru não moveu nada — a fixture não contém o fenómeno: {before:?} -> \
         {after:?}"
    );
}

/// **Prender a uma entidade que não existe não escreve nada** — prender sem saber pôr de volta
/// deixaria a forma num sítio que ninguém autorou.
#[test]
fn a_missing_parent_writes_nothing() {
    let (mut sim, scene, map, _body, bar) = rig();
    let before = world_box(&sim, &scene, &map, bar);
    assert!(!reparent_keeping_world(
        &mut sim,
        bar,
        Entity::from_bits(u64::MAX)
    ));
    let after = world_box(&sim, &scene, &map, bar);
    assert!((after.0[0] - before.0[0]).abs() < 1e-9);
    assert!(sim.world().get::<ph2d_ecs::ChildOf>(bar).is_none());
}
