//! ⭐⭐⭐ **AS DUAS PORTAS DE AUTORIA que o sistema vetorial tem — medidas no mecanismo GERAL**
//! (ADR-0164 / a pré-condição da F4.6c).
//!
//! # Por que estes gates existem antes de qualquer linha ser apagada
//!
//! A F4.6c apaga o `VecInstance` (~2 961 LOC em 24 ficheiros). A medição de 2026-08-27 nomeia
//! **duas** rotas que a contagem de verbos daquele sistema deixava de fora — `toggle_piece_visible`
//! e a swatch de cor **por peça** — e diz que elas são *«a única porta que PRODUZ um override no
//! sistema vetorial»*. ⇒ *um porte que apaga uma feature não é um porte.*
//!
//! ⚠️ **A pergunta certa não é «como as portamos?», é «elas já existem do lado geral?»** — a lei do
//! §5.0: *confira o CÓDIGO antes de acreditar numa ausência*. Estes dois gates respondem, e ficam
//! como **cerca**: quem apagar o sistema vetorial pode apontar para eles.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform, Visibility};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_render::Sprite;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::component_registry_for_tests::registo()
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

/// `Robot` > `Body` (uma sprite), e uma cópia dela.
fn scene() -> (SimWorld, ph2d_ecs::scene::ComponentRegistry, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Robot"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Body"),
        Sprite::atlas(0, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let inst = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    (sim, r, master, inst)
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

/// ⭐⭐⭐ **PORTA 1 — esconder UMA peça de UMA cópia fica, e não sobe à receita.**
///
/// É o `toggle_piece_visible` do sistema vetorial, feito pelo **olho da Hierarquia**, que é um gesto
/// da casa. ⚠️ A `Visibility` está no `ROOT_IS_ITS_OWN` só para a **raiz** — esconder uma **peça**
/// é autoria e propaga da receita para as cópias; o contrário (a cópia esconder a peça dela) tem de
/// virar excepção.
///
/// ⛔ Se este gate reprovar, a F4.6c **apaga uma feature**: o artista perde a única forma de tirar
/// uma peça de uma cópia sem a apagar.
#[test]
fn hiding_one_piece_of_one_copy_sticks_and_does_not_reach_the_recipe() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (mine, theirs) = (piece(&sim, inst, "Body"), piece(&sim, master, "Body"));

    sim.world_mut()
        .entity_mut(mine)
        .insert(Visibility { hidden: true });
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert!(
        sim.world()
            .get::<Visibility>(mine)
            .is_some_and(|v| v.hidden),
        "a peca da copia voltou a aparecer — o passe reescreveu a decisao do artista"
    );
    assert!(
        !sim.world()
            .get::<Visibility>(theirs)
            .is_some_and(|v| v.hidden),
        "esconder a peca de UMA copia escondeu-a na RECEITA"
    );
}

/// ⭐⭐⭐ **PORTA 2 — a COR de uma peça de uma cópia é dela, e sobrevive à edição seguinte da
/// receita.**
///
/// É a swatch de cor **por peça** do sistema vetorial. ⚠️ **A segunda metade é a que importa:** que
/// a cor fique enquanto ninguém toca no mestre não prova nada — o passe só reescreve o que o mestre
/// mexeu. O que faz disto um **override** é a cor da cópia sobreviver a uma edição da receita.
///
/// ⛔ Se este gate reprovar, a F4.6c apaga a única porta que PRODUZ uma excepção no sistema
/// vetorial.
#[test]
fn the_colour_of_one_piece_of_one_copy_survives_the_next_edit_of_the_recipe() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let (mine, theirs) = (piece(&sim, inst, "Body"), piece(&sim, master, "Body"));

    // O artista pinta a peça DESTA cópia de laranja.
    let orange = [0.95, 0.55, 0.20, 1.0];
    if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(mine) {
        s.tint = orange;
    }
    pass(&mut sim, &r, &mut echo);

    // E depois pinta a RECEITA de azul — a cópia tem de ficar laranja.
    let blue = [0.35, 0.55, 0.85, 1.0];
    if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(theirs) {
        s.tint = blue;
    }
    pass(&mut sim, &r, &mut echo);
    pass(&mut sim, &r, &mut echo);

    assert_eq!(
        sim.world().get::<Sprite>(mine).map(|s| s.tint),
        Some(orange),
        "a cor que o artista deu a' peca DESTA copia foi achatada pela receita — nao ha' excepcao"
    );
    assert_eq!(
        sim.world().get::<Sprite>(theirs).map(|s| s.tint),
        Some(blue),
        "a receita perdeu a propria cor"
    );
}
