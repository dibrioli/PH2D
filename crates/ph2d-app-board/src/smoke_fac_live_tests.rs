//! ⭐ A cena 13 ARMA o que as seis lições dizem, na ORDEM REAL, com duas janelas em duas threads (o
//! estado da família é por thread) e a rede local de verdade em 127.0.0.1: a Bia entra; a Ana liga o
//! modo privado pelo PAINEL; a nota que a Bia escreve chega à Ana SEM o texto (e o desenho dela não o
//! pinta); «Turn off private mode for all» revela-a a todos depois da contagem; a Ana abre a votação
//! pelo painel, a sessão chega à Bia, os boletins vão só ao cofre, e «End for all» põe os totais nos
//! dois quadros.

use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;
use crate::live::{fake_firewall_for_tests, reset_for_tests, service};
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, BoardOp, ElementId, Rgba};
use ph2d_board_net::firewall::Probe;
use ph2d_editor_core::documents::live::{LiveRequest, LiveRole};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::board_nav::facilitate::{
    self, FacButton, apply_event, fac_node,
};

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

/// Uma imagem: a família serve a rede, e o ecrã segue a sala (o que o desenho faz antes de pintar).
fn frame(h: &mut HeroScreen) {
    service(h);
    facilitate::follow(h);
}

fn until(h: &mut HeroScreen, secs: u64, mut done: impl FnMut(&mut HeroScreen) -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        frame(h);
        if done(h) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    false
}

fn room(h: &mut HeroScreen) -> Option<&mut Board> {
    let id = h.documents.live_view().board?;
    h.documents.board_mut(id)
}

fn text(h: &mut HeroScreen, id: ElementId) -> String {
    room(h)
        .and_then(|b| b.doc.get(id).and_then(|e| e.shape()).map(|s| s.text.as_str().to_owned()))
        .unwrap_or_default()
}

fn click(h: &mut HeroScreen, b: FacButton) {
    assert!(apply_event(h, WidgetEvent::Click(fac_node(b))), "{b:?}");
    frame(h);
}

/// Uma nota escrita no frame «Write privately», como a mão a deixa (uma operação no documento).
fn write_note(h: &mut HeroScreen, private: ElementId, x: f64, s: &str) -> ElementId {
    let b = room(h).unwrap();
    let ink = Rgba([20, 20, 20, 255]);
    let mut el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Shape {
            kind: ShapeType::Sticky,
            style: Style::new(Some(Rgba([255, 230, 120, 255])), None, ink),
            text: RichText::plain(s),
        },
        [x, 90.0, 150.0, 150.0],
    );
    el.frame = Some(private);
    let id = el.id;
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    id
}

#[test]
fn the_private_scene_hides_until_the_reveal_and_votes_without_peeking() {
    reset_for_tests();
    fake_firewall_for_tests(Probe::default(), vec!["192.168.15.0/24".parse().unwrap()]);
    let mut ana = hero("Ana");
    let l = scene_fac_live(&mut ana);
    frame(&mut ana);
    let LiveRole::Hosting { code, addrs } = ana.documents.live_view().role.clone() else {
        panic!("a cena não abriu a sala: {:?}", ana.documents.live_view().role);
    };
    assert!(facilitate::private_open(&ana), "o painel do modo privado não abriu");
    assert_eq!(ana.documents.active(), Some(l.board));
    let xs: Vec<f64> = l
        .lessons
        .iter()
        .map(|id| room(&mut ana).unwrap().doc.get(*id).expect("a lição").x)
        .collect();
    assert!(xs.windows(2).all(|w| w[0] < w[1]), "as lições pela ordem: {xs:?}");
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let (to_ana, from_bia) = mpsc::channel::<&'static str>();
    let (to_bia, from_ana) = mpsc::channel::<&'static str>();
    let (private, ideas) = (l.private, l.ideas);
    let wait = move |what: &'static str| {
        assert_eq!(from_ana.recv_timeout(Duration::from_secs(30)).unwrap(), what);
    };
    let bia = std::thread::spawn(move || {
        reset_for_tests();
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code,
        });
        assert!(until(&mut g, 10, |g| room(g).is_some_and(|b| b.doc.get(ideas[0]).is_some())));
        to_ana.send("entrei").unwrap();
        // 2 — o modo privado chega (ligado pela Ana).
        assert!(until(&mut g, 10, |g| g.documents.live_view().fac.private.is_some()));
        // 3 — escrevo em privado: a nota é minha, o olho está nela.
        let mine = write_note(&mut g, private, 600.0, "segredo da Bia");
        assert!(until(&mut g, 5, |g| g.documents.live_view().fac.mine_hidden.contains(&mine)));
        to_ana.send("escrevi").unwrap();
        // 4 — revelada.
        assert!(until(&mut g, 15, |g| g.documents.live_view().fac.private.is_none()));
        assert!(g.documents.live_view().fac.mine_hidden.is_empty());
        // 5 — a votação da Ana abre-se aqui: a sessão segue a rodada, e o meu boletim vai ao cofre.
        assert!(
            until(&mut g, 10, |g| g.documents.live_view().fac.vote.is_some()),
            "a votação não chegou"
        );
        assert!(facilitate::vote_in_session(&g), "a sessão do editor não seguiu a sala");
        let round = g.documents.live_view().fac.vote.as_ref().unwrap().round;
        g.documents.ask_live(LiveRequest::Ballot {
            round,
            votes: BTreeMap::from([(ideas[0], 2)]),
            done: true,
        });
        frame(&mut g);
        to_ana.send("votei").unwrap();
        // 6 — o fim: os totais no meu quadro, e o painel no resultado.
        assert!(
            until(&mut g, 10, |g| room(g).is_some_and(|b| b.doc.get(ideas[0]).unwrap().votes() == 3)),
            "os totais não chegaram"
        );
        assert!(!facilitate::vote_in_session(&g));
        to_ana.send("vi o fim").unwrap();
        wait("adeus");
        mine
    });
    let next = |ana: &mut HeroScreen, what: &str| {
        let end = Instant::now() + Duration::from_secs(30);
        loop {
            frame(ana);
            if let Ok(m) = from_bia.try_recv() {
                assert_eq!(m, what);
                return;
            }
            assert!(Instant::now() < end, "a Bia não disse «{what}»");
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    next(&mut ana, "entrei");
    // 2 — ligar pelo painel (o que a lição 2 diz), com os nomes anónimos: a Bia perde o nome e a cor
    // nos olhos da Ana, e ninguém se segue.
    click(&mut ana, FacButton::PrivateAnonymous);
    click(&mut ana, FacButton::PrivateOn);
    assert!(until(&mut ana, 5, |h| h.documents.live_view().fac.private.is_some()));
    let bia_peer = ana.documents.live_view().peers.iter().find(|p| !p.me).unwrap().peer;
    ana.documents.live_view_mut().following = Some(bia_peer);
    frame(&mut ana);
    let lv = ana.documents.live_view();
    assert!(lv.others.iter().all(|o| o.anonymous && o.name.is_empty()), "{:?}", lv.others);
    assert_eq!(lv.following, None, "com os nomes anónimos não se segue ninguém");
    // 3 — a nota da Bia chega SEM o texto; o desenho da Ana não a pinta.
    next(&mut ana, "escrevi");
    assert!(until(&mut ana, 5, |h| !h.documents.live_view().fac.hidden_to_me.is_empty()));
    let bias: BTreeSet<ElementId> = ana.documents.live_view().fac.hidden_to_me.clone();
    let id = *bias.iter().next().unwrap();
    assert_eq!(text(&mut ana, id), "", "o texto da Bia chegou à Ana");
    // 4 — revelar: a contagem, e depois o texto.
    click(&mut ana, FacButton::PrivateOff);
    let counting = ana.documents.live_view().fac.private.as_ref().and_then(|p| p.reveal_in_ms);
    assert!(counting.is_some_and(|ms| ms > 4_000), "{counting:?}");
    assert!(until(&mut ana, 10, |h| h.documents.live_view().fac.private.is_none()));
    assert!(until(&mut ana, 3, |h| text(h, id) == "segredo da Bia"));
    assert!(
        ana.documents.live_view().others.iter().all(|o| !o.anonymous && o.name == "Bia"),
        "o nome volta depois de revelar"
    );
    // 5 — votar: o painel abre, «Start voting» pede à sala (o quadro inteiro).
    facilitate::toggle_vote(&mut ana);
    click(&mut ana, FacButton::VoteStart);
    assert!(until(&mut ana, 5, |h| h.documents.live_view().fac.vote.is_some()));
    let round = ana.documents.live_view().fac.vote.as_ref().unwrap().round;
    ana.documents.ask_live(LiveRequest::Ballot {
        round,
        votes: BTreeMap::from([(ideas[0], 1)]),
        done: false,
    });
    next(&mut ana, "votei");
    // Quem iniciou vê quantos votos cada um usou, e quem acabou — nunca em quê.
    assert!(until(&mut ana, 5, |h| h
        .documents
        .live_view()
        .fac
        .vote
        .as_ref()
        .is_some_and(|v| v.done == 1 && v.used.iter().any(|(n, u, d)| n == "Bia" && *u == 2 && *d))));
    assert_eq!(room(&mut ana).unwrap().doc.get(ideas[0]).unwrap().votes(), 0, "nada antes do fim");
    // 6 — «End for all».
    click(&mut ana, FacButton::VoteEnd);
    assert_eq!(room(&mut ana).unwrap().doc.get(ideas[0]).unwrap().votes(), 3);
    next(&mut ana, "vi o fim");
    ana.documents.ask_live(LiveRequest::Leave);
    frame(&mut ana);
    to_bia.send("adeus").unwrap();
    let mine = bia.join().unwrap();
    assert_eq!(mine, id, "a nota escondida era a da Bia");
}
