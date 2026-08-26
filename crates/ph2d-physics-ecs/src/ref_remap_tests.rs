//! Os gates do remap de referências da física (ADR-0164 / F4.2).

use super::{remap_joint_refs, remap_wheel_refs};
use crate::{PhysicsJoint, PulleyWheel};
use bevy_ecs::prelude::World;
use std::collections::BTreeMap;

fn map(pairs: &[(u64, u64)]) -> BTreeMap<u64, u64> {
    pairs.iter().copied().collect()
}

/// ⭐ **Os dois lados da junta seguem o mapa** — e é isto que faz a junta da instância prender
/// os corpos dela.
#[test]
fn both_ends_of_a_joint_follow_the_map() {
    let mut w = World::new();
    let j = w
        .spawn(PhysicsJoint {
            body_a: 10,
            body_b: 11,
            ..PhysicsJoint::default()
        })
        .id();
    assert_eq!(
        remap_joint_refs(&mut w, &[j], &map(&[(10, 20), (11, 21)])),
        1
    );
    let got = w.get::<PhysicsJoint>(j).expect("junta");
    assert_eq!((got.body_a, got.body_b), (20, 21));
}

/// ⚠️ **Uma ponta FORA do que se copiou fica.** Um ragdoll pendurado num gancho do cenário
/// continua pendurado nesse gancho — a busca falha e o elo não se mexe.
///
/// (Mutação: `j.body_b = *by_id.get(&j.body_b).unwrap_or(&0)` ⇒ a ponta externa vira `0`, que é
/// *"nenhum"*, e a instância cai no chão. Este gate reprova.)
#[test]
fn an_end_outside_the_copy_is_left_alone() {
    let mut w = World::new();
    let j = w
        .spawn(PhysicsJoint {
            body_a: 10,
            body_b: 99,
            ..PhysicsJoint::default()
        })
        .id();
    assert_eq!(remap_joint_refs(&mut w, &[j], &map(&[(10, 20)])), 1);
    let got = w.get::<PhysicsJoint>(j).expect("junta");
    assert_eq!(got.body_a, 20);
    assert_eq!(got.body_b, 99, "a ponta de fora foi reescrita");
}

/// ⚠️ **Só as entidades DADAS.** Varrer o mundo reescreveria as juntas do próprio mestre.
///
/// (Mutação: trocar o laço por uma query de todas as juntas ⇒ este gate reprova.)
#[test]
fn only_the_listed_entities_are_touched() {
    let mut w = World::new();
    let mine = w
        .spawn(PhysicsJoint {
            body_a: 10,
            body_b: 11,
            ..PhysicsJoint::default()
        })
        .id();
    let masters = w
        .spawn(PhysicsJoint {
            body_a: 10,
            body_b: 11,
            ..PhysicsJoint::default()
        })
        .id();
    remap_joint_refs(&mut w, &[mine], &map(&[(10, 20), (11, 21)]));
    let untouched = w.get::<PhysicsJoint>(masters).expect("junta do mestre");
    assert_eq!(
        (untouched.body_a, untouched.body_b),
        (10, 11),
        "o remap reescreveu a junta do MESTRE — a copia passou a comandar o original"
    );
}

/// **A roldana: a corda e o corpo em que ela é montada.**
#[test]
fn a_wheel_follows_its_rope_and_its_body() {
    let mut w = World::new();
    let wheel = w
        .spawn(PulleyWheel {
            rope: 10,
            body: 11,
            ..PulleyWheel::default()
        })
        .id();
    assert_eq!(
        remap_wheel_refs(&mut w, &[wheel], &map(&[(10, 20), (11, 21)])),
        1
    );
    let got = w.get::<PulleyWheel>(wheel).expect("roldana");
    assert_eq!((got.rope, got.body), (20, 21));
}

/// ⚠️ **`body = 0` é *pregada no cenário*, nunca uma identidade** — e continua a ser depois do
/// remap.
#[test]
fn a_wheel_pinned_to_the_scenery_stays_pinned() {
    let mut w = World::new();
    let wheel = w
        .spawn(PulleyWheel {
            rope: 10,
            body: 0,
            ..PulleyWheel::default()
        })
        .id();
    remap_wheel_refs(&mut w, &[wheel], &map(&[(10, 20), (0, 999)]));
    let got = w.get::<PulleyWheel>(wheel).expect("roldana");
    assert_eq!(got.rope, 20);
    assert_eq!(
        got.body, 0,
        "o zero de *nenhum* foi tratado como identidade"
    );
}

/// **Uma entidade sem o componente é saltada** — a lista que chega é a subárvore inteira.
#[test]
fn an_entity_without_the_component_is_skipped() {
    let mut w = World::new();
    let plain = w.spawn_empty().id();
    assert_eq!(remap_joint_refs(&mut w, &[plain], &map(&[(1, 2)])), 0);
    assert_eq!(remap_wheel_refs(&mut w, &[plain], &map(&[(1, 2)])), 0);
}
