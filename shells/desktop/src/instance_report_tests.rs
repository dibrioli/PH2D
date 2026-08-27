//! Os gates dos **reports do smoke de 2026-08-26** — a pose que o *Revert* não mexe e os pixels
//! que sobem à receita.
//!
//! ⚠️ **Irmão de [`super::tests`] por assunto (e pelo tecto de 600 LOC):** lá mora a propagação e o
//! ponto fixo; aqui, o que o smoke do Enio devolveu.

use super::{MasterEcho, sync_instances};
use crate::instantiate::instantiate_master;
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// ⚠️ **Sem documentos vetoriais** — ver `crate::instance_sync_docs` para os que têm.
fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    bridge: &PhysicsBridge,
    echo: &mut MasterEcho,
) -> usize {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    sync_instances(
        sim,
        r,
        bridge,
        echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// ⚠️ **Sem documentos vetoriais** — idem.
fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
    parent: Option<Entity>,
) -> Result<Entity, crate::instantiate::Refusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    instantiate_master(
        sim,
        r,
        master,
        parent,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
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
    let inst = instantiate(&mut sim, &r, master, None).expect("instanciou");
    pass(&mut sim, &r, &bridge, &mut echo);

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
    pass(&mut sim, &r, &bridge, &mut echo);
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(inst)
            .map_or(0, |o| o.overrides.len()),
        2,
        "a fixtura tem de ter DUAS excepcoes (pose + cor), senao o gate nao separa nada"
    );

    let got = crate::instance_revert::revert_all_overrides(&mut sim, &mut echo, arm)
        .expect("e' uma instancia");
    assert_eq!(
        got,
        crate::instance_revert::Reverted {
            count: 1,
            poses_kept: 1
        },
        "o revert tem de devolver a COR e contar a pose que ficou"
    );
    // ⚠️ Depois do passe seguinte, que é onde a propagação de facto acontece.
    pass(&mut sim, &r, &bridge, &mut echo);
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

/// ⭐⭐⭐ **O OUTRO REPORT: pintar uma cópia LIGADA muda as irmãs** (Enio, 2026-08-26 → 27).
///
/// > *«Pintei uma sprite de uma instância e as outras não mudaram.»*
///
/// A edição de pixels sobe até à receita ([`crate::hero_intents::texture_rebind`]) e o passe
/// leva-a a toda a gente.
///
/// ⚠️ **Em 2026-08-27 isto passou a ser o modo LIGADO** (`Instantiate Linked`, o `Alt+D`) e deixou
/// de valer para toda cópia — porque valer para todas era metade de uma incoerência: a tinta subia
/// e a geometria vetorial da mesma cópia virava excepção. O irmão
/// [`painting_an_unlinked_copy_keeps_it_to_itself`] guarda o outro lado.
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
    let a = instantiate(&mut sim, &r, master, None).expect("instanciou A");
    let b = instantiate(&mut sim, &r, master, None).expect("instanciou B");
    // ⭐ As duas são LIGADAS — é este o modo cuja promessa o gate mede. (O `instantiate` local dá
    // `ArtLink::Own`, que é o outro lado e tem gate próprio no `texture_rebind`.)
    for root in [a, b] {
        for e in [root, piece(&sim, root, "Arm")] {
            sim.world_mut().entity_mut(e).insert(ph2d_ecs::LinkedArt);
        }
    }
    pass(&mut sim, &r, &bridge, &mut echo);

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
    pass(&mut sim, &r, &bridge, &mut echo);

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
        pass(&mut sim, &r, &bridge, &mut echo),
        0,
        "o passe deixou de ser ponto fixo depois de uma pintura"
    );
}
