use super::*;
use ph2d_field_ecs::FieldObject;

fn leaves(world: &bevy_ecs::world::World, root: bevy_ecs::entity::Entity) -> usize {
    ph2d_field_ecs::walk(world, root)
        .into_iter()
        .filter(|(e, _)| {
            matches!(
                world.get::<ph2d_field_ecs::FieldNode>(*e),
                Some(ph2d_field_ecs::FieldNode {
                    shape: ph2d_field::NodeShape::Leaf(_)
                })
            )
        })
        .count()
}

/// ⭐ **Dois Models do menu são duas peças** (spec/06 F3) — cada uma com `FieldObject` e a esfera
/// dela, com nomes que a Hierarquia distingue, UMA luz na cena, e a nascida pede o Edit.
///
/// (Mutações: plantar a 2.ª dentro da 1.ª, ou acender uma luz por peça ⇒ RED.)
#[test]
fn two_menu_models_are_two_pieces_with_one_light() {
    crate::model_mode::forget();
    let mut sim = SimWorld::new();
    let a = bevy_ecs::entity::Entity::from_bits(add(&mut sim));
    let b = bevy_ecs::entity::Entity::from_bits(add(&mut sim));
    assert_ne!(a, b);
    let world = sim.world();
    for root in [a, b] {
        assert!(
            world.get::<FieldObject>(root).is_some(),
            "a raiz não é um Model"
        );
        assert_eq!(leaves(world, root), 1, "um Model novo nasce com UMA forma");
    }
    let name = |e| world.get::<ph2d_ecs::Name>(e).map(|n| n.0.clone());
    assert_ne!(name(a), name(b), "duas linhas «Model» iguais na Hierarquia");
    let mut lights = sim.world_mut().query::<&ph2d_field_ecs::FieldLight>();
    assert_eq!(
        lights.iter(sim.world()).count(),
        1,
        "uma luz por CENA, não por peça"
    );
    let mut family = crate::model_mode::Family::new(&mut sim, false);
    let mut tools = ph2d_editor_core::ToolRegistry::new();
    assert_eq!(
        ph2d_editor_core::screens::hero::mode_drive::ModeFamily::wants(&mut family, &mut tools),
        Some((b.to_bits(), ph2d_editor_core::object_mode::ObjectMode::Edit)),
        "a peça nascida pede o Edit"
    );
}
