//! Os gates dos três verbos que fecham a tabela (ADR-0164 / F4.5).
//!
//! ⚠️ **O oráculo é o que o ARTISTA vê depois do gesto** — o que está na tela, o que a receita
//! passou a ter, o que as outras cópias receberam. Um gate que contasse chamadas ficaria verde
//! sobre um verbo que faz a coisa errada.

use super::{VerbRefusal, apply_to_master, detach, make_master};
use crate::instance_smoke::{spawn_master, spawn_ragdoll_scene};
use crate::instance_sync::{MasterEcho, sync_instances};
use crate::instantiate::instantiate_master;
use ph2d_ecs::{
    Children, Entity, InstanceOf, MasterRoot, Name, ObjectInstance, SimWorld, Transform, Visibility,
};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

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

/// Uma subárvore comum na cena: um corpo com uma peça pendurada.
fn plain_rig(sim: &mut SimWorld) -> Entity {
    let root = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(3.0, 1.0)),
            Name::new("Rig"),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Arm"),
        ph2d_render::Sprite::atlas(
            ph2d_render::WHITE_TILE_KEY,
            [1.0, 0.2],
            [0.5, 0.5, 0.5, 1.0],
        ),
        ph2d_ecs::ChildOf(root),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    root
}

// ── CRIAR COMPONENTE ───────────────────────────────────────────────────────────────────────

/// ⭐⭐ **A seleção vira RECEITA e uma INSTÂNCIA fica no lugar dela** — o gesto do Unity
/// *Create Prefab*.
///
/// (Mutação: não instanciar ⇒ o objeto some da tela, e o gate reprova nomeando a pose.)
#[test]
fn make_master_leaves_an_instance_in_its_place() {
    let mut sim = SimWorld::new();
    let r = reg();
    let rig = plain_rig(&mut sim);
    let where_it_was = sim.world().get::<Transform>(rig).expect("pose").translation;

    let (master, instance) = make_master(&mut sim, &r, rig).expect("o gesto");
    assert_eq!(
        master, rig,
        "a receita E' a subarvore que o artista escolheu"
    );
    assert!(sim.world().get::<MasterRoot>(master).is_some());
    assert!(sim.world().get::<InstanceOf>(instance).is_some());
    // ⚠️ E ela está no lugar porque a **cópia profunda leva o `Transform` verbatim** — não porque
    // o verbo o reescreva. A 1.ª versão reescrevia, e a prova de mutação mostrou a linha morta.
    assert_eq!(
        sim.world()
            .get::<Transform>(instance)
            .expect("pose")
            .translation,
        where_it_was,
        "a instancia nao ficou NO LUGAR da selecao"
    );
    // E ela traz a subárvore inteira.
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(piece(&sim, instance, "Arm"))
            .map(|s| s.size),
        Some([1.0, 0.2]),
        "a instancia nasceu sem a peca"
    );
}

/// ⚠️⚠️ **A RECEITA fica escondida e a INSTÂNCIA visível** — senão o artista vê dois objetos
/// empilhados, um que cai e outro que não, e lê isso como defeito.
///
/// ⚠️ E a segunda metade é a que quase escapou: sem a `Visibility` no `ROOT_IS_ITS_OWN`, o
/// `hidden` da receita **propagava** e a instância nascia invisível — o gesto apagaria da tela o
/// objeto que o artista acabou de transformar em componente.
///
/// (Mutação: tirar `"ph2d::ecs::Visibility"` do `ROOT_IS_ITS_OWN` ⇒ RED depois do 1.º sync.)
#[test]
fn the_recipe_hides_and_the_instance_does_not() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let rig = plain_rig(&mut sim);
    let (master, instance) = make_master(&mut sim, &r, rig).expect("o gesto");

    assert!(
        sim.world()
            .get::<Visibility>(master)
            .is_some_and(|v| v.hidden),
        "a receita ficou visivel — o artista ve' dois objetos empilhados"
    );
    for _ in 0..3 {
        sync_instances(&mut sim, &r, &bridge, &mut echo);
    }
    assert!(
        !sim.world()
            .get::<Visibility>(instance)
            .is_some_and(|v| v.hidden),
        "a instancia herdou o `hidden` da receita — o gesto APAGOU da tela o que o artista escolheu"
    );
}

/// ⛔ **Duas recusas, distinguíveis.**
#[test]
fn make_master_refuses_a_master_and_a_piece_of_an_instance() {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = spawn_master(&mut sim);
    assert_eq!(
        make_master(&mut sim, &r, master),
        Err(VerbRefusal::AlreadyAMaster)
    );
    let inst = instantiate_master(&mut sim, &r, master, None).expect("instancia");
    assert_eq!(
        make_master(&mut sim, &r, inst),
        Err(VerbRefusal::InsideAnInstance)
    );
    // ⚠️ E uma PEÇA no meio da cópia também: a pergunta é sobre os ANCESTRAIS.
    assert_eq!(
        {
            let arm = piece(&sim, inst, "Arm");
            make_master(&mut sim, &r, arm)
        },
        Err(VerbRefusal::InsideAnInstance)
    );
}

// ── DESTACAR ───────────────────────────────────────────────────────────────────────────────

/// ⭐ **Destacar corta o vínculo e não muda mais nada** — os objetos continuam iguais, só deixam
/// de seguir a receita.
///
/// (Mutação: não remover o `InstanceOf` das PEÇAS ⇒ o sync continua a alcançá-las e o gate
/// reprova quando a receita muda.)
#[test]
fn detaching_stops_the_following_and_changes_nothing_else() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    let before = tint(&sim, mine);
    // O gesto, feito a partir de uma PEÇA — e ele solta a instância INTEIRA.
    assert_eq!(detach(&mut sim, mine), Ok(4));
    assert!(sim.world().get::<InstanceOf>(roots[0]).is_none());
    assert!(sim.world().get::<InstanceOf>(mine).is_none());
    assert_eq!(tint(&sim, mine), before, "destacar mudou o que se ve'");

    // A receita muda; a solta não ouve mais, e as outras duas ouvem.
    let master_arm = piece(&sim, master, "Arm");
    paint(&mut sim, master_arm, [0.9, 0.9, 0.1, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        tint(&sim, mine),
        before,
        "a solta continuou a seguir a receita"
    );
    assert_eq!(
        tint(&sim, piece(&sim, roots[1], "Arm")),
        [0.9, 0.9, 0.1, 1.0],
        "as outras deixaram de seguir — destacar UMA soltou todas"
    );
}

/// ⛔ **Destacar o que não é instância é recusado** (a receita não é cópia de ninguém).
#[test]
fn detaching_something_that_is_not_an_instance_is_refused() {
    let mut sim = SimWorld::new();
    let master = spawn_master(&mut sim);
    assert_eq!(detach(&mut sim, master), Err(VerbRefusal::NotAnInstance));
}

// ── APLICAR AO MESTRE ──────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **APLICAR promove a excepção a padrão** — o valor entra na receita, e as OUTRAS cópias
/// recebem-no.
///
/// ⚠️ **É a régua inteira do verbo**: se ele só apagasse a excepção, a cópia voltaria à cor antiga
/// (isso é o *Redefinir*); se só escrevesse na receita sem limpar a chave, a cópia continuaria
/// surda e a diferença voltaria no gesto seguinte.
///
/// (Mutação: trocar o `insert_from_bytes` no mestre por um no-op ⇒ as outras não recebem nada.)
#[test]
fn applying_an_override_makes_it_the_recipe_for_everyone() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let mine = piece(&sim, roots[0], "Arm");
    paint(&mut sim, mine, [0.1, 0.2, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a fixtura tem de conter a excepcao, senao o gate nao mede nada"
    );

    assert_eq!(apply_to_master(&mut sim, &r, &mut echo, mine), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Arm")),
        [0.1, 0.2, 0.9, 1.0],
        "o valor nao chegou a' RECEITA"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        0,
        "a excepcao sobreviveu ao Apply — a copia fica surda ao proprio valor que promoveu"
    );

    sync_instances(&mut sim, &r, &bridge, &mut echo);
    for (i, &root) in roots.iter().enumerate() {
        assert_eq!(
            tint(&sim, piece(&sim, root, "Arm")),
            [0.1, 0.2, 0.9, 1.0],
            "a instancia {} nao recebeu o valor promovido",
            i + 1
        );
    }
}

/// ⚠️ **O ESCOPO é o que se clicou** — numa peça, só a excepção dela.
#[test]
fn applying_from_a_piece_promotes_only_that_piece() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    let arm = piece(&sim, roots[0], "Arm");
    let hub = piece(&sim, roots[0], "Hub");
    let hub_before = tint(&sim, piece(&sim, master, "Hub"));
    paint(&mut sim, arm, [0.1, 0.2, 0.9, 1.0]);
    paint(&mut sim, hub, [0.9, 0.1, 0.9, 1.0]);
    sync_instances(&mut sim, &r, &bridge, &mut echo);

    assert_eq!(apply_to_master(&mut sim, &r, &mut echo, arm), Ok(1));
    assert_eq!(
        tint(&sim, piece(&sim, master, "Hub")),
        hub_before,
        "aplicar o BRACO promoveu tambem o eixo — o escopo esta' errado"
    );
    assert_eq!(
        sim.world()
            .get::<ObjectInstance>(roots[0])
            .map_or(0, |o| o.overrides.len()),
        1,
        "a excepcao do eixo tinha de ficar"
    );
}

/// **Sem excepção nenhuma o verbo responde ZERO** — e não um erro: o artista clicou no sítio certo.
#[test]
fn applying_with_nothing_overridden_answers_zero() {
    let mut sim = SimWorld::new();
    let r = reg();
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let (_master, roots) = spawn_ragdoll_scene(&mut sim, &r);
    sync_instances(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(apply_to_master(&mut sim, &r, &mut echo, roots[0]), Ok(0));
}
