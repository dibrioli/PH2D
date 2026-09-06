//! Os gates da cena 7 (ADR-0164 / F5.12).
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente.*
//! Este gate corre o caminho que o passo impresso descreve — mudar o pai da peça **na receita** — e
//! mede o que a tela mostraria: o braço de cada cópia na cabeça **dela**, à altura da cabeça.

use super::*;
use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::Children;
use ph2d_physics_ecs::PhysicsBridge;

fn build() -> (
    SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    Entity,
    Vec<Entity>,
) {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (master, copies) = spawn_move_scene(
        &mut sim,
        &r,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    (sim, r, master, copies)
}

fn pass(sim: &mut SimWorld, r: &ph2d_ecs::scene::ComponentRegistry, echo: &mut MasterEcho) {
    for _ in 0..2 {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        sync_instances(
            sim,
            r,
            &PhysicsBridge::new(),
            echo,
            &mut OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
    }
}

fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(k) = sim.world().get::<Children>(e) {
            stack.extend(k.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name}");
}

/// ⭐⭐⭐ **O passo impresso é o que a cena faz: os TRÊS bracos sobem, e cada um para a cabeça DELE.**
///
/// ⚠️ **A segunda asserção é a que custa.** Que o pai mudou é estrutura; que o braço aparece **à
/// altura da cabeça** é o que o dono vê — e só é verdade porque a pose de uma peça é LOCAL e chega
/// verbatim da receita. Um passe que reparentasse e deixasse a pose de mundo intacta poria o braço
/// no mesmo sítio da tela, e o smoke leria *«não aconteceu nada»*.
///
/// (Mutação: apagar o bloco *«as que estão no SÍTIO ERRADO»* do `reconcile_one` ⇒ RED.)
#[test]
fn the_printed_step_moves_the_arm_in_every_copy() {
    let (mut sim, r, master, copies) = build();
    assert_eq!(copies.len(), 3, "o passo fala de TRES robos");
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    for &c in &copies {
        assert_eq!(
            sim.world()
                .get::<ChildOf>(piece(&sim, c, "Arm"))
                .map(|x| x.0),
            Some(piece(&sim, c, "Body")),
            "a fixtura tem de partir do braco no CORPO, senao mede outra coisa"
        );
    }
    let before = ph2d_ecs::world_transform(sim.world(), piece(&sim, copies[0], "Arm"))
        .expect("pose")
        .translation
        .y;

    // PASSO 1 — na receita, o braço passa para a cabeça.
    let head = piece(&sim, master, "Head");
    let arm = piece(&sim, master, "Arm");
    sim.world_mut().entity_mut(arm).insert(ChildOf(head));
    pass(&mut sim, &r, &mut echo);

    for (i, &c) in copies.iter().enumerate() {
        let (arm, head) = (piece(&sim, c, "Arm"), piece(&sim, c, "Head"));
        assert_eq!(
            sim.world().get::<ChildOf>(arm).map(|x| x.0),
            Some(head),
            "o robo {} ficou com o braco no pai antigo",
            i + 1
        );
        let up = ph2d_ecs::world_transform(sim.world(), arm)
            .expect("pose")
            .translation
            .y;
        assert!(
            (up - before - HEAD_AT.y).abs() < 1e-4,
            "o braco do robo {} subiu {} e a cabeca esta' a {} — a pose nao seguiu o pai novo",
            i + 1,
            up - before,
            HEAD_AT.y
        );
    }
}

/// ⚠️ **O braço nasce LONGE do corpo na horizontal**, senão o robô lê-se como um bloco só e o dono
/// não vê qual peça se mexeu.
#[test]
fn the_arm_sticks_out_far_enough_to_be_seen_moving() {
    let (sim, _r, master, _copies) = build();
    let half_body = sim
        .world()
        .get::<Sprite>(piece(&sim, master, "Body"))
        .expect("sprite")
        .size[0]
        / 2.0;
    let arm = piece(&sim, master, "Arm");
    let half_arm = sim.world().get::<Sprite>(arm).expect("sprite").size[0] / 2.0;
    let x = sim
        .world()
        .get::<Transform>(arm)
        .expect("transform")
        .translation
        .x;
    assert!(
        x - half_arm > half_body,
        "o braco encosta no corpo ({}) — o dono nao distingue a peca que se mexe",
        x - half_arm
    );
}

/// ⚠️ **`\\` num literal de Rust NÃO é continuação de linha** — a irmã desta régua nas cenas 5 e 6.
#[test]
fn the_printed_steps_have_no_stray_backslash() {
    let src = include_str!("instance_move_smoke.rs");
    for (i, l) in src.lines().enumerate() {
        let code = l.split_once("//").map_or(l, |(before, _)| before);
        assert!(
            !code.trim_end().ends_with("\\\\"),
            "linha {}: `\\\\` no fim de um literal parte a mensagem em duas",
            i + 1
        );
    }
}
