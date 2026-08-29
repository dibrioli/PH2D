//! Os gates da AUTORIA de uma variante (ADR-0164 / F5, critério 2) — o gesto que a faz nascer.
//!
//! ⚠️ **Irmão por ASSUNTO do [`super::tests`]**, e não um braço dele: os gates aqui são sobre
//! `instance_verbs::make_master`, mas o que eles medem é a **variante** — e o
//! `instance_verbs_tests.rs` bateu no tecto de 600 LOC ao recebê-los. A lei da casa é partir por
//! assunto, nunca subir a tolerância; o precedente no mesmo módulo é o `instance_place_tests.rs`
//! (*«onde a cópia ATERRA é outro assunto»*).

use ph2d_ecs::SimWorld;

use crate::instance_smoke::spawn_master;
use crate::instance_verbs::VerbRefusal;

/// ⭐⭐⭐ **A RAIZ de uma cópia PODE virar receita — e isso é uma VARIANTE** (F5, critério 2).
///
/// Ela fica `MasterRoot` **e** `InstanceOf` ao mesmo tempo: receita das cópias dela, instância da
/// base. É a *Prefab Variant* do Unity e o `IsA` do flecs, e o sync alcança-a sem uma linha nova.
///
/// ⚠️ **A recusa `InsideAnInstance` continua a valer para o MEIO** — o gate abaixo prova as duas
/// metades, porque a cura foi estreitar uma condição e não apagá-la.
///
/// (Mutação: voltar a recusar toda entidade dentro de instância ⇒ RED aqui.)
#[test]
fn the_root_of_a_copy_becomes_a_variant_and_a_piece_still_cannot() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = spawn_master(&mut sim);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let copy = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");

    // A PEÇA no meio da cópia: continua recusada, e pela razão de sempre.
    let piece = *sim
        .world()
        .get::<ph2d_ecs::Children>(copy)
        .expect("a copia tem pecas")
        .first()
        .expect("uma peca");
    assert_eq!(
        crate::instance_verbs::make_master(&mut sim, &r, piece, &mut docs).err(),
        Some(VerbRefusal::InsideAnInstance),
        "uma peca no MEIO de uma copia virou receita — isso encurta a sub-arvore de edicao"
    );

    // A RAIZ dela: vira variante.
    let made = crate::instance_verbs::make_master(&mut sim, &r, copy, &mut docs);
    assert!(
        made.is_ok(),
        "a raiz de uma copia nao virou variante: {made:?}"
    );
    assert!(
        sim.world().get::<ph2d_ecs::MasterRoot>(copy).is_some()
            && sim.world().get::<ph2d_ecs::InstanceOf>(copy).is_some(),
        "a variante tem de ser receita E instancia ao mesmo tempo — sem o elo ela deixa de seguir \
         a base, e passa a ser so' um segundo componente"
    );
}

/// ⭐⭐ **A cópia de uma VARIANTE nasce sem excepções** (F5).
///
/// A cópia profunda leva o `ObjectInstance` verbatim, e numa variante ele existe — chaveado pelas
/// peças da BASE. Numa cópia da variante essas chaves não alcançam nada: seriam lixo com cara de
/// excepção, e é o cartão do Inspector que o lê.
#[test]
fn a_copy_of_a_variant_is_born_with_no_overrides() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let master = spawn_master(&mut sim);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let variant = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    sim.world_mut()
        .entity_mut(variant)
        .insert(ph2d_ecs::MasterRoot);
    // A variante tem uma excepção sua, chaveada por uma peça da BASE.
    sim.world_mut()
        .entity_mut(variant)
        .insert(ph2d_ecs::ObjectInstance {
            overrides: [ph2d_ecs::OverrideKey {
                piece: 7,
                type_id: 9,
            }]
            .into_iter()
            .collect(),
            orphans: Default::default(),
        });
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let copy = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        variant,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou a variante");
    assert!(
        sim.world().get::<ph2d_ecs::ObjectInstance>(copy).is_none(),
        "a copia herdou as excepcoes da variante — elas chaveiam pecas da BASE e nao alcancam nada"
    );
}

/// ⭐⭐⭐ **A BIBLIOTECA ganha o nome novo; o que fica na TELA mantém o do artista** — report do
/// Enio, 2026-08-27: *«a mensagem [diz] Instance of "ele mesmo"»*.
///
/// O gesto promove o objeto escolhido a receita e põe uma cópia no lugar. Sobre uma **instância**
/// isso lia-se como auto-referência: o promovido chamava-se `Badge (1)`, a cópia nova ficava
/// `Badge (2)`, e o cartão dela dizia *«Instance of "Badge (1)"»* — o nome que estava na linha que
/// o artista acabara de clicar. *A identidade mudou de dono e o nome não.*
///
/// ⚠️ **As duas metades**, e a ordem entre elas: a variante é renomeada primeiro (o que **liberta**
/// o nome original) e só então a cópia o reclama. Sem a 1.ª metade a 2.ª devolve `Badge (1) (1)`.
///
/// (Mutação: não renomear a variante ⇒ a cópia não recupera o nome ⇒ RED.)
#[test]
fn making_a_variant_names_the_library_and_leaves_the_canvas_name_alone() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let g = spawn_master(&mut sim);
    // ⚠️ O `spawn_master` já devolve uma receita; a variante nasce da CÓPIA dela.
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let copy = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        g,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
    )
    .expect("instanciou");
    let on_canvas = name(&sim, copy);

    let (variant, replacement) =
        crate::instance_verbs::make_master(&mut sim, &r, copy, &mut docs).expect("variante");

    assert_eq!(
        name(&sim, variant),
        format!("{} Variant", name(&sim, g)),
        "a receita nova nao diz que e' uma variante — e' o idioma do Unity, e e' o que impede a \
         linha da biblioteca de se confundir com o objeto de tela"
    );
    assert_eq!(
        name(&sim, replacement),
        on_canvas,
        "o objeto que ficou na tela foi RENOMEADO. O artista clicou numa linha chamada {on_canvas:?} \
         e ela mudou de nome debaixo dele — e o cartao passa a apontar para o nome que ela tinha, \
         que e' exactamente o report da auto-referencia"
    );
}

/// ⚠️ **Um *Make Component* comum não inventa a palavra `Variant`** — o gesto sobre um objeto solto
/// não tem base nenhuma que o nome pudesse nomear.
#[test]
fn a_plain_make_component_keeps_the_old_naming() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let g = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::Name::new("Badge")))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    let (master, inst) =
        crate::instance_verbs::make_master(&mut sim, &r, g, &mut docs).expect("receita");
    assert_eq!(name(&sim, master), "Badge");
    assert!(
        !name(&sim, inst).contains("Variant"),
        "um objeto solto virou «Variant» de nada: {:?}",
        name(&sim, inst)
    );
}

fn name(sim: &SimWorld, e: ph2d_ecs::Entity) -> String {
    sim.world()
        .get::<ph2d_ecs::Name>(e)
        .map_or_else(|| "?".to_string(), |n| n.0.clone())
}
