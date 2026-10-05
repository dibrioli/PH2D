use super::*;
use crate::widget::command_palette::top_match;

const A: &[AddEntry] = &[
    AddEntry::new("object_add.flip", AddGroup::TwoD),
    AddEntry::new("object_add.vector.object", AddGroup::TwoD),
];
const B: &[AddEntry] = &[AddEntry::new("object_add.game.camera", AddGroup::Game)];

/// ⭐ **Cada entrada compilada tem UM item, e o id volta à mesma entrada.**
///
/// (Mutação: `entry_of_pick` a devolver a primeira entrada ⇒ RED.)
#[test]
fn every_entry_has_one_item_and_its_id_maps_back() {
    let fams: &[&[AddEntry]] = &[CORE, A, B];
    let model = build(fams, &|_| None);
    let all: Vec<AddEntry> = fams.iter().flat_map(|f| f.iter().copied()).collect();
    assert_eq!(model.item_count(), all.len());
    for e in &all {
        assert!(model.is_item(e.id()), "{} sem item", e.key.key());
        assert_eq!(entry_of_pick(fams, e.id()), Some(*e));
    }
    let mut ids: Vec<_> = all.iter().map(|e| e.id()).collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    assert_eq!(ids.len(), all.len(), "dois itens com o mesmo id");
}

/// **Uma família que não está compilada não deixa grupo** — o grupo vazio não é pintado.
#[test]
fn a_group_without_entries_is_not_shown() {
    let model = build(&[CORE], &|_| None);
    let titles: Vec<&str> = model.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles,
        [AddGroup::TwoD.title(), AddGroup::Empty.title()],
        "sem a família de Jogo, o grupo dela não pode aparecer"
    );
    assert_eq!(entry_of_pick(&[CORE], A[0].id()), None);
}

/// ⭐ **A busca acha cada tipo pelo NOME do tipo** — o rótulo do item leva-o, porque a busca da
/// paleta só lê o rótulo.
#[test]
fn the_search_finds_every_family_by_its_type_name() {
    let model = build(&[CORE, A, B], &|_| None);
    assert_eq!(top_match(&model, "flip"), Some(A[0].id()));
    assert_eq!(top_match(&model, "image"), Some(IMAGE.id()));
    for e in [CORE, A, B].iter().flat_map(|f| f.iter()) {
        assert_eq!(
            top_match(&model, e.key.tr()),
            Some(e.id()),
            "{}",
            e.key.key()
        );
    }
}

/// **Cada rótulo existe na tabela** — um `tr` que devolve a chave pintaria `object_add.…` no ecrã.
#[test]
fn every_label_is_translated() {
    for e in CORE.iter().chain(A).chain(B) {
        assert_ne!(e.key.tr(), e.key.key());
    }
    for g in AddGroup::ALL {
        assert!(!g.title().starts_with("object_add."));
    }
}

/// ⭐ **O que não pode nascer agora APARECE, depois do que pode, com a razão no rótulo** — e o id
/// continua a ser o da entrada, para o clique responder.
///
/// (Mutação: largar o `blocked` ⇒ RED; pô-lo antes do `ready` ⇒ RED.)
#[test]
fn a_blocked_entry_shows_after_the_ready_ones_with_its_reason() {
    let reason = "already one";
    let model = build(&[A], &|e| (e == A[0]).then_some(reason));
    let subs = &model.groups[0].subs;
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].items[0].id, A[1].id(), "o que pode vem primeiro");
    assert_eq!(subs[1].items[0].id, A[0].id());
    assert!(subs[1].items[0].label.ends_with(reason));
    assert!(subs[1].items[0].label.starts_with(A[0].key.tr()));
}
