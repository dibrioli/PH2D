//! Os gates do cartão de PROPRIEDADES ([`super`]).
//!
//! ⚠️ **O oráculo é o que o artista LÊ.** Os gates puros da lei (`variant_axes_tests`) medem a
//! derivação a partir de dados; o que só se mede aqui é **de onde esses dados são lidos**.

use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform};

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
        .spawn((Transform::IDENTITY, Name::new("Casa"), MasterRoot))
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
    sim.world_mut()
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa Variant")));
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

/// ⭐⭐⭐ **O cartão sai da FAMÍLIA** — uma fileira com o nome de cada versão.
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
    assert_eq!(got, vec![("", vec!["Casa", "Casa Variant"])]);
    // ⚠️ E a raiz viaja — é dela que a troca de versão precisa.
    assert_ne!(info.root_bits, 0);
}

/// ⛔⛔⛔ **RENOMEAR MUDA O RÓTULO E MAIS NADA** — nem a versão vigente, nem a família, nem a
/// identidade de ninguém.
///
/// # ⚠️ Este gate VIROU com a decisão, e a 1.ª redacção dele era FALSA
///
/// Ele nasceu a afirmar *«renomear não mexe no cartão»*, e isso era verdade enquanto o chip
/// mostrava o **valor de uma propriedade**. Com o mecanismo de propriedades adiado (Enio,
/// 2026-09-01), o chip mostra o **nome da versão** — logo renomear **muda o rótulo, e tem de
/// mudar**: é a natureza de um rótulo seguir o que ele rotula.
///
/// ⛔ **A afirmação que fica é a que importa e continua verdadeira:** o nome não decide nada. A
/// estrutura — quantas versões, quais são, qual está acesa — atravessa a renomeação intacta.
/// *Um gate que sobrevive a uma decisão revertida transforma-a em regressão silenciosa; quando a
/// decisão vira, o gate vira com ela e diz que virou.*
///
/// ⚠️ **A fixtura carrega o fenómeno**: os nomes novos têm chaves lá dentro, que é o que a lei
/// revogada lia. Com nomes limpos ela provaria nada.
#[test]
fn renaming_changes_the_label_and_nothing_else() {
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

    let ids_and_current = |i: &ph2d_editor::screens::hero::InspectorPropertiesInfo| {
        i.rows
            .iter()
            .map(|r| {
                r.options
                    .iter()
                    .map(|o| (o.master, o.current))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ids_and_current(&before),
        ids_and_current(&after),
        "renomear mexeu na ESTRUTURA — identidade ou versao vigente"
    );
    // ⭐ E o rótulo seguiu, verbatim — sem cortar as chaves, que já foram a doença.
    assert_eq!(after.rows[0].options[0].label, "Outra {Size=Zzz}");
}

/// ⛔ **Um objecto que não é cópia de nada não pinta cartão** — não há família, não há escolha.
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

/// ⭐⭐⭐ **A família é do MESTRE, não do exemplar** — renomear a cópia não a tira da família.
///
/// **Mutação que deve sangrar:** derivar a família da entidade selecionada.
#[test]
fn the_declaration_is_read_from_the_master_not_from_the_copy() {
    let mut sim = SimWorld::new();
    let (_base, _variant, copy) = family(&mut sim);
    sim.world_mut().entity_mut(copy).insert(Name::new("Bob"));
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    assert_eq!(
        info.rows.len(),
        1,
        "a copia renomeada perdeu a familia dela"
    );
    assert_eq!(info.rows[0].options.len(), 2);
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
