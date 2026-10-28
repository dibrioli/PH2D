//! ⭐ A fusão: comutativa e idempotente (a ordem, o atraso e a duplicação não mudam os bytes), e as
//! leis de cada conflito.

use ph2d_kanban_model::{CardId, History, KanbanOp, KanbanSet, Member, Record, RecordId};

use crate::KanbanReplica;
use crate::test_support::{K, Net, Rng, base, doc, edit_card, kanban, random_op};

fn card_ids(set: &KanbanSet) -> Vec<CardId> {
    kanban(set).cards.keys().copied().collect()
}

/// ⭐ N pares, gestos ao acaso no MESMO Kanban, entregues baralhados, aos bocados e em duplicado:
/// todos acabam com os MESMOS bytes de `KanbanSet::to_bytes`.
#[test]
fn the_merge_is_commutative_and_idempotent_over_random_ops() {
    for seed in 0..40u64 {
        let mut rng = Rng(seed);
        let n = 2 + (seed as usize % 3);
        let mut net = Net::new(n);
        for _ in 0..12 {
            for p in 0..n {
                for _ in 0..rng.below(4) {
                    random_op(&mut net.sets[p], &mut net.reps[p], p as u64 + 1, &mut rng);
                }
                net.send(p, &mut rng);
            }
            for q in 0..n {
                net.deliver(q, &mut rng, false);
            }
        }
        net.settle(&mut rng);
        let bytes = net.bytes();
        for (p, b) in bytes.iter().enumerate() {
            assert_eq!(b, &bytes[0], "seed {seed}: o par {p} divergiu");
        }
        for r in &net.reps {
            assert_eq!(r.pending(), 0, "seed {seed}");
        }
        // Nenhum par perdeu o que só ele fez: há registos de todos.
        assert!(kanban(&net.sets[0]).checklists.len() + kanban(&net.sets[0]).cards.len() > 6);
    }
}

/// Entregar o mesmo lote duas vezes, ou em ordens opostas, dá o mesmo.
#[test]
fn delivering_twice_or_reversed_changes_nothing() {
    let mut a = Net::new(3);
    let mut rng = Rng(7);
    for p in 0..3 {
        for _ in 0..6 {
            random_op(&mut a.sets[p], &mut a.reps[p], p as u64 + 1, &mut rng);
        }
    }
    let ws: Vec<_> = (1..3).map(|p| a.reps[p].local(&mut a.sets[p])).collect();
    let mut one = (a.sets[0].clone(), a.reps[0].clone());
    let mut two = (a.sets[0].clone(), a.reps[0].clone());
    for w in &ws {
        one.1.receive(&mut one.0, w);
        one.1.receive(&mut one.0, w);
    }
    for w in ws.iter().rev() {
        let mut rev = w.clone();
        rev.reverse();
        two.1.receive(&mut two.0, &rev);
    }
    assert_eq!(one.0.to_bytes().unwrap(), two.0.to_bytes().unwrap());
}

/// Dois colegas a arrastar cartões DIFERENTES: os dois arrastos ficam.
#[test]
fn two_colleagues_dragging_different_cards_merge_without_conflict() {
    let mut net = Net::new(2);
    let ids = card_ids(&net.sets[0]);
    let lists: Vec<_> = kanban(&net.sets[0]).lists.keys().copied().collect();
    let (a, b) = (ids[0], ids[1]);
    let pa = kanban(&net.sets[0]).pos_after_cards(lists[2]);
    edit_card(&mut net.sets[0], a, |c| {
        (c.list, c.pos) = (lists[2], pa.clone())
    });
    let pb = kanban(&net.sets[1]).pos_after_cards(lists[1]);
    edit_card(&mut net.sets[1], b, |c| {
        (c.list, c.pos) = (lists[1], pb.clone())
    });
    net.settle(&mut Rng(1));
    for s in &net.sets {
        assert_eq!(kanban(s).cards[&a].list, lists[2]);
        assert_eq!(kanban(s).cards[&b].list, lists[1]);
    }
    let b = net.bytes();
    assert_eq!(b[0], b[1]);
}

/// O MESMO campo editado pelos dois ao mesmo tempo: em todos, o carimbo maior (o par desempata).
#[test]
fn the_same_field_goes_to_the_bigger_lamport_on_every_peer() {
    let mut net = Net::new(3);
    let id = card_ids(&net.sets[0])[0];
    for p in 0..3 {
        edit_card(&mut net.sets[p], id, |c| c.title = format!("de {p}"));
    }
    net.settle(&mut Rng(2));
    // Todos com o contador 1 ⇒ ganha o par maior (o 3, índice 2).
    for s in &net.sets {
        assert_eq!(kanban(s).cards[&id].title, "de 2");
    }
    // E noutra ordem de entrega, o mesmo vencedor.
    let mut again = Net::new(3);
    for p in (0..3).rev() {
        edit_card(&mut again.sets[p], id, |c| c.title = format!("de {p}"));
    }
    again.settle(&mut Rng(99));
    assert_eq!(kanban(&again.sets[0]).cards[&id].title, "de 2");
}

/// Apagar e, ao mesmo tempo, editar outro campo: a lápide fica (a edição não ressuscita), e a edição
/// também (a lei do Quadro: apagar é um registo).
#[test]
fn a_delete_and_a_concurrent_edit_keep_the_tombstone() {
    let mut net = Net::new(2);
    let id = card_ids(&net.sets[0])[0];
    let _ = KanbanOp::Delete(RecordId::Card(id)).apply(doc(&mut net.sets[0]));
    edit_card(&mut net.sets[1], id, |c| c.title = "editado".into());
    net.settle(&mut Rng(3));
    for s in &net.sets {
        let c = &kanban(s).cards[&id];
        assert!(c.meta.deleted, "a edição não ressuscita");
        assert_eq!(c.title, "editado");
    }
    let b = net.bytes();
    assert_eq!(b[0], b[1]);
}

/// Dois votos ao mesmo tempo são dois votos; duas etiquetas, duas etiquetas; dois campos
/// personalizados diferentes, os dois.
#[test]
fn two_votes_at_the_same_time_are_two_votes() {
    let mut net = Net::new(2);
    let id = card_ids(&net.sets[0])[0];
    let labels: Vec<_> = kanban(&net.sets[0]).labels.keys().copied().collect();
    use ph2d_kanban_model::{FieldId, FieldValue};
    for p in 0..2 {
        let l = labels[p];
        edit_card(&mut net.sets[p], id, |c| {
            c.votes.push(Member(format!("p{p}")));
            c.labels.push(l);
            c.fields
                .insert(FieldId(p as u64 + 1), FieldValue::Number(p as f64));
        });
    }
    net.settle(&mut Rng(4));
    for s in &net.sets {
        let c = &kanban(s).cards[&id];
        assert_eq!(c.votes.len(), 2, "{:?}", c.votes);
        assert_eq!(c.labels.len(), 2);
        assert_eq!(c.fields.len(), 2);
    }
    // Tirar o meu voto não leva o do outro.
    edit_card(&mut net.sets[0], id, |c| c.votes.retain(|m| m.0 != "p0"));
    net.settle(&mut Rng(5));
    for s in &net.sets {
        assert_eq!(kanban(s).cards[&id].votes, vec![Member("p1".into())]);
    }
}

/// ⭐ O desfazer ao vivo: o meu `Ctrl+Z` repõe o MEU campo e não apaga a edição do colega, que
/// chegou depois, noutro campo do MESMO cartão.
#[test]
fn undo_in_a_live_session_reverts_only_my_fields() {
    let mut net = Net::new(2);
    let id = card_ids(&net.sets[0])[0];
    let mut h = History::default();
    let mut c = kanban(&net.sets[0]).cards[&id].clone();
    c.title = "meu".into();
    h.apply(
        doc(&mut net.sets[0]),
        vec![KanbanOp::put(Record::Card(c))],
        1,
    );
    net.settle(&mut Rng(6));
    edit_card(&mut net.sets[1], id, |c| c.description = "do colega".into());
    net.settle(&mut Rng(6));
    assert!(h.undo(doc(&mut net.sets[0]), 2));
    net.settle(&mut Rng(6));
    for s in &net.sets {
        let c = &kanban(s).cards[&id];
        assert_eq!(c.title, "card 0");
        assert_eq!(c.description, "do colega");
    }
}

/// A regra dos ficheiros por Kanban é a do conjunto (`KanbanSet::files_in_use`).
#[test]
fn files_of_one_kanban_match_the_sets_rule() {
    use ph2d_kanban_model::{
        Attachment, AttachmentId, Background, Cover, CoverFill, FracKey, Link, Meta, Moment,
    };
    let mut set = base();
    let a = set.put_file(vec![1u8, 2, 3], None);
    let b = set.put_file(vec![4u8], Some([1, 1]));
    let c = set.put_file(vec![5u8], None);
    let id = card_ids(&set)[0];
    edit_card(&mut set, id, |x| {
        x.cover = Some(Cover {
            fill: CoverFill::Image(a),
            full: false,
        });
    });
    let k = doc(&mut set);
    let att = Attachment {
        id: AttachmentId(k.mint()),
        card: id,
        pos: FracKey::between(None, None),
        name: "f".into(),
        link: Link::File(b),
        at: Moment(0),
        meta: Meta::default(),
    };
    let _ = KanbanOp::put(Record::Attachment(att)).apply(k);
    let mut h = k.header.clone();
    h.background = Background::Image(c);
    let _ = KanbanOp::Header(h).apply(k);
    assert_eq!(crate::table::files_of(kanban(&set)), set.files_in_use());
    assert_eq!(set.files_in_use().len(), 3);
}

/// Um ficheiro anexado por um convidado chega aos outros (uma vez).
#[test]
fn a_file_attached_by_a_guest_reaches_the_others() {
    use ph2d_kanban_model::{Cover, CoverFill};
    let mut net = Net::new(3);
    let id = card_ids(&net.sets[1])[0];
    let f = net.sets[1].put_file(vec![7u8; 64], Some([8, 8]));
    edit_card(&mut net.sets[1], id, |c| {
        c.cover = Some(Cover {
            fill: CoverFill::Image(f),
            full: true,
        });
    });
    net.settle(&mut Rng(8));
    for s in &net.sets {
        assert!(s.file(f).is_some());
    }
    let b = net.bytes();
    assert!(b.iter().all(|x| *x == b[0]));
}

/// Um Kanban partilhado a meio entra no projecto do outro com um id DELE (o meu Kanban 1 não é o
/// teu).
#[test]
fn a_kanban_shared_midway_arrives_with_a_new_local_id() {
    let mut net = Net::new(2);
    let mine = net.sets[1].create("Só meu");
    let other = net.sets[0].create("Do anfitrião");
    assert_eq!(mine, other, "os dois projectos dão o mesmo número");
    let wire = net.reps[1].share(&net.sets[1], mine).unwrap();
    assert_ne!(wire, mine, "o convidado sorteia o id da sessão");
    net.settle(&mut Rng(9));
    let local = net.reps[0].local_id(wire).unwrap();
    assert_ne!(local, other);
    assert_eq!(net.sets[0].get(local).unwrap().header.name, "Só meu");
    assert_eq!(net.sets[0].get(other).unwrap().header.name, "Do anfitrião");
}

/// Quem cai (perdendo o que ia no fio) e volta pelo retrato converge, sem perder o que fez.
#[test]
fn a_guest_that_drops_and_rejoins_converges() {
    let mut net = Net::new(2);
    let mut rng = Rng(10);
    let id = card_ids(&net.sets[1])[0];
    edit_card(&mut net.sets[1], id, |c| c.title = "antes de cair".into());
    let _lost = net.reps[1].local(&mut net.sets[1]);
    for _ in 0..5 {
        random_op(&mut net.sets[0], &mut net.reps[0], 1, &mut rng);
    }
    net.send(0, &mut rng);
    net.inbox[1].clear();
    let _ = net.reps[0].local(&mut net.sets[0]);
    let snap = net.reps[0].snapshot(&net.sets[0]);
    let (s, r) = (&mut net.sets[1], &mut net.reps[1]);
    r.rejoin(s, &snap).unwrap();
    net.settle(&mut rng);
    let b = net.bytes();
    assert_eq!(b[0], b[1]);
    let c = &kanban(&net.sets[0]).cards[&id];
    assert!(c.meta.deleted || c.title == "antes de cair" || c.title.starts_with('t'));
}

#[test]
fn session_ids_have_the_high_bit_and_differ_between_peers() {
    let set = base();
    let mut a = KanbanReplica::host(1, 5, &set, &[K]);
    let mut b = KanbanReplica::host(2, 6, &set, &[K]);
    let xs: Vec<u64> = (0..1000)
        .map(|_| a.mint())
        .chain((0..1000).map(|_| b.mint()))
        .collect();
    assert!(xs.iter().all(|x| x >> 63 == 1));
    let mut d = xs.clone();
    d.sort_unstable();
    d.dedup();
    assert_eq!(d.len(), xs.len());
}
