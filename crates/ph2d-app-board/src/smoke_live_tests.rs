//! ⭐ A cena 12 ARMA o que as seis lições dizem, na ORDEM REAL: a sala aberta com o código no painel;
//! a firewall fechada oferece o botão e diz onde ele está; a 2.ª janela entra pelo código e recebe a
//! «Retro»; o que uma desenha aparece na outra, com a seta e o nome; seguir leva a minha vista à dela;
//! «trazer todos» leva a dela à minha; «End session» acaba-a JÁ do outro lado (sem o minuto a tentar
//! voltar) e a cópia fica. Duas janelas = duas threads (o estado da família é por thread), a rede
//! local de verdade em 127.0.0.1.

use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;
use crate::live::{fake_firewall_for_tests, reset_for_tests, service};
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, BoardOp, Camera, Rgba};
use ph2d_board_net::firewall::Probe;
use ph2d_editor_core::documents::live::LiveRole;

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

fn until(h: &mut HeroScreen, secs: u64, mut done: impl FnMut(&mut HeroScreen) -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        service(h);
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

fn count(h: &mut HeroScreen) -> usize {
    room(h).map_or(0, |b| b.doc.live_len())
}

fn camera(h: &mut HeroScreen) -> Option<Camera> {
    room(h).map(|b| b.camera)
}

const THEIRS: Camera = Camera {
    center_x: 700.0,
    center_y: 900.0,
    zoom: 0.5,
};
const MINE: Camera = Camera {
    center_x: -300.0,
    center_y: 40.0,
    zoom: 1.5,
};

/// O que a 2.ª janela viu, para o anfitrião o afirmar depois de a juntar.
struct Seen {
    name: String,
    summoned: bool,
    ended_in: Option<Duration>,
    told: String,
    copy: usize,
}

#[test]
fn the_live_scene_hosts_and_a_second_window_joins_by_its_code() {
    reset_for_tests();
    let ufw = Probe {
        ufw_conf: Some("ENABLED=yes\n".into()),
        pkexec: true,
        ..Probe::default()
    };
    fake_firewall_for_tests(ufw, vec!["192.168.15.0/24".parse().unwrap()]);
    let mut host = hero("Ana");
    let l = scene_live(&mut host);
    service(&mut host);

    // 1 — a sala aberta, o painel aberto com o código, no quadro da cena.
    let LiveRole::Hosting { code, addrs } = host.documents.live_view().role.clone() else {
        panic!(
            "a cena não abriu a sala: {:?}",
            host.documents.live_view().role
        );
    };
    assert_eq!(code.replace('-', "").len(), 6, "{code}");
    assert!(live_board::is_open(&host), "o painel Live não abriu");
    assert_eq!(host.documents.active(), Some(l.board));
    assert_eq!(host.documents.live_view().board, Some(l.board));
    let n = count(&mut host);
    assert!(n > l.lessons.len() + 5, "a «Retro» sem formas e notas: {n}");

    // 2 — a firewall fechada: o botão e a frase que diz onde ele está.
    assert!(
        until(&mut host, 5, |h| h.documents.live_view().firewall.can_allow),
        "a firewall fechada não ofereceu o botão"
    );
    assert_eq!(
        host.documents.told(),
        Some(tr("board.live.fw_hint").as_str())
    );

    // 3 — a 2.ª janela entra pelo código.
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let (to_host, from_guest) = mpsc::channel::<()>();
    let (to_guest, from_host) = mpsc::channel::<()>();
    let guest = std::thread::spawn(move || {
        reset_for_tests();
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code: code.to_lowercase(),
        });
        assert!(
            until(&mut g, 10, |g| count(g) == n),
            "não recebeu a «Retro»"
        );
        let gid = g.documents.live_view().board.expect("a aba da sala");
        let name = g
            .documents
            .boards()
            .get(gid)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        // 4 — desenho e mexo a vista e o rato.
        let b = room(&mut g).unwrap();
        let ink = Rgba([20, 20, 20, 255]);
        let el = Element::new_shape(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Shape {
                kind: ShapeType::Rectangle,
                style: Style::new(Some(ink), None, ink),
                text: RichText::plain("Bia"),
            },
            [900.0, 900.0, 80.0, 60.0],
        );
        let _ = BoardOp::Put(el).apply(&mut b.doc);
        b.camera = THEIRS;
        g.documents.live_view_mut().my_cursor = Some([910.0, 910.0]);
        to_host.send(()).unwrap();
        // 6 — «trazer todos» chega, e depois o fim.
        let summoned = until(&mut g, 10, |g| camera(g) == Some(MINE));
        to_host.send(()).unwrap();
        from_host.recv_timeout(Duration::from_secs(10)).unwrap();
        let t0 = Instant::now();
        let ended = until(&mut g, 10, |g| {
            g.documents.live_view().role == LiveRole::Off
        });
        let copy = g
            .documents
            .boards()
            .get(gid)
            .map_or(0, |b| b.doc.live_len());
        Seen {
            name,
            summoned,
            ended_in: ended.then(|| t0.elapsed()),
            told: g.documents.told().unwrap_or_default().to_owned(),
            copy,
        }
    });
    let serve_until = |h: &mut HeroScreen, rx: &mpsc::Receiver<()>| {
        let end = Instant::now() + Duration::from_secs(15);
        while Instant::now() < end {
            service(h);
            if rx.try_recv().is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        false
    };
    assert!(
        serve_until(&mut host, &from_guest),
        "a 2.ª janela não entrou"
    );
    assert!(
        until(&mut host, 10, |h| count(h) == n + 1),
        "o desenho dela não apareceu cá"
    );
    assert!(
        until(&mut host, 10, |h| h
            .documents
            .live_view()
            .others
            .iter()
            .any(|o| o.name == "Bia" && o.cursor.is_some())),
        "a seta e o nome dela não apareceram"
    );

    // 5 — seguir leva a minha vista à dela.
    let bia = host
        .documents
        .live_view()
        .peers
        .iter()
        .find(|p| !p.me)
        .map(|p| p.peer)
        .unwrap();
    host.documents.live_view_mut().following = Some(bia);
    assert!(
        until(&mut host, 10, |h| camera(h) == Some(THEIRS)),
        "seguir não trouxe a vista dela"
    );

    // 6 — trazer todos, e fechar.
    host.documents.live_view_mut().following = None;
    room(&mut host).unwrap().camera = MINE;
    host.documents.ask_live(LiveRequest::Summon);
    assert!(serve_until(&mut host, &from_guest), "a 2.ª janela parou");
    host.documents.ask_live(LiveRequest::Leave);
    service(&mut host);
    to_guest.send(()).unwrap();
    let seen = guest.join().expect("a 2.ª janela falhou");
    assert_eq!(
        seen.name,
        tr("board.smoke.live.board"),
        "a lição 3 diz «Retro»"
    );
    assert!(
        seen.summoned,
        "«Bring everyone here» não levou a vista dela"
    );
    let ended = seen.ended_in.expect("a sessão dela não acabou");
    assert!(
        ended < Duration::from_secs(3),
        "o fim levou {ended:?}: o outro lado ficou a tentar voltar"
    );
    assert_eq!(seen.told, tr("board.live.host_left"));
    assert_eq!(seen.copy, n + 1, "a cópia não ficou");
    assert_eq!(host.documents.live_view().role, LiveRole::Off);
}
