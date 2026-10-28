//! ⭐ J4 — a sala do QUADRO leva os Kanbans dele, pela costura do hero: dois ecrãs (cada um na sua
//! thread), a rede local de verdade em 127.0.0.1. A Bia entra no quadro da Ana e recebe o Kanban
//! (no id DELA, e a referência do elemento aponta-lhe); mover um cartão num aparece no outro; um
//! Kanban posto a meio da sessão também chega.

use super::*;
use std::sync::mpsc;

use ph2d_a11y::NodeId;
use ph2d_board_model::{BoardKanban, BoardOp, Element, Rgba, Style};
use ph2d_editor_core::documents::live::LiveRequest;
use ph2d_kanban_model::{CardId, KanbanId, KanbanOp, ListId, Record};

const SECS: u64 = 20;

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

fn until(h: &mut HeroScreen, mut done: impl FnMut(&mut HeroScreen) -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(SECS);
    while Instant::now() < end {
        service(h);
        if done(h) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

/// Um Kanban novo no projecto e o seu elemento no quadro `board`.
fn place(h: &mut HeroScreen, board: BoardId, x: f64) -> KanbanId {
    let shared = h.documents.kanbans_shared();
    let (k, _) =
        ph2d_kanban_scenes::short_film_into(&mut shared.borrow_mut(), ph2d_kanban_model::Moment(0));
    let b = h.documents.board_mut(board).unwrap();
    let bk = BoardKanban::new(k, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_kanban(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        bk,
        [x, 0.0, 960.0, 540.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    k
}

/// Os Kanbans que o quadro da sala mostra: (id local, nome, nº de cartões).
fn on_board(h: &HeroScreen) -> Vec<(KanbanId, String, usize)> {
    let Some(b) = h
        .documents
        .live_view()
        .board
        .and_then(|id| h.documents.boards().get(id))
    else {
        return Vec::new();
    };
    let set = h.documents.kanbans();
    b.doc
        .live_with_hidden()
        .filter_map(|e| e.kanban().map(|k| k.kanban))
        .filter_map(|k| {
            set.get(k)
                .map(|x| (k, x.header.name.clone(), x.cards.len()))
        })
        .collect()
}

fn list_of(h: &HeroScreen, k: KanbanId, title: &str) -> Option<String> {
    let set = h.documents.kanbans();
    let kb = set.get(k)?;
    let c = kb.cards.values().find(|c| c.title == title)?;
    kb.lists.get(&c.list).map(|l| l.title.clone())
}

/// Move o cartão `title` para a lista `to` (a porta do modelo, como o arrastar).
fn move_card(h: &mut HeroScreen, k: KanbanId, title: &str, to: &str) {
    let shared = h.documents.kanbans_shared();
    let mut set = shared.borrow_mut();
    let kb = set.doc_mut(k).unwrap();
    let list: ListId = kb.lists.values().find(|l| l.title == to).unwrap().id;
    let mut c = kb
        .cards
        .values()
        .find(|c| c.title == title)
        .unwrap()
        .clone();
    c.list = list;
    let _ = KanbanOp::put(Record::card(c)).apply(kb);
}

fn first_card(h: &HeroScreen, k: KanbanId) -> (CardId, String, String) {
    let set = h.documents.kanbans();
    let kb = set.get(k).unwrap();
    let c = kb.cards.values().next().unwrap();
    (c.id, c.title.clone(), kb.lists[&c.list].title.clone())
}

#[test]
fn j4_the_board_session_carries_its_kanbans_and_a_moved_card_arrives_inside_the_board() {
    let mut ana = hero("Ana");
    let id = ana.documents.create("Retro".into());
    ana.documents.activate(Some(id));
    // A Ana já tem outros Kanbans no projecto: os ids dela e os da Bia NÃO coincidem.
    let _ = ana.documents.kanbans_shared().borrow_mut().create("Outro");
    let k = place(&mut ana, id, 0.0);
    let (_, title, from) = first_card(&ana, k);
    let to = if from == "Doing" { "Done" } else { "Doing" };
    ana.documents.ask_live(LiveRequest::Host);
    service(&mut ana);
    let LiveRole::Hosting { code, addrs } = ana.documents.live_view().role.clone() else {
        panic!("não abriu");
    };
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let (tx, rx) = mpsc::channel::<&'static str>();
    let (back_tx, back_rx) = mpsc::channel::<&'static str>();
    let t2 = title.clone();
    let from2 = from.clone();
    let kname = ana.documents.kanbans().get(k).unwrap().header.name.clone();
    let bia = std::thread::spawn(move || {
        let mut g = hero("Bia");
        // Dois Kanbans da Bia: o 2 dela é OUTRO que o 2 da Ana (sem a tradução, a referência caía nele).
        let mine = g.documents.kanbans_shared().borrow_mut().create("Meu");
        let _ = g.documents.kanbans_shared().borrow_mut().create("Meu 2");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code,
        });
        let mut got = None;
        assert!(
            until(&mut g, |g| {
                got = on_board(g).into_iter().find(|(_, _, n)| *n > 3);
                got.is_some()
            }),
            "o Kanban do quadro não chegou à Bia"
        );
        let (kb, name, _) = got.unwrap();
        assert_ne!(
            kb, mine,
            "a referência aponta ao Kanban recebido, não ao da Bia"
        );
        assert_eq!(name, kname);
        let _ = tx.send("recebi");
        assert!(
            until(&mut g, |g| list_of(g, kb, &t2).as_deref() == Some(to)),
            "o cartão movido pela Ana não chegou"
        );
        // A Bia devolve-o à lista de partida.
        move_card(&mut g, kb, &t2, &from2);
        let _ = tx.send("movi");
        // Um Kanban que a Ana põe a meio da sessão chega também.
        assert!(
            until(&mut g, |g| {
                let mut ids: Vec<KanbanId> = on_board(g)
                    .into_iter()
                    .filter(|(_, _, n)| *n > 3)
                    .map(|(k, _, _)| k)
                    .collect();
                ids.dedup();
                ids.len() == 2
            }),
            "o Kanban posto a meio não chegou"
        );
        let _ = back_rx.recv_timeout(Duration::from_secs(SECS));
        g.documents.ask_live(LiveRequest::Leave);
        service(&mut g);
        assert!(g.documents.kanbans().get(kb).is_some(), "a cópia fica");
    });
    let go = |rx: &mpsc::Receiver<&'static str>, ana: &mut HeroScreen| {
        let end = Instant::now() + Duration::from_secs(SECS);
        loop {
            if let Ok(m) = rx.try_recv() {
                return m;
            }
            assert!(Instant::now() < end, "a Bia não respondeu");
            service(ana);
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    assert_eq!(go(&rx, &mut ana), "recebi");
    move_card(&mut ana, k, &title, to);
    assert_eq!(go(&rx, &mut ana), "movi");
    assert!(
        until(&mut ana, |a| list_of(a, k, &title).as_deref()
            == Some(from.as_str())),
        "o cartão que a Bia moveu não voltou à Ana"
    );
    place(&mut ana, id, 1200.0);
    let _ = back_tx.send("ok");
    let end = Instant::now() + Duration::from_secs(SECS);
    while !bia.is_finished() && Instant::now() < end {
        service(&mut ana);
        std::thread::sleep(Duration::from_millis(5));
    }
    bia.join().expect("a Bia falhou");
    ana.documents.ask_live(LiveRequest::Leave);
    service(&mut ana);
    assert!(
        !ana.documents.kanbans().get(k).unwrap().in_session(),
        "a sala acabou: o Kanban volta ao contador local"
    );
}

/// J4 — cada «Join» reconhece o convite da OUTRA sala: diz o que é e leva-o ao sítio certo pela
/// porta das duas famílias (`ph2d_kanban_sync::port`): o do Kanban vai à família Kanban (que abre a
/// sala dela — `j7_porta` no `ph2d-app-kanban`); o do Quadro, colado no Kanban, entra no Quadro.
#[test]
fn j4_a_link_of_the_other_kind_is_told_and_routed_to_its_place() {
    reset_for_tests();
    internet_local_for_tests();
    ph2d_kanban_sync::port::reset_for_tests();
    let invite = net::Invite {
        key: net::InviteKey::random(),
        addrs: vec![],
    };
    let mut h = hero("Ana");
    let link = invite.link_as(net::InviteKind::Kanban);
    h.documents
        .ask_live(LiveRequest::JoinLink { link: link.clone() });
    service(&mut h);
    assert_eq!(
        h.documents.told(),
        Some(ph2d_i18n::tr("kanban.share.kanban_link"))
    );
    assert_eq!(
        h.documents.live_view().role,
        LiveRole::Off,
        "o Quadro não entra"
    );
    assert_eq!(
        ph2d_kanban_sync::port::take_kanban_link(),
        Some(link),
        "vai à família Kanban"
    );
    // O contrário: o convite do Quadro que a família Kanban recusou.
    ph2d_kanban_sync::port::post_board_link(invite.link());
    service(&mut h);
    assert_eq!(
        h.documents.live_view().role,
        LiveRole::Joining,
        "o Quadro entra"
    );
    reset_for_tests();
}

/// O que a sala do Quadro publica à família Kanban deste fio: quem tem o cartão `card` aberto no
/// Kanban LOCAL `k`, e se a sala está a voltar.
fn on_card(k: KanbanId, card: CardId) -> (Vec<String>, bool) {
    ph2d_kanban_sync::port::with_room(|r| {
        let Some(r) = r else {
            return (Vec::new(), false);
        };
        let Some((wire, _)) = r.pairs.iter().find(|(_, l)| *l == k) else {
            return (Vec::new(), r.reconnecting);
        };
        let who = r
            .viewers
            .on_card(*wire, card)
            .into_iter()
            .filter_map(|p| r.names.get(&p).cloned())
            .collect();
        (who, r.reconnecting)
    })
}

/// ⭐ J7 — a presença do Kanban DENTRO da sala do Quadro (que cartão cada um tem aberto) e a queda:
/// o fio da Bia cai, a sala dela diz que está a voltar, ela volta SOZINHA (o retrocesso partilhado
/// com o Kanban) JUNTO com os Kanbans do quadro, e o que cada uma moveu enquanto ela estava fora
/// chega à outra.
#[test]
fn j7_the_board_guest_drops_and_returns_with_its_kanbans_and_their_presence() {
    reset_for_tests();
    ph2d_kanban_sync::port::reset_for_tests();
    let mut ana = hero("Ana");
    let id = ana.documents.create("Retro".into());
    ana.documents.activate(Some(id));
    let _ = ana.documents.kanbans_shared().borrow_mut().create("Outro");
    let k = place(&mut ana, id, 0.0);
    let open: Vec<(CardId, String)> = {
        let set = ana.documents.kanbans();
        let kb = set.get(k).unwrap();
        kb.cards
            .values()
            .filter(|c| kb.lists.get(&c.list).is_some_and(|l| l.title != "Done"))
            .map(|c| (c.id, c.title.clone()))
            .collect()
    };
    let ((ca, ta), (cb, tb)) = (open[0].clone(), open[1].clone());
    // A Ana tem o cartão B aberto (o que a família Kanban diria pela porta).
    ph2d_kanban_sync::port::set_here(Some((k, Some(cb), None)));
    ana.documents.ask_live(LiveRequest::Host);
    service(&mut ana);
    let LiveRole::Hosting { code, addrs } = ana.documents.live_view().role.clone() else {
        panic!("não abriu");
    };
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let (tx, rx) = mpsc::channel::<&'static str>();
    let (back_tx, back_rx) = mpsc::channel::<()>();
    let (ta2, tb2) = (ta.clone(), tb.clone());
    let bia = std::thread::spawn(move || {
        ph2d_kanban_sync::port::reset_for_tests();
        let mut g = hero("Bia");
        let _ = g.documents.kanbans_shared().borrow_mut().create("Meu");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code,
        });
        let mut kb = KanbanId(0);
        assert!(
            until(&mut g, |g| {
                if let Some((x, _, _)) = on_board(g).into_iter().find(|(_, _, n)| *n > 3) {
                    kb = x;
                }
                kb != KanbanId(0) && on_card(kb, cb).0 == vec!["Ana".to_owned()]
            }),
            "a Bia não viu a Ana no cartão B: {:?}",
            on_card(kb, cb)
        );
        // A Bia abre o cartão A no Kanban dela.
        ph2d_kanban_sync::port::set_here(Some((kb, Some(ca), None)));
        let _ = tx.send("vi");
        assert!(
            until(&mut g, |_| back_rx.try_recv().is_ok()),
            "a Ana não me viu"
        );
        // Cai, e move o A FORA da sala.
        super::drop_guest_for_tests();
        move_card(&mut g, kb, &ta2, "Done");
        let _ = tx.send("caí");
        let mut voltando = false;
        assert!(
            until(&mut g, |g| {
                voltando |= on_card(kb, cb).1;
                voltando
                    && !on_card(kb, cb).1
                    && list_of(g, kb, &tb2).as_deref() == Some("Done")
                    && list_of(g, kb, &ta2).as_deref() == Some("Done")
                    && matches!(g.documents.live_view().role, LiveRole::Guest { .. })
            }),
            "a Bia não voltou com o Kanban (voltando={voltando}, B={:?})",
            list_of(&g, kb, &tb2)
        );
        let _ = tx.send("voltei");
        let _ = until(&mut g, |_| back_rx.try_recv().is_ok());
        g.documents.ask_live(LiveRequest::Leave);
        service(&mut g);
    });
    let go = |rx: &mpsc::Receiver<&'static str>, ana: &mut HeroScreen| {
        let end = Instant::now() + Duration::from_secs(SECS);
        loop {
            if let Ok(m) = rx.try_recv() {
                return m;
            }
            assert!(Instant::now() < end, "a Bia não respondeu");
            service(ana);
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    assert_eq!(go(&rx, &mut ana), "vi");
    assert!(
        until(&mut ana, |_| on_card(k, ca).0 == vec!["Bia".to_owned()]),
        "a Ana não viu a Bia no cartão A"
    );
    let _ = back_tx.send(());
    assert_eq!(go(&rx, &mut ana), "caí");
    // Enquanto a Bia está fora, a Ana move o B.
    move_card(&mut ana, k, &tb, "Done");
    assert!(
        until(&mut ana, |a| list_of(a, k, &ta).as_deref() == Some("Done")),
        "o que a Bia moveu fora não chegou à Ana"
    );
    assert_eq!(go(&rx, &mut ana), "voltei");
    assert!(
        until(&mut ana, |_| on_card(k, ca).0 == vec!["Bia".to_owned()]),
        "a presença da Bia não voltou"
    );
    let _ = back_tx.send(());
    let end = Instant::now() + Duration::from_secs(SECS);
    while !bia.is_finished() && Instant::now() < end {
        service(&mut ana);
        std::thread::sleep(Duration::from_millis(5));
    }
    bia.join().expect("a Bia falhou");
    ana.documents.ask_live(LiveRequest::Leave);
    service(&mut ana);
    assert!(
        ph2d_kanban_sync::port::with_room(|r| r.is_none()),
        "a sala acabou: a presença dela some"
    );
    reset_for_tests();
}
