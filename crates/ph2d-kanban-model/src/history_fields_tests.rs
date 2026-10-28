//! W5 — o desfazer POR CAMPO e a divisão em campos que a réplica ao vivo partilha.

use super::*;
use crate::test_support::{card_of, checklist, comment, sample};
use crate::{CardId, KanbanOp, Member, Record};

fn edit(k: &Kanban, id: CardId, f: impl FnOnce(&mut crate::Card)) -> KanbanOp {
    let mut c = card_of(k, id);
    f(&mut c);
    KanbanOp::put(Record::card(c))
}

#[test]
fn every_record_splits_into_fields_and_joins_back_byte_equal() {
    let mut k = sample();
    let id = k.cards.keys().copied().next().unwrap();
    for op in [checklist(&k, id), comment(&k, id, "Ana", "oi")] {
        let _ = op.apply(&mut k);
    }
    let all: Vec<Record> = k
        .lists
        .values()
        .cloned()
        .map(Record::List)
        .chain(k.cards.values().cloned().map(Record::card))
        .chain(k.checklists.values().cloned().map(Record::Checklist))
        .chain(k.comments.values().cloned().map(Record::Comment))
        .collect();
    for r in all {
        let f = r.fields();
        assert!(f.iter().any(|(n, _)| *n == "meta"), "{r:?}");
        let changes: Vec<(&str, &[u8])> = f.iter().map(|(n, b)| (*n, b.as_slice())).collect();
        let back = r.with_fields(&changes).expect("junta");
        assert!(back.same_content(&r), "{r:?}");
    }
    let h = k.header.fields();
    assert!(!h.iter().any(|(n, _)| *n == "version"));
    assert_eq!(k.header.with_fields(&[]).unwrap(), k.header);
}

#[test]
fn bytes_of_another_field_are_refused_not_misaligned() {
    let k = sample();
    let c = Record::card(k.cards.values().next().unwrap().clone());
    assert!(c.with_fields(&[("title", &[0xff, 0xff, 0xff])]).is_none());
}

/// ⭐ O meu desfazer devolve o MEU campo e deixa o do colega (escrito depois, noutro campo).
#[test]
fn undo_restores_my_field_and_keeps_the_field_a_colleague_changed() {
    let mut k = sample();
    let id = k.cards.keys().copied().next().unwrap();
    let mut h = History::default();
    let op = edit(&k, id, |c| {
        c.title = "mine".into();
        c.description = "mine too".into();
    });
    h.apply(&mut k, vec![op], 1);
    let theirs = edit(&k, id, |c| c.description = "theirs".into());
    apply_batch(&mut k, vec![theirs]);
    assert!(h.undo(&mut k, 2));
    let c = card_of(&k, id);
    assert_eq!(c.title, "Sketch", "o meu campo volta");
    assert_eq!(c.description, "theirs", "o do colega fica");
}

#[test]
fn undo_of_the_header_is_per_field_too() {
    let mut k = sample();
    let mut h = History::default();
    let mut mine = k.header.clone();
    mine.name = "Meu".into();
    mine.starred = true;
    h.apply(&mut k, vec![KanbanOp::Header(mine)], 1);
    let mut theirs = k.header.clone();
    theirs.starred = false;
    theirs.watchers = vec![Member("Bia".into())];
    apply_batch(&mut k, vec![KanbanOp::Header(theirs)]);
    assert!(h.undo(&mut k, 2));
    assert_eq!(k.header.name, "Projeto");
    assert!(!k.header.starred);
    assert_eq!(k.header.watchers, vec![Member("Bia".into())]);
}

/// Desfazer o nascimento de um cartão apaga-o mesmo que o colega lhe tenha mudado o título (a lápide
/// é um campo que ninguém mexeu) — e não o apaga se o colega já o apagou e reviveu… nem o pisa.
#[test]
fn undo_of_a_creation_is_the_tombstone_field() {
    let mut k = sample();
    let todo = k.lists_in_order()[0].id;
    let mut h = History::default();
    let op = crate::test_support::card(&k, todo, "novo");
    let id = match &op {
        KanbanOp::Put(r) => match &**r {
            Record::Card(c) => c.id,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    h.apply(&mut k, vec![op], 1);
    let theirs = edit(&k, id, |c| c.title = "deles".into());
    apply_batch(&mut k, vec![theirs]);
    assert!(h.undo(&mut k, 2));
    assert!(k.cards[&id].meta.deleted);
}

/// R1 — um passo cujos campos um colega mexeu TODOS ficou vazio: o desfazer DESCARTA-O e desfaz o
/// anterior (o idioma dos editores colaborativos), sem refazer vazio. Só vazios ⇒ `false`.
#[test]
fn an_undo_step_left_empty_by_a_colleague_is_skipped_not_counted() {
    let mut k = sample();
    let id = k.cards.keys().copied().next().unwrap();
    let mut h = History::default();
    let first = edit(&k, id, |c| c.description = "first".into());
    h.apply(&mut k, vec![first], 1);
    let second = edit(&k, id, |c| c.title = "mine".into());
    h.apply(&mut k, vec![second], 2);
    let theirs = edit(&k, id, |c| c.title = "theirs".into());
    apply_batch(&mut k, vec![theirs]);
    assert!(h.undo(&mut k, 3), "desfaz o passo de antes");
    let c = card_of(&k, id);
    assert_eq!(c.title, "theirs", "o título do colega fica");
    assert_ne!(c.description, "first", "o passo anterior foi desfeito");
    assert!(!h.can_undo());
    assert!(h.redo(&mut k, 4));
    assert_eq!(card_of(&k, id).description, "first");
    assert!(!h.can_redo(), "nenhum refazer vazio ficou na pilha");

    let mut h = History::default();
    let mine = edit(&k, id, |c| c.title = "mine again".into());
    h.apply(&mut k, vec![mine], 5);
    let theirs = edit(&k, id, |c| c.title = "theirs again".into());
    apply_batch(&mut k, vec![theirs]);
    assert!(
        !h.undo(&mut k, 6),
        "só havia um passo vazio: nada a desfazer"
    );
    assert!(!h.can_redo(), "sem refazer vazio");
}
