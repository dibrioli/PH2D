//! ⭐ **A BIBLIOTECA DE IMAGENS no undo** (pedido do Enio, 2026-08-30) — apagar uma gaveta, tirar
//! uma imagem, e a árvore de catálogos a voltar inteira.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::undo_tests`] responde pela captura do MUNDO e da geometria; este pela **biblioteca**,
//! que é um documento à parte que a captura carrega. O irmão passou as `600` linhas do gate de LOC
//! do shell — ⛔ *split, nunca a marca de isenção* — e o corte é por assunto.
//!
//! ⚠️ **O gate que o apanhou vive em `shells/desktop/tests/`**, e o `cargo test --bins` não lhe
//! toca: os `604` e `602` LOC entraram em três waves sem que nenhuma corrida os visse.

use super::*;
use ph2d_ecs::scene::ComponentRegistry;

// ── ⭐⭐⭐ A BIBLIOTECA DESFAZ (Enio, 2026-08-30) ────────────────────────────────────────────────

/// Captura com uma biblioteca escolhida — o irmão do [`capture`] acima.
fn capture_with_library(
    sim: &mut SimWorld,
    vec: &VecScene,
    reg: &ComponentRegistry,
    library: &crate::project_library::LibraryDoc,
) -> ProjectState {
    ProjectState::capture(
        &crate::preview_drive::PreviewDrive::default(),
        sim,
        vec,
        &FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        library,
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// ⭐⭐⭐ **APAGAR UMA GAVETA DESFAZ-SE** — o pedido do Enio, na régua que ele usaria.
///
/// ⚠️ **A régua é a ÁRVORE inteira, não uma contagem**: um undo que devolvesse os catálogos e
/// perdesse as atribuições passaria numa contagem, e o artista veria as gavetas de volta e vazias.
///
/// **Mutação que deve sangrar:** apagar o campo `library` do `ProjectState::capture`.
#[test]
fn deleting_a_catalog_is_undone_with_its_contents() {
    use ph2d_asset_index::{AssetRef, CatalogTree};

    let reg = registry();
    let (mut sim, vec) = scene();
    let mut tree = CatalogTree::new();
    let heroes = tree.create("Personagens/Herois");
    tree.assign(AssetRef::Component { stable_id: 7 }, heroes);
    tree.assign(AssetRef::Texture { asset: [4; 32] }, heroes);

    let mut cache = crate::project_library::LibraryCache::default();
    let before = capture_with_library(&mut sim, &vec, &reg, &cache.doc(&tree).clone());

    // O gesto: apagar a gaveta.
    tree.delete(heroes);
    assert!(
        tree.catalogs()
            .iter()
            .all(|c| c.path != "Personagens/Herois")
    );

    // Ctrl+Z.
    let back = {
        crate::project_library::apply_forgotten(&before.library);
        crate::project_library::apply_catalogs(&before.library)
    };
    assert_eq!(
        back.catalogs().len(),
        2,
        "o undo não devolveu a gaveta e o pai dela"
    );
    assert_eq!(
        back.catalog_of(&AssetRef::Component { stable_id: 7 }),
        Some(heroes),
        "a gaveta voltou VAZIA — o undo devolveu os catálogos e perdeu o que estava dentro"
    );
    assert_eq!(
        back.catalog_of(&AssetRef::Texture { asset: [4; 32] }),
        Some(heroes)
    );
}

/// ⭐⭐⭐ **TIRAR UMA IMAGEM DA BIBLIOTECA DESFAZ-SE** — o *«inclusive em del»* do pedido.
///
/// ⛔ Antes disto o gesto era **irreversível**: a biblioteca é reconstruída do mundo a cada quadro
/// e uma imagem sem utilizadores não tem quem a re-lembre, então esquecê-la era para sempre.
///
/// **Mutação que deve sangrar:** fazer o `forget` voltar a `entries.remove(&id)`.
#[test]
fn removing_an_image_from_the_library_is_undone() {
    let reg = registry();
    let (mut sim, vec) = scene();
    crate::asset_index_build::set_forgotten_textures(&[]);

    let mut cache = crate::project_library::LibraryCache::default();
    let tree = ph2d_asset_index::CatalogTree::new();
    let before = capture_with_library(&mut sim, &vec, &reg, &cache.doc(&tree).clone());
    assert!(before.library.forgotten.is_empty());

    // O gesto: `Remove from Library` sobre uma imagem sem utilizadores.
    crate::asset_index_build::forget_texture(ph2d_asset::AssetId::from_digest([7; 32]));
    assert_eq!(
        crate::asset_index_build::forgotten_textures(),
        vec![[7; 32]],
        "a lápide não foi posta"
    );

    // Ctrl+Z.
    let _ = {
        crate::project_library::apply_forgotten(&before.library);
        crate::project_library::apply_catalogs(&before.library)
    };
    assert!(
        crate::asset_index_build::forgotten_textures().is_empty(),
        "a imagem não voltou — o gesto continua irreversível"
    );
    crate::asset_index_build::set_forgotten_textures(&[]);
}

/// ⛔⛔ **E o mesmo estado capturado duas vezes NÃO regista passo.**
///
/// ⚠️ É a lei que a captura por DIFF exige, e a que uma biblioteca mal desenhada partiria: se o
/// `collect` não fosse determinístico, ou se a `revision` entrasse no `PartialEq`, todo quadro com
/// input viraria um passo de undo e o Ctrl+Z gastar-se-ia a desfazer nada.
///
/// **Mutação que deve sangrar:** pôr `revision` no `PartialEq` do `CatalogTree` — não a apanha
/// directamente, mas a irmã abaixo apanha.
#[test]
fn capturing_the_same_library_twice_is_not_a_step() {
    use ph2d_asset_index::{AssetRef, CatalogTree};

    let reg = registry();
    let (mut sim, vec) = scene();
    let mut tree = CatalogTree::new();
    let c = tree.create("A/B");
    tree.assign(AssetRef::Texture { asset: [1; 32] }, c);
    let mut cache = crate::project_library::LibraryCache::default();

    let a = capture_with_library(&mut sim, &vec, &reg, &cache.doc(&tree).clone());
    let b = capture_with_library(&mut sim, &vec, &reg, &cache.doc(&tree).clone());
    assert_eq!(
        a.library, b.library,
        "duas capturas do mesmo estado diferem"
    );
}

/// ⭐⭐ **E uma árvore RESTAURADA volta a codificar-se nos MESMOS bytes.**
///
/// ⚠️ Esta é a metade que a `revision` podia partir: a árvore restaurada nasce com revisão `0` e a
/// original tem `N`. Se a revisão fosse identidade, o quadro seguinte a um undo registaria um passo
/// espúrio — e o Ctrl+Z seguinte não iria a lado nenhum.
///
/// **Mutação que deve sangrar:** derivar `PartialEq` no `CatalogTree` (a revisão volta a contar).
#[test]
fn a_restored_tree_encodes_to_the_same_bytes() {
    use ph2d_asset_index::{AssetRef, CatalogTree};

    let mut tree = CatalogTree::new();
    let c = tree.create("A/B");
    tree.assign(AssetRef::Texture { asset: [1; 32] }, c);
    tree.rename(c, "C");
    // ⛔⛔ **O DELETE é o que faltava a esta fixtura** (auditoria de 2026-08-30). Sem um catálogo
    // apagado, o `next_id` derivado de `max(id) + 1` calha certo e o gate afirmava o round-trip
    // **sem nunca o pôr à prova** — a mutação que o volta a derivar SOBREVIVIA. Com o delete, o id
    // seguinte era RECICLADO: apagar o `Z`(3) e o próximo nascia com o `3` outra vez.
    let doomed = tree.create("Z");
    tree.delete(doomed);
    assert!(tree.revision() > 0, "a fixtura precisa de uma revisão > 0");

    let mut cache = crate::project_library::LibraryCache::default();
    let doc = cache.doc(&tree).clone();
    let back = {
        crate::project_library::apply_forgotten(&doc);
        crate::project_library::apply_catalogs(&doc)
    };
    assert_eq!(
        back.revision(),
        0,
        "a fixtura mede o caso em que elas diferem"
    );
    assert_eq!(back, tree, "a árvore restaurada não é igual à original");
    assert_eq!(
        back.next_id(),
        tree.next_id(),
        "o próximo id foi RECICLADO — um catálogo novo herdaria o id de um apagado"
    );

    let mut cache2 = crate::project_library::LibraryCache::default();
    assert_eq!(
        cache2.doc(&back).catalog_bytes,
        doc.catalog_bytes,
        "a árvore restaurada codifica-se noutros bytes — todo undo registaria um passo espúrio"
    );
}
