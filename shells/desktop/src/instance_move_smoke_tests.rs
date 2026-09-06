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

/// ⛔⛔⛔ **A LINHA QUE O PASSO NOMEIA TEM DE ESTAR NA LISTA.**
///
/// O passo 1 manda arrastar duas linhas **da receita** na Hierarquia. Mas uma receita não é uma
/// linha da cena: desde 2026-08-30 o `snapshots.rs` **retira** da lista tudo o que o
/// [`crate::render_loop::off_canvas::is_unedited_recipe`] acusa — e o `MasterRoot` também é
/// `MasterPiece`, logo a receita INTEIRA sai. ⇒ sem alguém a escolher, o passo nomeia linhas que
/// não existem, e o report que volta é *«não achei»* — indistinguível de um defeito real.
///
/// ⚠️ **As DUAS metades são o gate:** com a receita escolhida as quatro linhas estão lá; **sem**
/// ela, nenhuma está. A segunda metade é o que impede este gate de ser vácuo — sem ela, uma
/// implementação que nunca escondesse nada passaria.
///
/// (Mutação: tirar a selecção da receita do `instance_smoke_move` ⇒ RED na 1.ª metade.)
#[test]
fn every_row_the_step_names_is_actually_in_the_list() {
    let (mut sim, _r, master, _copies) = build();
    let rows = [
        master,
        piece(&sim, master, "Body"),
        piece(&sim, master, "Head"),
        piece(&sim, master, "Arm"),
    ];

    // ⚠️ **A metade JUSTA vem primeiro:** sem selecção nenhuma, a receita não está na lista.
    crate::render_loop::master_editing::mark(&mut sim, std::iter::empty());
    for (i, &e) in rows.iter().enumerate() {
        assert!(
            crate::render_loop::off_canvas::is_unedited_recipe(sim.world(), e),
            "a linha {i} ja' estaria na lista sem ninguem escolher a receita — este gate mediria \
             nada"
        );
    }

    // E é isto que a cena faz ao montar: escolhe a receita, e as quatro linhas aparecem.
    crate::render_loop::master_editing::mark(&mut sim, std::iter::once(master.to_bits()));
    for (i, &e) in rows.iter().enumerate() {
        assert!(
            !crate::render_loop::off_canvas::is_unedited_recipe(sim.world(), e),
            "a linha {i} que o PASSO 1 manda arrastar NAO esta' na Hierarquia"
        );
    }
}

/// ⛔⛔⛔ **O PASSO QUE DEMONSTRA UMA RECUSA TEM DE PEDIR UM GESTO QUE A RECUSA APANHE.**
///
/// Report do Enio (2026-09-06): *«a mensagem da recusa não apareceu»*. A guarda estava certa — o
/// passo é que pedia o gesto errado: ele mandava arrastar o braço da cópia para a **cabeça**, e
/// depois do passo 1 ele **já lá está**. Arrastar uma peça para o pai onde ela já está é o *mesmo
/// pai*, e a guarda deixa passar de propósito (é um reordenar, e a ordem é excepção da cópia).
///
/// ⚠️ **As duas metades são o gate, e a segunda é a que o report descreve:** o gesto que o passo
/// pede agora é apanhado; o que ele pedia antes **não é**. Sem a segunda, este gate ficaria verde
/// sobre o texto antigo.
///
/// (Mutação: tirar o `!same_parent` do `refuses_reparent` ⇒ RED na 2.ª metade.)
#[test]
fn the_step_that_shows_the_refusal_asks_for_a_gesture_the_guard_catches() {
    let (mut sim, r, master, copies) = build();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);

    // PASSO 1 — a receita move o braço para a cabeça, e as cópias seguem.
    let m_head = piece(&sim, master, "Head");
    let m_arm = piece(&sim, master, "Arm");
    sim.world_mut().entity_mut(m_arm).insert(ChildOf(m_head));
    pass(&mut sim, &r, &mut echo);

    let c = copies[1];
    let (arm, body, head) = (
        piece(&sim, c, "Arm"),
        piece(&sim, c, "Body"),
        piece(&sim, c, "Head"),
    );
    assert_eq!(
        sim.world().get::<ChildOf>(arm).map(|x| x.0),
        Some(head),
        "a fixtura tem de partir do estado que o PASSO 1 deixa"
    );
    assert!(
        crate::hero_intents::hierarchy::refuses_reparent(&mut sim, arm, Some(body)),
        "o gesto que o PASSO 2 pede nao e' recusado — o smoke promete uma mensagem que nao vem"
    );
    assert!(
        !crate::hero_intents::hierarchy::refuses_reparent(&mut sim, arm, Some(head)),
        "o gesto que o PASSO 2 PEDIA antes seria recusado — entao o report do dono nao teria \
         causa, e este gate esta' a medir outra coisa"
    );
}

/// ⛔⛔ **E o TEXTO do passo tem de nomear a peça que a guarda apanha.**
///
/// O gate acima mede a **lei** (qual arrasto é recusado); este mede a **instrução**, e sem ele o
/// texto pode voltar a pedir o gesto que passa — que foi exactamente o report de 2026-09-06.
/// *Duas metades: a lei e a frase que a manda exercer.*
///
/// ⚠️ A régua é derivada do estado que o PASSO 1 deixa: depois dele o braço está na **cabeça**,
/// logo o alvo que MUDA o pai é o **corpo**.
#[test]
fn the_printed_steps_name_the_piece_the_guard_catches() {
    let src = include_str!("instance_move_smoke.rs");
    let at = src
        .find("PASSO 2 (na LISTA)")
        .expect("a cena perdeu o passo da recusa");
    let step = &src[at..at + 220.min(src.len() - at)];
    assert!(
        step.contains("'Body'"),
        "o passo da recusa deixou de nomear o 'Body' — depois do PASSO 1 o braco esta' na cabeca, \
         e so' o corpo MUDA o pai:\n{step}"
    );
    assert!(
        !step.contains("para o 'Head'"),
        "o passo da recusa voltou a mandar arrastar para a cabeca, onde a peca ja' esta' — a \
         guarda deixa passar (e' o mesmo pai) e a mensagem nao aparece:\n{step}"
    );
}
