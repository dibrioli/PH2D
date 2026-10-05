//! Os gates do *Add ▸ Skeleton* (A14).

use super::*;

/// ⭐⭐ O esqueleto nasce um OBJECTO (o marcador) com UM osso filho dele, e pede o Edit. Controlo:
/// uma entrada de outra família não cria nada.
#[test]
fn the_menu_skeleton_is_born_an_object_with_one_bone_and_asks_for_edit() {
    let mut sim = SimWorld::default();
    let mut state = SkeletonState::default();
    let antes = sim.world().iter_entities().count();
    assert!(add(ph2d_editor_core::object_add::EMPTY, &mut sim, &mut state).is_none());
    assert_eq!(
        sim.world().iter_entities().count(),
        antes,
        "controlo: nada nasceu"
    );

    let bits = add(SKELETON, &mut sim, &mut state)
        .expect("é desta família")
        .expect("nasce");
    assert!(is_skeleton(&sim, bits));
    let ossos = ph2d_skeleton_ecs::bones_of(sim.world(), Entity::from_bits(bits));
    assert_eq!(ossos.len(), 1, "nasce com UM osso");
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ChildOf>(ossos[0])
            .map(ph2d_ecs::ChildOf::parent),
        Some(Entity::from_bits(bits)),
        "o 1.º osso é filho do esqueleto"
    );
    assert_eq!(state.born, Some(bits), "nasce a pedir o Edit");
    let segundo = add(SKELETON, &mut sim, &mut state)
        .expect("é desta família")
        .expect("nasce");
    let nome = |b: u64| {
        sim.world()
            .get::<Name>(Entity::from_bits(b))
            .map(|n| n.0.clone())
            .unwrap_or_default()
    };
    assert_ne!(
        nome(bits),
        nome(segundo),
        "o nome conta (Skeleton, Skeleton.001)"
    );
}
