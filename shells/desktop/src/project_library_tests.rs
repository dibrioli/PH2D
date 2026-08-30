//! Os gates do documento da biblioteca ([`super`]).

use super::{LibraryCache, LibraryDoc, apply};
use ph2d_asset_index::{AssetRef, CatalogTree};

/// ⭐⭐⭐ **A cache re-codifica quando a árvore muda, e NÃO quando ela não muda.**
///
/// ⚠️ É esta a lei que torna a taxonomia barata o suficiente para viver na captura do undo: sem
/// ela, o `collect` corria em **todo quadro com input** e custava 4,8 % de um quadro a 50
/// catálogos (`measure_catalog_capture_cost`).
///
/// **Mutação que deve sangrar:** apagar a guarda `if self.rev != Some(tree.revision())`.
#[test]
fn the_cache_re_encodes_on_a_change_and_only_then() {
    let mut tree = CatalogTree::new();
    let a = tree.create("Personagens");
    let mut cache = LibraryCache::default();

    let first = cache.doc(&tree).catalogs.clone();
    assert!(!first.is_empty());
    // Sem mutação, os MESMOS bytes e sem re-codificar — o oráculo do «sem re-codificar» é a
    // revisão, que é o que a cache lê.
    let rev = tree.revision();
    assert_eq!(cache.doc(&tree).catalogs, first);
    assert_eq!(tree.revision(), rev, "ler a cache mexeu na árvore");

    tree.rename(a, "Herois");
    assert_ne!(tree.revision(), rev, "renomear não moveu a revisão");
    assert_ne!(
        cache.doc(&tree).catalogs,
        first,
        "a árvore mudou e a cache devolveu os bytes velhos"
    );
}

/// ⛔⛔ **Uma árvore SUBSTITUÍDA por baixo invalida a cache — e a colisão de revisão é o caso
/// NORMAL, não o raro.**
///
/// ⚠️ A revisão é por-árvore e nasce em `0` a cada `restore`, então uma árvore restaurada tem
/// quase sempre uma revisão que a cache já viu. Sem o `invalidate`, o quadro seguinte a um undo
/// devolveria os bytes ANTIGOS e o passo seguinte gravaria a taxonomia errada.
///
/// **Mutação que deve sangrar:** fazer o `invalidate` não fazer nada.
#[test]
fn replacing_the_tree_underneath_needs_the_cache_invalidated() {
    let mut cache = LibraryCache::default();
    let mut old = CatalogTree::new();
    old.create("Antiga");
    let old_bytes = cache.doc(&old).catalogs.clone();

    // Uma árvore diferente com a MESMA revisão — o que um `restore` produz.
    let mut new = CatalogTree::new();
    new.create("Nova");
    assert_eq!(
        old.revision(),
        new.revision(),
        "esta fixtura precisa das duas revisões iguais para medir o que mede"
    );
    cache.invalidate();
    assert_ne!(
        cache.doc(&new).catalogs,
        old_bytes,
        "a cache devolveu a taxonomia antiga depois de a árvore ser substituída"
    );
}

/// ⭐⭐ **A ida-e-volta é exacta nas DUAS metades** — a taxonomia e as lápides.
#[test]
fn the_document_round_trips_both_halves() {
    let mut tree = CatalogTree::new();
    let a = tree.create("Personagens/Herois");
    tree.assign(AssetRef::Component { stable_id: 7 }, a);
    crate::asset_index_build::set_forgotten_textures(&[[3; 32], [9; 32]]);

    let mut cache = LibraryCache::default();
    let doc = cache.doc(&tree).clone();
    assert_eq!(doc.forgotten, vec![[3; 32], [9; 32]]);

    // Apaga tudo, e devolve pelo documento.
    crate::asset_index_build::set_forgotten_textures(&[]);
    let back = apply(&doc);
    assert_eq!(back, tree, "a taxonomia não voltou igual");
    assert_eq!(
        crate::asset_index_build::forgotten_textures(),
        vec![[3; 32], [9; 32]],
        "as lápides não voltaram"
    );
    crate::asset_index_build::set_forgotten_textures(&[]);
}

/// ⚠️ **Um documento vazio é uma biblioteca vazia**, e não um estouro — é o estado em que todo
/// projecto novo nasce.
#[test]
fn an_empty_document_is_an_empty_library() {
    let back = apply(&LibraryDoc::default());
    assert_eq!(back, CatalogTree::new());
    assert!(crate::asset_index_build::forgotten_textures().is_empty());
}
