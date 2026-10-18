//! B8 (W23) — 📏 o que um ARRASTO manda pela internet: um retransmissor UDP no meio conta os
//! datagramas e os bytes de cada sentido; as variantes (escritas a cada imagem × agrupadas) correm
//! intercaladas no MESMO processo. À mão: `--ignored --nocapture`, perfil smoke.

use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::*;
use crate::live::{internet_local_for_tests, reset_for_tests, service};
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, BoardOp, Element, Rgba, RichText, Shape, ShapeType, Style};
use ph2d_editor_core::documents::live::LiveRequest;

fn hero(name: &str) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.documents.set_author(name.into());
    h
}

fn rect(b: &mut Board) {
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
        [0.0, 0.0, 10.0, 10.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
}

fn until(h: &mut HeroScreen, secs: u64, mut done: impl FnMut(&mut HeroScreen) -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        service(h);
        if done(h) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    false
}

/// [datagramas, bytes] de cada sentido, contados só com `on`.
#[derive(Default)]
struct Count {
    on: AtomicBool,
    up: [AtomicU64; 2],
    down: [AtomicU64; 2],
    /// Os tamanhos vistos em cada sentido (quantos de cada).
    sizes: Mutex<[std::collections::BTreeMap<usize, u32>; 2]>,
}

/// Retransmite tudo entre o convidado (quem não é o anfitrião) e o anfitrião, e conta.
fn relay(host: SocketAddr, count: Arc<Count>, stop: Arc<AtomicBool>) -> SocketAddr {
    let s = UdpSocket::bind("127.0.0.1:0").unwrap();
    s.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
    let me = s.local_addr().unwrap();
    std::thread::spawn(move || {
        let mut guest = None;
        let mut buf = vec![0u8; 65_536];
        while !stop.load(SeqCst) {
            let Ok((n, from)) = s.recv_from(&mut buf) else {
                continue;
            };
            let (to, c) = if from == host {
                (guest, &count.down)
            } else {
                guest = Some(from);
                (Some(host), &count.up)
            };
            if count.on.load(SeqCst) {
                *count.sizes.lock().unwrap()[usize::from(from == host)]
                    .entry(n)
                    .or_default() += 1;
                c[0].fetch_add(1, SeqCst);
                c[1].fetch_add(n as u64, SeqCst);
            }
            if let Some(to) = to {
                let _ = s.send_to(&buf[..n], to);
            }
        }
    });
    me
}

fn x_of(h: &HeroScreen) -> f64 {
    h.documents
        .live_view()
        .board
        .and_then(|id| h.documents.boards().get(id))
        .and_then(|b| b.doc.live().next().map(|e| e.x))
        .unwrap_or(0.0)
}

const MOVES: u32 = 120;

fn round(every: u64) -> String {
    reset_for_tests();
    internet_local_for_tests();
    internet_pace_for_tests(every);
    let mut ana = hero("Ana");
    let id = ana.documents.create("Medida".into());
    ana.documents.activate(Some(id));
    rect(ana.documents.board_mut(id).unwrap());
    ana.documents.ask_live(LiveRequest::Host);
    service(&mut ana);
    ana.documents.ask_live(LiveRequest::InviteInternet);
    let mut link = None;
    assert!(until(&mut ana, 10, |h| {
        link = link.take().or_else(|| h.store.take_clipboard_copy());
        link.is_some()
    }));
    let mut invite = net::Invite::parse(&link.unwrap()).unwrap();
    let host = *invite
        .addrs
        .iter()
        .find(|a| a.ip().is_loopback() || a.is_ipv4())
        .unwrap();
    let host = SocketAddr::from(([127, 0, 0, 1], host.port()));
    let (count, stop) = (Arc::new(Count::default()), Arc::new(AtomicBool::new(false)));
    invite.addrs = vec![relay(host, Arc::clone(&count), Arc::clone(&stop))];
    let link = invite.link();
    let sent: Arc<Mutex<Vec<Instant>>> = Arc::default();
    let (go, done) = (
        Arc::new(AtomicBool::new(false)),
        Arc::new(AtomicBool::new(false)),
    );
    let bia = {
        let (sent, go, done, count) = (sent.clone(), go.clone(), done.clone(), count.clone());
        std::thread::spawn(move || {
            reset_for_tests();
            internet_local_for_tests();
            internet_pace_for_tests(every);
            let mut h = hero("Bia");
            h.documents.ask_live(LiveRequest::JoinLink { link });
            assert!(until(&mut h, 20, |h| h
                .documents
                .live_view()
                .board
                .is_some()));
            go.store(true, SeqCst);
            // Os dois lados assentam (o retrato, as primeiras presenças) antes de contar.
            until(&mut h, 1, |_| false);
            let b = h.documents.live_view().board.unwrap();
            count.on.store(true, SeqCst);
            let mut next = Instant::now();
            let mut k = 0;
            while k < MOVES {
                if Instant::now() >= next {
                    k += 1;
                    let bd = h.documents.board_mut(b).unwrap();
                    let mut el = bd.doc.live().next().unwrap().clone();
                    el.x = f64::from(k);
                    let _ = BoardOp::Put(el).apply(&mut bd.doc);
                    sent.lock().unwrap().push(Instant::now());
                    next += Duration::from_millis(16);
                }
                service(&mut h);
                std::thread::sleep(Duration::from_micros(500));
            }
            // O que ficou agrupado sai, e chega.
            until(&mut h, 1, |_| false);
            count.on.store(false, SeqCst);
            done.store(true, SeqCst);
            until(&mut h, 1, |_| false);
            h.documents.ask_live(LiveRequest::Leave);
            service(&mut h);
        })
    };
    let (mut last, mut seen, mut lat) = (0.0, 0u32, Vec::new());
    let t0 = Instant::now();
    while !bia.is_finished() && t0.elapsed() < Duration::from_secs(40) {
        service(&mut ana);
        let x = x_of(&ana);
        if x > last {
            seen += 1;
            if let Some(s) = sent.lock().unwrap().get(x as usize - 1) {
                lat.push(s.elapsed());
            }
            last = x;
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    bia.join().unwrap();
    stop.store(true, SeqCst);
    ana.documents.ask_live(LiveRequest::Leave);
    service(&mut ana);
    let _ = (go, done);
    lat.sort();
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let secs = f64::from(MOVES) * 0.016;
    let per_s = |a: &AtomicU64| a.load(SeqCst) as f64 / secs;
    let sizes = count.sizes.lock().unwrap();
    format!(
        "tamanhos sobe {:?} desce {:?}\n   escritas a cada {every:>2} ms · sobe {:>5.1} dgr/s {:>7.0} B/s · desce {:>5.1} dgr/s {:>7.0} B/s · \
         o anfitrião viu {seen} posições, a última {last} · latência mediana {:.1} ms p95 {:.1}",
        sizes[0],
        sizes[1],
        per_s(&count.up[0]),
        per_s(&count.up[1]),
        per_s(&count.down[0]),
        per_s(&count.down[1]),
        lat.get(lat.len() / 2).map_or(f64::NAN, |d| ms(*d)),
        lat.get(lat.len() * 95 / 100).map_or(f64::NAN, |d| ms(*d)),
    )
}

#[test]
#[ignore = "medição: corre-se à mão"]
fn measure_what_a_drag_sends_over_the_internet() {
    for every in [
        0,
        INTERNET_WRITE_EVERY_MS,
        0,
        INTERNET_WRITE_EVERY_MS,
        0,
        INTERNET_WRITE_EVERY_MS,
    ] {
        eprintln!("B8 {}", round(every));
    }
}
