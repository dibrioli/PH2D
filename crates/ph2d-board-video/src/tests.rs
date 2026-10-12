//! W18 — o tocador e a miniatura. Os que precisam do navegador real e da rede são `#[ignore]` (corre-os à
//! mão: `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release -- --ignored --nocapture`); um
//! deles é a régua do critério de morte do ADR-0185 (≥ 24 q/s a 560×315; clique → quadro ≤ 100 ms).

use std::time::{Duration, Instant};

use ph2d_board_model::Video;

use super::*;

/// Big Buck Bunny (Blender), o vídeo das medições de `youtube_motor/`.
fn bunny() -> Video {
    ph2d_board_embed::youtube("https://www.youtube.com/watch?v=aqz-KE-bpKQ").unwrap()
}

#[test]
fn the_player_page_carries_the_official_player_with_a_referrer_policy() {
    let p = player::page_for_test(&bunny());
    assert!(p.contains("https://www.youtube-nocookie.com/embed/aqz-KE-bpKQ?"));
    assert!(p.contains("referrerpolicy=\"strict-origin-when-cross-origin\""));
    assert!(p.contains("allow=\"autoplay; encrypted-media; fullscreen\""));
}

#[test]
fn a_screencast_jpeg_decodes_to_rgba() {
    // Um JPEG 2×1 mínimo feito pelo nosso codificador de imagens não existe aqui: usa-se a miniatura
    // gravada no repositório pelas medições.
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/MiroClone/ferramentas/youtube_motor/saidas/chromium_0.jpg"
    ))
    .unwrap();
    let f = decode_jpeg(&bytes).unwrap();
    assert_eq!(f.size, [560, 315]);
    assert_eq!(f.rgba.len(), 560 * 315 * 4);
}

/// A miniatura de verdade, pela rede (e a 2.ª vez pelo disco).
#[test]
#[ignore = "rede: corre à mão"]
fn fetches_a_real_thumbnail() {
    let t0 = Instant::now();
    let b = thumb::fetch(&bunny(), Duration::from_secs(10)).expect("a miniatura");
    let rede = t0.elapsed();
    let t1 = Instant::now();
    let again = thumb::fetch(&bunny(), Duration::from_secs(10)).unwrap();
    println!("miniatura: {} bytes, rede {:?}, disco {:?}", b.len(), rede, t1.elapsed());
    assert_eq!(b, again);
    let f = decode_jpeg(&b).unwrap();
    println!("tamanho {:?}", f.size);
}

/// ⭐⭐ A régua do critério de morte (ADR-0185): o tocador no navegador da pessoa dá ≥ 24 quadros
/// DISTINTOS por segundo a 560×315, e um clique (pausa) chega a um quadro em ≤ 100 ms.
#[test]
#[ignore = "navegador real + rede: corre à mão"]
fn the_person_s_browser_plays_at_24_fps_and_answers_a_click() {
    let mut p = Player::start(&bunny(), [560, 315]);
    let t0 = Instant::now();
    while p.state() == PlayerState::Starting {
        assert!(t0.elapsed() < Duration::from_secs(20), "não arrancou");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(p.state(), PlayerState::Playing);
    // 6 s para o vídeo começar, depois conta 5 s.
    std::thread::sleep(Duration::from_secs(6));
    let _ = p.take_frame();
    let (t1, mut n, mut last) = (Instant::now(), 0u32, Vec::new());
    while t1.elapsed() < Duration::from_secs(5) {
        if let Some(f) = p.take_frame()
            && f.rgba != last
        {
            n += 1;
            last = f.rgba;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    let fps = f64::from(n) / t1.elapsed().as_secs_f64();
    // O clique no meio (pausa): o 1.º quadro novo depois dele.
    let _ = p.take_frame();
    let tc = Instant::now();
    for m in [
        Mouse::Down { x: 280.0, y: 157.0, button: Button::Left, clicks: 1 },
        Mouse::Up { x: 280.0, y: 157.0, button: Button::Left, clicks: 1 },
    ] {
        p.mouse(m);
    }
    let mut answer = None;
    while tc.elapsed() < Duration::from_secs(2) {
        if p.take_frame().is_some() {
            answer = Some(tc.elapsed());
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    println!("quadros distintos {fps:.2}/s · clique → quadro {answer:?} · load {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());
    assert!(fps >= 24.0, "{fps:.2} q/s < 24");
    assert!(answer.is_some_and(|d| d <= Duration::from_millis(100)), "{answer:?}");
}
