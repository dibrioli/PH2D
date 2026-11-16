use ph2d_ecs::{ChildOf, Entity, Locked, SimWorld, Transform, is_locked_for_edit, pick_target};

fn arte(sim: &mut SimWorld, pai: Option<Entity>) -> Entity {
    let w = sim.world_mut();
    let mut e = w.spawn((
        Transform::IDENTITY,
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
    ));
    if let Some(p) = pai {
        e.insert(ChildOf(p));
    }
    e.id()
}

/// ⭐ **O cadeado de uma pasta COMUM tranca o conteúdo** (dono 15/11): um vazio com dois filhos com
/// arte; trancado, nenhum se arrasta nem se escolhe no canvas; destrancado, os dois voltam. Um
/// objecto COM arte trancado continua a trancar só a si (26/05).
/// **Mutações que sangram:** o `sync` sem o `insert` · `is_folder` sem o `Empty`.
#[test]
fn locking_a_common_folder_locks_what_is_inside() {
    let registry = crate::component_registry_for_tests::registo();
    let mut sim = SimWorld::new();
    let pasta = crate::object_add::spawn_empty_root(&mut sim, "Folder");
    let pasta = Entity::from_bits(pasta);
    let filhos = [arte(&mut sim, Some(pasta)), arte(&mut sim, Some(pasta))];
    super::sync(&mut sim, &registry);
    for f in filhos {
        assert!(!is_locked_for_edit(sim.world(), f), "unlocked moves");
        assert_eq!(pick_target(sim.world(), f), Some(f));
    }
    sim.world_mut().entity_mut(pasta).insert(Locked);
    assert_eq!(super::sync(&mut sim, &registry), 1);
    assert_eq!(super::sync(&mut sim, &registry), 0, "idempotent");
    for f in filhos {
        assert!(is_locked_for_edit(sim.world(), f), "the lock covers it");
        assert_eq!(pick_target(sim.world(), f), None, "a click cannot take it");
    }
    sim.world_mut().entity_mut(pasta).remove::<Locked>();
    super::sync(&mut sim, &registry);
    for f in filhos {
        assert!(!is_locked_for_edit(sim.world(), f), "unlocked again");
        assert_eq!(pick_target(sim.world(), f), Some(f));
    }
    // Objecto com arte: o cadeado trava só ele.
    let sprite = arte(&mut sim, None);
    let filho = arte(&mut sim, Some(sprite));
    sim.world_mut().entity_mut(sprite).insert(Locked);
    super::sync(&mut sim, &registry);
    assert!(is_locked_for_edit(sim.world(), sprite));
    assert!(
        !is_locked_for_edit(sim.world(), filho),
        "its child stays editable"
    );
    assert_eq!(pick_target(sim.world(), filho), Some(filho));
}
