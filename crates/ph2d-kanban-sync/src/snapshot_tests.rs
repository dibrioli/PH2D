//! O retrato compacto é SEM PERDA, e quem entra por blocos converge — também com escritas que
//! chegam a meio dos blocos. A sonda de tamanho e tempo (ignorada) escreve a tabela da MEDICOES.md.

use std::collections::BTreeMap;
use std::time::Instant;

use ph2d_kanban_model::{Card, CardId, FracKey, KanbanOp, KanbanSet, Member, Moment, Record};

use super::{SNAPSHOT_BLOCK, messages, pack, unpack};
use crate::test_support::{K, Net, Rng, base, doc, edit_card, kanban, random_op};
use crate::{GuestRoom, HostRoom, KMsg, KanbanReplica};

#[test]
fn the_packed_stamps_unpack_to_the_same_maps() {
    let mut net = Net::new(3);
    let mut rng = Rng(40);
    for _ in 0..8 {
        for p in 0..3 {
            for _ in 0..5 {
                random_op(&mut net.sets[p], &mut net.reps[p], p as u64 + 1, &mut rng);
            }
            net.send(p, &mut rng);
        }
    }
    net.settle(&mut rng);
    let r = &net.reps[1];
    assert!(!r.stamps.is_empty() && !r.elems.is_empty() && !r.tops.is_empty());
    let packed = pack(&r.stamps, &r.elems, &r.tops);
    let (mut s, mut e, mut t) = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    unpack(&packed, &mut s, &mut e, &mut t);
    assert_eq!(s, r.stamps);
    assert_eq!(e, r.elems);
    assert_eq!(t, r.tops);
}

/// Um Kanban com `n` cartões NASCIDOS na sessão (cada campo com carimbo), um a cada três com um voto.
fn hosted(n: usize) -> (KanbanSet, HostRoom) {
    let mut set = base();
    let mut room = HostRoom::new(KanbanReplica::host(1, 11, &set, &[K]));
    let _ = room.replica.local(&mut set);
    let lists: Vec<_> = kanban(&set).lists.keys().copied().collect();
    let k = doc(&mut set);
    let ids = k.mint_n(n);
    let mut last: BTreeMap<_, _> = lists.iter().map(|l| (*l, k.pos_after_cards(*l))).collect();
    for (i, id) in ids.into_iter().enumerate() {
        let list = lists[i % lists.len()];
        let pos = last[&list].clone();
        last.insert(list, FracKey::between(Some(&pos), None));
        let mut c = Card::new(CardId(id), list, pos, "", Moment(0));
        c.title = format!("Cartão {i} — rever a cena e o texto");
        if i % 3 == 0 {
            c.votes.push(Member("ana".into()));
        }
        let _ = KanbanOp::put(Record::card(c)).apply(k);
    }
    let _ = room.replica.local(&mut set);
    (set, room)
}

/// ⭐ Quem entra por um retrato em blocos converge com o anfitrião, e uma escrita que chega A MEIO
/// dos blocos (o anfitrião mexeu depois do retrato) não se perde.
#[test]
fn a_guest_joining_by_a_snapshot_in_blocks_converges() {
    let (mut hs, mut host) = hosted(300);
    let snap = host.replica.snapshot(&hs);
    let parts = messages(snap, 4096);
    assert!(parts.len() > 3, "{} blocos", parts.len());
    assert!(parts.iter().all(|m| matches!(m, KMsg::SnapshotPart { .. })));
    let mut gs = KanbanSet::default();
    let mut guest = GuestRoom::new(2, 77);
    let half = parts.len() / 2;
    let mut parts = parts.into_iter();
    for m in parts.by_ref().take(half) {
        let _ = guest.frame(&mut gs, m, 0);
    }
    assert!(guest.replica.is_none(), "ainda não entrou");
    let id = *kanban(&hs).cards.keys().last().unwrap();
    edit_card(&mut hs, id, |c| c.title = "depois do retrato".into());
    let ws = host.replica.local(&mut hs);
    let _ = guest.frame(&mut gs, KMsg::Writes(ws), 0);
    let mut back = Vec::new();
    for m in parts {
        back.extend(guest.frame(&mut gs, m, 0).0);
    }
    assert!(guest.replica.is_some(), "entrou no último bloco");
    assert_eq!(kanban(&gs).cards[&id].title, "depois do retrato");
    for m in back {
        let _ = host.frame(&mut hs, 9, 2, m, 0);
    }
    assert_eq!(hs.to_bytes().unwrap(), gs.to_bytes().unwrap());
    // E dali em diante é uma sessão como as outras: o convidado cria e o anfitrião recebe.
    let k = doc(&mut gs);
    assert!(k.in_session());
    let list = *k.lists.keys().next().unwrap();
    let c = Card::new(
        CardId(k.mint()),
        list,
        k.pos_after_cards(list),
        "g",
        Moment(0),
    );
    let _ = KanbanOp::put(Record::card(c)).apply(k);
    for m in guest.tick(&mut gs, 0) {
        let _ = host.frame(&mut hs, 9, 2, m, 0);
    }
    assert_eq!(hs.to_bytes().unwrap(), gs.to_bytes().unwrap());
}

/// O retrato que cabe num bloco vai numa mensagem só (o caminho de sempre).
#[test]
fn a_small_snapshot_is_one_message() {
    let (hs, host) = hosted(20);
    let m = messages(host.replica.snapshot(&hs), SNAPSHOT_BLOCK);
    assert!(matches!(m.as_slice(), [KMsg::Snapshot(_)]));
}

/// A sonda da MEDICOES.md (W5 S2): `bash scripts/ph2d-run.sh cargo test -p ph2d-kanban-sync
/// --profile smoke --lib -- --ignored sonda_retrato --nocapture`. O MÍNIMO de 5.
#[test]
#[ignore = "sonda de medição"]
fn sonda_retrato() {
    #[derive(serde::Serialize)]
    struct Flat<'a> {
        kanbans: &'a [(ph2d_kanban_model::KanbanId, Vec<u8>)],
        stamps: Vec<(&'a crate::msg::FieldKey, &'a ph2d_board_sync::Lamport)>,
        elems: Vec<(&'a crate::msg::ElemKey, &'a crate::msg::Elem)>,
        tops: Vec<(
            &'a (ph2d_kanban_model::KanbanId, crate::Target),
            &'a ph2d_board_sync::Lamport,
        )>,
        clock: u64,
    }
    println!(
        "| cartões | Kanban (postcard) | carimbos antes | carimbos agora | retrato antes | retrato agora | blocos | anfitrião: retrato+bytes | convidado: juntar+entrar |"
    );
    for n in [100usize, 1_000, 5_000, 20_000] {
        let (hs, host) = hosted(n);
        let r = &host.replica;
        let (mut t_host, mut t_guest) = (f64::MAX, f64::MAX);
        let mut sizes = (0, 0, 0, 0, 0, 0);
        for _ in 0..5 {
            let t = Instant::now();
            let snap = r.snapshot(&hs);
            let ms = messages(snap.clone(), SNAPSHOT_BLOCK);
            let enc: Vec<Vec<u8>> = ms.iter().map(KMsg::encode).collect();
            t_host = t_host.min(t.elapsed().as_secs_f64() * 1e3);
            let flat = Flat {
                kanbans: &snap.kanbans,
                stamps: r.stamps.iter().collect(),
                elems: r.elems.iter().collect(),
                tops: r.tops.iter().collect(),
                clock: snap.clock,
            };
            let before = postcard::to_allocvec(&flat).unwrap();
            let doc_b: usize = snap.kanbans.iter().map(|(_, b)| b.len()).sum();
            let st_new = postcard::to_allocvec(&snap.stamps).unwrap().len();
            let st_old = before.len() - doc_b;
            let total: usize = enc.iter().map(Vec::len).sum();
            sizes = (doc_b, st_old, st_new, before.len(), total, enc.len());
            let t = Instant::now();
            let mut gs = KanbanSet::default();
            let mut g = GuestRoom::new(2, 5);
            for b in &enc {
                let _ = g.frame_bytes(&mut gs, b, 0);
            }
            t_guest = t_guest.min(t.elapsed().as_secs_f64() * 1e3);
            assert!(g.replica.is_some());
            assert_eq!(kanban(&gs).cards.len(), kanban(&hs).cards.len());
        }
        let kb = |b: usize| format!("{:.0} KB", b as f64 / 1024.0);
        println!(
            "| {n} | {} | {} | {} | {} | {} | {} | {t_host:.1} ms | {t_guest:.1} ms |",
            kb(sizes.0),
            kb(sizes.1),
            kb(sizes.2),
            kb(sizes.3),
            kb(sizes.4),
            sizes.5
        );
    }
}

/// J4 — o retrato fora da janela compensa? Na janela, (a) hoje: retrato + blocos + bytes; (b) só o
/// que um fio de fundo exigiria na janela: CLONAR o Kanban e os mapas de carimbos. E as partes que
/// iriam para o fio. 20 000 cartões, 7 blocos intercalados no MESMO processo, o MÍNIMO.
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-kanban-sync --profile smoke --lib -- --ignored
/// sonda_retrato_no_fio --nocapture`.
#[test]
#[ignore = "sonda de medição"]
fn sonda_retrato_no_fio() {
    let (hs, host) = hosted(20_000);
    let r = &host.replica;
    let mut best = [f64::MAX; 5];
    let mut sink = 0usize;
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1e3;
    for _ in 0..7 {
        let t = Instant::now();
        let enc: Vec<Vec<u8>> = messages(r.snapshot(&hs), SNAPSHOT_BLOCK)
            .iter()
            .map(KMsg::encode)
            .collect();
        best[0] = best[0].min(ms(t));
        sink += enc.len();
        let t = Instant::now();
        let k = kanban(&hs).clone();
        let maps = (r.stamps.clone(), r.elems.clone(), r.tops.clone());
        best[1] = best[1].min(ms(t));
        sink += k.cards.len() + maps.0.len();
        let t = Instant::now();
        let kb = postcard::to_allocvec(&k.without_personal()).unwrap();
        best[2] = best[2].min(ms(t));
        let t = Instant::now();
        let p = pack(&maps.0, &maps.1, &maps.2);
        best[3] = best[3].min(ms(t));
        let t = Instant::now();
        let k2 = kanban(&hs).clone();
        best[4] = best[4].min(ms(t));
        sink += kb.len() + postcard::to_allocvec(&p).unwrap().len() + k2.cards.len();
    }
    println!("| (a) hoje, tudo na janela | {:.1} ms |", best[0]);
    println!(
        "| (b) na janela: clonar Kanban + carimbos | {:.1} ms |",
        best[1]
    );
    println!("|   — só o Kanban | {:.1} ms |", best[4]);
    println!("| no fio: Kanban → postcard | {:.1} ms |", best[2]);
    println!("| no fio: carimbos → pack | {:.1} ms |", best[3]);
    assert!(sink > 0);
}
