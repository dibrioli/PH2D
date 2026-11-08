//! A cena 120 migrada (`219 → 220`, GEF02 B2): cada forma passa a filha «Collision Shape», os mesmos
//! corpos, nomes e camadas, e a mesma simulação de 240 tiques. O arquivo fica em `project_formas_tests`.

use super::*;
use ph2d_ecs::{SimWorld, StableId};
use ph2d_physics_ecs::{Collider, PhysicsBridge, RigidBody, main_shape, migra_formas_para_filhos};

fn cena(migrada: bool) -> SimWorld {
    let mut sim = SimWorld::new();
    monta(sim.world_mut());
    if migrada {
        assert!(migra_formas_para_filhos(sim.world_mut()) >= 5);
    }
    sim
}

/// Os corpos por `StableId`: (id, nome, camada da forma principal, a forma é filha).
fn corpos(sim: &mut SimWorld) -> Vec<(u64, String, u8, bool)> {
    let w = sim.world_mut();
    let es: Vec<_> = w
        .query_filtered::<Entity, bevy_ecs::query::With<RigidBody>>()
        .iter(w)
        .collect();
    let mut v: Vec<_> = es
        .into_iter()
        .filter_map(|e| {
            let m = main_shape(w, e)?;
            Some((
                w.get::<StableId>(e).unwrap().0,
                w.get::<Name>(e).map_or_else(String::new, |n| n.0.clone()),
                w.get::<Collider>(m).unwrap().layer,
                m != e,
            ))
        })
        .collect();
    v.sort_by_key(|c| c.0);
    v
}

fn hash(sim: &mut SimWorld) -> u64 {
    let mut bridge = PhysicsBridge::new();
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    for t in 1..=240 {
        bridge.dispatch(sim, true, t);
        let w = sim.world_mut();
        let mut q: Vec<_> = w
            .query_filtered::<(&StableId, &ph2d_ecs::Transform), bevy_ecs::query::With<RigidBody>>()
            .iter(w)
            .map(|(s, t)| (s.0, *t))
            .collect();
        q.sort_by_key(|p| p.0);
        for (_, tr) in q {
            for v in [tr.translation.x, tr.translation.y, tr.rotation] {
                h = (h ^ u64::from(v.to_bits())).wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    h
}

#[test]
fn a_cena_120_migrada_tem_as_formas_filhas_e_simula_igual() {
    let (mut s219, mut s220) = (cena(false), cena(true));
    let (antes, depois) = (corpos(&mut s219), corpos(&mut s220));
    assert!(
        antes.len() >= 5 && antes.iter().all(|c| !c.3),
        "controlo: {antes:?}"
    );
    assert!(
        depois.iter().all(|c| c.3),
        "toda forma passou a filha: {depois:?}"
    );
    let sem_flag = |v: &[(u64, String, u8, bool)]| {
        v.iter()
            .map(|c| (c.0, c.1.clone(), c.2))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        sem_flag(&antes),
        sem_flag(&depois),
        "mesmos corpos, nomes e camadas"
    );
    assert_eq!(
        hash(&mut s219),
        hash(&mut s220),
        "a migração não muda a simulação"
    );
}
