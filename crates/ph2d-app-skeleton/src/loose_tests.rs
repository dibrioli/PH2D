//! O gate da porta das raízes soltas (A14).

use super::*;

fn osso(sim: &mut SimWorld, pos: [f32; 2], rot: f32, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                rotation: rot,
                ..Transform::IDENTITY
            },
            Bone {
                length: 10.0,
                ..Bone::default()
            },
        ))
        .id();
    match pai {
        Some(p) => {
            sim.world_mut().entity_mut(e).insert(ChildOf(p));
        }
        None => {
            sim.world_mut().entity_mut(e).insert(RootOrder(3));
        }
    }
    e
}

/// ⭐⭐⭐ **Um projecto de antes do A14 abre com um esqueleto por raiz, e a pose AO BIT** — duas
/// correntes soltas (uma de topo, outra dentro de um grupo posado) e uma que já tem esqueleto.
/// Controlo: a 2.ª passagem não cria nada, e a corrente que já tinha esqueleto fica onde estava.
#[test]
fn an_old_project_opens_with_one_skeleton_per_root_and_the_pose_to_the_bit() {
    let mut sim = SimWorld::default();
    let a = osso(&mut sim, [3.0, 4.0], 0.7, None);
    let _a2 = osso(&mut sim, [10.0, 0.0], -1.3, Some(a));
    let grupo = sim
        .world_mut()
        .spawn(Transform {
            translation: ph2d_core::Vec2::new(-7.0, 2.5),
            rotation: 0.4,
            ..Transform::IDENTITY
        })
        .id();
    let b = osso(&mut sim, [1.0, 1.0], 2.1, Some(grupo));
    let _b2 = osso(&mut sim, [10.0, 0.0], 0.9, Some(b));
    let ja = sim.world_mut().spawn((Transform::IDENTITY, Skeleton)).id();
    let c = osso(&mut sim, [0.0, 0.0], 0.0, Some(ja));
    let antes = ph2d_skeleton_live::skin_live::bone_polylines(&sim);

    let novos = adopt_loose_roots(&mut sim);
    assert_eq!(novos.len(), 2, "uma raiz solta, um esqueleto");
    let w = sim.world();
    let dono = |e: Entity| ph2d_skeleton_ecs::skeleton_of(w, e).map(Entity::to_bits);
    assert!(novos.contains(&dono(a).expect("A ganhou esqueleto")));
    assert!(novos.contains(&dono(b).expect("B ganhou esqueleto")));
    assert_eq!(
        dono(c),
        Some(ja.to_bits()),
        "controlo: a que já tinha esqueleto"
    );
    let esq_b = Entity::from_bits(dono(b).expect("dono"));
    assert_eq!(
        w.get::<ChildOf>(esq_b).map(|p| p.parent()),
        Some(grupo),
        "o esqueleto fica no lugar da raiz (dentro do grupo)"
    );
    assert_eq!(
        w.get::<RootOrder>(Entity::from_bits(dono(a).expect("dono"))),
        Some(&RootOrder(3)),
        "o esqueleto de topo herda a ordem da raiz"
    );
    // (sobrevivente M10) a raiz deixou de ser raiz: a ordem dela fica com o esqueleto, não em dobro.
    assert_eq!(
        w.get::<RootOrder>(a),
        None,
        "a raiz adoptada guardou uma RootOrder obsoleta"
    );
    assert_eq!(
        ph2d_skeleton_live::skin_live::bone_polylines(&sim),
        antes,
        "a pose de algum osso mudou — a migração mexeu no desenho"
    );
    assert!(
        adopt_loose_roots(&mut sim).is_empty(),
        "controlo: a 2.ª passagem não cria nada"
    );
}
