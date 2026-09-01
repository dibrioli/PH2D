//! Os gates de *«as chaves no nome de uma cópia valem»* ([`super`]).
//!
//! ⚠️ **O oráculo é o que MUDA no mundo**, e nunca «a função devolveu `Some`»: as duas metades da
//! lei escrevem em sítios diferentes (uma troca o elo, a outra renomeia a receita), e um gate que
//! só olhasse o `Applied` passaria com as duas trocadas.

use super::Applied;
use ph2d_ecs::{ChildOf, Entity, InstanceOf, MasterRoot, Name, SimWorld, Transform};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma base com uma peça, e a cópia que o *Make Prefab* deixa no lugar.
fn base_and_copy(base_name: &str) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let r = reg();
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new(base_name), MasterRoot))
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
    (sim, master, copy)
}

fn name_of(sim: &SimWorld, e: Entity) -> String {
    sim.world().get::<Name>(e).expect("nome").0.clone()
}

/// ⭐⭐⭐ **Um valor NOVO no nome da cópia autora-o na RECEITA** — decisão do Enio, 2026-08-31:
/// *«por que não funciona mudando o nome entre as chaves? Tem que funcionar!»*.
///
/// ⚠️ **E o nome da CÓPIA fica como ele o escreveu** — ela é a etiqueta dele; reescrevê-la seria o
/// app a corrigir o que o artista acabou de digitar.
///
/// (Mutação: `apply` devolver `None` antes do braço da autoria ⇒ RED.)
#[test]
fn a_new_value_typed_on_a_copy_is_authored_on_the_recipe() {
    let (mut sim, master, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Casa {Size=Big} (1)"));
    let out = super::apply(&mut sim, &mut echo, copy);
    assert!(
        matches!(&out, Some(Applied::Authored { key, value }) if key == "Size" && value == "Big"),
        "nao autorou: {:?}",
        out.is_some()
    );
    assert_eq!(name_of(&sim, master), "Casa {Size=Big}");
    assert_eq!(
        name_of(&sim, copy),
        "Casa {Size=Big} (1)",
        "o app reescreveu o nome que o artista digitou"
    );
}

/// ⭐⭐ **Um valor que a família JÁ TEM faz a cópia TROCAR de versão** — e não uma segunda receita
/// a dizer o mesmo, que é o estado que colapsa o eixo.
///
/// ⚠️ **O oráculo é o ELO** (`InstanceOf::master`), não o `Applied`.
///
/// (Mutação: trocar a ordem — autorar antes de procurar — ⇒ RED.)
#[test]
fn a_value_the_family_already_has_switches_instead_of_authoring() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    // A variante, feita como o produto a faz: promover uma cópia a receita.
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let r = reg();
    let sibling = crate::instantiate::instantiate_master(
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
    sim.world_mut()
        .entity_mut(sibling)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let sibling_id = sim
        .world()
        .get::<ph2d_ecs::StableId>(sibling)
        .expect("sid")
        .0;

    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Casa {Size=Big} (1)"));
    let out = super::apply(&mut sim, &mut echo, copy);
    assert!(
        matches!(out, Some(Applied::Switched)),
        "nao trocou para a variante que ja' declarava isto"
    );
    // ⛔ E a BASE ficou intacta — autorar por cima criaria duas receitas a dizer `Big`.
    assert_eq!(name_of(&sim, base), "Casa {Size=Small}");
    let root = crate::instance_verbs::instance_root_of(&mut sim, copy).expect("raiz");
    assert_eq!(
        sim.world().get::<InstanceOf>(root).map(|l| l.master),
        Some(sibling_id),
        "o elo da copia nao aponta a variante"
    );
}

/// ⛔ **Nada acontece quando não há nada a fazer** — e são os casos comuns.
///
/// ⚠️ Sem esta metade, um `apply` que agisse sempre renomearia a receita a cada commit de nome de
/// qualquer objecto da cena.
#[test]
fn nothing_happens_when_there_is_nothing_to_do() {
    let mut echo = crate::instance_sync::MasterEcho::default();
    // (a) o nome declara o MESMO que a receita.
    let (mut sim, _m, copy) = base_and_copy("Casa {Size=Small}");
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Casa {Size=Small} (1)"));
    assert!(super::apply(&mut sim, &mut echo, copy).is_none());
    // (b) o nome não declara nada.
    sim.world_mut().entity_mut(copy).insert(Name::new("Bob"));
    assert!(super::apply(&mut sim, &mut echo, copy).is_none());
    // (c) declara uma chave que o componente NÃO tem — mudar a forma da família é outro gesto.
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Casa {Tag=City}"));
    assert!(super::apply(&mut sim, &mut echo, copy).is_none());
    // (d) o objecto não é cópia de nada — ali o nome dele JÁ é a declaração.
    let lone = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Muro {Size=Big}")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(super::apply(&mut sim, &mut echo, lone).is_none());
}
