//! A sala em estrela: a aba partilhada SOZINHA, a sala do Quadro que leva os Kanbans, a presença.

use ph2d_board_sync::{ConnId, PROTOCOL, To};
use ph2d_kanban_model::{KanbanSet, Member};

use super::*;
use crate::test_support::{K, Rng, base, edit_card, kanban, random_op};
use crate::{KMsg, KanbanPresence};

/// A sala de mentira: o anfitrião e os convidados (ligação `i + 1`), e o que ia no fio.
struct Star {
    host: KanbanHost,
    hset: KanbanSet,
    guests: Vec<(KanbanGuest, KanbanSet)>,
    to_host: Vec<(ConnId, Vec<u8>)>,
    to_guest: Vec<Vec<Vec<u8>>>,
}

impl Star {
    fn new(n: usize) -> Self {
        let hset = base();
        let host = KanbanHost::open(1, "Ana".into(), 9, &hset, &[K]);
        let mut s = Self {
            host,
            hset,
            guests: Vec::new(),
            to_host: Vec::new(),
            to_guest: Vec::new(),
        };
        for i in 0..n {
            let g = KanbanGuest::new(10 + i as u64, format!("g{i}"), 50 + i as u64, 77 + i as u64);
            s.to_host.push((i as u64 + 1, g.hello().encode()));
            s.guests.push((g, KanbanSet::default()));
            s.to_guest.push(Vec::new());
        }
        s
    }

    fn route(&mut self, out: Vec<(To, SoloMsg)>) {
        for (to, m) in out {
            for i in 0..self.guests.len() {
                let conn = i as u64 + 1;
                let send = match to {
                    To::All => true,
                    To::One(c) => c == conn,
                    To::AllBut(c) => c != conn,
                };
                if send {
                    self.to_guest[i].push(m.encode());
                }
            }
        }
    }

    fn pump(&mut self, now: u64) -> Vec<KEvent> {
        let mut events = Vec::new();
        for _ in 0..20 {
            let mut moved = false;
            for (conn, b) in std::mem::take(&mut self.to_host) {
                moved = true;
                let (out, ev) = self.host.frame(&mut self.hset, conn, &b, now);
                events.extend(ev);
                self.route(out);
            }
            let out = self.host.tick(&mut self.hset, now);
            self.route(out);
            for i in 0..self.guests.len() {
                let (g, set) = &mut self.guests[i];
                for b in std::mem::take(&mut self.to_guest[i]) {
                    moved = true;
                    let (out, ev) = g.frame(set, &b, now);
                    events.extend(ev);
                    self.to_host
                        .extend(out.into_iter().map(|m| (i as u64 + 1, m.encode())));
                }
                let out = g.tick(set, now);
                self.to_host
                    .extend(out.into_iter().map(|m| (i as u64 + 1, m.encode())));
            }
            if !moved && self.to_host.is_empty() {
                break;
            }
        }
        events
    }
}

/// ⭐ Uma aba de Kanban partilhada SOZINHA: dois convidados e o anfitrião a mexer ao mesmo tempo
/// acabam com os mesmos bytes.
#[test]
fn a_kanban_shared_alone_converges_through_the_star() {
    let mut s = Star::new(2);
    let ev = s.pump(0);
    assert!(ev.contains(&KEvent::Arrived(vec![K])), "{ev:?}");
    assert_eq!(s.host.peers().len(), 3);
    let mut rng = Rng(42);
    for round in 0..10 {
        for _ in 0..3 {
            let r = &mut s.host.room.replica;
            random_op(&mut s.hset, r, 1, &mut rng);
            for (i, (g, set)) in s.guests.iter_mut().enumerate() {
                let r = g.room.replica.as_mut().unwrap();
                random_op(set, r, 10 + i as u64, &mut rng);
            }
        }
        s.pump(round * 16);
    }
    s.pump(1_000);
    let want = s.hset.to_bytes().unwrap();
    for (g, set) in &s.guests {
        assert_eq!(set.to_bytes().unwrap(), want, "{:?}", g.me);
    }
}

#[test]
fn another_protocol_and_an_impostor_are_refused() {
    let mut s = Star::new(1);
    s.pump(0);
    let set = &mut s.hset;
    let old = SoloMsg::Hello {
        protocol: PROTOCOL - 1,
        peer: 99,
        name: "x".into(),
        key: 1,
    };
    let (out, _) = s.host.frame(set, 9, &old.encode(), 0);
    assert_eq!(
        out,
        vec![(To::One(9), SoloMsg::Refused { protocol: PROTOCOL })]
    );
    let thief = SoloMsg::Hello {
        protocol: PROTOCOL,
        peer: 10,
        name: "g0".into(),
        key: 12345,
    };
    let (out, _) = s.host.frame(set, 9, &thief.encode(), 0);
    assert_eq!(
        out,
        vec![(To::One(9), SoloMsg::Refused { protocol: PROTOCOL })]
    );
}

/// Quem vê o Kanban e que cartão tem aberto (os avatares), o cursor, e quem sai some.
#[test]
fn presence_says_who_sees_the_kanban_and_which_card() {
    let mut s = Star::new(2);
    s.pump(0);
    let card = *kanban(&s.hset).cards.keys().next().unwrap();
    let p = KanbanPresence {
        peer: 0,
        kanban: Some(K),
        card: Some(card),
        cursor: Some([10.0, 20.0]),
    };
    let (g, _) = &mut s.guests[0];
    let out = g.presence(5, p.clone());
    s.to_host.extend(out.into_iter().map(|m| (1, m.encode())));
    let out = s.host.presence(5, KanbanPresence { card: None, ..p });
    s.route(out);
    s.pump(5);
    let v = s.guests[1].0.viewers();
    assert_eq!(v.on_kanban(K), vec![1, 10]);
    assert_eq!(v.on_card(K, card), vec![10]);
    assert_eq!(v.cursor(10, 500), Some([10.0, 20.0]));
    let out = s.host.left(1);
    s.route(out);
    s.pump(6);
    assert_eq!(s.guests[1].0.viewers().on_kanban(K), vec![1]);
    assert_eq!(s.host.viewers().on_card(K, card), Vec::<u64>::new());
}

/// O cursor sai no máximo a ~30 Hz; mudar de cartão sai já.
#[test]
fn the_cursor_is_paced_and_a_card_change_is_not() {
    let mut s = Star::new(1);
    s.pump(0);
    let card = *kanban(&s.hset).cards.keys().next().unwrap();
    let mut p = KanbanPresence {
        peer: 0,
        kanban: Some(K),
        card: None,
        cursor: Some([0.0, 0.0]),
    };
    let (g, _) = &mut s.guests[0];
    assert_eq!(g.presence(100, p.clone()).len(), 1);
    p.cursor = Some([1.0, 0.0]);
    assert!(g.presence(110, p.clone()).is_empty(), "antes dos 33 ms");
    p.card = Some(card);
    assert_eq!(
        g.presence(111, p.clone()).len(),
        1,
        "abrir um cartão sai já"
    );
}

/// ⭐ A sala de um QUADRO leva os Kanbans que ele referencia: os bytes do Kanban vão em
/// `Msg::Kanban`, e o convidado pede-os depois do `Welcome`.
#[test]
fn the_board_session_carries_its_kanbans() {
    use ph2d_board_model::BoardSet;
    use ph2d_board_sync::{Guest, Host, Msg};
    let mut bs = BoardSet::default();
    let bid = bs.create("Quadro".into());
    let mut hb = bs.get_mut(bid).unwrap().clone();
    let mut bhost = Host::open(1, "Ana".into(), &mut hb, 3);
    let mut hset = base();
    let mut hroom = crate::HostRoom::new(crate::KanbanReplica::host(1, 9, &hset, &[K]));
    let mut bguest = Guest::new(2, "Bia".into(), 4, 5);
    let mut gset = KanbanSet::default();
    let mut groom = crate::GuestRoom::new(2, 6);
    let (out, _) = bhost.frame(&mut hb, 7, &bguest.hello().encode(), 0);
    let mut gb = None;
    for (_, m) in out {
        let (_, ev) = bguest.frame(gb.as_mut(), &m.encode(), 0);
        for e in ev {
            if let ph2d_board_sync::Event::Arrived(b) = e {
                gb = Some(*b);
            }
        }
    }
    // Entrou no Quadro: pede os Kanbans dele.
    let want = Guest::kanban(groom.want().encode()).encode();
    let _ = bhost.frame(&mut hb, 7, &want, 1);
    let id = *kanban(&hset).cards.keys().next().unwrap();
    edit_card(&mut hset, id, |c| c.members.push(Member("Ana".into())));
    for (conn, from, b) in bhost.take_kanban() {
        let (out, _) = hroom.frame_bytes(&mut hset, conn, from, &b, 1);
        for (to, m) in out {
            if !matches!(to, To::One(7) | To::All) {
                continue;
            }
            let bytes = Msg::Kanban(m.encode()).encode();
            let _ = bguest.frame(gb.as_mut(), &bytes, 1);
        }
    }
    let mut events = Vec::new();
    for b in bguest.take_kanban() {
        let (out, ev) = groom.frame_bytes(&mut gset, &b, 1);
        assert!(
            out.iter()
                .all(|m| matches!(m, KMsg::Writes(_) | KMsg::Presence(_)))
        );
        events.extend(ev);
    }
    assert_eq!(events, vec![KEvent::Arrived(vec![K])]);
    assert_eq!(gset.to_bytes().unwrap(), hset.to_bytes().unwrap());
    assert_eq!(kanban(&gset).cards[&id].members, vec![Member("Ana".into())]);
}
