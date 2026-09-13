//! Os gates da porta [`super::apply`] — a metade do LOAD que o gate do undo não pode ver (os nossos
//! próprios bytes nunca têm gémeos).

use ph2d_app_components::tags_doc::SavedTag;
use ph2d_ecs::tags::Tags;
use ph2d_ecs::{Name, Transform, World};
use ph2d_tags::TagId;

/// Um documento com dois caminhos iguais e ids diferentes — o fenómeno medido no ficheiro de
/// catálogos que o Blender distribui (2 de 63), com o gémeo DOBRADO que a decisão D2 acrescenta.
fn documento_com_gemeos() -> Vec<u8> {
    postcard::to_allocvec(&(
        ph2d_app_components::tags_doc::TAGS_DOC_VERSION,
        vec![
            SavedTag {
                id: 3,
                path: "Enemy".into(),
            },
            SavedTag {
                id: 9,
                path: "énemy".into(),
            },
        ],
        10u64,
    ))
    .expect("serializa")
}

/// ⭐⭐⭐ **O objecto do gémeo DESCARTADO não perde a tag** — a árvore funde, e a pertença acompanha
/// no mesmo gesto.
///
/// **Mutação que deve sangrar:** a porta a devolver só a árvore (sem o `remap`) — o Goblin ficaria
/// com um id que a árvore já não tem, e a tag dele sumia sem uma linha de aviso.
#[test]
fn loading_a_document_with_twins_keeps_every_member() {
    let mut w = World::new();
    let goblin = w
        .spawn((
            Transform::default(),
            Name::new("Goblin"),
            Tags::from_ids([TagId(9)]),
        ))
        .id();
    let bat = w
        .spawn((
            Transform::default(),
            Name::new("Bat"),
            Tags::from_ids([TagId(3)]),
        ))
        .id();

    let (tree, reapontados) = super::apply(&documento_com_gemeos(), &mut w);

    assert_eq!(tree.len(), 1, "os gemeos nao fundiram");
    let ficou = tree.find("Enemy").expect("a tag que ficou");
    assert_eq!(ficou, TagId(3), "a fusao tem de ser no MENOR id");
    assert_eq!(reapontados, 1, "so' o objecto do gemeo descartado muda");
    for (nome, e) in [("Goblin", goblin), ("Bat", bat)] {
        assert_eq!(
            w.get::<Tags>(e),
            Some(&Tags::from_ids([ficou])),
            "{nome} ficou com uma tag que a arvore nao tem"
        );
    }
}

/// ⚠️ **Sem gémeos, a porta não toca em objecto nenhum** — o caso de todo documento nosso.
#[test]
fn a_document_without_twins_touches_nobody() {
    let mut tree = ph2d_tags::TagTree::new();
    let enemy = tree.create("Enemy").expect("cria");
    let bytes = ph2d_app_components::tags_doc::collect(&tree);
    let mut w = World::new();
    let e = w
        .spawn((
            Transform::default(),
            Name::new("Goblin"),
            Tags::from_ids([enemy]),
        ))
        .id();

    let (de_volta, reapontados) = super::apply(&bytes, &mut w);
    assert_eq!(reapontados, 0);
    assert_eq!(de_volta, tree);
    assert_eq!(w.get::<Tags>(e), Some(&Tags::from_ids([enemy])));
}

/// ⚠️ **Um blob vazio abre uma árvore vazia** — é o que um ficheiro migrado de v128 traz.
#[test]
fn an_empty_blob_opens_an_empty_tree() {
    let mut w = World::new();
    let (tree, reapontados) = super::apply(&[], &mut w);
    assert!(tree.is_empty());
    assert_eq!(reapontados, 0);
}
