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

// ── F4.4 — os OVERRIDES ────────────────────────────────────────────────────────────────────

/// A cor de uma peça, o valor que estes gates movem.
fn tint(sim: &SimWorld, e: Entity) -> [f32; 4] {
    sim.world()
        .get::<ph2d_render::Sprite>(e)
        .expect("sprite")
        .tint
}

fn paint(sim: &mut SimWorld, e: Entity, c: [f32; 4]) {
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(e)
        .copied()
        .expect("sprite");
    spr.tint = c;
    sim.world_mut().entity_mut(e).insert(spr);
}

/// ⭐⭐⭐ **O QUE A F4.4 ENTREGA: uma edição na instância SOBREVIVE à edição seguinte do mestre.**
///
/// ⚠️ Sem isto o app tem um modo de falha que apaga trabalho: o artista pinta uma cópia, edita a
/// receita mais tarde, e a pintura dele desaparece **sem aviso**.
///
/// (Mutação: apagar o `if overrides.overrides.contains(&key) { continue }` ⇒ RED.)
#[test]
fn an_edit_on_an_instance_survives_the_next_master_edit() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo); // o eco nasce

    // O artista pinta o braço da PRIMEIRA instância de azul.
    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, mine),
        [0.1, 0.2, 0.9, 1.0],
        "o sync achatou a edicao do artista no mesmo quadro em que ele a fez"
    );

    // Mais tarde, ele pinta a RECEITA de verde.
    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, master_arm, [0.1, 0.9, 0.2, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(
        tint(&sim, mine),
        [0.1, 0.2, 0.9, 1.0],
        "a edicao do artista foi apagada pela receita — o override nao segurou"
    );
    for root in &roots[1..] {
        assert_eq!(
            tint(&sim, piece(&sim, *root, "Arm")),
            [0.1, 0.9, 0.2, 1.0],
            "as OUTRAS instancias tinham de receber a cor da receita"
        );
    }
}

/// ⚠️⚠️ **Editar a RECEITA não cria override nenhum** — e este é o gate que impede a leitura
/// errada do diff.
///
/// Ler *«mestre != instância»* como *«a instância mexeu-se»* transformaria cada edição da receita
/// num override em **todas** as instâncias: a difusão pararia para sempre, no gesto que a pediu.
///
/// (Mutação: trocar `!master_moved && echo.master.contains_key(&echo_key)` por `true` ⇒ RED.)
#[test]
fn editing_the_master_creates_no_overrides() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, master_arm, [0.9, 0.9, 0.1, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    for (i, &root) in roots.iter().enumerate() {
        let n = sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(root)
            .map_or(0, |o| o.overrides.len());
        assert_eq!(
            n,
            0,
            "a instancia {} ganhou {n} override(s) por o MESTRE ter mudado",
            i + 1
        );
        assert_eq!(tint(&sim, piece(&sim, root, "Arm")), [0.9, 0.9, 0.1, 1.0]);
    }
}

/// **O override fica REGISTADO na raiz da instância** — é o que o faz sobreviver ao save e ao undo.
#[test]
fn the_override_is_recorded_on_the_instance_root() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let master_arm_id = sim
        .world()
        .get::<StableId>(piece(&sim, master, "Arm"))
        .expect("id")
        .0;
    let ov = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(roots[0])
        .expect("a raiz tem o conjunto")
        .clone();
    assert!(
        ov.overrides.contains(&ph2d_ecs::OverrideKey {
            piece: master_arm_id,
            type_id: ph2d_ecs::scene::stable_type_id("ph2d::render::Sprite"),
        }),
        "o override nao foi registado com a chave (peca do MESTRE, componente): {ov:?}"
    );
    // E só a instância que o artista tocou o tem.
    for root in &roots[1..] {
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::ObjectInstance>(*root)
                .map_or(0, |o| o.overrides.len()),
            0
        );
    }
}

/// ⚠️ **O EMPATE está declarado: quando os dois mudam no mesmo quadro, a RECEITA ganha.**
///
/// Editar a receita é uma difusão deliberada. ⛔ A alternativa (a instância ganhar) faria o gesto
/// mais explícito do app — *mudar o molde* — não chegar exatamente às cópias que alguém acabou de
/// tocar, que é onde ele menos se explicaria.
#[test]
fn when_both_move_in_the_same_pass_the_master_wins() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    paint(&mut sim, master_arm, [0.9, 0.9, 0.1, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(
        tint(&sim, mine),
        [0.9, 0.9, 0.1, 1.0],
        "a receita tinha de ganhar o empate"
    );
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        0,
        "o empate nao pode deixar um override para tras"
    );
}

/// ⭐ **DEVOLVER a peça ao mestre** — o inverso, e a saída de um override.
///
/// ⚠️ Sem este verbo um override é uma armadilha: o artista muda um valor por engano e a peça fica
/// surda à receita **para sempre**, sem nada na tela a dizer porquê.
#[test]
fn revert_gives_the_piece_back_to_the_master() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let key = ph2d_ecs::OverrideKey {
        piece: sim
            .world()
            .get::<StableId>(piece(&sim, master, "Arm"))
            .expect("id")
            .0,
        type_id: ph2d_ecs::scene::stable_type_id("ph2d::render::Sprite"),
    };
    assert!(super::revert_override(&mut sim, &mut echo, roots[0], key));
    assert!(
        !super::revert_override(&mut sim, &mut echo, roots[0], key),
        "devolver duas vezes tem de dizer que nao havia o que devolver"
    );
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, mine),
        tint(&sim, piece(&sim, master, "Arm")),
        "depois do revert a peca tinha de voltar a ouvir a receita"
    );
}

/// ⚠️ **A pose que o solver possui NUNCA vira override** — a outra metade da condição (b).
///
/// O corpo cai, o `Transform` dele muda por tique, e nada disso é autoria. Sem esta metade, cada
/// instância com um corpo dinâmico ganharia um override por quadro e ficaria surda à receita.
#[test]
fn a_pose_the_solver_owns_never_becomes_an_override() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = super::MasterEcho::default();
    let (_master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    let mut bridge = PhysicsBridge::new();
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    for t in 1..=60 {
        bridge.dispatch(&mut sim, true, t);
        sync_instances(&mut sim, &r, &bridge, &mut echo);
    }
    for (i, &root) in roots.iter().enumerate() {
        let n = sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(root)
            .map_or(0, |o| o.overrides.len());
        // ⚠️ O diagnóstico que tornou este defeito ACHÁVEL: sem o nome do componente, um
        // *«ganhou 1 override»* manda procurar em 76 tipos. Foi assim que o `PhysicsJoint`
        // apareceu.
        if n != 0 {
            let ov = sim.world().get::<ph2d_ecs::ObjectInstance>(root).unwrap();
            for k in &ov.overrides {
                let nm = r
                    .get_by_id(k.type_id)
                    .map(|e| e.canonical_name)
                    .unwrap_or("?");
                println!("SONDA override: peca={} componente={nm}", k.piece);
            }
        }
        assert_eq!(
            n,
            0,
            "a instancia {} ganhou {n} override(s) por CAIR — o solver nao e' o artista",
            i + 1
        );
    }
}

/// ⭐ **O override SOBREVIVE ao save e ao undo** — porque é um componente registado como qualquer
/// outro, e não uma tabela paralela.
///
/// ⚠️ Sem isto o artista perde toda excepção ao fechar o projeto, e o sync achata-as no primeiro
/// quadro depois de abrir.
#[test]
fn an_override_survives_a_capture_and_restore() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (_master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    let want = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(roots[0])
        .cloned()
        .expect("o conjunto existe");
    assert_eq!(want.overrides.len(), 1);

    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = ph2d_ecs::scene::WorldSnapshot::default();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap)
        .expect("captura");
    let mut fresh = SimWorld::new();
    ph2d_ecs::scene::snapshot_to_world(fresh.world_mut(), &snap, &r).expect("restore");

    let mut q = fresh.world_mut().query::<&ph2d_ecs::ObjectInstance>();
    let all: Vec<_> = q.iter(fresh.world()).cloned().collect();
    assert_eq!(
        all,
        vec![want],
        "os overrides nao voltaram do restore — o artista perde as excepcoes ao reabrir"
    );
}

/// ⛔⛔ **DECLARADO: um componente que carrega REFERÊNCIA propaga, mas nunca captura override.**
///
/// Medido ao escrever o gate irmão (`a_pose_the_solver_owns_never_becomes_an_override`): o solver
/// **escreve dentro do `PhysicsJoint`** — ele semeia `local_a`/`local_b` e vira o `anchored` no
/// 1.º reconcile —, e de fora isso é indistinguível de uma edição do artista. A 1.ª versão
/// capturava, e toda instância com uma junta ganhava um override no primeiro tique, ficando surda
/// à receita para sempre.
///
/// ⇒ editar a junta de uma instância vale **até o mestre mexer na dele**. O gate mede as duas
/// metades: a edição fica, e a edição do mestre leva-a.
#[test]
fn a_ref_carrying_component_never_captures_an_override_and_that_is_declared() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    // O artista mexe na junta de UMA instância.
    let mine = piece(&sim, roots[0], "Pin");
    let mut j = sim
        .world()
        .get::<PhysicsJoint>(mine)
        .copied()
        .expect("junta");
    j.motor_max_force = 42.0;
    sim.world_mut().entity_mut(mine).insert(j);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<PhysicsJoint>(mine)
            .unwrap()
            .motor_max_force,
        42.0,
        "a edicao do artista foi apagada no mesmo quadro"
    );
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        0,
        "um componente com referencia NAO pode capturar override (ver o doc)"
    );

    // E o mestre leva-a quando mexer na junta dele — a outra metade da declaração.
    let master_pin = piece(&sim, master, "Pin");
    let mut mj = sim
        .world()
        .get::<PhysicsJoint>(master_pin)
        .copied()
        .expect("junta");
    mj.motor_max_force = 7.0;
    sim.world_mut().entity_mut(master_pin).insert(mj);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<PhysicsJoint>(mine)
            .unwrap()
            .motor_max_force,
        7.0,
        "a receita tinha de levar a junta da instancia"
    );
}

/// ⭐ **O verbo INTEIRO: devolver a instância à receita** — o que o menu da Hierarquia chama.
///
/// ⚠️ **As três respostas são distinguíveis de propósito**: `None` = não é instância (o menu é
/// plano e o item aparece em toda linha, então o caminho negativo tem de saber dizer porquê),
/// `Some(0)` = é instância e não tinha excepção, `Some(n)` = devolveu `n`. *Um `None` para os dois
/// primeiros daria o mesmo aviso a duas situações que não são a mesma.*
#[test]
fn reverting_a_whole_instance_answers_the_three_cases() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    // (a) não é instância — o mestre não é uma instância de nada.
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, master),
        None
    );
    // (b) é instância e não tem excepção.
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, roots[0]),
        Some(0)
    );
    // (c) tem uma, e volta a ouvir a receita.
    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, roots[0]),
        Some(1)
    );
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(tint(&sim, mine), tint(&sim, piece(&sim, master, "Arm")));
}
