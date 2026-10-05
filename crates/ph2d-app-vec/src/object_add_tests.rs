use super::*;

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector) — **a entrada cria um objecto vetorial VAZIO, no sítio pedido,
/// e ele pede o Edit**: o `VecObject` (o marcador do tipo), nenhuma forma, e o `born` armado.
///
/// (Mutação: tirar o `vec.edit.born(bits)` ⇒ RED — o objecto nasceria em Object e o artista não
/// teria onde desenhar.)
#[test]
fn the_entry_creates_an_empty_vector_object_that_asks_for_edit() {
    let mut sim = SimWorld::new();
    let mut vec = crate::state::VecState::default();
    let bits = add(VECTOR_OBJECT, &mut sim, &mut vec, [10.0, 20.0]).expect("é desta família");
    let e = ph2d_ecs::Entity::from_bits(bits);
    let w = sim.world();
    assert!(w.get::<ph2d_ecs::VecObject>(e).is_some());
    assert!(w.get::<ph2d_ecs::VecPathRef>(e).is_none());
    assert!(
        w.get::<ph2d_ecs::Children>(e).is_none(),
        "nasceu com formas"
    );
    let at = w
        .get::<ph2d_ecs::Transform>(e)
        .expect("tem pose")
        .translation;
    assert_eq!((at.x, at.y), (10.0, 20.0));
    let mut tools = ph2d_editor_core::ToolRegistry::new();
    let mut family = crate::vector_mode::Family::new(&mut vec, &mut sim);
    use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
    assert_eq!(
        family.wants(&mut tools),
        Some((bits, ph2d_editor_core::object_mode::ObjectMode::Edit))
    );
}

/// **UMA entrada** (escolha do dono, 04/10), e uma de outra família não é desta — nada nasce.
#[test]
fn one_entry_and_the_entry_of_another_family_is_not_ours() {
    assert_eq!(ENTRIES, &[VECTOR_OBJECT]);
    let mut sim = SimWorld::new();
    let mut vec = crate::state::VecState::default();
    let empty = ph2d_editor_core::object_add::EMPTY;
    assert!(add(empty, &mut sim, &mut vec, [0.0; 2]).is_none());
    let mut q = sim.world_mut().query::<&ph2d_ecs::VecObject>();
    assert_eq!(q.iter(sim.world()).count(), 0, "nasceu um objecto vetorial");
}
