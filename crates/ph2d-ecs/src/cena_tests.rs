use super::*;
use crate::scene::ComponentRegistry;
use crate::scene::incremental::{CaptureCache, capture_incremental};
use crate::{StableIdCounter, Transform, assign_missing_stable_ids};

/// Idempotente: a mesma entidade, com o id reservado e sem `Transform`.
#[test]
fn garante_cena_e_idempotente() {
    let mut w = World::new();
    let a = garante_cena(&mut w);
    let b = garante_cena(&mut w);
    assert_eq!(a, b);
    assert_eq!(w.get::<StableId>(a), Some(&StableId::CENA));
    assert!(
        w.get::<Transform>(a).is_none(),
        "fora da hierarquia e do save"
    );
    assert_eq!(w.query::<&Cena>().iter(&w).count(), 1);
    // Um mundo que a perdeu (o load reconstrói-o) recebe-a de volta.
    w.despawn(a);
    let c = garante_cena(&mut w);
    assert_eq!(w.get::<StableId>(c), Some(&StableId::CENA));
}

/// ⛔ O contador nunca alcança o id da Cena: a reconciliação contra o mundo não a conta, e os ids
/// novos continuam a seguir os dos objectos.
#[test]
fn o_contador_nunca_alcanca_o_id_da_cena() {
    let mut w = World::new();
    garante_cena(&mut w);
    w.spawn(Transform::IDENTITY);
    w.spawn(Transform::IDENTITY);
    assign_missing_stable_ids(&mut w);
    let next = w.resource::<StableIdCounter>().next_free();
    assert_eq!(next, StableId::FIRST + 2, "a Cena não puxou o contador");
    let ids: Vec<u64> = w
        .query::<&StableId>()
        .iter(&w)
        .map(|s| s.0)
        .filter(|&i| i != StableId::CENA.0)
        .collect();
    assert!(ids.iter().all(|&i| i < StableId::CENA.0), "{ids:?}");
    assert!(StableId::CENA.reservado() && !StableId(next).reservado());
}

/// ⛔ A Cena não vai para o ficheiro nem para o passo de undo: a captura incremental (a do undo)
/// e o save só levam os objectos.
#[test]
fn a_cena_nao_vai_para_o_ficheiro() {
    let mut w = World::new();
    w.spawn(Transform::IDENTITY);
    garante_cena(&mut w);
    let mut reg = ComponentRegistry::new();
    crate::scene::registry::register_ecs_components(&mut reg);
    let mut cache = CaptureCache::new();
    let mut out = crate::scene::WorldSnapshot::new();
    capture_incremental(&mut w, &mut cache, &reg, &mut out).expect("captura");
    assert_eq!(out.entities.len(), 1, "só o objecto");
    assert!(out.entities.iter().all(|r| r.id != StableId::CENA));
    let mut state = crate::TransformPropagationState::new(&mut w);
    let mut wl = crate::WorklistBuf::default();
    let mut snap = crate::scene::WorldSnapshot::new();
    crate::scene::world_to_snapshot(&mut w, &mut state, &mut wl, &reg, &mut snap).expect("save");
    assert_eq!(snap.entities.len(), 1);
    assert!(snap.entities.iter().all(|r| r.id != StableId::CENA));
}
