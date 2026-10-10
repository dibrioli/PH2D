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
