//! Os gates das duas perguntas do esqueleto (A14).

use super::*;
use crate::Bone;

/// ⭐ Um osso sobe a corrente até ao esqueleto; os ossos de um esqueleto não incluem os de outro
/// nem os soltos. Controlo: o osso solto não tem dono.
#[test]
fn a_bone_climbs_to_its_skeleton_and_only_its_bones_answer() {
    let mut world = World::new();
    let esq = world.spawn(Skeleton).id();
    let raiz = world.spawn((Bone::default(), ChildOf(esq))).id();
    let ponta = world.spawn((Bone::default(), ChildOf(raiz))).id();
    let outro = world.spawn(Skeleton).id();
    let do_outro = world.spawn((Bone::default(), ChildOf(outro))).id();
    let solto = world.spawn(Bone::default()).id();

    assert_eq!(skeleton_of(&world, ponta), Some(esq));
    assert_eq!(
        skeleton_of(&world, esq),
        Some(esq),
        "o esqueleto é dono de si"
    );
    assert_eq!(skeleton_of(&world, solto), None, "controlo: o osso solto");
    let mut esperado = vec![raiz, ponta];
    esperado.sort_by_key(|e| e.to_bits());
    assert_eq!(bones_of(&world, esq), esperado);
    assert_eq!(bones_of(&world, outro), vec![do_outro]);
}
