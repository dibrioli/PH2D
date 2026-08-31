//! ⭐⭐ **O que a BIBLIOTECA deve ao arquivo** — cortado do [`super::tests`] pelo tecto de 600 LOC
//! (HR-18), e o corte é por responsabilidade: aqui vive *a taxonomia e as lápides a atravessarem o
//! `.ph2dproj`*.

use crate::undo::ProjectState;
use ph2d_ecs::scene::WorldSnapshot;
use ph2d_vec_scene::VecScene;

/// ⭐⭐⭐ **A BIBLIOTECA atravessa o ARQUIVO** — a taxonomia e as lápides.
///
/// ⛔⛔ Achado da auditoria de 2026-08-30: os gates do undo chamavam `ProjectState::capture` e
/// `project_library::apply` **directamente**, e o único round-trip pelo ficheiro
/// (`project_file_round_trips_through_postcard`) punha uma `LibraryDoc::default()` — o valor
/// **vazio**, três linhas abaixo de um comentário que avisa que é assim *«que uma mutacao
/// sobreviveu a 10.503 testes»*. ⇒ este é o irmão do `the_ui_states_travel_in_the_file`, com o
/// documento POPULADO, que é a única forma de o arquivo poder perder alguma coisa.
///
/// ⚠️ **E as duas metades são afirmadas uma a uma**: um `assert_eq` do `LibraryDoc` inteiro
/// compara o que voltou com o que foi, e um campo que NÃO viaja volta no default **dos dois
/// lados** — a igualdade fica verde sobre um arquivo que perdeu o dado.
#[test]
fn the_library_travels_in_the_file() {
    use ph2d_asset_index::{AssetRef, CatalogTree};

    let mut tree = CatalogTree::new();
    let heroes = tree.create("Personagens/Herois");
    tree.assign(AssetRef::Component { stable_id: 7 }, heroes);
    tree.assign(AssetRef::Texture { asset: [4; 32] }, heroes);
    // Um catálogo apagado, para o `next_id` ficar ADIANTE do maior id vivo.
    let doomed = tree.create("Z");
    tree.delete(doomed);

    let library = crate::project_library::LibraryDoc {
        catalog_bytes: crate::project_catalogs::collect(&tree),
        forgotten: vec![[9; 32]],
    };
    let state = ProjectState {
        world: WorldSnapshot::new(),
        vec: VecScene::new(),
        flip: ph2d_flip::FlipDoc::new(),
        guides: ph2d_guides::GuideSet::default(),
        ui_states: ph2d_ui_state::StateSets::default(),
        library,
    };
    let bytes = postcard::to_allocvec(&state).unwrap();
    let back: ProjectState = postcard::from_bytes(&bytes).unwrap();

    let back_tree = crate::project_catalogs::restore(&back.library.catalog_bytes);
    assert_eq!(back_tree, tree, "a taxonomia nao atravessou o arquivo");
    assert_eq!(
        back_tree.catalog_of(&AssetRef::Texture { asset: [4; 32] }),
        Some(heroes),
        "os catalogos voltaram VAZIOS — as atribuicoes nao atravessaram"
    );
    assert_eq!(
        back_tree.next_id(),
        tree.next_id(),
        "o proximo id foi RECICLADO ao atravessar o arquivo"
    );
    assert_eq!(
        back.library.forgotten,
        vec![[9u8; 32]],
        "as lapides nao atravessaram — reabrir o projecto traria de volta o que o artista tirou"
    );
}
