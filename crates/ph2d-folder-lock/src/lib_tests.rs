use ph2d_ecs::{
    ChildOf, Entity, Locked, Name, SimWorld, Transform, World, is_locked_for_edit, pick_target,
};
use ph2d_editor_core::hier_types::ObjectType;

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

/// O tipo da linha como a shell o dá, reduzido ao que este gate precisa: com Sprite é Image, sem
/// arte é Empty — e a lei de pasta é a do `ObjectType::is_folder`.
fn sync(sim: &mut SimWorld) -> usize {
    super::sync(sim, |_| {
        |w: &World, e: Entity| {
            let t = if w.get::<ph2d_render::Sprite>(e).is_some() {
                ObjectType::Image
            } else {
                ObjectType::Empty
            };
            t.is_folder()
        }
    })
}

/// ⭐ **O cadeado de uma pasta COMUM tranca o conteúdo** (dono 15/11): um vazio com dois filhos com
/// arte; trancado, nenhum se arrasta nem se escolhe no canvas; destrancado, os dois voltam. Um
/// objecto COM arte trancado continua a trancar só a si (26/05).
/// **Mutações que sangram:** o `sync` sem o `insert` · `is_folder` sem o `Empty`.
#[test]
fn locking_a_common_folder_locks_what_is_inside() {
    let mut sim = SimWorld::new();
    let pasta = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Folder")))
        .id();
    let filhos = [arte(&mut sim, Some(pasta)), arte(&mut sim, Some(pasta))];
    sync(&mut sim);
    for f in filhos {
        assert!(!is_locked_for_edit(sim.world(), f), "unlocked moves");
        assert_eq!(pick_target(sim.world(), f), Some(f));
    }
    sim.world_mut().entity_mut(pasta).insert(Locked);
    assert_eq!(sync(&mut sim), 1);
    assert_eq!(sync(&mut sim), 0, "idempotent");
    for f in filhos {
        assert!(is_locked_for_edit(sim.world(), f), "the lock covers it");
        assert_eq!(pick_target(sim.world(), f), None, "a click cannot take it");
    }
    sim.world_mut().entity_mut(pasta).remove::<Locked>();
    sync(&mut sim);
    for f in filhos {
        assert!(!is_locked_for_edit(sim.world(), f), "unlocked again");
        assert_eq!(pick_target(sim.world(), f), Some(f));
    }
    // Objecto com arte: o cadeado trava só ele.
    let sprite = arte(&mut sim, None);
    let filho = arte(&mut sim, Some(sprite));
    sim.world_mut().entity_mut(sprite).insert(Locked);
    sync(&mut sim);
    assert!(is_locked_for_edit(sim.world(), sprite));
    assert!(
        !is_locked_for_edit(sim.world(), filho),
        "its child stays editable"
    );
    assert_eq!(pick_target(sim.world(), filho), Some(filho));
}
