//! Gate do duplicar ao lado da original.

use super::super::{bits, group_entities, setup, sync};
use super::*;
use ph2d_vec_scene::rectangle;

/// ⭐⭐ GATE (escolha do dono, 04/10: *«duplicar e permanecer como filho do mesmo pai»*) — **a cópia
/// de uma forma nasce IRMÃ da original**, logo a seguir a ela entre as irmãs, e não solta na raiz.
#[test]
fn a_duplicated_shape_stays_beside_its_source() {
    let (mut sim, mut scene, mut map) = setup();
    let a = scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    let b = scene.push_path(rectangle([3.0, 0.0], [4.0, 1.0]));
    sync(&mut sim, &mut scene, &mut map);
    let (ea, eb) = (bits(&map, a), bits(&map, b));
    let g = group_entities(&mut sim, &[ea.to_bits(), eb.to_bits()], "Group".into())
        .map(Entity::from_bits)
        .expect("o grupo");
    ph2d_ecs::assign_missing_sibling_order(sim.world_mut());
    let clip = scene.copy_paths(&[a]);
    let copies = scene.paste_clip(&clip, 0.5, 0.0);
    place_beside(&mut sim, &mut scene, &mut map, &copies, ea);
    let ec = bits(&map, copies[0]);
    let parent = |e| sim.world().get::<ChildOf>(e).map(ChildOf::parent);
    assert_eq!(parent(ec), Some(g), "a cópia saiu do pai da original");
    let order = |e| sim.world().get::<SiblingOrder>(e).expect("ordem").0;
    assert!(
        order(ea) < order(ec) && order(ec) < order(eb),
        "a cópia não ficou ao lado"
    );
}
