//! Os gates do sync vivo (ADR-0164 / F4.3).
//!
//! ⚠️ **O oráculo é o VALOR na instância depois do passe**, e nunca «o sync correu»: um gate que
//! contasse escritas ficaria verde sobre um passe que escreve a coisa errada.

use super::sync_instances;
use crate::instance_smoke::{spawn_master, spawn_ragdoll_scene};
use crate::instantiate::instantiate_master;
use ph2d_ecs::{
    Children, Entity, InstanceOf, MasterRoot, Name, SimWorld, StableId, Transform, Visibility,
};
use ph2d_physics_ecs::{PhysicsBridge, PhysicsJoint};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Os descendentes de `root` com um nome dado.
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

/// ⭐⭐ **O SMOKE-GATE 2 DO PLANO: editar a receita muda as três instâncias.**
///
/// (Mutação: trocar o `insert_from_bytes` por um no-op ⇒ RED nomeando a cor que não chegou.)
#[test]
fn editing_the_master_changes_every_instance() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    assert_eq!(roots.len(), 3);

    // O artista pinta o braço da RECEITA de verde.
    let master_arm = piece(&sim, master, "Arm");
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(master_arm)
        .copied()
        .expect("a peca tem sprite");
    spr.tint = [0.1, 0.9, 0.2, 1.0];
    sim.world_mut().entity_mut(master_arm).insert(spr);

    assert!(
        sync_instances(&mut sim, &r, &bridge, &mut echo) > 0,
        "o sync nao escreveu nada"
    );

    for (i, &root) in roots.iter().enumerate() {
        let got = sim
            .world()
            .get::<ph2d_render::Sprite>(piece(&sim, root, "Arm"))
            .expect("a peca da instancia tem sprite")
            .tint;
        assert_eq!(
            got,
            [0.1, 0.9, 0.2, 1.0],
            "a instancia {} nao recebeu a cor da receita",
            i + 1
        );
    }
}

/// ⭐ **E ele é um PONTO FIXO** — correr outra vez não escreve nada.
///
/// ⚠️ É esta a propriedade que impede um passo de undo por quadro, para sempre. Sem a comparação
/// por bytes, todo quadro escreveria e a fila de undo encheria sozinha.
///
/// (Mutação: apagar o `if want == have { continue }` ⇒ RED com a contagem.)
#[test]
fn a_second_pass_writes_nothing() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sync_instances(&mut sim, &r, &bridge, &mut echo),
        0,
        "o 2.o passe escreveu — o sync nao e' ponto fixo, e a fila de undo enche sozinha"
    );
}

/// ⚠️⚠️ **A instância NÃO vira mestre.** O `MasterRoot` está no mestre; propagá-lo faria a
/// instância parar de cair — o defeito da F4.1 de volta pela porta do sync.
///
/// (Mutação: tirar `"ph2d::ecs::MasterRoot"` do `NEVER_PROPAGATES` ⇒ RED.)
#[test]
fn the_sync_never_turns_an_instance_into_a_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = spawn_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert!(
        sim.world().get::<MasterRoot>(inst).is_none(),
        "o sync marcou a instancia como receita — ela deixaria de ser simulada"
    );
    assert!(
        sim.world().get::<InstanceOf>(inst).is_some(),
        "o sync apagou o ELO — no quadro seguinte esta entidade ja' nao e' uma instancia"
    );
    assert!(
        sim.world()
            .get::<InstanceOf>(piece(&sim, inst, "Arm"))
            .is_some(),
        "a PECA perdeu o elo — a correspondencia morre e o sync deixa de a alcancar"
    );
}

/// ⚠️ **A RAIZ da instância é dela** — onde está e como se chama não vêm do mestre.
///
/// (Mutação: esvaziar o `ROOT_IS_ITS_OWN` ⇒ as três instâncias saltam para cima da receita e
/// passam a chamar-se todas "Ragdoll".)
#[test]
fn the_instance_root_keeps_its_own_place_and_name() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    let master_at = sim
        .world()
        .get::<Transform>(master)
        .expect("pose")
        .translation;
    let before: Vec<_> = roots
        .iter()
        .map(|&e| sim.world().get::<Transform>(e).expect("pose").translation)
        .collect();
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    for (i, &root) in roots.iter().enumerate() {
        let now = sim
            .world()
            .get::<Transform>(root)
            .expect("pose")
            .translation;
        assert_eq!(now, before[i], "a instancia {} saltou", i + 1);
        assert_ne!(
            now,
            master_at,
            "a instancia {} foi para cima da receita",
            i + 1
        );
        let name = sim.world().get::<Name>(root).expect("nome").0.clone();
        assert_ne!(
            name,
            "Ragdoll",
            "a instancia {} roubou o nome da receita",
            i + 1
        );
    }
}

/// ⚠️ **E a POSE de uma PEÇA propaga** — é a receita. *Um tipo, duas respostas, escolhidas pelo
/// lugar*: a regra da raiz acima não pode engolir a peça.
///
/// A peça é o **eixo**, que é um corpo `Static`: o solver não escreve a pose de um estático, então
/// o dono dela é o documento. *É o caso que prova que a regra é sobre o DONO, e não sobre «tem
/// corpo».*
///
/// (Mutação: pôr o `Transform` numa lista global de exclusão ⇒ RED.)
#[test]
fn a_piece_pose_does_propagate() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = spawn_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    let master_hub = piece(&sim, master, "Hub");
    sim.world_mut()
        .entity_mut(master_hub)
        .insert(Transform::from_translation(ph2d_core::Vec2::new(2.5, 0.0)));
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<Transform>(piece(&sim, inst, "Hub"))
            .expect("pose")
            .translation
            .x,
        2.5,
        "a pose da PECA nao propagou — a regra da raiz engoliu a peca"
    );
}

/// ⛔⛔ **DECLARADO, e medido ao escrever este gate: a pose de repouso de uma peça DINÂMICA não
/// propaga.**
///
/// O dono do `Transform` de um corpo dinâmico é o solver — sempre, e não só enquanto o relógio
/// anda: a resposta sai do `BodyKind`, que não sabe se a cena está tocando. Então mover o braço da
/// receita **não** move o braço das instâncias, nem depois de um Reset.
///
/// ⚠️ **Isto é a condição (b) da refutação 1 cumprida à letra**, e é a irmã exata da limitação que
/// o plano já declara para a config de física (*«aplicada pelo sync, mas só chega ao solver no
/// próximo Reset»*). Levantá-la exige uma pergunta de produto — *quando é que a pose autorada de
/// uma peça simulada volta a valer?* — e não mais uma linha aqui.
///
/// ⚠️ **O gate existe para que a ausência tenha um autor.** Um comportamento declarado e um buraco
/// leem-se igual num app; só um deles tem um teste a dizer o número.
#[test]
fn the_rest_pose_of_a_simulated_piece_does_not_propagate_and_that_is_declared() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = spawn_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    let before = sim
        .world()
        .get::<Transform>(piece(&sim, inst, "Arm"))
        .expect("pose")
        .translation;
    let master_arm = piece(&sim, master, "Arm");
    sim.world_mut()
        .entity_mut(master_arm)
        .insert(Transform::from_translation(ph2d_core::Vec2::new(9.0, 0.0)));
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<Transform>(piece(&sim, inst, "Arm"))
            .expect("pose")
            .translation,
        before,
        "a pose de uma peca DINAMICA propagou — o sync escreveu na celula do solver"
    );
}

/// ⭐⭐ **A JUNTA propagada prende os corpos DA INSTÂNCIA.**
///
/// Os bytes que chegam nomeiam os corpos do MESTRE; sem o religamento a instância larga o rig
/// dela na primeira propagação — o defeito que a F4.2 curou, de volta pela porta do sync.
///
/// (Mutação: apagar o `remap_object_refs` do fim do laço ⇒ RED.)
#[test]
fn the_propagated_joint_still_binds_the_instances_own_bodies() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = spawn_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    let inst_hub = sim
        .world()
        .get::<StableId>(piece(&sim, inst, "Hub"))
        .expect("id")
        .0;
    let inst_arm = sim
        .world()
        .get::<StableId>(piece(&sim, inst, "Arm"))
        .expect("id")
        .0;

    // O artista mexe num número da junta da receita — e com ele viajam as duas pontas.
    let master_pin = piece(&sim, master, "Pin");
    let mut j = sim
        .world()
        .get::<PhysicsJoint>(master_pin)
        .copied()
        .expect("junta");
    j.motor_max_force += 1.0;
    sim.world_mut().entity_mut(master_pin).insert(j);

    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let got = sim
        .world()
        .get::<PhysicsJoint>(piece(&sim, inst, "Pin"))
        .copied()
        .expect("junta da instancia");
    assert_eq!(
        (got.body_a, got.body_b),
        (inst_hub, inst_arm),
        "a junta da instancia passou a prender os corpos do MESTRE"
    );
}

/// ⚠️⚠️ **A pose de um corpo DINÂMICO não é escrita pelo sync** — o dono dela é o solver, e dois
/// autores na mesma célula dão um corpo teleportado por tique.
///
/// ⚠️ **O oráculo é a CENA**: a instância cai 120 tiques, e depois de um sync ela tem de continuar
/// onde caiu — não de volta à pose da receita.
///
/// (Mutação: tirar o `document_owns_pose` ⇒ o braço volta à pose autorada e o gate reprova.)
#[test]
fn the_sync_never_writes_a_pose_the_solver_owns() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    let arm = piece(&sim, inst, "Arm");
    let authored = sim.world().get::<Transform>(arm).expect("pose").translation;

    let mut bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    for t in 1..=120 {
        bridge.dispatch(&mut sim, true, t);
    }
    let fell = sim.world().get::<Transform>(arm).expect("pose").translation;
    assert!(
        (fell - authored).length() > 0.1,
        "o controle nao caiu ({fell:?}) — a fixtura nao contem o fenomeno"
    );

    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let after = sim.world().get::<Transform>(arm).expect("pose").translation;
    assert_eq!(
        after, fell,
        "o sync reescreveu a pose que o SOLVER possui — o corpo teleporta por tique"
    );
}

/// **Um mestre sem instância nenhuma não custa nada, e um elo pendurado não estraga nada.**
#[test]
fn a_dangling_link_is_left_alone() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let orphan = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Orphan"),
            Visibility { hidden: true },
            InstanceOf { master: 9_999 },
        ))
        .id();
    assert_eq!(sync_instances(&mut sim, &r, &bridge, &mut echo), 0);
    assert!(
        sim.world()
            .get::<Visibility>(orphan)
            .is_some_and(|v| v.hidden),
        "o sync mexeu numa entidade cujo mestre nao existe"
    );
}

/// ⭐⭐⭐ **O ELO SOBREVIVE AO PASSE — e o gate existe porque ele NÃO sobrevivia.**
///
/// ⚠️ **Este é o defeito que a prova de mutação encontrou** (F4.3): o remap do sync reescrevia o
/// `InstanceOf.master` da raiz — que É a identidade do mestre, logo uma chave do mapa — para a
/// identidade da **própria instância**. A partir do 2.º quadro a instância dizia-se instância de
/// si mesma, o sync deixava de a encontrar, e **nada mais propagava**. Calado, com todos os outros
/// gates verdes: nenhum deles corria o passe DUAS vezes antes de medir.
///
/// ⇒ a régua é a que faltava: **sincronize, e SÓ ENTÃO edite o mestre.**
///
/// (Mutação: trocar o `remap_object_refs_except` por `remap_object_refs` ⇒ RED.)
#[test]
fn the_link_survives_a_sync_so_the_next_edit_still_arrives() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);

    // Alguns quadros de app antes de o artista tocar em nada.
    for _ in 0..3 {
        sync_instances(&mut sim, &r, &bridge, &mut echo);
    }
    // O elo continua a apontar para o MESTRE, e não para a própria instância.
    let master_id = sim.world().get::<StableId>(master).expect("id").0;
    for (i, &root) in roots.iter().enumerate() {
        let link = sim.world().get::<InstanceOf>(root).expect("elo").master;
        let own = sim.world().get::<StableId>(root).expect("id").0;
        assert_ne!(
            link,
            own,
            "a instancia {} passou a ser instancia de SI PROPRIA",
            i + 1
        );
        assert_eq!(link, master_id, "a instancia {} perdeu o mestre", i + 1);
    }

    // E a edição seguinte ainda chega — que é o que o artista sente.
    let master_arm = piece(&sim, master, "Arm");
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(master_arm)
        .copied()
        .expect("sprite");
    spr.tint = [0.9, 0.1, 0.1, 1.0];
    sim.world_mut().entity_mut(master_arm).insert(spr);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    for (i, &root) in roots.iter().enumerate() {
        assert_eq!(
            sim.world()
                .get::<ph2d_render::Sprite>(piece(&sim, root, "Arm"))
                .expect("sprite")
                .tint,
            [0.9, 0.1, 0.1, 1.0],
            "depois de 3 quadros parados, a instancia {} deixou de ouvir a receita",
            i + 1
        );
    }
}

// ───────────────── O report do Enio de 2026-08-26 ─────────────────

/// Mestre simples — raiz sem geometria + uma peça com sprite, e **sem física**.
///
/// ⚠️ O ragdoll da outra fixtura tem o braço **dinâmico**, e a pose de um corpo dinâmico nem
/// sincroniza nem vira excepção (a condição (b) da refutação 1). Um gate sobre a POSE medido lá
/// estaria a medir o `pose_owner`, não o revert.
fn plain_master(sim: &mut SimWorld) -> Entity {
    use ph2d_ecs::ChildOf;
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Rig"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(1.0, 0.0)),
        Name::new("Arm"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(root),
    ));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    root
}

/// ⭐⭐⭐ **O REPORT: devolver à receita não pode TELETRANSPORTAR a peça** (Enio, 2026-08-26).
///
/// > *«Revert to master modifica a posição global do objeto e isso não é uma boa idéia. Melhor o
/// > objeto ficar onde está.»*
///
/// ⚠️ **Os dois lados, no mesmo gesto:** a cor volta à receita (senão o verbo deixou de servir para
/// alguma coisa) e a pose **não se mexe**. Um gate só sobre a pose ficaria verde num verbo que não
/// faz nada.
///
/// (Mutação: apagar o `if key.type_id == pose { continue }` ⇒ RED na pose.)
#[test]
fn reverting_does_not_move_what_the_artist_placed() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = plain_master(&mut sim);
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instanciou");
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    // O artista arrasta a peça E pinta-a.
    let arm = piece(&sim, inst, "Arm");
    let placed = ph2d_core::Vec2::new(5.0, 3.0);
    sim.world_mut()
        .entity_mut(arm)
        .insert(Transform::from_translation(placed));
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(arm)
        .copied()
        .expect("a peca tem sprite");
    spr.tint = [0.1, 0.2, 0.9, 1.0];
    sim.world_mut().entity_mut(arm).insert(spr);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(inst)
            .map_or(0, |o| o.overrides.len()),
        2,
        "a fixtura tem de ter DUAS excepcoes (pose + cor), senao o gate nao separa nada"
    );

    let got = super::revert_all_overrides(&mut sim, &mut echo, arm).expect("e' uma instancia");
    assert_eq!(
        got,
        super::Reverted {
            count: 1,
            poses_kept: 1
        },
        "o revert tem de devolver a COR e contar a pose que ficou"
    );
    // ⚠️ Depois do passe seguinte, que é onde a propagação de facto acontece.
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<Transform>(arm)
            .expect("a peca existe")
            .translation,
        placed,
        "a peca TELETRANSPORTOU-SE ao devolver a receita — o report do Enio"
    );
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(arm)
            .expect("a peca tem sprite")
            .tint,
        sim.world()
            .get::<ph2d_render::Sprite>(piece(&sim, master, "Arm"))
            .expect("a receita tem sprite")
            .tint,
        "a cor nao voltou a ouvir a receita — o verbo deixou de servir para alguma coisa"
    );
}

/// ⭐⭐⭐ **O OUTRO REPORT: pintar uma cópia muda as irmãs** (Enio, 2026-08-26).
///
/// > *«Pintei uma sprite de uma instância e as outras não mudaram.»*
///
/// A edição de pixels sobe até à receita ([`crate::hero_intents::texture_rebind`]) e o passe
/// leva-a a toda a gente.
///
/// ⚠️ **A metade que mantém o ponto fixo:** ela **não** pode virar excepção. Se virasse, a cópia
/// pintada ficava surda à receita para sempre — e o gate mede isso ao lado do resultado visível.
///
/// (Mutação: fazer `write_through_targets` devolver só a entidade ⇒ RED na irmã.)
#[test]
fn painting_one_copy_reaches_the_others() {
    use crate::hero_intents::texture_rebind::{SamplingWindow, rebind_to_individual};
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let master = plain_master(&mut sim);
    let a = instantiate_master(&mut sim, &r, master, None).expect("instanciou A");
    let b = instantiate_master(&mut sim, &r, master, None).expect("instanciou B");
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let pixels = ph2d_asset::AssetId::from_bytes(b"os pixels pintados");
    rebind_to_individual(
        piece(&sim, a, "Arm"),
        &mut sim,
        7,
        pixels,
        [1.0, 1.0],
        false,
        SamplingWindow::Dies,
    );
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    for (name, root) in [("a receita", master), ("a irma", b)] {
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::SpritePixels>(piece(&sim, root, "Arm"))
                .map(|p| p.0),
            Some(pixels),
            "{name} nao recebeu os pixels pintados"
        );
    }
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(a)
            .map_or(0, |o| o.overrides.len()),
        0,
        "pintar criou uma EXCEPCAO — a copia pintada fica surda a' receita para sempre"
    );
    assert_eq!(
        sync_instances(&mut sim, &r, &bridge, &mut echo),
        0,
        "o passe deixou de ser ponto fixo depois de uma pintura"
    );
}
