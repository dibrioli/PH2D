//! Os gates do cartão de PROPRIEDADES ([`super`]).
//!
//! ⚠️ **O oráculo é o que o artista LÊ.** Os gates puros da lei (`variant_axes_tests`) medem a
//! derivação a partir de nomes; o que só se mede aqui é **de que nome ela parte**.

use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform};

fn spawn_named(sim: &mut SimWorld, name: &str) -> Entity {
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(name)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    e
}

/// ⭐⭐⭐ **UM OBJECTO SOLTO TEM CARTÃO** — o report do Enio de 2026-08-31.
///
/// *«quando mudo o conteúdo entre `{}` o inspector não muda»*: o construtor da seção COMPONENT sai
/// por `?` no `InstanceOf`, então sobre um objecto que não é cópia de nada **não havia superfície
/// nenhuma** a ler as chaves. O selo `*²` da Hierarquia prometia que alguém as lia.
///
/// **Mutação que deve sangrar:** exigir `InstanceOf` no `build_properties_info`.
#[test]
fn an_object_that_is_no_copy_still_shows_its_properties() {
    let mut sim = SimWorld::new();
    let e = spawn_named(&mut sim, "Casa {Size=Small, State=Idle}");
    let info = super::build_properties_info(&mut sim, Some(e.to_bits())).expect("o cartao");
    let got: Vec<(&str, &str)> = info
        .rows
        .iter()
        .map(|r| (r.name.as_str(), r.options[0].label.as_str()))
        .collect();
    assert_eq!(got, vec![("Size", "Small"), ("State", "Idle")]);
    // ⛔ Sem cópia não há raiz — e é isso que impede um `root_bits` inventado de viajar no
    // barramento se alguém alcançar o braço da troca.
    assert_eq!(info.root_bits, 0);
}

/// ⛔ **Reescrever as chaves REESCREVE o cartão** — é literalmente o report.
///
/// ⚠️ **Duas leituras do MESMO construtor**, e não uma comparação com uma constante: um cartão
/// congelado num cache passaria num gate que só olhasse a 1.ª leitura.
#[test]
fn rewriting_the_braces_rewrites_the_card() {
    let mut sim = SimWorld::new();
    let e = spawn_named(&mut sim, "Casa {Size=Small}");
    let before = super::build_properties_info(&mut sim, Some(e.to_bits())).expect("antes");
    sim.world_mut()
        .entity_mut(e)
        .insert(Name::new("Casa {Size=Big, State=Run}"));
    let after = super::build_properties_info(&mut sim, Some(e.to_bits())).expect("depois");
    assert_eq!(before.rows.len(), 1);
    assert_eq!(after.rows.len(), 2);
    assert_eq!(after.rows[0].options[0].label, "Big");
}

/// ⛔ **Um nome sem chaves e sem família não pinta cartão** — a lei da F3.
#[test]
fn a_plain_name_paints_no_card() {
    let mut sim = SimWorld::new();
    let e = spawn_named(&mut sim, "Casa");
    assert!(super::build_properties_info(&mut sim, Some(e.to_bits())).is_none());
}

/// ⭐⭐⭐ **A DECLARAÇÃO é do MESTRE, não do exemplar.**
///
/// Uma propriedade é do componente. Se o artista renomear a cópia para `Bob`, ela continua a ser a
/// `Casa {Size=Small}` — e ler o nome próprio faria as propriedades **desaparecerem** ao renomear.
///
/// **Mutação que deve sangrar:** ler sempre o `Name` da entidade selecionada.
#[test]
fn the_declaration_is_read_from_the_master_not_from_the_copy() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Casa {Size=Small, State=Idle}"),
            MasterRoot,
        ))
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
    let copy = crate::instantiate::instantiate_master(
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
    sim.world_mut().entity_mut(copy).insert(Name::new("Bob"));
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    let got: Vec<&str> = info.rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(
        got,
        vec!["Size", "State"],
        "uma cópia renomeada perdeu as propriedades do componente dela"
    );
    // ⚠️ E a raiz viaja — é dela que a troca de variante precisa quando houver para onde ir.
    assert_ne!(info.root_bits, 0);
}

/// ⭐⭐⭐ **O cartão diz DE QUEM são as propriedades quando não são do próprio objecto.**
///
/// Report do Enio com foto (2026-08-31): ele renomeou uma **cópia** para `Canvas{Size=Big}` e o
/// cartão continuou a dizer `Size  Small`. O cartão estava certo — a propriedade é do componente —
/// mas na tela liam-se um nome a dizer `Big` e uma linha a dizer `Small`, sem nada entre os dois.
///
/// ⚠️ **E num objecto SOLTO ele fica calado**: nomear a fonte ali seria dizer-lhe o nome dele
/// próprio.
///
/// (Mutação: `source_name` fixo em `None` ⇒ RED.)
#[test]
fn the_card_names_whose_properties_these_are() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Canvas {Size=Small}"),
            MasterRoot,
        ))
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
    let copy = crate::instantiate::instantiate_master(
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
    // O gesto do report: renomear a CÓPIA.
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Canvas {Size=Big}"));
    let info = super::build_properties_info(&mut sim, Some(copy.to_bits())).expect("o cartao");
    assert_eq!(
        info.source_name.as_deref(),
        Some("Canvas"),
        "o cartao nao diz de quem sao as propriedades — e o nome da copia contradi-lo na tela"
    );
    // ⛔ E a fonte é o nome CURTO, nunca o cru com chaves.
    assert!(!info.source_name.unwrap().contains('{'));

    // ⛔ Num objecto solto, calado.
    let lone = spawn_named(&mut sim, "Muro {Size=Small}");
    let lone_info = super::build_properties_info(&mut sim, Some(lone.to_bits())).expect("cartao");
    assert!(lone_info.source_name.is_none());
}
