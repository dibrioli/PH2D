//! **A migração `219 → 220`** (GEF02 B2): a forma de cada corpo passa a uma filha «Collision Shape»,
//! e a cena simula byte a byte como antes.

use super::forma_filha::{cena, corre};
use ph2d_ecs::{ChildOf, InstanceOf, Name, ObjectInstance, OverrideKey, StableId, Transform};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, Collider, ColliderShape, LADO_FORMA, MaterialCombine, NOME_DA_FORMA,
    OneWayPlatform, RigidBody, SignalOnHit, main_shape, migra_formas_para_filhos,
    migra_formas_para_filhos_com_pares,
};

#[test]
fn a_cena_migrada_simula_igual_e_a_segunda_corrida_nao_move_nada() {
    let mut antes = cena(false);
    let mut depois = cena(false);
    let m = migra_formas_para_filhos_com_pares(depois.sim.world_mut());
    assert_eq!(m.pares.len(), 6);
    assert_eq!(
        migra_formas_para_filhos(depois.sim.world_mut()),
        0,
        "idempotente"
    );
    let w = depois.sim.world();
    for (i, &corpo) in depois.corpos.iter().enumerate() {
        let filha = main_shape(w, corpo).expect("forma");
        assert_ne!(filha, corpo, "a forma saiu do corpo");
        assert_eq!(w.get::<ChildOf>(filha).map(ChildOf::parent), Some(corpo));
        assert_eq!(
            w.get::<Name>(filha).map(|n| n.0.as_str()),
            Some(NOME_DA_FORMA)
        );
        assert_eq!(w.get::<Transform>(filha), Some(&Transform::IDENTITY));
        assert!(
            w.get::<ph2d_ecs::Visibility>(filha)
                .is_some_and(|v| v.hidden)
        );
        assert!(w.get::<Collider>(corpo).is_none());
        // Os pares vêm por ordem de `StableId` do corpo e as filhas recebem ids na mesma ordem.
        let sid = |e| w.get::<StableId>(e).unwrap().0;
        assert_eq!(m.pares[i], (sid(corpo), sid(filha)));
        assert!(w.get::<Name>(corpo).is_some(), "o nome do corpo fica");
    }
    // O lado-forma inteiro foi com ela (a borda, o chão, a zona).
    let f = |i: usize| main_shape(w, depois.corpos[i]).unwrap();
    assert!(w.get::<OneWayPlatform>(f(1)).is_some());
    assert!(w.get::<OneWayPlatform>(depois.corpos[1]).is_none());
    assert!(w.get::<MaterialCombine>(f(0)).is_some());
    assert!(w.get::<MaterialCombine>(depois.corpos[0]).is_none());
    assert!(w.get::<AreaEffector>(f(3)).is_some() && w.get::<SignalOnHit>(f(3)).is_some());
    assert!(w.get::<AreaEffector>(depois.corpos[3]).is_none());
    let (ha, _) = corre(&mut antes);
    let (hb, _) = corre(&mut depois);
    assert_eq!(ha, hb, "a migração não muda a simulação");
}

fn caixa(h: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: h,
            half_y: h,
        },
        ..Collider::default()
    }
}

/// A filha nasce PRIMEIRA entre as irmãs: um corpo que já tinha uma peça continua com a sua forma
/// como principal.
#[test]
fn a_forma_migrada_e_a_primeira_irma_e_a_principal() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let w = sim.world_mut();
    let corpo = w
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            caixa(1.0),
            Transform::IDENTITY,
        ))
        .id();
    let peca = w
        .spawn((caixa(0.2), Transform::IDENTITY, ChildOf(corpo)))
        .id();
    ph2d_ecs::assign_missing_sibling_order(w);
    assert_eq!(migra_formas_para_filhos(w), 1);
    let filhos = ph2d_ecs::ordered_children(w, corpo);
    assert_eq!(filhos.len(), 2);
    assert_eq!(filhos[1], peca);
    assert_eq!(main_shape(w, corpo), Some(filhos[0]));
    assert!(matches!(
        w.get::<Collider>(filhos[0]).unwrap().shape,
        ColliderShape::Cuboid { half_x, .. } if half_x == 1.0
    ));
}

/// ⛔ Instâncias: a peça de uma instância nunca é migrada (a reconciliação materializa a filha do
/// mestre — duplicaria); um corpo do mestre cujo lado-forma uma instância sobrepõe fica e é contado.
#[test]
fn a_instancia_nao_ganha_forma_e_o_override_do_lado_forma_recusa_o_mestre() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let w = sim.world_mut();
    let mestre = w
        .spawn((
            RigidBody::default(),
            caixa(1.0),
            Transform::IDENTITY,
            StableId(10),
        ))
        .id();
    let outro = w
        .spawn((
            RigidBody::default(),
            caixa(1.0),
            Transform::IDENTITY,
            StableId(11),
        ))
        .id();
    let inst = w
        .spawn((
            RigidBody::default(),
            caixa(1.0),
            Transform::IDENTITY,
            StableId(20),
            InstanceOf { master: 10 },
        ))
        .id();
    let mut ov = ObjectInstance::default();
    ov.overrides.insert(OverrideKey {
        piece: 11,
        type_id: ph2d_ecs::scene::registry::stable_type_id("ph2d::physics::Collider"),
    });
    w.entity_mut(inst).insert(ov);
    let m = migra_formas_para_filhos_com_pares(w);
    assert_eq!(m.recusados, 1);
    assert_eq!(m.pares.len(), 1);
    assert_eq!(m.pares[0].0, 10);
    assert_ne!(main_shape(w, mestre), Some(mestre));
    assert_eq!(main_shape(w, outro), Some(outro), "recusado: fica no corpo");
    assert_eq!(main_shape(w, inst), Some(inst), "a instância não é tocada");
    assert!(w.get::<bevy_ecs::hierarchy::Children>(inst).is_none());
}

/// A fonte única cobre o catálogo: toda linha do cartão da forma é lado-forma.
#[test]
fn o_lado_forma_cobre_o_catalogo() {
    let linhas: Vec<&str> = ph2d_component_desc::catalog::all()
        .filter(|d| d.row_of == "ph2d::physics::Collider")
        .map(|d| d.canonical_name)
        .collect();
    assert!(linhas.len() >= 11, "controlo positivo: {linhas:?}");
    for n in linhas {
        assert!(LADO_FORMA.contains(&n), "{n} é linha da forma e não migra");
    }
    for n in LADO_FORMA {
        assert!(
            ph2d_component_desc::catalog::desc_for(n).is_some(),
            "{n} não está no catálogo"
        );
    }
}
