//! Os gates da FORMA de uma instância (ADR-0164 / F5.1).
//!
//! ⚠️ **O oráculo é a ÁRVORE da instância depois do passe**, e nunca «o passe correu»: um gate que
//! contasse materializações ficaria verde sobre um passe que põe a peça no pai errado.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Children, Entity, MasterRoot, Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

fn pass(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
) -> usize {
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
    )
}

fn instantiate(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: Entity,
) -> Entity {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou")
}

/// Uma receita de uma peça, e uma instância dela.
fn scene() -> (SimWorld, ph2d_ecs::scene::ComponentRegistry, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, master);
    (sim, r, master, inst)
}

fn names(sim: &SimWorld, root: Entity) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root
            && let Some(n) = sim.world().get::<Name>(e)
        {
            out.push(n.0.clone());
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    out.sort();
    out
}

/// ⭐⭐⭐ **Acrescentar uma peça ao mestre MATERIALIZA-A em todas as cópias** — a promessa da tabela
/// do doc 04 §2.6, que nada cumpria.
///
/// ⛔ Medido por sonda em 2026-08-27: `a_inst tem 0 filho(s) depois do passe`. O laço de valores
/// percorre **pares**, e uma peça que só existe do lado do mestre não forma par nenhum — ela é
/// invisível para ele por construção. Para o artista: *«acrescentei uma peça ao componente e as
/// cópias não mudaram»*.
///
/// ⚠️ E os **dois** lados: a peça aparece **e** traz o valor do mestre no MESMO passe. Materializar
/// sem sincronizar deixaria a cópia com o valor do momento, até alguém tocar no mestre outra vez.
///
/// (Mutação: não chamar o `reconcile` no `sync_instances` ⇒ RED na ausência.)
#[test]
fn a_piece_added_to_the_master_materialises_in_every_copy() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(names(&sim, inst), vec!["Box".to_string()]);

    // O gesto: o artista acrescenta uma peça à receita.
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(0.0, 2.0)),
        Name::new("Label"),
        ph2d_render::Sprite::atlas(0, [0.5, 0.2], [0.25, 0.5, 0.75, 1.0]),
        ChildOf(master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");
    assert_eq!(
        names(&sim, inst),
        vec!["Box".to_string(), "Label".to_string()],
        "a copia nao recebeu a peca nova"
    );
    // ⭐ E ela chega com o VALOR do mestre, no mesmo passe.
    let label = {
        let mut found = None;
        for e in sim
            .world()
            .get::<Children>(inst)
            .map(|c| c.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default()
        {
            if sim.world().get::<Name>(e).is_some_and(|n| n.0 == "Label") {
                found = Some(e);
            }
        }
        found.expect("a peca nova")
    };
    assert_eq!(
        sim.world()
            .get::<ph2d_render::Sprite>(label)
            .expect("sprite")
            .tint,
        [0.25, 0.5, 0.75, 1.0],
        "a peca nova chegou sem o valor da receita"
    );
    // ⚠️ E o passe assenta: a forma é um ponto fixo como os valores.
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe nao assentou");
}

/// ⭐⭐ **E apagar uma peça do mestre TIRA-A das cópias** — a outra metade, e ela não pode ir
/// sozinha: acrescentar sem remover deixa na cena um objeto que o artista apagou da biblioteca.
///
/// (Mutação: apagar o laço das que SOBRAM ⇒ RED.)
#[test]
fn a_piece_deleted_in_the_master_leaves_every_copy() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    assert_eq!(names(&sim, inst), vec!["Box".to_string()]);

    let box_piece = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca do mestre");
    sim.world_mut().entity_mut(box_piece).despawn();
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");
    assert!(
        names(&sim, inst).is_empty(),
        "a copia ficou com uma peca que a receita ja' nao tem: {:?}",
        names(&sim, inst)
    );
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe nao assentou");
}

/// ⛔⛔ **O que o ARTISTA pendurou numa cópia NÃO é uma peça a mais** — ele nunca veio do mestre,
/// logo apagá-lo seria apagar trabalho que ninguém pediu.
///
/// ⚠️ É a fronteira que separa *«a forma segue a receita»* de *«a receita é dona de tudo o que está
/// aqui dentro»*. O sinal é o elo: só o que a receita deu é que a receita tira.
///
/// (Mutação: tratar uma entidade sem `InstanceOf` como sobra ⇒ RED.)
#[test]
fn what_the_artist_hung_on_a_copy_is_not_a_leftover_piece() {
    let (mut sim, r, _master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let mine = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Mine"), ChildOf(inst)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    pass(&mut sim, &r, &mut echo);
    assert!(
        sim.world().get_entity(mine).is_ok(),
        "o passe apagou o que o artista pendurou na copia"
    );
}

/// ⛔ **Uma instância cujo MESTRE inteiro desapareceu fica em paz** — é a lei que já existia
/// (`a_dangling_link_is_left_alone`), e este passe não pode passar por cima dela.
///
/// ⚠️ A diferença é o SUJEITO: mestre presente e peça ausente é uma peça a mais; mestre ausente é
/// uma instância órfã, e apagá-la seria o passe a comer a cena por causa de um `Delete`.
#[test]
fn an_instance_whose_master_is_gone_keeps_its_pieces() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    for e in [master]
        .into_iter()
        .chain(
            sim.world()
                .get::<Children>(master)
                .map(|c| c.iter().copied().collect::<Vec<_>>())
                .unwrap_or_default(),
        )
        .collect::<Vec<_>>()
    {
        if let Ok(em) = sim.world_mut().get_entity_mut(e) {
            em.despawn();
        }
    }
    assert_eq!(pass(&mut sim, &r, &mut echo), 0, "o passe mexeu numa orfa");
    assert_eq!(
        names(&sim, inst),
        vec!["Box".to_string()],
        "a instancia orfa perdeu as pecas dela"
    );
}

/// ⭐⭐ **E uma peça acrescentada FUNDO aterra debaixo do pai DELA** — não na raiz da cópia.
///
/// ⛔⛔ **Este gate existe porque a mutação do irmão SOBREVIVEU.** A fixtura dele é plana (a peça
/// nova é filha da raiz), então *«pôr no pai certo»* e *«pôr na raiz»* dão o mesmo resultado — e a
/// mutação que troca um pelo outro passava. *Uma fixtura de um nível não pode medir de que nível a
/// peça é.*
///
/// (Mutação: usar `root` como `host` em vez de `have[parent_sid]` ⇒ RED.)
#[test]
fn a_piece_added_deep_lands_under_its_own_parent() {
    let (mut sim, r, master, inst) = scene();
    let mut echo = MasterEcho::default();
    pass(&mut sim, &r, &mut echo);
    let box_master = sim
        .world()
        .get::<Children>(master)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca do mestre");
    // O gesto: uma peça NETA — filha da peça, não da raiz.
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Pip"),
        ph2d_render::Sprite::atlas(0, [0.2, 0.2], [1.0; 4]),
        ChildOf(box_master),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    assert!(pass(&mut sim, &r, &mut echo) > 0, "o passe nao fez nada");

    let box_inst = sim
        .world()
        .get::<Children>(inst)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca da copia");
    let under_box: Vec<String> = sim
        .world()
        .get::<Children>(box_inst)
        .map(|c| {
            c.iter()
                .filter_map(|&e| sim.world().get::<Name>(e).map(|n| n.0.clone()))
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        under_box,
        vec!["Pip".to_string()],
        "a peca neta nao aterrou debaixo do pai dela — a arvore da copia deixou de ser a do mestre"
    );
}
