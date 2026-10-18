//! A12 (W23) — a entrada por LINK na família: a frase de cada falha no painel, a sala de outra chave
//! de ponta a ponta, e a volta depois de uma queda pela mesma sessão (nó e ponto de encontro).

use std::io::ErrorKind as K;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use super::*;
use crate::live::{internet_local_for_tests, reset_for_tests, service};
use ph2d_a11y::NodeId;
use ph2d_editor_core::documents::live::LiveRequest;

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

/// Uma sala só com a entrada pela internet, nesta máquina.
fn internet_room() -> (net::Host, InternetHost) {
    let cfg = net::HostConfig {
        port: 0,
        tries: 1,
        mdns: false,
        ..net::HostConfig::default()
    };
    let ann = net::Announce {
        board: "Q".into(),
        host: "Ana".into(),
    };
    let mut h = net::Host::open_with(&ann, &cfg, std::sync::Arc::new(|| {})).unwrap();
    let ih = InternetHost::open(&mut h, &internet_config(true)).unwrap();
    (h, ih)
}

fn arrived(h: &net::Host) -> (net::ConnId, SocketAddr) {
    let end = Instant::now() + Duration::from_secs(10);
    while Instant::now() < end {
        if let Some(j) = h.poll().into_iter().find_map(|e| match e {
            net::NetEvent::Joined(id, peer) => Some((id, peer)),
            _ => None,
        }) {
            return j;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("ninguém chegou");
}

fn finished(d: &Dialing) -> Result<net::Guest, net::JoinError> {
    d.rx.recv_timeout(Duration::from_secs(20))
        .expect("a tentativa não acabou")
}

#[test]
fn every_way_a_link_fails_has_its_sentence_in_the_panel() {
    let link = Route::Link {
        guest: LinkGuest::new(net::Invite {
            key: net::InviteKey::random(),
            addrs: vec![],
        }),
        local: true,
    };
    for (e, key) in [
        (net::JoinError::WrongCode, "board.live.link_old"),
        (net::JoinError::InvalidCode, "board.live.link_old"),
        (
            net::JoinError::Unreachable(K::ConnectionRefused),
            "board.live.link_closed",
        ),
        (
            net::JoinError::Unreachable(K::NetworkUnreachable),
            "board.live.link_offline",
        ),
        (
            net::JoinError::Unreachable(K::NetworkDown),
            "board.live.link_offline",
        ),
        (
            net::JoinError::Unreachable(K::AddrNotAvailable),
            "board.live.link_offline",
        ),
        (
            net::JoinError::Unreachable(K::TimedOut),
            "board.live.link_unreachable",
        ),
    ] {
        let mut g = hero("Bia");
        assert!(link_failed(&mut g, &link, &e), "{e:?}");
        assert_eq!(g.documents.told(), Some(tr(key)), "{e:?}");
        assert_eq!(
            g.documents.live_view().link.failed.as_deref(),
            Some(tr(key)),
            "{e:?}: a frase não ficou no painel"
        );
    }
    // Pela rede local, a frase é a de sempre (não é desta família).
    let lan = Route::Lan {
        addr: SocketAddr::from(([127, 0, 0, 1], 1)),
        code: "ABC-DEF".into(),
    };
    let mut g = hero("Bia");
    assert!(!link_failed(&mut g, &lan, &net::JoinError::WrongCode));
    assert_eq!(g.documents.live_view().link.failed, None);
}

/// A sala responde mas a chave do link não é a dela (outra sala, ou fechou e abriu de novo).
#[test]
fn a_link_with_another_rooms_key_says_the_link_is_old() {
    reset_for_tests();
    internet_local_for_tests();
    let (_h, ih) = internet_room();
    let mut other = ih.invite().clone();
    other.key = net::InviteKey::random();
    let mut g = hero("Bia");
    g.documents
        .ask_live(LiveRequest::JoinLink { link: other.link() });
    let end = Instant::now() + Duration::from_secs(10);
    while g.documents.live_view().link.failed.is_none() && Instant::now() < end {
        service(&mut g);
        std::thread::sleep(Duration::from_millis(5));
    }
    let old = tr("board.live.link_old");
    assert_eq!(g.documents.live_view().link.failed.as_deref(), Some(old));
    assert_eq!(g.documents.told(), Some(old));
}

/// ⭐ Depois de cair, a volta pela rota é a MESMA sessão: o anfitrião vê-a chegar do mesmo endereço.
#[test]
fn coming_back_by_the_link_comes_from_the_same_port() {
    let (h, ih) = internet_room();
    let route = Route::Link {
        guest: LinkGuest::new(ih.invite().clone()),
        local: true,
    };
    let g = finished(&dial(&route)).expect("entra");
    let (id, first) = arrived(&h);
    h.kick(id);
    drop(g);
    let _g = finished(&dial(&route)).expect("volta");
    let (_, again) = arrived(&h);
    assert_eq!(again, first, "a volta abriu outro nó");
}
