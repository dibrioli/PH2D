//! Os gates da taxonomia ([`super`]).
//!
//! ⚠️ **O oráculo é o que a UI desenharia e o que a grade mostraria**, nunca o estado interno: os
//! caminhos por ordem, quem sobrevive a um renomear, e o que a escolha de um catálogo alcança.

use super::{Catalog, CatalogScope, CatalogTree};
use crate::{AssetRef, CatalogId};

fn prefab(n: u64) -> AssetRef {
    AssetRef::Component { stable_id: n }
}

/// ⭐⭐ **Criar `"A/B"` cria `"A"`** — senão a árvore derivada desenha um filho sem pai.
///
/// **Mutação que deve sangrar:** criar só o nível final.
#[test]
fn creating_a_nested_catalog_creates_the_ancestors_it_needs() {
    let mut t = CatalogTree::new();
    let deep = t.create("Personagens/Heróis/Espadachins");
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "Personagens",
            "Personagens/Heróis",
            "Personagens/Heróis/Espadachins"
        ],
        "os três níveis têm de existir, por ordem"
    );
    assert_eq!(t.get(deep).map(|c| c.label()), Some("Espadachins"));
    assert_eq!(t.get(deep).map(super::Catalog::depth), Some(2));
}

/// Criar o mesmo caminho duas vezes devolve o MESMO catálogo — o gesto é idempotente.
#[test]
fn creating_the_same_path_twice_is_the_same_catalog() {
    let mut t = CatalogTree::new();
    let a = t.create("Props");
    let b = t.create("Props");
    assert_eq!(a, b);
    assert_eq!(t.catalogs().len(), 1);
}

/// ⭐⭐⭐ **Renomear leva os FILHOS e não mexe nos assets.**
///
/// ⚠️ A fronteira é o SEPARADOR: sem ele, renomear `"Hero"` reescreveria `"Heroine"` — um prefixo
/// de texto não é um prefixo de caminho.
///
/// **Mutação que deve sangrar:** usar `starts_with(&old)` em vez de `starts_with("{old}/")`.
#[test]
fn renaming_carries_the_children_and_never_the_neighbour() {
    let mut t = CatalogTree::new();
    let hero = t.create("Hero");
    t.create("Hero/Armas");
    t.create("Heroine");
    let asset = prefab(7);
    t.assign(asset, hero);

    assert!(t.rename(hero, "Herói"));
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    // ⚠️ **`"Heroine"` vem ANTES de `"Herói"`, e isso é a dívida de colação declarada no
    // `sort_as_a_tree`:** a ordem é por segmento em minúsculas, sem tabela de acentos — a mesma
    // que a grade de cartões já tem. *Duas metades do mesmo painel com ordens diferentes seria
    // pior que esta.*
    assert_eq!(paths, vec!["Heroine", "Herói", "Herói/Armas"]);
    assert_eq!(
        t.catalog_of(&asset),
        Some(hero),
        "o asset continua no MESMO catálogo — a identidade não é o caminho"
    );
}

/// Um nome com separador seria «mover» escondido dentro de «renomear» — recusado.
#[test]
fn renaming_refuses_a_name_that_would_move_the_catalog() {
    let mut t = CatalogTree::new();
    let id = t.create("Props");
    assert!(!t.rename(id, "Cenário/Props"));
    assert!(!t.rename(id, "   "));
    assert_eq!(t.catalogs()[0].path, "Props");
}

/// ⭐⭐ **Apagar leva os descendentes, e os assets voltam a *Unassigned* — nunca são apagados.**
///
/// **Mutação que deve sangrar:** apagar também as entradas do asset em vez das atribuições.
#[test]
fn deleting_takes_the_descendants_and_frees_the_assets() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("A/B");
    let other = t.create("Z");
    let x = prefab(1);
    let y = prefab(2);
    t.assign(x, b);
    t.assign(y, other);

    t.delete(a);
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, vec!["Z"], "o pai e o filho saíram");
    assert_eq!(t.catalog_of(&x), None, "o asset ficou sem catálogo");
    assert_eq!(t.catalog_of(&y), Some(other), "e o vizinho não se mexeu");
}

/// ⭐⭐ **Escolher um catálogo alcança os DESCENDENTES dele.**
///
/// **Mutação que deve sangrar:** devolver `These(vec![id])`.
#[test]
fn choosing_a_catalog_reaches_what_is_inside_its_children() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("A/B");
    let z = t.create("Z");
    let CatalogScope::These(ids) = t.scope_of(a) else {
        panic!("um catálogo existente tem escopo");
    };
    assert!(ids.contains(&a) && ids.contains(&b), "{ids:?}");
    assert!(!ids.contains(&z));
}

/// A contagem que a linha mostra soma os descendentes.
#[test]
fn the_row_count_sums_the_descendants() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("A/B");
    t.assign(prefab(1), a);
    t.assign(prefab(2), b);
    t.assign(prefab(3), b);
    assert_eq!(t.count_in(a), 3);
    assert_eq!(t.count_in(b), 2);
}

/// ⚠️ **Um asset está em UM catálogo** — atribuir de novo tira-o do anterior sem gesto nenhum.
#[test]
fn assigning_again_moves_the_asset_instead_of_duplicating_it() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("B");
    let x = prefab(1);
    t.assign(x, a);
    t.assign(x, b);
    assert_eq!(t.catalog_of(&x), Some(b));
    assert_eq!(t.count_in(a), 0);
    assert_eq!(t.count_in(b), 1);
}

/// Níveis vazios e espaços não produzem linhas em branco que ninguém consegue escolher.
#[test]
fn a_path_with_empty_levels_does_not_make_nameless_rows() {
    let mut t = CatalogTree::new();
    t.create("  A / / B  ");
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, vec!["A", "A/B"]);
}

/// ⭐ **O `next_id` do restore salta o que chegou** — senão o gesto seguinte sentava-se em cima de
/// um catálogo carregado, e os assets dele apareceriam dentro do novo.
///
/// **Mutação que deve sangrar:** `next_id: 1` no `restore`.
#[test]
fn a_restored_tree_never_hands_out_an_id_it_already_has() {
    let mut t = CatalogTree::restore(
        vec![Catalog {
            id: CatalogId(42),
            path: "Velho".into(),
        }],
        std::collections::BTreeMap::new(),
    );
    let novo = t.create("Novo");
    assert_ne!(novo, CatalogId(42), "o id novo colidiu com o carregado");
    assert_eq!(t.catalogs().len(), 2);
}

/// ⭐⭐⭐ **UM PAI VEM IMEDIATAMENTE ANTES DOS FILHOS DELE** — e um irmão nunca se mete no meio.
///
/// ⛔ A árvore que a UI desenha é derivada da PROFUNDIDADE de cada linha, então esta ordem **é** a
/// árvore. Ordenar pelo texto cru parte-a: `'-'` (0x2D) é menor que `'/'` (0x2F), e `"A-x"` cai
/// entre `"A"` e `"A/B"` ⇒ o filho aparece indentado debaixo de um irmão.
///
/// **Mutação que deve sangrar:** voltar a `sort_by(|a, b| a.path.cmp(&b.path))`.
#[test]
fn a_parent_is_always_immediately_followed_by_its_children() {
    let mut t = CatalogTree::new();
    t.create("A/B");
    t.create("A-x");
    t.create("A");
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["A", "A/B", "A-x"],
        "o irmão `A-x` meteu-se entre o pai e o filho — a árvore desenha-se errada"
    );
}
