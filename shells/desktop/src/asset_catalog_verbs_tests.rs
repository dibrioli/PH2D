//! Os gates dos verbos de catálogo ([`super`]).

use ph2d_asset_index::{AssetRef, CatalogId, CatalogTree};
use ph2d_editor::action_bus::CatalogVerb;
use ph2d_editor::interaction::drag_payload::DragPayload;

fn voz() -> ph2d_editor::ToastQueue {
    ph2d_editor::ToastQueue::default()
}

/// ⭐⭐ **Dois `+` seguidos dão DOIS catálogos.**
///
/// ⛔ O `create` do modelo é idempotente **por caminho**, então sem o sufixo único o segundo `+`
/// devolveria o primeiro e o botão pareceria morto — um gesto que não faz nada e não diz nada.
///
/// **Mutação que deve sangrar:** devolver sempre `BASE` no `free_path`.
#[test]
fn two_clicks_on_new_make_two_catalogs() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    assert!(super::drain(
        &CatalogVerb::New { parent: None },
        &mut t,
        &mut v
    ));
    assert!(super::drain(
        &CatalogVerb::New { parent: None },
        &mut t,
        &mut v
    ));
    assert_eq!(t.catalogs().len(), 2, "o segundo `+` não criou nada");
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, vec!["Catalog", "Catalog 2"]);
}

/// ⭐ **O catálogo nasce DENTRO do escolhido** — é o que torna a hierarquia alcançável sem um campo
/// de caminho.
#[test]
fn a_new_catalog_is_born_inside_the_chosen_one() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    let parent = t.create("Personagens");
    super::drain(
        &CatalogVerb::New {
            parent: Some(parent.0),
        },
        &mut t,
        &mut v,
    );
    let paths: Vec<&str> = t.catalogs().iter().map(|c| c.path.as_str()).collect();
    assert_eq!(paths, vec!["Personagens", "Personagens/Catalog"]);
}

/// ⚠️ **Um nome ilegal RECUSA em voz alta e não muda nada** — o silêncio deixaria o artista a olhar
/// para o nome antigo sem saber porquê.
#[test]
fn an_illegal_name_is_refused_out_loud_and_changes_nothing() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    let id = t.create("Props");
    assert!(!super::drain(
        &CatalogVerb::Rename {
            id: id.0,
            name: "Cenário/Props".into()
        },
        &mut t,
        &mut v
    ));
    assert_eq!(t.catalogs()[0].path, "Props");
    assert!(!v.is_empty(), "a recusa tem de falar");
}

/// ⭐⭐ **Apagar um catálogo NÃO apaga os assets** — eles voltam a *Unassigned*, e a voz di-lo.
#[test]
fn deleting_a_catalog_frees_the_assets_instead_of_deleting_them() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    let id = t.create("Props");
    let asset = AssetRef::Component { stable_id: 3 };
    t.assign(asset, id);
    assert!(super::drain(
        &CatalogVerb::Delete { id: id.0 },
        &mut t,
        &mut v
    ));
    assert!(t.is_empty());
    assert_eq!(t.catalog_of(&asset), None);
}

/// ⭐ **Atribuir e desatribuir**, pelas duas famílias de endereço.
#[test]
fn assigning_moves_the_asset_and_none_takes_it_out() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    let a = t.create("A");
    let img = DragPayload::Image { asset: [4; 32] };
    let key = AssetRef::Texture { asset: [4; 32] };
    assert!(super::drain(
        &CatalogVerb::Assign {
            asset: img,
            catalog: Some(a.0)
        },
        &mut t,
        &mut v
    ));
    assert_eq!(t.catalog_of(&key), Some(a));
    assert!(super::drain(
        &CatalogVerb::Assign {
            asset: img,
            catalog: None
        },
        &mut t,
        &mut v
    ));
    assert_eq!(t.catalog_of(&key), None);
}

/// ⚠️ **Atribuir a um catálogo que não existe não inventa um** — e devolve `false`, para o título
/// não se marcar sujo sobre um gesto que não mudou nada.
#[test]
fn assigning_to_a_catalog_that_is_gone_does_nothing() {
    let mut t = CatalogTree::new();
    let mut v = voz();
    assert!(!super::drain(
        &CatalogVerb::Assign {
            asset: DragPayload::Prefab { stable_id: 1 },
            catalog: Some(CatalogId(999).0)
        },
        &mut t,
        &mut v
    ));
    assert!(t.is_empty());
}
