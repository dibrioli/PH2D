use super::*;

/// ⭐ **O Model do menu nasce como objecto Model** — uma raiz com `FieldObject` e a esfera
/// dentro — e um segundo pedido é RECUSADO com a razão, nunca uma raiz morta.
///
/// (Mutação: tirar a guarda do `why_not` no `add` ⇒ RED.)
#[test]
fn the_menu_model_is_born_once_and_the_second_is_refused() {
    let mut sim = SimWorld::new();
    assert_eq!(why_not(&mut sim), None);
    let bits = add(&mut sim).expect("o primeiro nasce");
    let world = sim.world();
    let root = bevy_ecs::entity::Entity::from_bits(bits);
    assert!(
        world.get::<FieldObject>(root).is_some(),
        "a raiz não é um Model"
    );
    let leaves = ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter(|(e, _)| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e),
                Some(ph2d_field_ecs::FieldNode {
                    shape: ph2d_field::NodeShape::Leaf(_)
                })
            )
        })
        .count();
    assert_eq!(leaves, 1, "o Model novo nasce com UMA forma");

    let reason = add(&mut sim).expect_err("o segundo seria uma raiz que ninguém coze");
    assert_eq!(Some(reason), why_not(&mut sim));
    assert_ne!(
        reason, "object_add.model.one_per_scene",
        "a razão não foi traduzida"
    );
    let mut q = sim
        .world_mut()
        .query::<(bevy_ecs::entity::Entity, &FieldObject)>();
    assert_eq!(q.iter(sim.world()).count(), 1);
}
