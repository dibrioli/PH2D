//! Os gates do blob da taxonomia ([`super`]).

use ph2d_asset_index::{AssetRef, CatalogTree};

/// ⭐⭐ **A ida-e-volta é EXACTA** — os catálogos, os caminhos e as atribuições das DUAS famílias.
///
/// ⚠️ **O oráculo é a árvore inteira**, e não uma contagem: um round-trip que perdesse os caminhos
/// e mantivesse os ids passaria numa contagem, e o artista reabriria o projecto com as gavetas sem
/// nome.
///
/// **Mutação que deve sangrar:** gravar `Vec::new()` nas atribuições.
#[test]
fn a_saved_taxonomy_reopens_exactly_as_it_was() {
    let mut t = CatalogTree::new();
    let heroes = t.create("Personagens/Heróis");
    let props = t.create("Cenário/Props");
    let prefab = AssetRef::Component { stable_id: 7 };
    let image = AssetRef::Texture { asset: [9; 32] };
    t.assign(prefab, heroes);
    t.assign(image, props);

    let back = super::restore(&super::collect(&t));
    assert_eq!(back, t, "a taxonomia não sobreviveu à ida-e-volta");
    assert_eq!(back.catalog_of(&prefab), Some(heroes));
    assert_eq!(back.catalog_of(&image), Some(props));
}

/// ⚠️ **Determinístico** (HR-5): dois saves da mesma taxonomia dão os MESMOS bytes.
#[test]
fn two_saves_of_the_same_taxonomy_are_byte_identical() {
    let mut t = CatalogTree::new();
    t.create("B");
    t.create("A/Z");
    let a = t.create("A");
    t.assign(AssetRef::Component { stable_id: 1 }, a);
    assert_eq!(super::collect(&t), super::collect(&t));
}

/// Um projecto sem taxonomia nenhuma grava vazio e volta vazio — sem erro e sem ruído.
#[test]
fn an_empty_taxonomy_round_trips_as_empty() {
    let t = CatalogTree::new();
    assert!(super::restore(&super::collect(&t)).is_empty());
    assert!(super::restore(&[]).is_empty());
}

/// ⛔ **Um blob ilegível não estoura e não fica em silêncio** — ele devolve vazio e diz. *Um
/// projecto que abrisse sem catálogos sem uma linha de log faria o artista concluir que o trabalho
/// de arrumação se perdeu, sem nada a que agarrar.*
#[test]
fn an_unreadable_blob_opens_empty_instead_of_exploding() {
    assert!(super::restore(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).is_empty());
}

/// ⭐ **O `next_id` sobrevive ao ficheiro** — um catálogo criado depois de abrir não pode sentar-se
/// em cima de um carregado, senão os assets dele apareceriam dentro do novo.
///
/// **Mutação que deve sangrar:** o `restore` da árvore repor `next_id: 1`.
#[test]
fn a_reopened_taxonomy_never_hands_out_an_id_it_already_has() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("B");
    let mut back = super::restore(&super::collect(&t));
    let novo = back.create("Novo");
    assert_ne!(novo, a);
    assert_ne!(novo, b);
}
