//! Os gates dos OVERRIDES (ADR-0164 / F4.4) — irmão de [`super::tests`] pelo teto de 600 LOC da
//! shell, que os dois juntos estouraram em 870.
//!
//! ⚠️ **O corte é por ASSUNTO:** lá fica o que prova que a receita CHEGA às cópias; aqui, o que
//! prova que a EXCEPÇÃO do artista resiste a ela. ⛔ Não devolva um teste ao irmão — o teto volta a
//! estourar no gate seguinte, e o corte teria sido pago à toa.

use super::sync_instances;
use crate::instance_smoke::spawn_ragdoll_scene;
use ph2d_ecs::{Children, Entity, Name, SimWorld, StableId};
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

    // (a) não pertence a instância nenhuma — a receita não é cópia de ninguém.
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, master),
        None
    );
    // ⚠️ E nem uma PEÇA da receita: subir por `ChildOf` a partir dela chega ao mestre, e um
    // mestre não é uma instância. *A metade que impede o verbo de morder a própria biblioteca.*
    let master_arm = piece(&sim, master, "Arm");
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, master_arm),
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

/// ⭐⭐⭐ **O GESTO DO ENIO (report 2026-08-26): botão direito na PEÇA que ele acabou de pintar.**
///
/// ⛔ **A 1.ª versão respondia *«Not an instance»*** — e estava tecnicamente certa e
/// **inutilmente** certa: o verbo só aceitava a RAIZ da instância, e para pintar o braço o artista
/// teve de selecionar a linha do **braço**. É lá que a mão dele está quando ele quer desfazer.
///
/// ⚠️ *Um aviso que diz o que a coisa NÃO é, sem dizer o que fazer, é um botão mudo com legenda.*
///
/// ⇒ o verbo aceita qualquer peça, sobe até à raiz, e **devolve o que se clicou**: numa peça, só a
/// excepção dela; na raiz, todas.
#[test]
fn reverting_from_the_piece_the_artist_touched_works() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = super::MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let arm = piece(&sim, roots[0], "Arm");
    let hub = piece(&sim, roots[0], "Hub");
    paint(&mut sim, arm, [0.1, 0.2, 0.9, 1.0]);
    paint(&mut sim, hub, [0.9, 0.1, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        2,
        "a fixtura tem de conter DUAS excepcoes, senao o escopo abaixo nao mede nada"
    );

    // O gesto: botão direito no BRAÇO.
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, arm),
        Some(1),
        "clicar na peca tem de devolver a excepcao DELA"
    );
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, arm),
        tint(&sim, piece(&sim, master, "Arm")),
        "o braco nao voltou a ouvir a receita"
    );
    assert_eq!(
        tint(&sim, hub),
        [0.9, 0.1, 0.9, 1.0],
        "devolver o BRACO levou tambem a excepcao do EIXO — o escopo esta' errado"
    );

    // E na RAIZ continua a devolver tudo o que resta.
    assert_eq!(
        super::revert_all_overrides(&mut sim, &mut echo, roots[0]),
        Some(1)
    );
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(tint(&sim, hub), tint(&sim, piece(&sim, master, "Hub")));
}
