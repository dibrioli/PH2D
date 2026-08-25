//! Testes de [`super::resolve_body_names`].

use super::*;
use crate::{JointKind, PhysicsJoint, PulleyWheel, WrapSide};
use ph2d_ecs::Transform;

fn body(world: &mut World, name: &str) -> Entity {
    world.spawn((Transform::IDENTITY, Name::new(name))).id()
}

/// **Uma junta autorada por NOME passa a apontar pela IDENTIDADE.**
#[test]
fn a_joint_authored_by_name_ends_up_pointing_by_identity() {
    let mut w = World::new();
    let a = body(&mut w, "Post");
    let b = body(&mut w, "Plank");
    let j = w
        .spawn((
            Transform::IDENTITY,
            Name::new("Post : Plank"),
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();

    let n = resolve_body_names(&mut w);
    assert_eq!(n.joints, 1);

    let id_a = w.get::<StableId>(a).unwrap().0;
    let id_b = w.get::<StableId>(b).unwrap().0;
    let joint = w.get::<PhysicsJoint>(j).unwrap();
    assert_eq!(joint.body_a, id_a, "o lado A aponta a identidade do Post");
    assert_eq!(joint.body_b, id_b, "o lado B aponta a identidade do Plank");
}

/// ⭐ **A cura, medida: RENOMEAR deixa de desligar a junta.**
///
/// É o defeito que o `name.rs` documentava desde antes desta wave (*"renomear um objeto muda
/// o id dele, e portanto desliga o que apontava para ele"*). Depois da resolução, o nome é só
/// um rótulo — mudá-lo não toca no que a junta guarda.
#[test]
fn renaming_a_body_no_longer_disconnects_its_joint() {
    let mut w = World::new();
    let a = body(&mut w, "Post");
    let b = body(&mut w, "Plank");
    let j = w
        .spawn((
            Transform::IDENTITY,
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();
    resolve_body_names(&mut w);
    let before = *w.get::<PhysicsJoint>(j).unwrap();

    // O artista renomeia o corpo na Hierarquia.
    *w.get_mut::<Name>(a).unwrap() = Name::new("Pillar");

    let after = *w.get::<PhysicsJoint>(j).unwrap();
    assert_eq!(
        after.body_a, before.body_a,
        "renomear nao pode mexer no que a junta guarda",
    );
    assert_eq!(
        after.body_a,
        w.get::<StableId>(a).unwrap().0,
        "e ela continua a apontar para o MESMO corpo, agora com outro nome",
    );
    let _ = b;
}

/// ⭐ **E a outra metade: uma CÓPIA deixa de prender nos corpos do original.**
///
/// A cópia recebe o nome `" (1)"`; com hash de nome a junta dela continuava a nomear o
/// original (ADR-0164 — *"a junta da cópia prenderia os corpos do MESTRE"*). Com identidade,
/// dois corpos distintos são dois ids distintos, e cada junta prende os seus.
#[test]
fn a_copy_binds_its_own_bodies_not_the_originals() {
    let mut w = World::new();
    let a1 = body(&mut w, "Post");
    let b1 = body(&mut w, "Plank");
    let j1 = w
        .spawn((
            Transform::IDENTITY,
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();
    resolve_body_names(&mut w);

    // A copia: nomes sufixados, e a junta dela autorada contra ESSES nomes.
    let a2 = body(&mut w, "Post (1)");
    let b2 = body(&mut w, "Plank (1)");
    let j2 = w
        .spawn((
            Transform::IDENTITY,
            PhysicsJoint {
                body_a: stable_name_id("Post (1)"),
                body_b: stable_name_id("Plank (1)"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();
    resolve_body_names(&mut w);

    let (ja, jb) = {
        let j = w.get::<PhysicsJoint>(j2).unwrap();
        (j.body_a, j.body_b)
    };
    assert_eq!(ja, w.get::<StableId>(a2).unwrap().0);
    assert_eq!(jb, w.get::<StableId>(b2).unwrap().0);
    assert_ne!(
        ja,
        w.get::<StableId>(a1).unwrap().0,
        "a junta da COPIA nao pode prender o corpo do original",
    );
    // E a do original continua onde estava.
    let j = w.get::<PhysicsJoint>(j1).unwrap();
    assert_eq!(j.body_a, w.get::<StableId>(a1).unwrap().0);
    let _ = b1;
}

/// **A roldana traduz a corda e o corpo em que é montada.**
#[test]
fn a_pulley_wheel_resolves_its_rope_and_its_mount() {
    let mut w = World::new();
    let rope_e = body(&mut w, "Hoist");
    let mount = body(&mut w, "Cart");
    let wheel = w
        .spawn((
            Transform::IDENTITY,
            PulleyWheel {
                rope: stable_name_id("Hoist"),
                order: 0,
                radius: 0.2,
                radius_out: 0.0,
                wrap: WrapSide::Auto,
                motor_speed: 0.0,
                body: stable_name_id("Cart"),
                local: [0.0, 0.0],
                mounted: true,
                break_enabled: false,
                break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            },
        ))
        .id();

    assert_eq!(resolve_body_names(&mut w).wheels, 1);
    let pw = w.get::<PulleyWheel>(wheel).unwrap();
    assert_eq!(pw.rope, w.get::<StableId>(rope_e).unwrap().0);
    assert_eq!(pw.body, w.get::<StableId>(mount).unwrap().0);
}

/// **`body: 0` significa «no cenário» e continua `0`** — traduzir o zero prenderia a roldana
/// a um objeto que ninguém escolheu.
#[test]
fn a_scenery_wheel_keeps_its_zero() {
    let mut w = World::new();
    body(&mut w, "Hoist");
    let wheel = w
        .spawn((
            Transform::IDENTITY,
            PulleyWheel {
                rope: stable_name_id("Hoist"),
                order: 0,
                radius: 0.2,
                radius_out: 0.0,
                wrap: WrapSide::Auto,
                motor_speed: 0.0,
                body: 0,
                local: [0.0, 0.0],
                mounted: false,
                break_enabled: false,
                break_force: PulleyWheel::DEFAULT_BREAK_FORCE,
            },
        ))
        .id();
    resolve_body_names(&mut w);
    assert_eq!(w.get::<PulleyWheel>(wheel).unwrap().body, 0);
}

/// **Idempotente** — correr duas vezes não move nada.
///
/// ⚠️ Vale porque um `StableId` é pequeno e sequencial e o mapa só tem hashes FNV-1a de nomes,
/// que são grandes. É a razão de a chamada poder viver num roteador sem ninguém se preocupar
/// com quantas vezes ela corre.
#[test]
fn running_twice_moves_nothing() {
    let mut w = World::new();
    body(&mut w, "Post");
    body(&mut w, "Plank");
    let j = w
        .spawn((
            Transform::IDENTITY,
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();
    resolve_body_names(&mut w);
    let once = *w.get::<PhysicsJoint>(j).unwrap();
    assert_eq!(
        resolve_body_names(&mut w).joints,
        0,
        "a 2a corrida nao acha nada"
    );
    assert_eq!(*w.get::<PhysicsJoint>(j).unwrap(), once);
}

/// **Um nome que já não existe não traduz, e a referência fica como estava** — apontando para
/// nada, que é exatamente o que ela já fazia. ⛔ Inventar um alvo seria pior que não prender.
#[test]
fn a_dangling_name_stays_dangling() {
    let mut w = World::new();
    body(&mut w, "Post");
    let j = w
        .spawn((
            Transform::IDENTITY,
            PhysicsJoint {
                body_a: stable_name_id("Post"),
                body_b: stable_name_id("Ghost"),
                ..PhysicsJoint::of_kind(JointKind::Pin)
            },
        ))
        .id();
    resolve_body_names(&mut w);
    assert_eq!(
        w.get::<PhysicsJoint>(j).unwrap().body_b,
        stable_name_id("Ghost"),
        "o lado sem dono fica como estava",
    );
}
