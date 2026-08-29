//! Os gates da seção COMPONENT (ADR-0164 / F5).
//!
//! ⚠️ **O oráculo é o que o artista LÊ**, e nunca «o info existe»: uma seção publicada com a lista
//! vazia sobre uma cópia que tem excepções é exactamente o defeito que ela existe para curar.

use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma receita de uma peça + uma instância dela, já sincronizada.
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
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let inst = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instance_sync::sync_instances(
        &mut sim,
        &r,
        &ph2d_physics_ecs::PhysicsBridge::new(),
        &mut echo,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    );
    (sim, r, master, inst)
}

fn piece(sim: &SimWorld, root: Entity) -> Entity {
    sim.world()
        .get::<ph2d_ecs::Children>(root)
        .and_then(|c| c.iter().next().copied())
        .expect("a peca")
}

/// ⛔ **Um objeto que não é cópia de nada NÃO tem a seção** — a lei da F3 (o Inspector mostra o que
/// o objeto TEM), e a metade que impede doze seções de zeros.
///
/// ⚠️ **A mutação que o mata tem de remover as DUAS guardas juntas** (o elo e a raiz), e as duas
/// tentativas de o matar com uma só SOBREVIVERAM: cada guarda é neutralizada pela outra, porque um
/// objeto solto falha as duas. *Isso não é um gate fraco — é uma propriedade com duas defesas, e a
/// mutação honesta tem de tirar as duas.*
#[test]
fn a_plain_object_gets_no_component_section() {
    let (mut sim, r, master, _inst) = scene();
    let loose = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Loose")))
        .id();
    for (what, e) in [("um objeto solto", loose), ("a propria receita", master)] {
        assert!(
            super::build_instance_info(&mut sim, &r, Some(e.to_bits())).is_none(),
            "{what} publicou a secao COMPONENT"
        );
    }
}

/// ⭐⭐⭐ **A seção NOMEIA o componente que está overridado nesta peça** — o item que a lista de
/// abertos carregava desde 26/08 (*«nada na tela MOSTRA que campo está overridado»*).
///
/// ⚠️ **Os dois lados:** sem excepção nenhuma ela diz *«segue a receita»* — que é informação, e é a
/// diferença entre *«não mexi nesta»* e *«mexi e não vejo onde»*.
///
/// (Mutação: devolver a lista de overrides INTEIRA (sem o filtro por peça) ⇒ passa aqui e falha no
/// irmão `the_list_is_of_the_selected_piece_not_of_the_whole_copy`.)
#[test]
fn the_section_names_the_overridden_component_of_this_piece() {
    let (mut sim, r, _master, inst) = scene();
    let p = piece(&sim, inst);
    let before = super::build_instance_info(&mut sim, &r, Some(p.to_bits()))
        .expect("a peca de uma copia tem a secao");
    assert_eq!(before.master_name, "Badge");
    assert!(
        before.overridden.is_empty() && before.summary() == "Follows the component",
        "uma copia intacta ja' aparece com excepcoes: {before:?}"
    );

    // ⚠️ **O eco tem de existir ANTES da edição, e o gate apanhou-me a construí-lo depois.** Sem
    // eco não há atribuição: o passe cai na regra do 1.º encontro e **o mestre ganha**, então a
    // tinta era desfeita em vez de virar excepção. *Uma fixtura que semeia o eco depois do gesto
    // mede o app a arrancar, não o artista a editar.*
    let mut echo = crate::instance_sync::MasterEcho::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut run = |sim: &mut SimWorld, echo: &mut crate::instance_sync::MasterEcho| {
        crate::instance_sync::sync_instances(
            sim,
            &r,
            &ph2d_physics_ecs::PhysicsBridge::new(),
            echo,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
        );
    };
    run(&mut sim, &mut echo); // semeia
    // O gesto: o artista pinta a peça da cópia.
    let mut spr = sim
        .world()
        .get::<ph2d_render::Sprite>(p)
        .copied()
        .expect("sprite");
    spr.tint = [0.9, 0.1, 0.1, 1.0];
    sim.world_mut().entity_mut(p).insert(spr);
    run(&mut sim, &mut echo);
    let after = super::build_instance_info(&mut sim, &r, Some(p.to_bits())).expect("a secao");
    assert_eq!(
        after.overridden,
        vec!["Sprite".to_string()],
        "a secao nao nomeou o componente overridado — o artista continua sem o ver"
    );
    assert_eq!(after.summary(), "1 override(s) on this piece");
}

/// ⚠️ **A lista é da PEÇA selecionada, não da cópia inteira.** O conjunto mora na RAIZ e chaveia
/// por `(peça, tipo)`; mostrá-lo todo diria ao artista que ele mexeu em coisas que estão noutro
/// sítio da cópia.
///
/// (Mutação: tirar o `filter(|k| k.piece == link.master)` ⇒ RED.)
#[test]
fn the_list_is_of_the_selected_piece_not_of_the_whole_copy() {
    let (mut sim, r, _master, inst) = scene();
    let p = piece(&sim, inst);
    // Uma excepção fabricada numa peça QUE NÃO É a selecionada.
    let mut o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .unwrap_or_default();
    o.overrides.insert(ph2d_ecs::OverrideKey {
        piece: 999_999,
        type_id: ph2d_ecs::scene::stable_type_id("ph2d::render::Sprite"),
    });
    sim.world_mut().entity_mut(inst).insert(o);
    let info = super::build_instance_info(&mut sim, &r, Some(p.to_bits())).expect("a secao");
    assert!(
        info.overridden.is_empty(),
        "a secao mostrou a excepcao de OUTRA peca: {:?}",
        info.overridden
    );
}

/// ⭐⭐ **Os ÓRFÃOS contam-se, e o gesto que os limpa só toca NELES** (F5.3).
///
/// ⚠️ Eles são da instância INTEIRA — um órfão não tem peça, logo não há peça onde listá-lo. É o
/// mesmo sítio onde o Unity os põe.
///
/// (Mutação: o `clear_orphans` limpar também os `overrides` ⇒ RED na 2.ª metade.)
#[test]
fn the_orphans_are_counted_and_the_gesture_touches_only_them() {
    let (mut sim, r, _master, inst) = scene();
    let p = piece(&sim, inst);
    let live = ph2d_ecs::OverrideKey {
        piece: sim
            .world()
            .get::<ph2d_ecs::InstanceOf>(p)
            .expect("elo")
            .master,
        type_id: ph2d_ecs::scene::stable_type_id("ph2d::render::Sprite"),
    };
    let orphan = ph2d_ecs::OverrideKey {
        piece: 424_242,
        type_id: live.type_id,
    };
    let mut o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .unwrap_or_default();
    o.overrides.insert(live);
    o.orphans.insert(orphan, vec![1, 2, 3]);
    sim.world_mut().entity_mut(inst).insert(o);

    let info = super::build_instance_info(&mut sim, &r, Some(p.to_bits())).expect("a secao");
    assert_eq!(info.orphans, 1);
    assert_eq!(
        info.summary(),
        "1 override(s) on this piece \u{b7} 1 unused"
    );

    assert_eq!(super::clear_orphans(&mut sim, info.root_bits), 1);
    let after = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .expect("a raiz");
    assert!(after.orphans.is_empty(), "o gesto nao limpou os orfaos");
    assert!(
        after.overrides.contains(&live),
        "o gesto apagou uma excepcao VIVA — isso e' o *Revert to Master* com outro nome"
    );
}
