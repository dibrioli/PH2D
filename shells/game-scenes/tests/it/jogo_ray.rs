//! A cena 26 (`PH2D_GAME_SMOKE=26`, ficha GEF 07): o filtro do Ray Sensor — o «Eye Ray» só vê a tag Enemy e,
//! sem Walls block, vê o «Enemy» através da «Crate», de pé, no primeiro tique.

use ph2d_app_game::smoke::ray;
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_physics_ecs::{PhysicsBridge, RaySensor};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    let e = q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e);
    e.unwrap_or_else(|| panic!("{nome}"))
}

/// O que o raio `nome` vê depois de alguns tiques de pé (o herói não anda).
fn ve(sim: &mut SimWorld, tree: &ph2d_tags::TagTree, nome: &str) -> Option<Entity> {
    let raio = por_nome(sim, nome);
    let mut bridge = PhysicsBridge::new();
    bridge.set_tag_tree(tree);
    for tick in 1..=5_u64 {
        bridge.dispatch(sim, true, tick);
    }
    bridge.ray_sensor_hits().get(&raio).map(|h| h.body)
}

/// ⭐ A tag nasce na árvore do projecto e no Enemy; o Eye Ray vê o Enemy através da Crate, e com Walls block a
/// caixa tapa-o. O Wall Ray de pé não vê nada (a luz da parede só acende ao andar). (Mutação: o Eye Ray com Walls
/// block de fábrica ⇒ RED.)
#[test]
fn na_cena_26_o_eye_ray_ve_o_inimigo_atraves_da_caixa() {
    let mut sim = SimWorld::new();
    let (escolhido, tree) = crate::comp::montar_com_tags(&mut sim, ray::CENA);
    assert_eq!(por_nome(&mut sim, ray::RAIO_PAREDE), escolhido, "o Wall Ray continua escolhido");
    let tag = tree.find(ray::TAG_INIMIGO).expect("a tag está na árvore do projecto");
    let inimigo = por_nome(&mut sim, ray::INIMIGO);
    let tags = sim.world().get::<ph2d_ecs::tags::Tags>(inimigo).expect("o Enemy tem tags");
    assert!(ph2d_ecs::tags::belongs(tags, &tree, tag));
    let olho = por_nome(&mut sim, ray::RAIO_OLHO);
    let r = *sim.world().get::<RaySensor>(olho).expect("Eye Ray");
    assert_eq!(r.see_tag, tag.0);
    assert!(!r.walls_block);
    assert_eq!(ve(&mut sim, &tree, ray::RAIO_OLHO), Some(inimigo), "vê através da caixa");
    assert_eq!(ve(&mut sim, &tree, ray::RAIO_PAREDE), None, "a parede está longe");

    let mut sim = SimWorld::new();
    let (_, tree) = crate::comp::montar_com_tags(&mut sim, ray::CENA);
    let olho = por_nome(&mut sim, ray::RAIO_OLHO);
    if let Some(mut r) = sim.world_mut().get_mut::<RaySensor>(olho) {
        r.walls_block = true;
    }
    assert_eq!(ve(&mut sim, &tree, ray::RAIO_OLHO), None, "com Walls block a caixa tapa");
}
