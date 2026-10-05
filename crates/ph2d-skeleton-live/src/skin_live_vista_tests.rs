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

/// ⭐⭐ **O esqueleto de um osso é o OBJECTO inteiro** (A14): duas raízes do mesmo esqueleto entram
/// as duas no Bind, e a recusa «vários esqueletos» conta objectos. Controlo: sem o objecto, cada
/// raiz é o seu esqueleto (e com duas, o Bind sem osso escolhido recusa).
#[test]
fn the_skeleton_of_a_bone_is_the_whole_object() {
    use crate::recusa_do_osso::recusa_do_bind;
    let mut sim = SimWorld::default();
    let a = osso(&mut sim, "A", [0.0, 0.0], 10.0, None);
    let a2 = osso(&mut sim, "A2", [10.0, 0.0], 10.0, Some(a));
    let b = osso(&mut sim, "B", [0.0, 30.0], 10.0, None);
    assert_eq!(
        skeleton_of(&sim, Some(a)).len(),
        2,
        "controlo: sem objecto, a corrente"
    );
    assert!(
        recusa_do_bind(&sim, None).is_some(),
        "controlo: duas correntes soltas"
    );

    let obj = sim.world_mut().spawn(ph2d_skeleton_ecs::Skeleton).id();
    for r in [a, b] {
        sim.world_mut().entity_mut(r).insert(ph2d_ecs::ChildOf(obj));
    }
    let mut todos = vec![a, a2, b];
    todos.sort_by_key(|e| e.to_bits());
    assert_eq!(
        skeleton_of(&sim, Some(a2)),
        todos,
        "as duas raízes do objecto"
    );
    assert_eq!(
        recusa_do_bind(&sim, None),
        None,
        "um objecto é UM esqueleto"
    );
}

/// ⭐⭐⭐ **Mover o ESQUELETO em Object leva a forma presa junto** (A14): o gizmo escreve o `Transform`
/// do objecto esqueleto, o mundo de cada osso compõe-no, e a pele segue. Controlo: antes de mover,
/// o desenho é o de depois do Bind.
#[test]
fn moving_the_skeleton_object_carries_the_bound_shape() {
    use crate::test_support::quadro;
    let (mut sim, mut scene, map, id, [raiz, _]) = super::tests::palco();
    let esq = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_skeleton_ecs::Skeleton))
        .id();
    sim.world_mut()
        .entity_mut(raiz)
        .insert(ph2d_ecs::ChildOf(esq));
    assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
    let antes = quadro(&sim, &mut scene, id);
    assert_eq!(
        quadro(&sim, &mut scene, id).verts,
        antes.verts,
        "controlo: parado"
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(esq)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(100.0, 0.0);
    let depois = quadro(&sim, &mut scene, id);
    assert_eq!(depois.verts.len(), antes.verts.len());
    for (a, d) in antes.verts.iter().zip(&depois.verts) {
        let dx = d.anchor[0] - a.anchor[0];
        let dy = d.anchor[1] - a.anchor[1];
        assert!(
            (dx - 100.0).abs() < 1e-6 && dy.abs() < 1e-6,
            "a forma presa não seguiu o esqueleto: deslocou ({dx}, {dy})"
        );
    }
}
