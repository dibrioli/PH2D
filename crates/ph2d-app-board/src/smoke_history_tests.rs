//! ⭐ A cena 14 ARMA o que as sete lições dizem, na ORDEM REAL, pelo PAINEL (os ids que a pintura
//! regista) e com a rede local de verdade em 127.0.0.1 para a 2.ª janela: a lista com a mais nova em
//! cima e mostrada; abrir a antiga mostra-a; renomeá-la pelo duplo clique; o toggle esconde e volta;
//! votar de novo pela sala dá a «Voting 3» em cima; a Bia, que entra depois, recebe o MESMO
//! histórico; o lixo (dois cliques) apaga nos dois.

use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::Duration;

use super::*;
use crate::live::{fake_firewall_for_tests, reset_for_tests};
use crate::smoke::fac_live::tests::{Hand, click, frame, hero, room, until};
use ph2d_board_model::{Poll, Team};
use ph2d_board_net::firewall::Probe;
use ph2d_editor_core::documents::live::{LiveRequest, LiveRole};
use ph2d_editor_core::ids::BOARD_POLL_NAME_INPUT;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::screens::hero::board_nav::facilitate::{
    self, FacButton, apply_event, fac_node, poll_row_node,
};

fn team(h: &mut HeroScreen) -> Team {
    room(h).expect("o quadro da sala").team.clone()
}

/// A fila `k` da página da lista, como a pintura a registou.
fn row(h: &mut HeroScreen, hand: &mut Hand, k: usize) {
    hand.paint(h);
    assert!(apply_event(h, WidgetEvent::Click(poll_row_node(k))), "a fila {k}");
    frame(h);
}

fn lesson(h: &mut HeroScreen, id: ElementId) -> String {
    let b = room(h).unwrap();
    b.doc.get(id).and_then(|e| e.shape()).map(|s| s.text.as_str().to_owned()).unwrap_or_default()
}

fn tr(key: &str) -> &'static str {
    ph2d_i18n::tr(key)
}

/// Serve a sala (a Ana é o anfitrião) até a Bia dizer `what`.
fn wait(ana: &mut HeroScreen, from_bia: &mpsc::Receiver<&'static str>, what: &str) {
    let end = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(m) = from_bia.try_recv() {
            assert_eq!(m, what);
            return;
        }
        assert!(std::time::Instant::now() < end, "a Bia não disse «{what}»");
        frame(ana);
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A mais nova (a que acabou por último).
fn newest(t: &Team) -> &Poll {
    t.polls_newest_first().next().expect("uma sessão")
}

#[test]
fn the_history_scene_does_what_its_lessons_say() {
    reset_for_tests();
    fake_firewall_for_tests(Probe::default(), vec!["192.168.15.0/24".parse().unwrap()]);
    let mut ana = hero("Ana");
    let l = scene_history(&mut ana);
    frame(&mut ana);
    let LiveRole::Hosting { code, addrs } = ana.documents.live_view().role.clone() else {
        panic!("a cena não abriu a sala: {:?}", ana.documents.live_view().role);
    };
    assert!(facilitate::vote_open(&ana), "o painel da votação não abriu");
    let xs: Vec<f64> = l
        .lessons
        .iter()
        .map(|id| room(&mut ana).unwrap().doc.get(*id).expect("a lição").x)
        .collect();
    assert!(xs.windows(2).all(|w| w[0] < w[1]), "as lições pela ordem: {xs:?}");
    // As lições dizem as palavras que o ecrã pinta (uma lição com a palavra errada ensina errado).
    let say = |ana: &mut HeroScreen, i: usize, keys: &[&str]| {
        let t = lesson(ana, l.lessons[i]);
        for k in keys {
            assert!(t.contains(tr(k)), "a lição {} não diz «{}»: {t}", i + 1, tr(k));
        }
    };
    say(&mut ana, 0, &["board.vote.results"]);
    say(&mut ana, 1, &["board.vote.deleted_row"]);
    say(&mut ana, 2, &[]);
    assert!(tr("board.vote.rename_tip").starts_with("Double-click"));
    assert!(lesson(&mut ana, l.lessons[2]).starts_with("3. Double-click"));
    say(&mut ana, 3, &["board.vote.show_on_board"]);
    say(&mut ana, 4, &["board.vote.tab_new", "board.vote.start", "board.vote.end"]);
    let [old, new] = l.polls;
    // 1 — a lista: a mais nova em cima, e é a que o quadro mostra.
    let t = team(&mut ana);
    assert_eq!(newest(&t).id, new);
    assert_eq!(t.shown, Some(new));
    // 2 — abrir a Voting 1 pela 2.ª fila: passa a ser a mostrada; a nota apagada fica na sessão.
    let mut hand = Hand::new();
    row(&mut ana, &mut hand, 1);
    let t = team(&mut ana);
    assert_eq!(t.shown, Some(old), "a antiga não passou a ser a mostrada");
    assert!(t.poll(old).unwrap().totals.contains_key(&l.gone));
    assert!(room(&mut ana).unwrap().doc.get(l.gone).is_none());
    // 3 — renomear: o duplo clique no título abre o campo; Enter grava.
    assert!(apply_event(&mut ana, WidgetEvent::DoubleClick(fac_node(FacButton::PollRename))));
    let name = "Sprint retro";
    ana.store.register(
        BOARD_POLL_NAME_INPUT,
        InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Focused,
            text: name.to_owned(),
            caret: name.len(),
            selection_anchor: None,
        },
    );
    assert!(apply_event(&mut ana, WidgetEvent::Submit(BOARD_POLL_NAME_INPUT)));
    frame(&mut ana);
    assert_eq!(team(&mut ana).poll(old).unwrap().name, name);
    // 4 — o toggle: esconde os selos e volta a mostrá-los (os da sessão aberta).
    click(&mut ana, FacButton::ShowOnBoard);
    assert_eq!(team(&mut ana).shown, None);
    click(&mut ana, FacButton::ShowOnBoard);
    assert_eq!(team(&mut ana).shown, Some(old));
    // 5 — votar de novo, pela SALA: a sessão nova fica em cima, mostrada.
    click(&mut ana, FacButton::VoteTabNew);
    click(&mut ana, FacButton::VoteStart);
    assert!(until(&mut ana, 10, |h| facilitate::vote_in_session(h)), "a votação não abriu");
    hand.plus(&mut ana, l.ideas[0]);
    hand.plus(&mut ana, l.ideas[0]);
    click(&mut ana, FacButton::VoteEnd);
    assert!(until(&mut ana, 10, |h| team(h).polls.len() == 3), "a sessão nova não chegou");
    let t = team(&mut ana);
    let third = newest(&t);
    assert_eq!((third.number, third.by.as_str()), (3, "Ana"));
    assert_eq!(third.totals.get(&l.ideas[0]), Some(&2));
    assert_eq!(t.shown, Some(third.id));
    let third = third.id;
    // 7 — a Bia entra DEPOIS e recebe o mesmo histórico (o nome novo incluído).
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let want = t.polls.clone();
    let (to_ana, from_bia) = mpsc::channel::<&'static str>();
    let (to_bia, from_ana) = mpsc::channel::<&'static str>();
    let bia = std::thread::spawn(move || {
        reset_for_tests();
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code,
        });
        assert!(
            until(&mut g, 10, |g| room(g).is_some_and(|b| b.team.polls == want)),
            "a Bia não recebeu o histórico"
        );
        to_ana.send("vi").unwrap();
        assert_eq!(from_ana.recv_timeout(Duration::from_secs(30)).unwrap(), "apaguei");
        // 6 — o lixo da Ana chega aqui.
        assert!(
            until(&mut g, 10, |g| room(g).is_some_and(|b| b.team.polls.len() == 2)),
            "o apagar não chegou"
        );
        to_ana.send("adeus").unwrap();
    });
    wait(&mut ana, &from_bia, "vi");
    // 6 — apagar a nova: abri-la (1.ª fila), o lixo arma, o 2.º clique apaga.
    click(&mut ana, FacButton::PollBack);
    row(&mut ana, &mut hand, 0);
    assert_eq!(team(&mut ana).shown, Some(third));
    click(&mut ana, FacButton::PollDelete);
    assert_eq!(team(&mut ana).polls.len(), 3, "o 1.º clique só arma");
    click(&mut ana, FacButton::PollDelete);
    let t = team(&mut ana);
    assert_eq!(t.polls.len(), 2);
    assert_eq!(t.shown, None, "a mostrada saiu: o quadro deixa de a mostrar");
    to_bia.send("apaguei").unwrap();
    wait(&mut ana, &from_bia, "adeus");
    bia.join().unwrap();
    ana.documents.ask_live(LiveRequest::Leave);
    frame(&mut ana);
}
