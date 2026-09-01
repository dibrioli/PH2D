//! Os gates da porta de ESCRITA das variações ([`super`]).
//!
//! ⚠️ **O oráculo é o MUNDO** — o elo, as declarações e as excepções —, nunca o valor devolvido:
//! uma função que devolvesse `Ok` e não escrevesse nada passaria num gate que só olhasse o retorno.

use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, VariantValues};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma base com uma peça, e a cópia que o *Make Prefab* deixa no lugar.
fn base_and_copy() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
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
    let copy = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (sim, base, copy)
}

fn save(
    sim: &mut SimWorld,
    copy: Entity,
    property: &str,
    value: &str,
    existing: Option<&str>,
) -> Result<super::Saved, super::SaveRefusal> {
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::save_variation(
        sim,
        &r,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        copy.to_bits(),
        property,
        value,
        existing,
    )
}

fn values_of_entity(sim: &SimWorld, e: Entity) -> Vec<(String, String)> {
    sim.world()
        .get::<VariantValues>(e)
        .map(|v| v.values.clone().into_iter().collect())
        .unwrap_or_default()
}

/// ⭐⭐⭐ **Gravar dá uma VERSÃO nova que declara o valor, e a cópia passa a segui-la.**
///
/// É o gesto que o Enio descreveu a 2026-09-01: *«Ao criar e modificar uma instância surge no card
/// um botão do tipo "Salvar Variação"… com o momento de colocar o nome que vai gerar o botão
/// seletor da variação»*.
///
/// (Mutação: não escrever o `VariantValues` na receita nova ⇒ RED.)
#[test]
fn saving_a_variation_creates_a_version_that_declares_the_value() {
    let (mut sim, base, copy) = base_and_copy();
    let saved = save(&mut sim, copy, "Size", "Big", Some("Small")).expect("gravou");
    assert_eq!(saved.property, "Size");
    assert_eq!(saved.value, "Big");
    let recipe = crate::instance_verbs_walk::entity_for_stable_id(&mut sim, saved.recipe)
        .map(Entity::from_bits)
        .expect("a receita nova existe");
    assert_eq!(
        values_of_entity(&sim, recipe),
        vec![("Size".to_string(), "Big".to_string())]
    );
    // ⭐ E ela deriva da BASE — o elo de família continua a ser o `InstanceOf` de sempre.
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("id").0;
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::InstanceOf>(recipe)
            .map(|l| l.master),
        Some(base_id)
    );
}

/// ⭐⭐⭐ **A propriedade que NASCE dá nome ao que já existia, em TODA a família.**
///
/// Sem esta metade a fileira nova nasceria com um botão em branco — e uma fileira de um valor só
/// nem sequer é oferecida, então o artista veria o gesto não fazer nada.
///
/// (Mutação: saltar o laço que escreve as irmãs ⇒ RED.)
#[test]
fn a_new_property_names_what_already_existed() {
    let (mut sim, base, copy) = base_and_copy();
    save(&mut sim, copy, "Size", "Big", Some("Small")).expect("gravou");
    assert_eq!(
        values_of_entity(&sim, base),
        vec![("Size".to_string(), "Small".to_string())],
        "a base ficou sem valor na propriedade que acabou de nascer"
    );
}

/// ⭐⭐ **A modificação é ABSORVIDA: a cópia que fica no lugar não tem excepção nenhuma.**
///
/// ⚠️ O oráculo é a cópia NOVA (a que o verbo põe no lugar), e o número de excepções absorvidas é
/// o que a voz diz ao artista.
#[test]
fn the_overrides_are_absorbed_by_the_new_version() {
    let (mut sim, _base, copy) = base_and_copy();
    // Uma excepção de verdade na raiz da cópia.
    sim.world_mut()
        .entity_mut(copy)
        .insert(ph2d_ecs::ObjectInstance {
            overrides: [ph2d_ecs::OverrideKey {
                piece: 1,
                type_id: 2,
            }]
            .into_iter()
            .collect(),
            ..Default::default()
        });
    let saved = save(&mut sim, copy, "Size", "Big", Some("Small")).expect("gravou");
    assert_eq!(saved.absorbed, 1, "a voz tem de dizer quantas absorveu");
    let recipe = crate::instance_verbs_walk::entity_for_stable_id(&mut sim, saved.recipe)
        .map(Entity::from_bits)
        .expect("a receita");
    // A cópia que ficou no lugar é filha da receita nova e nasce SEM excepções.
    let placed = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &ph2d_ecs::InstanceOf), bevy_ecs::prelude::Without<MasterRoot>>();
        let rid = sim.world().get::<ph2d_ecs::StableId>(recipe).expect("id").0;
        q.iter(sim.world())
            .find(|(_, l)| l.master == rid)
            .map(|(e, _)| e)
            .expect("a copia nova segue a receita nova")
    };
    assert_eq!(
        sim.world()
            .get::<ph2d_ecs::ObjectInstance>(placed)
            .map_or(0, |o| o.overrides.len()),
        0
    );
}

/// ⛔⛔ **Duas versões nunca declaram a MESMA combinação** — é o estado em que a fileira colapsa
/// para um valor só e o cartão desce ao modo plano.
///
/// (Mutação: tirar a pergunta `sibling_declaring` da porta ⇒ RED.)
#[test]
fn two_versions_never_declare_the_same_combination() {
    let (mut sim, _base, copy) = base_and_copy();
    save(&mut sim, copy, "Size", "Big", Some("Small")).expect("a primeira");
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let r = reg();
    let the_base = {
        let mut q = sim.world_mut().query_filtered::<Entity, (
            bevy_ecs::prelude::With<MasterRoot>,
            bevy_ecs::prelude::Without<ph2d_ecs::InstanceOf>,
        )>();
        q.iter(sim.world()).next().expect("a base")
    };
    let another = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        the_base,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert_eq!(
        save(&mut sim, another, "Size", "Big", None),
        Err(super::SaveRefusal::Duplicate)
    );
}

/// ⛔ **Vazio recusa em voz alta** — um campo que come o texto em silêncio é o defeito que este
/// gesto existe para curar.
#[test]
fn an_empty_name_is_refused_out_loud() {
    let (mut sim, _base, copy) = base_and_copy();
    assert_eq!(
        save(&mut sim, copy, "Size", "   ", None),
        Err(super::SaveRefusal::Empty)
    );
    assert_eq!(
        save(&mut sim, copy, "", "Big", None),
        Err(super::SaveRefusal::Empty)
    );
}

/// ⛔ **Sem cópia não há o que gravar** — e o caminho fala.
#[test]
fn saving_over_something_that_is_no_copy_is_refused() {
    let (mut sim, base, _copy) = base_and_copy();
    assert_eq!(
        save(&mut sim, base, "Size", "Big", None),
        Err(super::SaveRefusal::NotAnInstance)
    );
}

/// ⭐⭐ **Renomear um valor reescreve a DECLARAÇÃO, e não o nome.**
#[test]
fn renaming_a_value_rewrites_the_declaration() {
    let (mut sim, base, copy) = base_and_copy();
    let saved = save(&mut sim, copy, "Size", "Big", Some("Small")).expect("gravou");
    let name_before = sim.world().get::<Name>(base).expect("nome").0.clone();
    assert_eq!(
        super::rename_value(&mut sim, saved.recipe, "Size", "Grande"),
        super::Renamed::Written
    );
    let recipe = crate::instance_verbs_walk::entity_for_stable_id(&mut sim, saved.recipe)
        .map(Entity::from_bits)
        .expect("a receita");
    assert_eq!(
        values_of_entity(&sim, recipe),
        vec![("Size".to_string(), "Grande".to_string())]
    );
    // ⛔ E o `Name` de ninguém se mexeu — a declaração saiu do nome de vez.
    assert_eq!(sim.world().get::<Name>(base).expect("nome").0, name_before);
}

/// ⭐⭐⭐ **Escrever um valor que já existe manda TROCAR, não escrever por cima.**
///
/// Escrever por cima criaria duas receitas a declarar o mesmo — o colapso que a wave existe para
/// curar. ⚠️ A escolha vive na LEI, e não no dreno: no dreno ela seria inalcançável de um teste.
///
/// (Mutação: devolver `Written` sempre ⇒ RED.)
#[test]
fn writing_a_value_that_already_exists_asks_for_a_switch() {
    let (mut sim, base, copy) = base_and_copy();
    let saved = save(&mut sim, copy, "Size", "Big", Some("Small")).expect("gravou");
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("id").0;
    assert_eq!(
        super::rename_value(&mut sim, saved.recipe, "Size", "Small"),
        super::Renamed::Switch(base_id)
    );
}
