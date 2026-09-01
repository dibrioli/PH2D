//! Os gates de *«as chaves no nome de uma cópia valem»* ([`super`]).
//!
//! ⚠️ **O oráculo é o que MUDA no mundo**, e nunca «a função devolveu `Some`»: as duas metades da
//! lei escrevem em sítios diferentes (uma troca o elo, a outra renomeia a receita), e um gate que
//! só olhasse o `Applied` passaria com as duas trocadas.

use super::Applied;
use ph2d_ecs::{ChildOf, Entity, InstanceOf, MasterRoot, Name, SimWorld, Transform};

pub(super) fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma base com uma peça, e a cópia que o *Make Prefab* deixa no lugar.
pub(super) fn base_and_copy(base_name: &str) -> (SimWorld, Entity, Entity) {
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

pub(super) fn name_of(sim: &SimWorld, e: Entity) -> String {
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

/// ⭐⭐⭐ **O ELO SEGUE AS CHAVES** — report do Enio com duas fotos, 2026-08-31:
/// *«O objeto deve ler o que está nas Chaves. não tem porque está Small e o Botão ficar Big»*.
///
/// Ele fotografou o estado em que o nome dizia `{Size=Small}` e o botão aceso dizia `Big`. Elas
/// eram fontes **independentes**: o elo mudava por clique, o nome por escrita, e nada as obrigava a
/// concordar.
///
/// ⚠️ **O `follow` corre a cada quadro**, então ele tem de ser IDEMPOTENTE: a segunda corrida sobre
/// o mesmo estado não pode fazer nada. *Sem essa metade, um gate que só chamasse uma vez deixaria
/// passar um laço que troca ida-e-volta para sempre.*
///
/// (Mutação: `follow` devolver `false` sem olhar ⇒ RED.)
#[test]
fn the_link_follows_the_braces_and_settles() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
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
    let big = sim
        .world()
        .get::<ph2d_ecs::StableId>(sibling)
        .expect("sid")
        .0;

    // O estado da FOTO: o nome diz Big, o elo aponta a base (Small).
    sim.world_mut()
        .entity_mut(copy)
        .insert(Name::new("Casa {Size=Big} (1)"));
    assert!(super::follow(&mut sim, &mut echo, copy), "nao seguiu");
    let root = crate::instance_verbs::instance_root_of(&mut sim, copy).expect("raiz");
    assert_eq!(
        sim.world().get::<InstanceOf>(root).map(|l| l.master),
        Some(big),
        "o elo nao seguiu as chaves"
    );
    // ⛔ E a 2.ª corrida NÃO faz nada — ele corre a cada quadro.
    assert!(
        !super::follow(&mut sim, &mut echo, copy),
        "o follow nao assentou: a cada quadro ele voltaria a trocar"
    );
}

/// ⛔⛔ **Um clique no chip REESCREVE as chaves da cópia** — a metade sem a qual isto seria uma
/// BRIGA.
///
/// Sem ela: o clique troca o elo, o `follow` vê o nome antigo no quadro seguinte e troca de volta.
/// Todo quadro. *Uma fonte única só é única se TODO gesto escrever nela.*
///
/// ⚠️ **O oráculo é o par**: as chaves passam a dizer o novo valor **e** o `follow` a seguir não
/// mexe. Só a primeira metade deixaria passar um espelho que escreve o valor errado.
///
/// (Mutação: `mirror_onto_copy` não fazer nada ⇒ RED na 2.ª asserção.)
#[test]
fn a_swap_rewrites_the_copys_braces_so_the_follow_settles() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
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
    let big = sim
        .world()
        .get::<ph2d_ecs::StableId>(sibling)
        .expect("sid")
        .0;

    // O gesto do chip: trocar pela porta, e espelhar.
    let root = crate::instance_verbs::instance_root_of(&mut sim, copy).expect("raiz");
    crate::instance_variant::swap(&mut sim, &mut echo, root, big).expect("trocou");
    super::mirror_onto_copy(&mut sim, root);
    assert_eq!(
        name_of(&sim, root),
        "Casa {Size=Big} (1)",
        "as chaves da copia nao acompanharam a troca"
    );
    assert!(
        !super::follow(&mut sim, &mut echo, root),
        "o follow trocou de volta — isto e' a briga que a metade do espelho existe para evitar"
    );
}

/// ⭐⭐ **Renomear o valor de uma receita arrasta as CÓPIAS dela** — e só as dela.
///
/// Sem isto, `Small` → `Grande` deixa a cópia com `{Size=Small}`: um rótulo a apontar para um valor
/// que já não existe, e o `follow` não o pode curar (não há a quem trocar). *A etiqueta mentiria
/// para sempre — que é o que as duas fotos mostraram.*
///
/// (Mutação: `mirror_onto_copies_of` não fazer nada ⇒ RED.)
#[test]
fn renaming_a_value_drags_the_copies_that_follow_it() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("sid").0;
    // Um estranho, com o MESMO texto no nome, que não segue esta receita.
    let stranger = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Outro {Size=Small}")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    sim.world_mut()
        .entity_mut(base)
        .insert(Name::new("Casa {Size=Grande}"));
    super::mirror_onto_copies_of(&mut sim, base_id);
    assert_eq!(name_of(&sim, copy), "Casa {Size=Grande} (1)");
    assert_eq!(
        name_of(&sim, stranger),
        "Outro {Size=Small}",
        "arrastou quem nao segue esta receita"
    );
}

/// ⭐⭐⭐ **Renomear a RECEITA pelo commit de nome ARRASTA as cópias** — o «duas portas, duas leis»
/// da auditoria (achado 1 do auditor 1, achado 2 do auditor 2).
///
/// O caminho do CHIP arrastava; o Enter da Hierarquia e o campo de nome não — e recriavam o estado
/// das duas fotos pela porta mais óbvia.
///
/// (Mutação: tirar o braço `MasterRoot` do `apply` ⇒ RED.)
#[test]
fn renaming_a_recipe_through_the_name_commit_drags_its_copies() {
    let (mut sim, base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    // O que o `hierarchy_rename` faz: escreve o Name e chama `apply` sobre a entidade renomeada.
    sim.world_mut()
        .entity_mut(base)
        .insert(Name::new("Casa {Size=Grande}"));
    let out = super::apply(&mut sim, &mut echo, base);
    assert!(
        out.is_none(),
        "sobre uma receita o apply arrasta e cala — nao troca nem autora"
    );
    assert_eq!(
        name_of(&sim, copy),
        "Casa {Size=Grande} (1)",
        "a copia ficou com a etiqueta morta — o estado das duas fotos, pela porta da Hierarquia"
    );
}

/// ⛔⛔ **O arrasto NÃO leva a RECEITA-VARIANTE** — ela não é uma cópia (sonda B do auditor 4).
///
/// Sem a cerca, renomear o valor da base fazia as DUAS receitas declararem a mesma combinação — o
/// estado que colapsa o eixo, criado pela própria cura.
///
/// (Mutação: tirar o `Without<MasterRoot>` do `mirror_onto_copies_of` ⇒ RED.)
#[test]
fn the_drag_never_touches_a_variant_recipe() {
    let (mut sim, base, _copy) = base_and_copy("Casa {Size=Small}");
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let variant = crate::instantiate::instantiate_master(
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
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let base_id = sim.world().get::<ph2d_ecs::StableId>(base).expect("sid").0;

    sim.world_mut()
        .entity_mut(base)
        .insert(Name::new("Casa {Size=Grande}"));
    super::mirror_onto_copies_of(&mut sim, base_id);
    assert_eq!(
        name_of(&sim, variant),
        "Casa {Size=Big}",
        "a variante foi arrastada como se fosse copia — duas receitas com a mesma combinacao"
    );
}

/// ⭐⭐⭐ **O *Make Variant* escreve nas chaves da cópia que deixa** — a 3.ª porta sem espelho
/// (sonda C do auditor 4: sem isto, o follow da seleção DESFAZIA o gesto no quadro seguinte).
///
/// (Mutação: tirar o `mirror_onto_copy` do `make_master` ⇒ RED na 2.ª asserção.)
#[test]
fn make_variant_writes_the_copys_braces_so_the_follow_settles() {
    let (mut sim, _base, copy) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    // O verbo REAL, sobre a cópia — como o menu faz.
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let (variant, left_behind) = crate::instance_verbs::make_master(
        &mut sim,
        &r,
        copy,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
    .expect("virou variante");
    let variant_id = sim
        .world()
        .get::<ph2d_ecs::StableId>(variant)
        .expect("sid")
        .0;
    // A cópia deixada DECLARA o que segue…
    let n = name_of(&sim, left_behind);
    assert!(
        n.contains("{Size=Small 2}"),
        "a copia nao declara o combo da variante que segue: «{n}»"
    );
    // …e o follow da mudança de seleção NÃO desfaz o gesto.
    assert!(
        !super::follow(&mut sim, &mut echo, left_behind),
        "o follow desfez o *Make Variant* no quadro seguinte"
    );
    assert_eq!(
        sim.world().get::<InstanceOf>(left_behind).map(|l| l.master),
        Some(variant_id)
    );
}

/// ⛔ **O espelho passa pelo funil de unicidade** — dois nomes nunca colidem (achado 4).
///
/// (Mutação: tirar o `unique_name_excluding` do `write_combo` ⇒ RED.)
#[test]
fn the_mirror_never_collides_two_names() {
    let (mut sim, base, copy_a) = base_and_copy("Casa {Size=Small}");
    let mut echo = crate::instance_sync::MasterEcho::default();
    let r = reg();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let variant = crate::instantiate::instantiate_master(
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
        .entity_mut(variant)
        .insert((MasterRoot, Name::new("Casa {Size=Big}")));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let big = sim
        .world()
        .get::<ph2d_ecs::StableId>(variant)
        .expect("sid")
        .0;
    // Uma cópia da VARIANTE com o mesmo sufixo « (1)» que a da base tem.
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let copy_b = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        variant,
        None,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    // O gesto do achado: a cópia da BASE troca para Big — o espelho vai escrever «Casa {Size=Big}…»
    let root = crate::instance_verbs::instance_root_of(&mut sim, copy_a).expect("raiz");
    crate::instance_variant::swap(&mut sim, &mut echo, root, big).expect("trocou");
    super::mirror_onto_copy(&mut sim, root);
    let a = name_of(&sim, root);
    let b = name_of(&sim, copy_b);
    assert_ne!(
        a, b,
        "duas entidades ficaram com o Name byte-identico: «{a}»"
    );
}
