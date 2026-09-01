//! Os gates do cartão de PROPRIEDADES ([`super`]).
//!
//! ⚠️ **O oráculo é o que o artista LÊ.** Os gates puros da lei (`variant_axes_tests`) medem a
//! derivação a partir de dados; o que só se mede aqui é **de onde esses dados são lidos**.

use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, VariantValues};

fn values(pairs: &[(&str, &str)]) -> VariantValues {
    VariantValues {
        values: pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
    }
}

fn spawn_named(sim: &mut SimWorld, name: &str) -> Entity {
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    e
}

/// Uma família com duas versões e uma cópia da base. Devolve `(base, variante, cópia)`.
fn family(sim: &mut SimWorld) -> (Entity, Entity, Entity) {
    let r = crate::init::build_component_registry();
    let base = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Casa"),
            MasterRoot,
            values(&[("Size", "Small")]),
        ))
        .id();
    sim.world_mut().spawn((
        Transform::IDENTITY,
        Name::new("Box"),
        ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        ChildOf(base),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let variant = crate::instantiate::instantiate_master(
        sim,
        &r,
        base,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou a variante");
    sim.world_mut().entity_mut(variant).insert((
        MasterRoot,
        Name::new("Casa Variant"),
        values(&[("Size", "Big")]),
    ));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let copy = crate::instantiate::instantiate_master(
        sim,
        &r,
        base,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou a copia");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (base, variant, copy)
}

/// ⭐⭐⭐ **O cartão sai da FAMÍLIA, e as fileiras são as propriedades declaradas.**
#[test]
fn the_card_comes_from_the_family_declarations() {
    let mut sim = SimWorld::new();
    let (_base, _variant, copy) = family(&mut sim);
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    let got: Vec<(&str, Vec<&str>)> = info
        .rows
        .iter()
        .map(|r| {
            (
                r.name.as_str(),
                r.options.iter().map(|o| o.label.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(got, vec![("Size", vec!["Small", "Big"])]);
    // ⚠️ E a raiz viaja — é dela que a troca de versão precisa.
    assert_ne!(info.root_bits, 0);
}

/// ⛔⛔⛔ **RENOMEAR SEJA O QUE FOR NÃO MEXE NO CARTÃO** — a ordem do Enio de 2026-09-01.
///
/// Até 31/08 este ficheiro tinha o gate CONTRÁRIO (`rewriting_the_braces_rewrites_the_card`), e a
/// lei que ele defendia custou seis reports com foto: pôr a declaração no `Name` faz de renomear
/// uma operação estrutural.
///
/// ⚠️ **A fixtura carrega o fenómeno**: os nomes novos têm chaves lá dentro, que é o que a lei
/// velha lia. *Com nomes limpos este gate ficaria verde sem provar nada.*
///
/// (Mutação: voltar a derivar a fileira do `Name` ⇒ RED.)
#[test]
fn renaming_anything_never_changes_the_card() {
    let mut sim = SimWorld::new();
    let (base, variant, copy) = family(&mut sim);
    let before = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("antes");
    for (e, n) in [
        (base, "Outra {Size=Zzz}"),
        (variant, "Mais outra {Size=Nada, State=Nada}"),
        (copy, "Bob {Size=Big}"),
    ] {
        sim.world_mut().entity_mut(e).insert(Name::new(n));
    }
    let after = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("depois");
    assert_eq!(before.rows, after.rows, "renomear mexeu numa fileira");
}

/// ⛔ **Um objecto que não é cópia de nada não pinta cartão** — uma propriedade é do COMPONENTE.
///
/// ⚠️ Até 31/08 o gate era o oposto: um objecto solto com chaves no nome tinha cartão. Aquele
/// desenho morreu com a gramática — sem família não há segunda versão, logo não há o que escolher.
#[test]
fn a_lone_object_paints_no_card() {
    let mut sim = SimWorld::new();
    let e = spawn_named(&mut sim, "Casa {Size=Small, State=Idle}");
    assert!(super::build_properties_info(&mut sim, Some(e.to_bits())).is_none());
}

/// ⛔ **Um nome comum e sem família também não pinta** — a lei da F3.
#[test]
fn a_plain_name_paints_no_card() {
    let mut sim = SimWorld::new();
    let e = spawn_named(&mut sim, "Casa");
    assert!(super::build_properties_info(&mut sim, Some(e.to_bits())).is_none());
}

/// ⭐⭐⭐ **A declaração é do MESTRE, não do exemplar** — renomear a cópia não lhe tira as
/// propriedades do componente dela.
///
/// **Mutação que deve sangrar:** ler a declaração da entidade selecionada.
#[test]
fn the_declaration_is_read_from_the_master_not_from_the_copy() {
    let mut sim = SimWorld::new();
    let (_base, _variant, copy) = family(&mut sim);
    sim.world_mut().entity_mut(copy).insert(Name::new("Bob"));
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    let got: Vec<&str> = info.rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        got,
        vec!["Size"],
        "uma copia renomeada perdeu as propriedades do componente dela"
    );
}

/// ⭐⭐⭐ **O título nomeia o objecto SELECIONADO, e VERBATIM** (Enio, 2026-08-31 + 2026-09-01).
///
/// ⚠️ Ele já cortou as chaves, porque elas eram mecanismo e faziam a frase estourar. Hoje não há o
/// que cortar — e comer pedaços de um nome que o artista escreveu seria o app a corrigi-lo.
///
/// (Mutação: `source_name` fixo em `None` ⇒ RED.)
#[test]
fn the_card_title_names_the_selected_object_verbatim() {
    let mut sim = SimWorld::new();
    let (_base, _variant, copy) = family(&mut sim);
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Canvas {Size=Big} (2)"));
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    assert_eq!(info.source_name.as_deref(), Some("Canvas {Size=Big} (2)"));
}
