//! O gate de [`super::visible_bone_polylines`] — o olho da Hierarquia esconde os ossos.

use super::tests::osso;
use super::*;
use ph2d_ecs::{ChildOf, Name, Visibility};

fn bits(v: &[(u64, Vec<[f64; 2]>)]) -> Vec<u64> {
    v.iter().map(|(b, _)| *b).collect()
}

/// ⭐⭐ **Fechar o olho de um ANCESTRAL esconde a corrente inteira dele; a outra fica.**
/// Controlo: com os olhos abertos os quatro ossos estão à vista.
#[test]
fn a_closed_eye_on_an_ancestor_hides_its_whole_chain_and_only_it() {
    let mut sim = SimWorld::default();
    let grupo = sim.world_mut().spawn(Name::new("Group")).id();
    let a = osso(&mut sim, "A", [0.0, 0.0], 10.0, None);
    let a2 = osso(&mut sim, "A2", [10.0, 0.0], 10.0, Some(a));
    sim.world_mut().entity_mut(a).insert(ChildOf(grupo));
    let b = osso(&mut sim, "B", [0.0, 50.0], 10.0, None);
    let b2 = osso(&mut sim, "B2", [10.0, 0.0], 10.0, Some(b));
    let mut todos = vec![a.to_bits(), a2.to_bits(), b.to_bits(), b2.to_bits()];
    todos.sort_unstable();

    assert_eq!(
        bits(&visible_bone_polylines(&sim)),
        todos,
        "controlo: olhos abertos"
    );

    sim.world_mut()
        .entity_mut(grupo)
        .insert(Visibility::hidden());
    let mut so_b = vec![b.to_bits(), b2.to_bits()];
    so_b.sort_unstable();
    assert_eq!(
        bits(&visible_bone_polylines(&sim)),
        so_b,
        "o grupo escondido leva A e A2"
    );
    assert_eq!(
        bone_polylines(&sim).len(),
        4,
        "a cinemática continua a ver os quatro"
    );

    sim.world_mut().entity_mut(b2).insert(Visibility::hidden());
    assert_eq!(
        bits(&visible_bone_polylines(&sim)),
        vec![b.to_bits()],
        "o olho do próprio osso"
    );
}
