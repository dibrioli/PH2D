//! O quadro ao vivo de ponta a ponta: dois ecrãs (cada um na sua thread — o estado da família é por
//! thread), a rede local de verdade em 127.0.0.1, o código do anfitrião.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, BoardOp, Element, Rgba, RichText, Shape, ShapeType, Style};
use ph2d_editor_core::documents::live::LiveRequest;

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

fn rect(b: &mut Board, x: f64) {
    let ink = Rgba([20, 20, 20, 255]);
    let shape = Shape {
        kind: ShapeType::Rectangle,
        style: Style::new(Some(ink), None, ink),
        text: RichText::plain("x"),
    };
    let el = Element::new_shape(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        shape,
        [x, 0.0, 10.0, 10.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
}

fn shapes(h: &HeroScreen) -> usize {
    h.documents
        .live_view()
        .board
        .and_then(|id| h.documents.boards().get(id))
        .map_or(0, |b| b.doc.live_len())
}

/// Serve até `done` ou até ao prazo.
fn until(h: &mut HeroScreen, secs: u64, mut done: impl FnMut(&HeroScreen) -> bool) -> bool {
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

/// ⭐ O anfitrião abre, o convidado entra com o CÓDIGO e recebe o quadro numa aba; o que um cria
/// aparece no outro; o cursor do anfitrião aparece ao convidado; sair deixa a cópia.
#[test]
fn a_guest_joins_by_code_and_both_see_each_others_work() {
    let mut host = hero("Ana");
    let id = host.documents.create("Retro".into());
    host.documents.activate(Some(id));
    rect(host.documents.board_mut(id).unwrap(), 0.0);
    host.documents.ask_live(LiveRequest::Host);
    service(&mut host);
    let LiveRole::Hosting { code, addrs } = host.documents.live_view().role.clone() else {
        panic!("não abriu: {:?}", host.documents.live_view().role);
    };
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let (tx, rx) = mpsc::channel::<&'static str>();
    let guest = std::thread::spawn(move || {
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code: code.to_lowercase(),
        });
        assert!(
            until(&mut g, 10, |g| shapes(g) == 1),
            "não recebeu o quadro"
        );
        assert!(matches!(
            g.documents.live_view().role,
            LiveRole::Guest { .. }
        ));
        let b = g.documents.live_view().board.unwrap();
        rect(g.documents.board_mut(b).unwrap(), 50.0);
        let _ = tx.send("criei");
        assert!(
            until(&mut g, 10, |g| shapes(g) == 3),
            "não viu a forma do anfitrião"
        );
        assert!(
            until(&mut g, 10, |g| g
                .documents
                .live_view()
                .others
                .iter()
                .any(|o| o.name == "Ana" && o.cursor.is_some())),
            "não viu o cursor do anfitrião"
        );
        g.documents.ask_live(LiveRequest::Leave);
        service(&mut g);
        assert_eq!(g.documents.live_view().role, LiveRole::Off);
        assert_eq!(shapes(&g), 0, "a vista da sala fechou");
        assert_eq!(
            g.documents.boards().get(b).unwrap().doc.live_len(),
            3,
            "a cópia fica"
        );
    });
    assert!(
        until(&mut host, 10, |h| h.documents.live_view().peers.len() == 2),
        "o convidado não entrou"
    );
    assert!(
        until(&mut host, 10, |h| shapes(h) == 2),
        "não viu a forma do convidado"
    );
    let _ = rx.recv_timeout(Duration::from_secs(5));
    rect(host.documents.board_mut(id).unwrap(), 90.0);
    host.documents.live_view_mut().my_cursor = Some([5.0, 5.0]);
    let end = Instant::now() + Duration::from_secs(15);
    while !guest.is_finished() && Instant::now() < end {
        service(&mut host);
        std::thread::sleep(Duration::from_millis(5));
    }
    guest.join().expect("o convidado falhou");
    host.documents.ask_live(LiveRequest::Leave);
    service(&mut host);
    assert_eq!(host.documents.live_view().role, LiveRole::Off);
}

/// Um código errado é dito ao artista e a sessão não abre.
#[test]
fn a_wrong_code_is_said_and_nothing_opens() {
    let mut host = hero("Ana");
    let id = host.documents.create("Retro".into());
    host.documents.activate(Some(id));
    host.documents.ask_live(LiveRequest::Host);
    service(&mut host);
    let LiveRole::Hosting { code, addrs } = host.documents.live_view().role.clone() else {
        panic!();
    };
    let port = addrs[0].parse::<SocketAddr>().unwrap().port();
    let wrong: String = code
        .chars()
        .map(|c| {
            if c == 'A' {
                'B'
            } else if c == '-' {
                '-'
            } else {
                'A'
            }
        })
        .collect();
    let guest = std::thread::spawn(move || {
        let mut g = hero("Bia");
        g.documents.ask_live(LiveRequest::Join {
            addr: Some(format!("127.0.0.1:{port}")),
            code: wrong,
        });
        until(&mut g, 10, |g| {
            g.documents.live_view().role == LiveRole::Off && shapes(g) == 0
        });
        g.documents.live_view().role.clone()
    });
    let end = Instant::now() + Duration::from_secs(12);
    while !guest.is_finished() && Instant::now() < end {
        service(&mut host);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(guest.join().unwrap(), LiveRole::Off);
    assert_eq!(host.documents.live_view().peers.len(), 1);
    host.documents.ask_live(LiveRequest::Leave);
    service(&mut host);
}

/// ⭐ O quadro da sala SAI do anfitrião sem «End session» — a aba apagada, outro projecto carregado
/// (os ids recomeçam: o mesmo id seria OUTRO quadro), a janela fechada —, e a sala fecha como «End
/// session»: a votação aberta acaba e o resultado chega à Bia, e ela sai com o adeus — nunca passa por
/// «a religar» (antes: os fios cortavam-se no `Drop` e ela religava até a porta a recusar).
#[test]
fn the_room_closes_like_end_session_when_its_board_leaves_the_host() {
    type Leave = fn(&mut HeroScreen, ph2d_board_model::BoardId);
    let ways: [(&str, Leave); 3] = [
        ("apagar a aba", |h, id| {
            assert!(h.documents.remove(id).is_some());
        }),
        ("carregar outro projecto", |h, _| {
            let mut other = ph2d_board_model::BoardSet::default();
            let id = other.create("Outro".into());
            rect(other.get_mut(id).unwrap(), 0.0);
            h.documents.replace(other);
        }),
        ("fechar a janela", |h, _| shutdown(h)),
    ];
    for (how, leave) in ways {
        reset_for_tests();
        let mut ana = hero("Ana");
        let id = ana.documents.create("Retro".into());
        ana.documents.activate(Some(id));
        rect(ana.documents.board_mut(id).unwrap(), 0.0);
        let target = ana
            .documents
            .boards()
            .get(id)
            .unwrap()
            .doc
            .live_in_z_order()[0]
            .id;
        ana.documents.ask_live(LiveRequest::Host);
        service(&mut ana);
        let LiveRole::Hosting { code, addrs } = ana.documents.live_view().role.clone() else {
            panic!("{how}: não abriu");
        };
        let port = addrs[0].parse::<SocketAddr>().unwrap().port();
        let (tx, rx) = mpsc::channel::<()>();
        let bia = std::thread::spawn(move || {
            let mut g = hero("Bia");
            g.documents.ask_live(LiveRequest::Join {
                addr: Some(format!("127.0.0.1:{port}")),
                code,
            });
            assert!(until(&mut g, 10, |g| g
                .documents
                .live_view()
                .fac
                .vote
                .is_some()));
            let b = g.documents.live_view().board.unwrap();
            let _ = tx.send(());
            let mut religou = false;
            let off = until(&mut g, 10, |g| {
                religou |= g.documents.told() == Some(tr("board.live.reconnecting"));
                g.documents.live_view().role == LiveRole::Off
            });
            let polls = g.documents.boards().get(b).unwrap().team.polls.len();
            (off, religou, g.documents.told().map(str::to_owned), polls)
        });
        assert!(
            until(&mut ana, 10, |h| h.documents.live_view().peers.len() == 2),
            "{how}: a Bia não entrou"
        );
        ana.documents.ask_live(LiveRequest::VoteStart {
            setup: ph2d_board_model::VoteSetup::default(),
            targets: [target].into(),
        });
        let end = Instant::now() + Duration::from_secs(10);
        while rx.try_recv().is_err() && Instant::now() < end {
            service(&mut ana);
            std::thread::sleep(Duration::from_millis(5));
        }
        leave(&mut ana, id);
        service(&mut ana);
        assert_eq!(ana.documents.live_view().role, LiveRole::Off, "{how}");
        let (off, religou, told, polls) = bia.join().expect("a Bia falhou");
        assert!(off, "{how}: a Bia ficou na sala");
        assert!(
            !religou,
            "{how}: a Bia passou por «a religar» — sem o adeus"
        );
        assert_eq!(told.as_deref(), Some(tr("board.live.host_left")), "{how}");
        assert_eq!(polls, 1, "{how}: o fim da votação não chegou");
    }
}

/// O anfitrião CAI sem o adeus (o programa morreu) e abre logo outra sala: quem estava a voltar acaba
/// em segundos com a cópia (a porta recusa, ou ali já há outro código), e quem traz o código NOVO
/// entra sem «travado» — antes eram 60 s de tentativas com o código velho, e elas travavam a sala nova.
#[test]
fn a_vanished_host_ends_the_guest_soon_and_its_old_code_never_locks_the_next_room() {
    reset_for_tests();
    let open = |name: &str| {
        let mut h = hero(name);
        let id = h.documents.create("Retro".into());
        h.documents.activate(Some(id));
        rect(h.documents.board_mut(id).unwrap(), 0.0);
        h.documents.ask_live(LiveRequest::Host);
        service(&mut h);
        let LiveRole::Hosting { code, addrs } = h.documents.live_view().role.clone() else {
            panic!("não abriu");
        };
        let port = addrs[0].parse::<SocketAddr>().unwrap().port();
        (h, code, port)
    };
    let join = |code: String, port: u16, joined: mpsc::Sender<()>| {
        std::thread::spawn(move || {
            let mut g = hero("Bia");
            g.documents.ask_live(LiveRequest::Join {
                addr: Some(format!("127.0.0.1:{port}")),
                code,
            });
            let inside = until(&mut g, 10, |g| shapes(g) == 1);
            let _ = joined.send(());
            let t0 = Instant::now();
            let off = until(&mut g, 20, |g| {
                g.documents.live_view().role == LiveRole::Off
            });
            (
                inside,
                off.then(|| t0.elapsed()),
                g.documents.told().map(str::to_owned),
            )
        })
    };
    let (mut ana, code, port) = open("Ana");
    let (tx, rx) = mpsc::channel();
    let bia = join(code, port, tx);
    let end = Instant::now() + Duration::from_secs(10);
    while rx.try_recv().is_err() && Instant::now() < end {
        service(&mut ana);
        std::thread::sleep(Duration::from_millis(5));
    }
    // A queda: a sessão desta thread some sem o adeus (o transporte fecha os fios no Drop).
    reset_for_tests();
    drop(ana);
    let (mut ana2, code2, port2) = open("Ana");
    let (tx, _rx) = mpsc::channel();
    let caio_after = Instant::now() + Duration::from_secs(8);
    let mut caio = None;
    let end = Instant::now() + Duration::from_secs(25);
    while !(bia.is_finished()
        && caio
            .as_ref()
            .is_some_and(|c: &std::thread::JoinHandle<_>| c.is_finished()))
        && Instant::now() < end
    {
        service(&mut ana2);
        if caio.is_none() && Instant::now() > caio_after {
            // O Caio chega com o código NOVO depois de a Bia ter tido tempo de bater com o velho.
            caio = Some(std::thread::spawn({
                let (code2, tx) = (code2.clone(), tx.clone());
                move || {
                    let mut c = hero("Caio");
                    c.documents.ask_live(LiveRequest::Join {
                        addr: Some(format!("127.0.0.1:{port2}")),
                        code: code2,
                    });
                    let ok = until(&mut c, 10, |c| {
                        matches!(c.documents.live_view().role, LiveRole::Guest { .. })
                    });
                    let _ = tx.send(());
                    (ok, c.documents.told().map(str::to_owned))
                }
            }));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let (inside, ended, told) = bia.join().unwrap();
    assert!(inside, "a Bia não entrou");
    let ended = ended.expect("a Bia ficou a tentar voltar");
    assert!(
        ended < Duration::from_secs(6),
        "a Bia levou {ended:?} a saber"
    );
    assert_eq!(told.as_deref(), Some(tr("board.live.host_left")));
    let (ok, said) = caio.expect("o Caio não chegou").join().unwrap();
    assert!(ok, "o Caio, com o código certo, não entrou: {said:?}");
    ana2.documents.ask_live(LiveRequest::Leave);
    service(&mut ana2);
}

/// A voltar, o que diz que a sala JÁ NÃO É ESTA acaba a sessão; o que pode ser o Wi-Fi tenta outra
/// vez. (A via da porta recusada corre de ponta a ponta no teste acima; a do código errado precisa de
/// outra sala na MESMA porta, e numa máquina partilhada essa corrida não se ganha — decide-se aqui.)
#[test]
fn a_redial_ends_on_a_closed_or_replaced_room_and_retries_on_a_flaky_network() {
    use std::io::ErrorKind::{ConnectionRefused, HostUnreachable, NetworkUnreachable, TimedOut};
    for e in [
        net::JoinError::WrongCode,
        net::JoinError::Locked(Duration::from_secs(30)),
        net::JoinError::Unreachable(ConnectionRefused),
    ] {
        assert!(room_is_gone(&e), "{e:?}");
    }
    for e in [
        net::JoinError::Unreachable(TimedOut),
        net::JoinError::Unreachable(HostUnreachable),
        net::JoinError::Unreachable(NetworkUnreachable),
        net::JoinError::Broken,
    ] {
        assert!(!room_is_gone(&e), "{e:?}");
    }
}

/// 📏 Medição (à mão, `--ignored --nocapture`, perfil smoke): a latência ponta-a-ponta de um ARRASTO
/// com 2, 4 e 8 pessoas na rede local real — um convidado mexe uma forma a cada 16 ms (60 Hz) durante
/// 2 s; cada um dos outros regista quando viu cada posição. Resumo: mínimo, mediana, p95, máximo.
#[test]
#[ignore = "medição: corre-se à mão"]
fn measure_drag_latency_with_2_4_and_8_people() {
    for n in [2usize, 4, 8] {
        let mut host = hero("Ana");
        let id = host.documents.create("Medida".into());
        host.documents.activate(Some(id));
        rect(host.documents.board_mut(id).unwrap(), 0.0);
        host.documents.ask_live(LiveRequest::Host);
        service(&mut host);
        let LiveRole::Hosting { code, addrs } = host.documents.live_view().role.clone() else {
            panic!();
        };
        let port = addrs[0].parse::<SocketAddr>().unwrap().port();
        let sent: Arc<std::sync::Mutex<Vec<Instant>>> = Arc::default();
        let seen: Arc<std::sync::Mutex<Vec<Duration>>> = Arc::default();
        let go = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut threads = Vec::new();
        for g in 1..n {
            let (code, sent, seen, go) = (code.clone(), sent.clone(), seen.clone(), go.clone());
            threads.push(std::thread::spawn(move || {
                let mut h = hero(&format!("p{g}"));
                h.documents.ask_live(LiveRequest::Join {
                    addr: Some(format!("127.0.0.1:{port}")),
                    code,
                });
                assert!(until(&mut h, 15, |h| shapes(h) == 1));
                go.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                while go.load(std::sync::atomic::Ordering::SeqCst) < n {
                    service(&mut h);
                    std::thread::sleep(Duration::from_millis(1));
                }
                let b = h.documents.live_view().board.unwrap();
                let x_of = |h: &HeroScreen| {
                    h.documents
                        .boards()
                        .get(b)
                        .unwrap()
                        .doc
                        .live()
                        .next()
                        .map_or(0.0, |e| e.x)
                };
                let mut last = 0.0;
                let end = Instant::now() + Duration::from_secs(4);
                let mut k = 0;
                let mut next = Instant::now();
                while Instant::now() < end {
                    if g == 1 && k < 120 && Instant::now() >= next {
                        k += 1;
                        let bd = h.documents.board_mut(b).unwrap();
                        let mut el = bd.doc.live().next().unwrap().clone();
                        el.x = f64::from(k);
                        let _ = BoardOp::Put(el).apply(&mut bd.doc);
                        sent.lock().unwrap().push(Instant::now());
                        next += Duration::from_millis(16);
                    }
                    service(&mut h);
                    let x = x_of(&h);
                    if g != 1 && x > last {
                        let k = x as usize;
                        let t = Instant::now();
                        if let Some(s) = sent.lock().unwrap().get(k - 1) {
                            seen.lock().unwrap().push(t - *s);
                        }
                        last = x;
                    }
                    std::thread::sleep(Duration::from_micros(500));
                }
            }));
        }
        let end = Instant::now() + Duration::from_secs(30);
        let mut last = 0.0;
        while threads.iter().any(|t| !t.is_finished()) && Instant::now() < end {
            if go.load(std::sync::atomic::Ordering::SeqCst) == n - 1 {
                go.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            service(&mut host);
            // O anfitrião também observa (com 2 pessoas é o único).
            let x = host
                .documents
                .boards()
                .get(id)
                .unwrap()
                .doc
                .live()
                .next()
                .map_or(0.0, |e| e.x);
            if x > last {
                let t = Instant::now();
                if let Some(s) = sent.lock().unwrap().get(x as usize - 1) {
                    seen.lock().unwrap().push(t - *s);
                }
                last = x;
            }
            std::thread::sleep(Duration::from_micros(500));
        }
        for t in threads {
            t.join().unwrap();
        }
        let mut v = seen.lock().unwrap().clone();
        v.sort();
        let ms = |d: Duration| d.as_secs_f64() * 1e3;
        if v.is_empty() {
            eprintln!("{n} pessoas: nenhuma amostra");
        } else {
            eprintln!(
                "{n} pessoas: {} amostras · mín {:.2} ms · mediana {:.2} · p95 {:.2} · máx {:.2}",
                v.len(),
                ms(v[0]),
                ms(v[v.len() / 2]),
                ms(v[v.len() * 95 / 100]),
                ms(v[v.len() - 1])
            );
        }
        host.documents.ask_live(LiveRequest::Leave);
        service(&mut host);
    }
}
