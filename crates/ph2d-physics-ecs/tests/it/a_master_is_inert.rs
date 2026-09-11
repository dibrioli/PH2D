//! ⭐ **A RECEITA NÃO CAI** — um mestre no mesmo `World` é invisível ao solver (ADR-0164 / F4.1).
//!
//! Esta é a condição (a) da [refutação 1], e ela não é higiene: sem o filtro, uma peça de mestre
//! com `RigidBody` **é simulada**, o `readback` carimba o `Transform` dela por tique, e um sync por
//! change-tick propagaria a **pose SIMULADA do mestre** a todas as instâncias. Pausado, o `settle`
//! veria `Transform != pose do corpo` e **teleportaria, zerando a velocidade**, todo o quadro.
//!
//! ⚠️ **O oráculo é a CENA, nunca «o filtro está no tipo»**: o corpo do mestre tem de estar onde o
//! artista o largou depois de 120 tiques de gravidade, e o irmão idêntico fora da biblioteca tem de
//! ter caído. *Um gate que lesse o tipo da consulta ficaria verde sobre um filtro que ninguém corre.*
//!
//! [refutação 1]: ../../../docs/Components/pesquisa/instancias_2026-08-21/refutacao_1_sync_determinismo.md

use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, Transform, assign_master_pieces};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsJoint, PulleyWheel, RigidBody,
};

fn body(
    sim: &mut SimWorld,
    name: &str,
    y: f32,
    parent: Option<ph2d_ecs::Entity>,
) -> ph2d_ecs::Entity {
    let mut e = sim.world_mut().spawn((
        Name::new(name),
        Transform::from_translation(ph2d_core::Vec2::new(0.0, y)),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.3,
                half_y: 0.3,
            },
            ..Collider::default()
        },
    ));
    if let Some(p) = parent {
        e.insert(ChildOf(p));
    }
    e.id()
}

fn y_of(sim: &SimWorld, e: ph2d_ecs::Entity) -> f32 {
    sim.world()
        .get::<Transform>(e)
        .expect("o corpo existe")
        .translation
        .y
}

/// ⭐ **O corpo do mestre NÃO cai; o irmão fora da biblioteca cai.**
///
/// (Mutação: tirar o `NotAMaster` da `BodyQuery` ⇒ os dois caem e o gate reprova nomeando o `y`.)
#[test]
fn a_master_body_does_not_fall_and_a_loose_one_does() {
    let mut sim = SimWorld::new();
    let master = sim
        .world_mut()
        .spawn((Name::new("Ragdoll"), Transform::IDENTITY, MasterRoot))
        .id();
    let inside = body(&mut sim, "Torso", 5.0, Some(master));
    let outside = body(&mut sim, "Crate", 5.0, None);
    assign_master_pieces(sim.world_mut());

    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }

    let (yi, yo) = (y_of(&sim, inside), y_of(&sim, outside));
    assert!(
        (yi - 5.0).abs() < 1.0e-6,
        "a peca do MESTRE simulou: saiu de 5.000 para {yi:.3} — a receita caiu"
    );
    assert!(
        yo < 4.0,
        "o controle nao caiu ({yo:.3}) — a fixture nao contem o fenomeno, e o gate nao mede nada"
    );
}

/// ⚠️ **Uma PEÇA de mestre (collider sem corpo) também não entra** — e a configuração que o revela
/// é a que uma prova de mutação foi obrigada a achar.
///
/// Uma peça acha o dono **subindo o `ChildOf` no mundo** (`parts::owner_body`), e essa travessia
/// não sabe o que é uma biblioteca. Enquanto a fixture tinha o mestre solto na raiz, tirar o filtro
/// **não mudava nada** — a peça não tinha corpo nenhum acima, e ficava inerte por outra razão.
/// *Uma cura medida numa fixtura sem o fenómeno lê-se como inútil.*
///
/// A configuração observável é o mestre **pendurado num corpo da cena**: aí a prancha da biblioteca
/// vira um collider composto de um objeto real, e um caixote que devia passar ao lado pousa nela.
#[test]
fn a_master_part_never_becomes_a_collider_of_a_scene_body() {
    let mut sim = SimWorld::new();
    // O chão, lá em baixo.
    sim.world_mut().spawn((
        Name::new("Floor"),
        Transform::from_translation(ph2d_core::Vec2::new(0.0, -3.0)),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
    ));
    // Um corpo REAL da cena, estreito, com a biblioteca pendurada nele.
    let cart = sim
        .world_mut()
        .spawn((
            Name::new("Cart"),
            Transform::from_translation(ph2d_core::Vec2::new(0.0, 0.0)),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
        ))
        .id();
    let master = sim
        .world_mut()
        .spawn((
            Name::new("Lib"),
            Transform::IDENTITY,
            MasterRoot,
            ChildOf(cart),
        ))
        .id();
    // A prancha LARGA, dentro da biblioteca.
    sim.world_mut().spawn((
        Name::new("Plank"),
        Transform::IDENTITY,
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        ChildOf(master),
    ));
    // O caixote cai a `x = 4`: fora do carrinho (meia-largura `0,3`), dentro da prancha (`5`).
    let crate_e = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            Transform::from_translation(ph2d_core::Vec2::new(4.0, 4.0)),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
        ))
        .id();
    assign_master_pieces(sim.world_mut());

    let mut bridge = PhysicsBridge::new();
    for t in 1..=240 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = y_of(&sim, crate_e);
    assert!(
        y < -1.5,
        "o caixote parou em {y:.3} — a PRANCHA da biblioteca virou collider do carrinho da CENA"
    );
}

/// ⭐ **E o JOINT de um mestre não prende corpos reais** — o defeito que a refutação nomeia como o
/// pior de todos, porque a resolução é por identidade e o mestre e a cena partilham o mundo.
#[test]
fn a_master_joint_never_binds_a_scene_body() {
    let mut sim = SimWorld::new();
    let free = body(&mut sim, "Crate", 5.0, None);
    let anchor = sim
        .world_mut()
        .spawn((
            Name::new("Anchor"),
            Transform::from_translation(ph2d_core::Vec2::new(0.0, 8.0)),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider::default(),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (a, b) = {
        let w = sim.world();
        (
            w.get::<ph2d_ecs::StableId>(anchor).expect("id").0,
            w.get::<ph2d_ecs::StableId>(free).expect("id").0,
        )
    };
    // O joint vive DENTRO de um mestre e aponta para dois corpos da CENA.
    let master = sim
        .world_mut()
        .spawn((Name::new("Rig"), Transform::IDENTITY, MasterRoot))
        .id();
    sim.world_mut().spawn((
        Name::new("Rope"),
        Transform::IDENTITY,
        PhysicsJoint {
            body_a: a,
            body_b: b,
            ..PhysicsJoint::default()
        },
        ChildOf(master),
    ));
    assign_master_pieces(sim.world_mut());

    let mut bridge = PhysicsBridge::new();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let y = y_of(&sim, free);
    assert!(
        y < 3.0,
        "o caixote ficou pendurado em {y:.3} — o joint DE DENTRO da biblioteca prendeu um corpo da cena"
    );
}

/// ⚠️ **A ROLDANA é a SEXTA consulta, e a refutação só nomeava cinco** — ela cita uma faixa de
/// linhas de um ficheiro, e a roldana nasceu noutro depois disso.
///
/// Uma roldana é alcançada pelo **NOME** da corda: uma dentro da biblioteca não só entraria no
/// sistema vivo como **disputaria a resolução** com a da cena. Aqui a régua é direta — a roldana do
/// mestre não pode aparecer na rota que a ponte colheu.
#[test]
fn a_master_pulley_wheel_is_not_harvested() {
    let mut sim = SimWorld::new();
    // ⚠️ **A colheita de roldanas só corre se houver um JOINT na cena** — sem ele o `reconcile_joints`
    // sai cedo, e o controle abaixo apanhava *a colheita a não acontecer*, não o filtro a funcionar.
    // Foi o controle positivo que o disse; sem ele este gate teria ficado verde sobre nada.
    let p = body(&mut sim, "A", 0.0, None);
    let q = body(&mut sim, "B", 1.0, None);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (pa, qb) = {
        let w = sim.world();
        (
            w.get::<ph2d_ecs::StableId>(p).expect("id").0,
            w.get::<ph2d_ecs::StableId>(q).expect("id").0,
        )
    };
    sim.world_mut().spawn((
        Name::new("Rope"),
        Transform::IDENTITY,
        PhysicsJoint {
            body_a: pa,
            body_b: qb,
            ..PhysicsJoint::default()
        },
    ));
    let master = sim
        .world_mut()
        .spawn((Name::new("Lib"), Transform::IDENTITY, MasterRoot))
        .id();
    let wheel = sim
        .world_mut()
        .spawn((
            Name::new("Wheel"),
            Transform::IDENTITY,
            PulleyWheel {
                rope: 1,
                radius: 0.2,
                ..PulleyWheel::default()
            },
            ChildOf(master),
        ))
        .id();
    // ⚠️ **O CONTROLE POSITIVO** — uma roldana idêntica FORA da biblioteca. Sem ele este gate fica
    // verde sempre que a colheita não correr por qualquer outra razão, que é o modo de falha mais
    // caro: ele afirmaria o filtro sem nunca o ter exercido.
    let loose = sim
        .world_mut()
        .spawn((
            Name::new("SceneWheel"),
            Transform::IDENTITY,
            PulleyWheel {
                rope: 1,
                radius: 0.2,
                ..PulleyWheel::default()
            },
        ))
        .id();
    assign_master_pieces(sim.world_mut());

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, true, 1);
    let harvested = bridge.harvested_wheels();
    assert!(
        harvested.contains(&loose),
        "o CONTROLE nao foi colhido — a colheita nao correu, e o gate nao mede nada: {harvested:?}"
    );
    assert!(
        !harvested.contains(&wheel),
        "a roldana do MESTRE foi colhida — ela disputa a corda com a da cena, por NOME"
    );
}
