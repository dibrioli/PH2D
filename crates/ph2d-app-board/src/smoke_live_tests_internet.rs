//! A cena 22 e o link pela internet (partido de `smoke_live_tests.rs`).

use super::*;

/// ⭐ A cena 22 (ADR-0187) ARMA o convite pela internet: a sala aberta, o link COPIADO (o que o
/// painel mostra é o mesmo), e a 2.ª janela cola-o e recebe a «Retro online» pelo fio da internet
/// (QUIC sobre UDP, sem STUN nem DHT: só os endereços desta máquina, que vão no link).
#[test]
fn the_internet_scene_copies_a_link_and_a_second_window_joins_by_it() {
    use crate::live::internet_local_for_tests;
    use ph2d_editor_core::documents::live::LiveInternet;
    reset_for_tests();
    internet_local_for_tests();
    fake_firewall_for_tests(Probe::default(), vec![]);
    let mut host = hero("Ana");
    let l = scene_internet(&mut host);
    assert!(
        until(&mut host, 10, |h| matches!(
            h.documents.live_view().internet,
            LiveInternet::Ready { .. }
        )),
        "o link não ficou pronto: {:?}",
        host.documents.live_view().internet
    );
    let link = host
        .store
        .take_clipboard_copy()
        .expect("o link foi copiado");
    assert!(link.starts_with(ph2d_board_net::SCHEME), "{link}");
    assert_eq!(
        ph2d_editor_core::screens::hero::board_nav::team::live_board::LINK_SCHEME,
        ph2d_board_net::SCHEME,
        "o ecrã reconhece o link pelo mesmo começo"
    );
    assert_eq!(
        ph2d_editor_core::screens::hero::board_nav::team::live_board::KANBAN_LINK_SCHEME,
        ph2d_board_net::KANBAN_SCHEME,
        "e o do Kanban também"
    );
    assert_eq!(
        host.documents.live_view().internet,
        LiveInternet::Ready { link: link.clone() }
    );
    assert!(matches!(
        host.documents.live_view().role,
        LiveRole::Hosting { .. }
    ));
    assert_eq!(host.documents.active(), Some(l.board));
    let n = count(&mut host);

    let (to_host, from_guest) = mpsc::channel();
    let (to_guest, from_host) = mpsc::channel::<()>();
    let guest = std::thread::spawn(move || {
        reset_for_tests();
        internet_local_for_tests();
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::JoinLink { link });
        let got = until(&mut g, 20, |g| count(g) == n);
        let _ = to_host.send((got, g.documents.live_view().role.clone()));
        // Fica na sala até o anfitrião a ver.
        until(&mut g, 10, |_| from_host.try_recv().is_ok());
    });
    let mut seen = None;
    while seen.is_none() && !guest.is_finished() {
        service(&mut host);
        seen = from_guest.try_recv().ok();
        std::thread::sleep(Duration::from_millis(5));
    }
    let (got, role) = seen.expect("a 2.ª janela parou sem responder");
    assert!(got, "a 2.ª janela não recebeu a «Retro online»: {role:?}");
    assert!(matches!(role, LiveRole::Guest { .. }), "{role:?}");
    let both = until(&mut host, 5, |h| h.documents.live_view().peers.len() == 2);
    let _ = to_guest.send(());
    guest.join().unwrap();
    assert!(both, "o anfitrião não vê a Bia");
}

/// ADR-0187 — o anfitrião numa rede de NAT SIMÉTRICO (os STUN «medem» `Dependent`) fica a saber, no
/// painel, que quem está fora pode não entrar; o controlo (sem NAT) não diz nada.
#[test]
fn a_host_on_a_strict_network_is_told_the_link_may_not_get_through() {
    use crate::live::fake_nat_for_tests;
    use ph2d_board_internet::Mapping;
    use ph2d_editor_core::documents::live::LiveInternet;
    for strict in [false, true] {
        reset_for_tests();
        crate::live::internet_local_for_tests();
        fake_firewall_for_tests(Probe::default(), vec![]);
        fake_nat_for_tests(if strict {
            Mapping::Dependent
        } else {
            Mapping::Independent
        });
        let mut host = hero("Ana");
        scene_internet(&mut host);
        assert!(until(&mut host, 10, |h| matches!(
            h.documents.live_view().internet,
            LiveInternet::Ready { .. }
        )));
        assert_eq!(host.documents.live_view().link.strict_nat, strict);
        host.documents.ask_live(LiveRequest::Leave);
        service(&mut host);
        assert!(
            !host.documents.live_view().link.strict_nat,
            "a medição ficou depois da sessão"
        );
    }
}

/// Um link para um endereço onde nada responde à sala: um nó vivo e SEM entrada (recusa).
fn closed_room_link() -> (ph2d_board_internet::Node, String) {
    let n = ph2d_board_internet::Node::bind(&ph2d_board_internet::NodeConfig::default()).unwrap();
    let invite = ph2d_board_net::Invite {
        key: ph2d_board_net::InviteKey::random(),
        addrs: vec![SocketAddr::from(([127, 0, 0, 1], n.port()))],
    };
    (n, invite.link())
}

/// ⭐ ADR-0187 + auditoria A2 — quem entra por link vê que se PROCURA na internet (e até quando), não o
/// «Connecting…» da rede local; a sala fechada responde logo (não aos 30 s) e a frase, que fica no
/// painel, diz o que fazer.
#[test]
fn joining_by_link_searches_the_internet_and_a_closed_room_says_what_to_do() {
    reset_for_tests();
    crate::live::internet_local_for_tests();
    let (_n, link) = closed_room_link();
    let mut g = hero("Bia");
    g.documents.ask_live(LiveRequest::JoinLink { link });
    service(&mut g);
    assert_eq!(g.documents.live_view().role, LiveRole::Joining);
    assert_eq!(g.documents.live_view().link.searching, Some(30));
    let t = Instant::now();
    assert!(until(&mut g, 10, |g| g.documents.live_view().role
        == LiveRole::Off));
    assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
    let closed = tr("board.live.link_closed");
    assert_eq!(g.documents.told(), Some(closed.as_str()));
    let l = &g.documents.live_view().link;
    assert_eq!(
        l.failed.as_deref(),
        Some(closed.as_str()),
        "a frase saiu do painel"
    );
    assert_eq!(l.searching, None);
}

/// O convidado numa rede de NAT simétrico ouve, ao falhar, que a rede DELE pode bloquear e o que
/// tentar; sem a medição, a frase é a de sempre. (Só IPv6 no link e o nó sem IPv6: nenhum caminho.)
#[test]
fn a_guest_on_a_strict_network_hears_that_it_may_be_blocking() {
    use crate::live::fake_nat_for_tests;
    use ph2d_board_internet::Mapping;
    let link = ph2d_board_net::Invite {
        key: ph2d_board_net::InviteKey::random(),
        addrs: vec!["[2001:db8::7]:4000".parse().unwrap()],
    }
    .link();
    for (m, key) in [
        (Mapping::Independent, "board.live.link_unreachable"),
        (Mapping::Dependent, "board.live.link_unreachable_strict"),
    ] {
        reset_for_tests();
        crate::live::internet_local_for_tests();
        fake_nat_for_tests(m);
        let mut g = hero("Bia");
        g.documents
            .ask_live(LiveRequest::JoinLink { link: link.clone() });
        assert!(until(&mut g, 10, |g| g
            .documents
            .live_view()
            .link
            .failed
            .is_some()));
        assert_eq!(g.documents.told(), Some(tr(key).as_str()), "{m:?}");
    }
}

/// 📷 A cena 22 (ADR-0187) nos estados que a foto da JANELA não alcança (pedem um clique): o anfitrião
/// com o link pronto numa rede de NAT simétrico, e quem entra por link — à procura (um endereço que
/// não responde: a espera inteira) com a rede dele a bloquear, e a sala fechada com a frase do que
/// fazer. Corre à mão e LÊ-SE cada foto:
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-board --lib fotos_dos_estados_da_cena_22 -- --ignored --nocapture
/// ```
#[test]
#[ignore = "fotografias: correm-se à mão com a placa"]
fn fotos_dos_estados_da_cena_22() {
    use crate::live::{fake_nat_for_tests, internet_local_for_tests};
    use crate::smoke::shots::Shots;
    use ph2d_board_internet::Mapping;
    use ph2d_editor_core::documents::live::LiveInternet;
    let mut cam = Shots::new("ph2d_w22c_fotos");
    let fresh = || {
        reset_for_tests();
        internet_local_for_tests();
        fake_firewall_for_tests(Probe::default(), vec![]);
        fake_nat_for_tests(Mapping::Dependent);
    };
    fresh();
    let mut ana = hero("Ana");
    scene_internet(&mut ana);
    assert!(until(&mut ana, 10, |h| matches!(
        h.documents.live_view().internet,
        LiveInternet::Ready { .. }
    )));
    cam.shot(&mut ana, "a0_link_pronto_rede_estrita");
    drop(ana);

    fresh();
    let mut bia = hero("Bia");
    super::super::super::scene_two_boards(&mut bia);
    live_board::toggle(&mut bia);
    let silent = ph2d_board_net::Invite {
        key: ph2d_board_net::InviteKey::random(),
        addrs: vec![SocketAddr::from(([192, 0, 2, 1], 9))],
    };
    bia.documents.ask_live(LiveRequest::JoinLink {
        link: silent.link(),
    });
    assert!(until(&mut bia, 5, |g| g
        .documents
        .live_view()
        .link
        .strict_nat));
    cam.shot(&mut bia, "b0_a_procurar_na_internet");
    bia.documents.ask_live(LiveRequest::Leave);
    service(&mut bia);

    let (_n, closed) = closed_room_link();
    bia.documents
        .ask_live(LiveRequest::JoinLink { link: closed });
    assert!(until(&mut bia, 10, |g| g
        .documents
        .live_view()
        .link
        .failed
        .is_some()));
    cam.shot(&mut bia, "b1_a_sala_fechou");
}

#[test]
fn a_link_that_is_not_ours_is_refused_with_its_sentence() {
    reset_for_tests();
    let mut g = hero("Bia");
    g.documents.ask_live(LiveRequest::JoinLink {
        link: "ph2d-board://v9/#AAAA".into(),
    });
    service(&mut g);
    assert_eq!(g.documents.live_view().role, LiveRole::Off);
    assert_eq!(g.documents.told(), Some(tr("board.live.link_bad").as_str()));
}

/// W23 — a cena 23 arma o lado de quem ENTRA: a «Retro» sem sessão, o painel aberto (procura quadros
/// na rede) e nada aberto à rede nem à internet.
#[test]
fn the_join_scene_opens_the_panel_without_a_session() {
    reset_for_tests();
    let mut h = hero("Bia");
    let l = super::super::scene_join(&mut h);
    assert!(live_board::is_open(&h));
    assert_eq!(h.documents.active(), Some(l.board));
    assert_eq!(h.documents.take_live(), vec![LiveRequest::Discover]);
    assert_eq!(h.documents.live_view().role, LiveRole::Off);
    assert!(room(&mut h).is_none(), "a cena abriu uma sala");
}
