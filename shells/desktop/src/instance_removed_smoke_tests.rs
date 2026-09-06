//! Os gates da cena 5 (ADR-0164 / F5.10).
//!
//! ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente.*
//! Estes gates correm o CAMINHO INTEIRO que os passos impressos descrevem — arrastar, recusar,
//! devolver — e medem o que a tela mostraria no fim.

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
    let (master, copies) = spawn_removed_scene(
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

/// A peça chamada `name` desta cópia, se ela existir.
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Option<Entity> {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return Some(e);
        }
        if let Some(k) = sim.world().get::<Children>(e) {
            stack.extend(k.iter().copied());
        }
    }
    None
}

fn arm_x(sim: &SimWorld, root: Entity) -> f32 {
    sim.world()
        .get::<Transform>(piece(sim, root, "Arm").expect("o braco"))
        .expect("transform")
        .translation
        .x
}

/// ⭐⭐⭐ **O CAMINHO INTEIRO dos passos impressos**, medido: arrastar · recusar · devolver.
///
/// ⚠️ As **quatro** asserções são as quatro promessas dos `println!`, uma a uma. A que mais custa é
/// a última: *«volta ONDE VOCÊ O TINHA POSTO»* — ela só é verdade porque a recusa passa pelo passe
/// estrutural, que **sepulta** a excepção da peça ao tirá-la e a **exuma** ao pô-la de volta.
///
/// (Mutação: o `refuse_pieces` a despawnar em vez de marcar ⇒ a 4.ª sangra.)
#[test]
fn the_printed_steps_are_what_the_scene_actually_does() {
    let (mut sim, r, master, copies) = build();
    assert_eq!(copies.len(), 3, "os passos falam de TRES robos");
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (mid, witness) = (copies[1], copies[2]);

    // PASSO 2 — arrastar o braço do robô do meio.
    let arm = piece(&sim, mid, "Arm").expect("o braco do meio");
    let moved = arm_x(&sim, mid) + 0.9;
    sim.world_mut()
        .get_mut::<Transform>(arm)
        .expect("transform")
        .translation
        .x = moved;
    pass(&mut sim, &r, &mut echo);
    assert!(
        (arm_x(&sim, witness) - moved).abs() > 0.5,
        "arrastar o braco de um robo mexeu no do vizinho"
    );

    // PASSO 3 — recusar a peça nesta cópia.
    assert_eq!(
        crate::instance_structure::refuse_pieces(&mut sim, &[arm.to_bits()]),
        1
    );
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    assert!(
        piece(&sim, mid, "Arm").is_none(),
        "o braco nao saiu do robo do meio"
    );
    assert!(
        piece(&sim, witness, "Arm").is_some() && piece(&sim, master, "Arm").is_some(),
        "sumiu o braco de mais alguem — a testemunha ou a receita"
    );

    // PASSO 4 — o *Put back*, e a promessa de que a mudança do passo 2 volta junto.
    let sid = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(mid)
        .expect("a copia guarda a decisao")
        .removed
        .iter()
        .copied()
        .next()
        .expect("uma peca recusada");
    assert!(crate::instance_structure::restore_piece(
        &mut sim,
        mid.to_bits(),
        sid
    ));
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);
    let back = arm_x(&sim, mid);
    assert!(
        (back - moved).abs() < 1e-5,
        "o braco voltou em {back} e nao em {moved} — a excepcao do artista nao foi exumada"
    );
}

/// ⚠️ **`\\` num literal de Rust NÃO é continuação de linha.** A instrução ao dono é uma superfície
/// de produto, e era a única deste módulo sem régua.
#[test]
fn the_printed_steps_have_no_stray_backslash() {
    let src = include_str!("instance_removed_smoke.rs");
    // ⚠️ A própria linha do doc acima contém o padrão — por isso a varredura é só de CÓDIGO.
    for (i, l) in src.lines().enumerate() {
        let code = l.split_once("//").map_or(l, |(before, _)| before);
        assert!(
            !code.trim_end().ends_with("\\\\"),
            "linha {}: `\\\\` no fim de um literal parte a mensagem em duas",
            i + 1
        );
    }
}

/// ⭐⭐ **O braço fica LONGE do corpo** — o passo 1 manda clicar nele na tela, e uma peça encostada
/// não se distingue do corpo num clique de canvas.
#[test]
fn the_arm_is_far_enough_from_the_body_to_be_clicked() {
    let (sim, _r, master, _copies) = build();
    let body = piece(&sim, master, "Body").expect("o corpo");
    let arm = piece(&sim, master, "Arm").expect("o braco");
    let half_body = sim
        .world()
        .get::<ph2d_render::Sprite>(body)
        .expect("sprite")
        .size[0]
        / 2.0;
    let (arm_x, arm_half) = (
        sim.world()
            .get::<Transform>(arm)
            .expect("transform")
            .translation
            .x,
        sim.world()
            .get::<ph2d_render::Sprite>(arm)
            .expect("sprite")
            .size[0]
            / 2.0,
    );
    assert!(
        arm_x - arm_half > half_body,
        "o braco encosta no corpo ({}) — o passo 1 manda clicar nele",
        arm_x - arm_half
    );
}
