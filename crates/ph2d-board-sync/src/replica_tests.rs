//! A fusão por registo contra os casos do oráculo (`docs/MiroClone/ferramentas/colab_oracle/`,
//! `08_ao_vivo.md` §1) e a convergência com uma rede que atrasa, reordena e duplica.

use super::*;
use ph2d_board_model::{
    BoardSet, History, MessageId, Pin, Rgba, RichText, Shape, ShapeType, Stamp, Style, ThreadId,
};

const RED: Rgba = Rgba([200, 0, 0, 255]);
const BLUE: Rgba = Rgba([0, 0, 200, 255]);

struct Peer {
    r: Replica,
    b: Board,
    h: History,
}

fn rect(b: &mut Board, x: f64, text: &str) -> ElementId {
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: RichText::plain(text),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [x, 0.0, 10.0, 10.0],
    );
    let id = el.id;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    id
}

/// Um anfitrião com um rectângulo `X` já no quadro e `n - 1` convidados que entraram.
fn room(n: usize) -> (Vec<Peer>, ElementId) {
    let mut set = BoardSet::default();
    let id = set.create("Quadro".into());
    let mut board = set.get_mut(id).unwrap().clone();
    let x = rect(&mut board, 0.0, "x");
    let r = Replica::host(1, &mut board, 100);
    let snap = r.snapshot(&board);
    let mut peers = vec![Peer {
        r,
        b: board,
        h: History::default(),
    }];
    for i in 1..n as u64 {
        let (r, b) = Replica::join(i + 1, &snap, 100 + i).unwrap();
        peers.push(Peer {
            r,
            b,
            h: History::default(),
        });
    }
    (peers, x)
}

fn edit(p: &mut Peer, id: ElementId, f: impl FnOnce(&mut Element)) {
    let mut el = p.b.doc.get(id).unwrap().clone();
    f(&mut el);
    p.h.apply(&mut p.b.doc, vec![BoardOp::Put(el)]);
}

/// O conteúdo partilhado, byte a byte: TODOS os elementos (lápides incluídas), o nome e o rascunho.
/// Fica de fora o que é de cada um: a câmara, e as caches do documento (`top_z` — a maior chave que
/// JÁ passou por este par, que viu chaves intermédias diferentes; `next_id`, o contador local).
fn state(p: &Peer) -> Vec<u8> {
    let mut els: Vec<&Element> = p.b.doc.live_with_hidden().collect();
    let ids: Vec<ElementId> = p.r.tops.keys().copied().collect();
    els.extend(
        ids.iter()
            .filter_map(|id| p.b.doc.get_any(*id))
            .filter(|e| e.deleted),
    );
    els.sort_by_key(|e| e.id);
    els.dedup_by_key(|e| e.id);
    postcard::to_allocvec(&(els, &p.b.name, p.b.sketch, &p.b.team)).unwrap()
}

/// Igual byte a byte; senão, o 1.º elemento que difere (e não os bytes todos).
fn same(a: &Peer, b: &Peer, ctx: &str) {
    if state(a) == state(b) {
        return;
    }
    let ids: std::collections::BTreeSet<ElementId> =
        a.b.doc
            .live_with_hidden()
            .chain(b.b.doc.live_with_hidden())
            .map(|e| e.id)
            .collect();
    for id in ids.iter().copied().chain(a.r.stamps.keys().map(|k| k.0)) {
        let (x, y) = (a.b.doc.get_any(id), b.b.doc.get_any(id));
        if x != y {
            let what = match (x, y) {
                (Some(x), Some(y)) => format!(
                    "registos {:?}; org {:?} × {:?}; version/nonce {:?} × {:?}; carimbos {:?} × {:?}",
                    x.changed_fields(y),
                    x.org,
                    y.org,
                    (x.version, x.nonce),
                    (y.version, y.nonce),
                    x.stamps(),
                    y.stamps()
                ),
                _ => format!("existe {} × {}", x.is_some(), y.is_some()),
            };
            panic!("{ctx}: o elemento {id:?} difere: {what}");
        }
    }
    let (ta, tb) = (&a.b.team, &b.b.team);
    let brief = |t: &ph2d_board_model::Team| {
        format!(
            "fios {:?}; versões {:?}; slides {:?}",
            t.threads
                .iter()
                .map(|t| (
                    t.id.0 % 1000,
                    t.resolved,
                    t.messages
                        .iter()
                        .map(|m| (m.id.0 % 1000, m.text.clone(), m.at))
                        .collect::<Vec<_>>(),
                    t.read_by.clone()
                ))
                .collect::<Vec<_>>(),
            t.versions
                .iter()
                .map(|v| (v.id.0 % 1000, v.name.clone(), v.at))
                .collect::<Vec<_>>(),
            t.hidden_slides
        )
    };
    if ta != tb {
        let ids: std::collections::BTreeSet<_> =
            ta.threads.iter().chain(&tb.threads).map(|t| t.id).collect();
        for id in ids {
            let (x, y) = (ta.thread(id), tb.thread(id));
            if x != y {
                panic!(
                    "{ctx}: o fio {} difere (apagado/pendentes A {:?} B {:?}):\nA {x:?}\nB {y:?}",
                    id.0 % 1000,
                    a.r.team.probe(id),
                    b.r.team.probe(id)
                );
            }
        }
        panic!("{ctx}: a equipa difere:\nA {}\nB {}", brief(ta), brief(tb));
    }
    panic!(
        "{ctx}: difere fora dos elementos ({:?}/{} × {:?}/{})",
        a.b.name, a.b.sketch, b.b.name, b.b.sketch
    );
}

/// A e B mudam ao mesmo tempo (cada um ainda não viu o do outro), e cada um recebe o do outro.
fn concurrently(a: &mut Peer, b: &mut Peer) {
    let wa = a.r.local(&mut a.b);
    let wb = b.r.local(&mut b.b);
    b.r.receive(&mut b.b, &wa);
    a.r.receive(&mut a.b, &wb);
    assert!(
        a.r.local(&mut a.b).is_empty() && b.r.local(&mut b.b).is_empty(),
        "eco"
    );
    same(a, b, "divergiram");
}

fn two() -> (Peer, Peer, ElementId) {
    let (mut v, x) = room(2);
    let b = v.pop().unwrap();
    let a = v.pop().unwrap();
    (a, b, x)
}

/// c02 — A move, B pinta: fica movido E pintado (o Excalidraw perde a cor inteira).
#[test]
fn c02_a_move_and_a_colour_at_once_both_stay() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| el.x = 70.0);
    edit(&mut b, x, |el| el.style_mut().fill = Some(RED));
    concurrently(&mut a, &mut b);
    let el = a.b.doc.get(x).unwrap();
    assert_eq!((el.x, el.style().fill), (70.0, Some(RED)));
}

/// c03 — A move, B reescreve o texto: os dois ficam.
#[test]
fn c03_a_move_and_a_new_text_at_once_both_stay() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| el.x = 70.0);
    edit(&mut b, x, |el| {
        el.shape_mut().unwrap().text = RichText::plain("novo")
    });
    concurrently(&mut a, &mut b);
    let el = b.b.doc.get(x).unwrap();
    assert_eq!((el.x, el.shape().unwrap().text.as_str()), (70.0, "novo"));
}

/// c01/c09 — o MESMO registo dos dois lados: ganha um inteiro (o carimbo maior), igual nos dois.
#[test]
fn c01_c09_the_same_register_at_once_keeps_one_whole() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| {
        el.shape_mut().unwrap().text = RichText::plain("xaaa")
    });
    edit(&mut b, x, |el| {
        el.shape_mut().unwrap().text = RichText::plain("xbbb")
    });
    concurrently(&mut a, &mut b);
    let t =
        a.b.doc
            .get(x)
            .unwrap()
            .shape()
            .unwrap()
            .text
            .as_str()
            .to_owned();
    assert!(t == "xaaa" || t == "xbbb", "{t}");
}

/// c04 — A apaga, B move: fica apagado (o Excalidraw ressuscita).
#[test]
fn c04_a_delete_and_a_move_at_once_stays_deleted() {
    let (mut a, mut b, x) = two();
    a.h.apply(&mut a.b.doc, vec![BoardOp::Delete(x)]);
    edit(&mut b, x, |el| el.x = 70.0);
    concurrently(&mut a, &mut b);
    assert!(a.b.doc.get(x).is_none() && b.b.doc.get(x).is_none());
}

/// c05 — os dois criam no topo ao mesmo tempo: as duas formas ficam, por uma ordem igual nos dois.
#[test]
fn c05_two_new_shapes_at_once_get_one_order() {
    let (mut a, mut b, _) = two();
    let ra = rect(&mut a.b, 5.0, "a");
    let rb = rect(&mut b.b, 5.0, "b");
    concurrently(&mut a, &mut b);
    let za = a.b.doc.get(ra).unwrap().z.clone();
    let zb = a.b.doc.get(rb).unwrap().z.clone();
    assert_ne!(za, zb);
}

/// c11/c12 ao vivo — o desfazer de A depois de B pintar / apagar: volta o sítio, fica a cor de B; o
/// apagado por B não ressuscita. E chega a B igual.
#[test]
fn c11_c12_undo_after_the_other_keeps_the_others_work() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| el.x = 70.0);
    concurrently(&mut a, &mut b);
    edit(&mut b, x, |el| el.style_mut().fill = Some(BLUE));
    concurrently(&mut a, &mut b);
    assert!(a.h.undo(&mut a.b.doc));
    concurrently(&mut a, &mut b);
    let el = b.b.doc.get(x).unwrap();
    assert_eq!((el.x, el.style().fill), (0.0, Some(BLUE)));
    assert!(a.h.redo(&mut a.b.doc));
    concurrently(&mut a, &mut b);
    b.h.apply(&mut b.b.doc, vec![BoardOp::Delete(x)]);
    concurrently(&mut a, &mut b);
    a.h.undo(&mut a.b.doc);
    concurrently(&mut a, &mut b);
    assert!(a.b.doc.get(x).is_none(), "ressuscitou");
}

/// c16/c17 — a mesma escrita duas vezes, e duas escritas pela ordem inversa: o resultado é o da
/// última de A.
#[test]
fn c16_c17_duplicated_and_reordered_writes_change_nothing() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| el.x = 10.0);
    let w1 = a.r.local(&mut a.b);
    edit(&mut a, x, |el| el.x = 20.0);
    let w2 = a.r.local(&mut a.b);
    b.r.receive(&mut b.b, &w2);
    b.r.receive(&mut b.b, &w1);
    b.r.receive(&mut b.b, &w2);
    assert_eq!(b.b.doc.get(x).unwrap().x, 20.0);
    same(&a, &b, "divergiram");
}

/// Dois carimbos ao mesmo tempo SOMAM, e a lista fica pela mesma ordem nos dois (a de nascença).
#[test]
fn two_stamps_at_once_add_up_in_one_order() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| {
        el.add_stamp(Stamp::Heart, 1);
    });
    edit(&mut b, x, |el| {
        el.add_stamp(Stamp::Star, 1);
        el.add_stamp(Stamp::Heart, 1);
    });
    concurrently(&mut a, &mut b);
    let s = a.b.doc.get(x).unwrap().stamps().to_vec();
    let heart = s.iter().find(|c| c.stamp == Stamp::Heart).unwrap().count;
    assert_eq!((s.len(), heart), (2, 2));
}

/// c14 — B cai, os dois mudam, B volta: nada se perde (o Excalidraw perde a cor de B).
#[test]
fn c14_after_a_drop_nothing_is_lost() {
    let (mut a, mut b, x) = two();
    edit(&mut a, x, |el| el.x = 70.0);
    let w = rect(&mut a.b, 30.0, "w");
    let _lost_on_the_wire = a.r.local(&mut a.b);
    edit(&mut b, x, |el| el.style_mut().fill = Some(RED));
    let v = rect(&mut b.b, 40.0, "v");
    let _never_sent = b.r.local(&mut b.b);
    let snap = a.r.snapshot(&a.b);
    b.r.rejoin(&mut b.b, &snap, 7).unwrap();
    let resend = b.r.local(&mut b.b);
    a.r.receive(&mut a.b, &resend);
    same(&a, &b, "divergiram");
    let el = a.b.doc.get(x).unwrap();
    assert_eq!((el.x, el.style().fill), (70.0, Some(RED)));
    assert!(a.b.doc.get(w).is_some() && a.b.doc.get(v).is_some());
}

/// O nome e o rascunho do quadro também são registos.
#[test]
fn the_board_name_and_sketch_travel() {
    let (mut a, mut b, _) = two();
    a.b.name = "Retro".into();
    b.b.sketch = true;
    concurrently(&mut a, &mut b);
    assert_eq!((b.b.name.as_str(), a.b.sketch), ("Retro", true));
}

/// Gerador determinista (xorshift).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn random_edit(p: &mut Peer, rng: &mut Rng) {
    let ids: Vec<ElementId> = p.b.doc.live().map(|e| e.id).collect();
    let pick = (!ids.is_empty()).then(|| ids[rng.below(ids.len())]);
    if rng.below(4) == 0 {
        team_edit(p, rng);
        return;
    }
    match (rng.below(9), pick) {
        (0, _) | (_, None) => {
            rect(&mut p.b, rng.below(500) as f64, "n");
        }
        (1, Some(id)) => edit(p, id, |el| el.x = rng.below(500) as f64),
        (2, Some(id)) => edit(p, id, |el| {
            el.style_mut().fill = Some(Rgba([rng.below(255) as u8, 0, 0, 255]));
        }),
        (3, Some(id)) => edit(p, id, |el| {
            if let Some(s) = el.shape_mut() {
                s.text = RichText::plain(&format!("t{}", rng.below(99)));
            }
        }),
        (4, Some(id)) => p.h.apply(&mut p.b.doc, vec![BoardOp::Delete(id)]),
        (5, Some(id)) => edit(p, id, |el| {
            el.add_stamp(
                Stamp::ALL[rng.below(3)],
                if rng.below(4) == 0 { -1 } else { 1 },
            );
        }),
        (6, Some(id)) => {
            let z = p.b.doc.z_on_top();
            edit(p, id, |el| el.z = z);
        }
        (7, _) => {
            p.h.undo(&mut p.b.doc);
        }
        _ => {
            if rng.below(2) == 0 {
                p.b.name = format!("q{}", rng.below(9));
            } else {
                p.b.sketch = !p.b.sketch;
            }
        }
    }
}

/// ⭐⭐ Convergência: N pares, edições sorteadas (com desfazer), uma rede que ATRASA, REORDENA e
/// DUPLICA. No fim, todos com o MESMO quadro, byte a byte no postcard. Várias sementes.
#[test]
fn every_peer_ends_with_the_same_board_whatever_the_network_does() {
    for seed in 1..=40u64 {
        let n = 2 + (seed % 3) as usize;
        let (mut peers, _) = room(n);
        let mut rng = Rng(seed * 0x9e37_79b9 + 1);
        let mut wire: Vec<(usize, Vec<Write>)> = Vec::new();
        for _ in 0..300 {
            if rng.below(3) == 0 && !wire.is_empty() {
                let k = rng.below(wire.len());
                let (to, ws) = if rng.below(6) == 0 {
                    wire[k].clone()
                } else {
                    wire.swap_remove(k)
                };
                let p = &mut peers[to];
                p.r.receive(&mut p.b, &ws);
            } else {
                let from = rng.below(n);
                random_edit(&mut peers[from], &mut rng);
                let p = &mut peers[from];
                let ws = p.r.local(&mut p.b);
                if !ws.is_empty() {
                    for to in (0..n).filter(|t| *t != from) {
                        wire.push((to, ws.clone()));
                    }
                }
            }
        }
        while !wire.is_empty() {
            let (to, ws) = wire.swap_remove(rng.below(wire.len()));
            let p = &mut peers[to];
            p.r.receive(&mut p.b, &ws);
            let from = to;
            let p = &mut peers[from];
            let echo = p.r.local(&mut p.b);
            for t in (0..n).filter(|t| *t != from) {
                if !echo.is_empty() {
                    wire.push((t, echo.clone()));
                }
            }
        }
        for (i, p) in peers.iter().enumerate() {
            assert_eq!(p.r.pending(), 0, "semente {seed}: par {i} com pendentes");
            same(&peers[0], p, &format!("semente {seed}: o par {i}"));
        }
    }
}

const WHO: [&str; 3] = ["ana", "bia", "caio"];

/// Uma mudança sorteada na equipa: abrir, responder, ler, editar, apagar, resolver, versão, slide.
fn team_edit(p: &mut Peer, rng: &mut Rng) {
    let me = WHO[rng.below(3)];
    let at = 1_000 + rng.below(50) as i64;
    let threads: Vec<ThreadId> = p.b.team.threads.iter().map(|t| t.id).collect();
    let t = (!threads.is_empty()).then(|| threads[rng.below(threads.len())]);
    match (rng.below(8), t) {
        (0, _) | (_, None) => {
            p.b.team.open(Pin::Point([1.0, 2.0]), me, "oi", at);
        }
        (1, Some(t)) => {
            p.b.team.reply(t, me, &format!("r{}", rng.below(9)), at);
        }
        (2, Some(t)) => p.b.team.mark_read(t, me),
        (3, Some(t)) => {
            let msgs: Vec<(MessageId, String)> =
                p.b.team
                    .thread(t)
                    .unwrap()
                    .messages
                    .iter()
                    .map(|m| (m.id, m.author.clone()))
                    .collect();
            let (m, author) = msgs[rng.below(msgs.len())].clone();
            if rng.below(2) == 0 {
                p.b.team.edit(t, m, &author, &format!("e{}", rng.below(9)));
            } else {
                p.b.team.delete(t, m, &author);
            }
        }
        (4, Some(t)) => {
            p.b.team.set_resolved(t, rng.below(2) == 0);
        }
        (5, _) => p.b.save_version(format!("v{}", rng.below(9)), at),
        (6, _) => {
            if let Some(v) = p.b.team.versions.first_mut() {
                v.name = format!("n{}", rng.below(9));
            }
        }
        _ => {
            let f = ElementId(rng.below(3) as u64 + 1);
            p.b.team.set_slide_hidden(f, rng.below(2) == 0);
        }
    }
}

/// Duas respostas ao mesmo fio ao mesmo tempo ficam AS DUAS, pela mesma ordem nos dois.
#[test]
fn two_replies_at_once_both_stay_in_one_order() {
    let (mut a, mut b, _) = two();
    let t =
        a.b.team
            .open(Pin::Point([0.0, 0.0]), "ana", "pergunta", 10)
            .unwrap();
    concurrently(&mut a, &mut b);
    a.b.team.reply(t, "ana", "eu", 20);
    b.b.team.reply(t, "bia", "eu também", 20);
    concurrently(&mut a, &mut b);
    let texts: Vec<String> =
        b.b.team
            .thread(t)
            .unwrap()
            .messages
            .iter()
            .map(|m| m.text.clone())
            .collect();
    assert_eq!(texts.len(), 3, "{texts:?}");
}

/// Apagar o fio (a 1.ª mensagem) enquanto outro responde: o fio fica apagado nos dois.
#[test]
fn a_thread_deleted_while_another_replies_stays_deleted() {
    let (mut a, mut b, _) = two();
    let t =
        a.b.team
            .open(Pin::Point([0.0, 0.0]), "ana", "x", 10)
            .unwrap();
    concurrently(&mut a, &mut b);
    let first = a.b.team.thread(t).unwrap().messages[0].id;
    assert!(a.b.team.delete(t, first, "ana"));
    b.b.team.reply(t, "bia", "y", 11);
    concurrently(&mut a, &mut b);
    assert!(a.b.team.thread(t).is_none() && b.b.team.thread(t).is_none());
}

/// Duas pessoas a ler ao mesmo tempo ficam as duas como «leu»; uma resposta nova volta a pôr por ler
/// quem não a escreveu.
#[test]
fn two_readers_at_once_both_count_and_a_reply_resets_the_others() {
    let (mut a, mut b, _) = two();
    let t =
        a.b.team
            .open(Pin::Point([0.0, 0.0]), "caio", "x", 10)
            .unwrap();
    concurrently(&mut a, &mut b);
    a.b.team.mark_read(t, "ana");
    b.b.team.mark_read(t, "bia");
    concurrently(&mut a, &mut b);
    let read = |p: &Peer| p.b.team.thread(t).unwrap().read_by.clone();
    assert_eq!(read(&a), vec!["ana", "bia", "caio"]);
    b.b.team.reply(t, "bia", "y", 11);
    concurrently(&mut a, &mut b);
    assert_eq!(read(&a), vec!["bia"]);
}

/// Uma versão guardada por A chega a B; renomear e apagar ao mesmo tempo: fica apagada.
#[test]
fn a_saved_version_travels_and_a_delete_beats_a_rename() {
    let (mut a, mut b, _) = two();
    a.b.save_version("antes".into(), 5);
    concurrently(&mut a, &mut b);
    assert_eq!(b.b.team.versions.len(), 1);
    a.b.team.versions[0].name = "outro".into();
    b.b.team.versions.clear();
    concurrently(&mut a, &mut b);
    assert!(a.b.team.versions.is_empty());
}

/// O retrato compacta os carimbos por elemento e devolve-os IGUAIS a quem entra; e mede quanto pesa
/// por elemento um quadro feito ao vivo (impresso: 193 B, 129 do quadro e 63 dos carimbos, 10/10;
/// sem compactar eram 603).
#[test]
fn the_snapshot_packs_the_stamps_and_gives_them_back_equal() {
    let (mut a, _, x) = two();
    let n = 1000;
    for i in 0..n {
        rect(&mut a.b, i as f64, "texto de uma nota");
    }
    edit(&mut a, x, |el| el.x = 9.0);
    let _ = a.r.local(&mut a.b);
    let snap = a.r.snapshot(&a.b);
    let (joined, _) = Replica::join(9, &snap, 9).unwrap();
    assert_eq!(joined.stamps, a.r.stamps);
    let bytes = postcard::to_allocvec(&snap).unwrap().len();
    let board = snap.board.len();
    eprintln!(
        "retrato: {} B/elemento ({} do quadro, {} dos carimbos)",
        bytes / n,
        board / n,
        (bytes - board) / n
    );
}
