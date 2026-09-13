//! ⭐⭐⭐ **A ÁRVORE DE TAGS no undo** — gate 18 do plano de Tags
//! (`docs/Components/08_plano_tags.md` §5.2, W2).
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] responde pela captura do MUNDO e da geometria e o `undo_library_tests` pela
//! biblioteca; este pela **árvore de tags**, que é o terceiro documento que a captura carrega. O
//! corte é por assunto, e o teto de LOC do shell exige-o — ⛔ *split, nunca a marca de isenção*.
//!
//! ⚠️ **A régua é a árvore INTEIRA mais a PERTENÇA**: um undo que devolvesse as tags e deixasse os
//! objectos sem elas passaria numa contagem de tags, e o artista veria as tags de volta e vazias.

use super::*;
use ph2d_app_components::tags_doc::TagsCache;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::tags::Tags;
use ph2d_tags::{TagId, TagTree};

/// Captura com uma árvore de tags escolhida — o irmão do `capture_with_library`.
fn capture_with_tags(
    sim: &mut SimWorld,
    vec: &VecScene,
    reg: &ComponentRegistry,
    tags: &[u8],
) -> ProjectState {
    ProjectState::capture(
        &ph2d_preview_drive::PreviewDrive::default(),
        sim,
        vec,
        &FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        tags,
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// A entidade com este nome — depois de um restauro os bits são novos, e o nome é o que sobrevive.
fn por_nome(sim: &SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let mut q = sim
        .world()
        .try_query::<(ph2d_ecs::Entity, &ph2d_ecs::Name)>();
    let mut q = q.take().expect("o mundo ja' viu um Name");
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("nao ha' objecto chamado {nome:?}"))
}

/// ⭐⭐⭐ **Apagar uma tag é UM passo, com a pertença lá dentro** — e o `Ctrl+Z` devolve as duas
/// metades.
///
/// **Mutações que devem sangrar:** apagar o campo `tags` do `ProjectState::capture` (a árvore não
/// volta) · o restauro sem o `scrub`/`remap` no mesmo gesto (a pertença não volta com ela).
#[test]
fn the_tag_tree_travels_in_the_project_and_through_undo() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let mut arvore = TagTree::new();
    let boss = arvore.create("Enemy/Flying/Boss").expect("cria");
    let enemy = arvore.find("Enemy").expect("ancestral");
    sim.world_mut().spawn((
        ph2d_ecs::Transform::default(),
        ph2d_ecs::Name::new("Dragon"),
        ph2d_ecs::StableId(1),
        Tags::from_ids([boss]),
    ));
    let mut cache = TagsCache::default();

    let antes = capture_with_tags(&mut sim, &vec, &reg, cache.doc(&arvore));

    // O gesto: apagar `Enemy` — a subárvore sai da árvore e a pertença sai dos objectos, JUNTAS.
    let saem = arvore.delete(enemy);
    assert_eq!(saem.len(), 3, "a subarvore inteira");
    assert_eq!(
        ph2d_ecs::tags::scrub(sim.world_mut(), &saem),
        1,
        "o Dragon nao perdeu a tag"
    );
    let depois = capture_with_tags(&mut sim, &vec, &reg, cache.doc(&arvore));
    assert_eq!(
        antes.parts_that_differ(&depois),
        vec!["world", "tags"],
        "o gesto tem de mexer nas DUAS metades — e em mais nenhuma"
    );

    // Ctrl+Z: a árvore volta…
    let (de_volta, remap) = ph2d_app_components::tags_doc::restore(&antes.tags);
    assert!(remap.is_empty(), "os nossos proprios bytes nao tem gemeos");
    assert_eq!(de_volta.len(), 3);
    assert_eq!(de_volta.find("Enemy/Flying/Boss"), Some(boss));
    // …e a PERTENÇA volta com ela.
    let _ = antes.restore(&mut sim, &reg);
    let dragao = por_nome(&sim, "Dragon");
    assert_eq!(
        sim.world().get::<Tags>(dragao),
        Some(&Tags::from_ids([boss])),
        "o undo devolveu a arvore e deixou o objecto sem tag"
    );
}

/// ⚠️ **Duas capturas do mesmo estado não diferem em nada** — senão a árvore de tags registaria um
/// passo espúrio por quadro, que é o defeito que o `canonicalize` existia para curar.
#[test]
fn an_untouched_tag_tree_does_not_register_a_step() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let mut arvore = TagTree::new();
    arvore.create("Enemy").expect("cria");
    let mut cache = TagsCache::default();
    let a = capture_with_tags(&mut sim, &vec, &reg, cache.doc(&arvore));
    let b = capture_with_tags(&mut sim, &vec, &reg, cache.doc(&arvore));
    assert!(
        a.parts_that_differ(&b).is_empty(),
        "o mesmo estado deu bytes diferentes"
    );
    // E a árvore que sai dos bytes re-codifica igual — é isso que faz um restauro ser um ponto fixo.
    let (de_volta, _) = ph2d_app_components::tags_doc::restore(&a.tags);
    assert_eq!(
        ph2d_app_components::tags_doc::collect(&de_volta),
        a.tags,
        "re-codificar a arvore restaurada deu outros bytes"
    );
    let _ = TagId(0);
}
