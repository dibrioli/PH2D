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
    o.orphans.insert(
        orphan,
        ph2d_ecs::OrphanOverride {
            bytes: vec![1, 2, 3],
            piece_name: "Arm".into(),
        },
    );
    sim.world_mut().entity_mut(inst).insert(o);

    let info = super::build_instance_info(&mut sim, &r, Some(p.to_bits())).expect("a secao");
    assert_eq!(info.orphans(), 1);
    // ⭐ E a linha NOMEIA a peça que morreu — sem isto o botão apaga o que ninguém viu.
    assert_eq!(
        info.orphan_rows[0].label(),
        "Sprite \u{2014} was on \u{201c}Arm\u{201d}"
    );
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

/// ⭐⭐ **A fileira de variantes lista a FAMÍLIA, e marca a vigente** (F5, critério 2).
///
/// ⚠️ **O oráculo é o conjunto de `StableId`s, e não a contagem**: um construtor que devolvesse
/// «duas» com o mestre errado lá dentro passaria num gate que contasse.
#[test]
fn the_card_lists_the_family_and_marks_the_current_one() {
    let (mut sim, r, base, variant) = family();
    let inst = instantiate(&mut sim, &r, base);
    let info =
        super::super::inspector_properties::build_properties_info(&mut sim, Some(inst.to_bits()))
            .expect("o cartao de propriedades");
    // ⚠️ **Um eixo agora, e no modo plano ele chama-se `Variant`** — a fileira é a mesma; o que
    // mudou é que ela passou a saber ter irmãs (a fatia dos eixos, 2026-08-30).
    // ⚠️ **E ela MUDOU-SE de cartão** (2026-08-31): as fileiras são do cartão de PROPRIEDADES, que
    // existe sem o de instância. Ver o cabeçalho do `inspector_properties`.
    let all: Vec<_> = info.rows.iter().flat_map(|a| a.options.iter()).collect();
    let got: Vec<u64> = all.iter().map(|v| v.master).collect();
    let (base_id, variant_id) = (sid(&sim, base), sid(&sim, variant));
    assert!(
        got.contains(&base_id) && got.contains(&variant_id),
        "a familia nao tem os dois mestres: {got:?}"
    );
    assert_eq!(
        all.iter().filter(|v| v.current).count(),
        1,
        "a fileira tem de dizer onde a copia esta'"
    );
    assert!(
        all.iter().any(|v| v.current && v.master == base_id),
        "a vigente marcada nao e' o mestre da copia"
    );
}

/// ⛔ **Um mestre SOZINHO não pinta fileira nenhuma** — *um valor que não leva a lado nenhum não é
/// oferecido*.
///
/// ⚠️ E o gate mede a AUSÊNCIA no sítio em que ela decide: com um chip único e já escolhido, a
/// fileira ocuparia uma linha do cartão para não permitir gesto nenhum.
#[test]
fn a_lonely_master_offers_no_variant_row() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Solo"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, master);
    let info =
        super::super::inspector_properties::build_properties_info(&mut sim, Some(inst.to_bits()));
    // ⚠️ **`None`, e não «uma lista vazia»**: `Solo` não declara propriedade nenhuma e não tem
    // família, então não há cartão — que é a lei da F3 (*o Inspector mostra o que o objeto TEM*).
    assert!(
        info.is_none(),
        "um mestre sem familia e sem chaves pintou um cartao: {info:?}"
    );
}

/// ⛔ **Um mestre NÃO aparentado fica de fora da fileira** — o construtor filtra pela MESMA
/// pergunta que a troca faz, e é isso que impede um chip que recusa ao ser clicado.
#[test]
fn an_unrelated_master_is_not_offered_as_a_variant() {
    let (mut sim, r, base, _variant) = family();
    let other = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Other"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let inst = instantiate(&mut sim, &r, base);
    let info =
        super::super::inspector_properties::build_properties_info(&mut sim, Some(inst.to_bits()))
            .expect("o cartao de propriedades");
    let other_id = sid(&sim, other);
    assert!(
        !info
            .rows
            .iter()
            .flat_map(|a| a.options.iter())
            .any(|v| v.master == other_id),
        "um mestre sem antepassado comum foi oferecido — o chip recusaria ao ser clicado"
    );
}

fn sid(sim: &ph2d_ecs::SimWorld, e: ph2d_ecs::Entity) -> u64 {
    sim.world().get::<ph2d_ecs::StableId>(e).expect("sid").0
}

fn instantiate(
    sim: &mut ph2d_ecs::SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    master: ph2d_ecs::Entity,
) -> ph2d_ecs::Entity {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    crate::instantiate::instantiate_master(
        sim,
        r,
        master,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou")
}

/// Uma base com uma peça, e uma variante DERIVADA dela (um mestre que também é instância).
fn family() -> (
    ph2d_ecs::SimWorld,
    ph2d_ecs::scene::ComponentRegistry,
    ph2d_ecs::Entity,
    ph2d_ecs::Entity,
) {
    let mut sim = ph2d_ecs::SimWorld::new();
    let r = crate::init::build_component_registry();
    let base = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::Name::new("Base"),
            ph2d_ecs::MasterRoot,
        ))
        .id();
    sim.world_mut().spawn((
        ph2d_ecs::Transform::IDENTITY,
        ph2d_ecs::Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ph2d_ecs::ChildOf(base),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let copy = instantiate(&mut sim, &r, base);
    sim.world_mut()
        .entity_mut(copy)
        .insert(ph2d_ecs::MasterRoot);
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (sim, r, base, copy)
}

/// ⭐⭐⭐ **O cartão diz o que o objeto É, não só o que ele segue** — report do Enio, 2026-08-27.
///
/// Uma variante é `MasterRoot` **e** `InstanceOf`: ela segue a base *e* é a receita das cópias
/// dela. Chamar-lhe *«Instance of "Base"»* é verdade e esconde a metade que decide — quem edita uma
/// variante muda todas as cópias dela.
///
/// ⚠️ **E a peça DENTRO da variante lê o mesmo**: a pergunta *«de que sou cópia?»* é da raiz.
///
/// (Mutação: `is_variant` fixo em `false` ⇒ RED.)
#[test]
fn the_card_calls_a_variant_a_variant_and_a_copy_a_copy() {
    let (mut sim, r, base, variant) = family();
    let plain = instantiate(&mut sim, &r, base);

    let v = super::build_instance_info(&mut sim, &r, Some(variant.to_bits())).expect("variante");
    assert!(v.is_variant, "a variante nao se declarou receita");
    assert!(
        v.provenance().starts_with("Variant of"),
        "o cartao da variante ainda diz «Instance»: {:?}",
        v.provenance()
    );

    let c = super::build_instance_info(&mut sim, &r, Some(plain.to_bits())).expect("copia");
    assert!(!c.is_variant, "uma copia comum declarou-se receita");
    assert!(
        c.provenance().starts_with("Instance of"),
        "o cartao de uma copia comum deixou de dizer «Instance»: {:?}",
        c.provenance()
    );

    // A peça DENTRO da variante: mesma resposta, porque a pergunta é da raiz.
    let piece = *sim
        .world()
        .get::<ph2d_ecs::Children>(variant)
        .expect("a variante tem pecas")
        .first()
        .expect("uma peca");
    let p = super::build_instance_info(&mut sim, &r, Some(piece.to_bits())).expect("peca");
    assert!(
        p.is_variant,
        "uma peca dentro de uma variante leu-se como copia comum — a pergunta e' da RAIZ"
    );
}

/// ⛔⛔ **A proveniência mostra o nome TAL COMO ELE É** (Enio, 2026-09-01).
///
/// Até 31/08 ela **cortava** as chaves (`Canvas {Size=Small} Variant` → `Canvas Variant`), porque
/// a declaração vivia no nome e a frase inteira quebrava em duas linhas num cartão cuja altura é
/// contada em linhas de texto. O mecanismo de propriedades foi recusado e está adiado ⇒ **não há
/// nada a cortar**: um nome comprido é escolha do artista, e comer-lhe pedaços seria o app a
/// corrigir o que ele escreveu.
///
/// ⚠️ **A fixtura carrega chaves de propósito** — é o que a lei velha comia. Com um nome limpo
/// este gate ficaria verde sem provar nada.
///
/// (Mutação: reintroduzir um corte por `{` ⇒ RED.)
#[test]
fn the_provenance_line_shows_the_name_verbatim() {
    let info = ph2d_editor::screens::hero::InspectorInstanceInfo {
        master_name: "Canvas {Size=Small} Variant".into(),
        is_variant: false,
        ..Default::default()
    };
    assert_eq!(
        info.provenance(),
        "Instance of \u{201c}Canvas {Size=Small} Variant\u{201d}"
    );
}

/// ⭐⭐⭐ **Largar UMA excepção sem alvo tira EXACTAMENTE aquela** (F5.3-ter).
///
/// ⚠️ Duas metades. A CHAVE tem de chegar à linha — sem ela o `✕` não tem o que apontar, e os dois
/// campos de texto são rótulos que duas peças podem partilhar. E o gesto tem de tirar **uma**: um
/// braço que caísse no `clear` seria o botão de baixo com outro ícone.
///
/// (Mutação: o construtor não escrever `piece_id`/`type_id` ⇒ a 1.ª metade sangra. O `drop_orphan`
/// a chamar `orphans.clear()` ⇒ a 2.ª.)
#[test]
fn dropping_one_unused_override_leaves_the_others_alone() {
    let (mut sim, r, _master, inst) = scene();
    let p = piece(&sim, inst);
    let sprite = ph2d_ecs::scene::stable_type_id("ph2d::render::Sprite");
    let mut o = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .unwrap_or_default();
    for (piece_sid, name) in [(111_u64, "Arm"), (222, "Leg")] {
        o.orphans.insert(
            ph2d_ecs::OverrideKey {
                piece: piece_sid,
                type_id: sprite,
            },
            ph2d_ecs::OrphanOverride {
                bytes: vec![9],
                piece_name: name.into(),
            },
        );
    }
    sim.world_mut().entity_mut(inst).insert(o);

    let info = super::build_instance_info(&mut sim, &r, Some(p.to_bits())).expect("a secao");
    assert_eq!(info.orphan_rows.len(), 2);
    // ⚠️ A CHAVE, e não o rótulo: é ela que o `✕` manda pelo barramento.
    let arm = info
        .orphan_rows
        .iter()
        .find(|row| row.piece == "Arm")
        .expect("a linha do Arm");
    assert_eq!(
        (arm.piece_id, arm.type_id),
        (111, sprite),
        "a chave nao chegou a' linha — o `x` dela nao teria o que apontar"
    );

    assert!(super::drop_orphan(
        &mut sim,
        info.root_bits,
        arm.piece_id,
        arm.type_id
    ));
    let after = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(inst)
        .cloned()
        .expect("a raiz");
    assert_eq!(after.orphans.len(), 1, "o gesto levou mais do que uma");
    assert!(
        after
            .orphans
            .values()
            .all(|orphan| orphan.piece_name == "Leg"),
        "levou a errada"
    );
    // ⛔ E uma chave que já não está lá não mexe em nada, nem finge que mexeu.
    assert!(!super::drop_orphan(&mut sim, info.root_bits, 111, sprite));
}
