//! W18 — o tocador e a miniatura. Os que precisam do navegador real e da rede são `#[ignore]` (corre-os à
//! mão: `bash scripts/ph2d-run.sh cargo test -p ph2d-board-video --release -- --ignored --nocapture`); um
//! deles é a régua do critério de morte do ADR-0185 (≥ 24 q/s a 560×315; clique → quadro ≤ 100 ms).

use std::time::{Duration, Instant};

use ph2d_board_model::Video;
use serde_json::json;

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
    // W19: sem a permissão de ecrã inteiro (o nosso ecrã inteiro é do PH2D; o do tocador deformava).
    assert!(p.contains("allow=\"autoplay; encrypted-media\""));
    assert!(!p.contains("fullscreen"));
    // A página fala com o tocador pela API oficial e passa o estado ao PH2D.
    assert!(p.contains("event:'listening'") && p.contains("self.ph2dState(e.data)"));
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

/// ⭐ Os ids que o leitor manda ao CDP são inteiros de 32 bits e distintos, também ao dar a volta (a
/// mutação `1 << 40` sobrevivia: o `min` antigo escondia-a repetindo `i32::MAX`).
#[test]
fn reader_ids_stay_32_bit_and_distinct_across_the_wrap() {
    use std::sync::atomic::AtomicU64;
    let start = player::READER_IDS;
    let end = player::READER_IDS_END;
    assert!(start < end && end <= i32::MAX as u64);
    let ids = AtomicU64::new(start);
    let a: Vec<u64> = (0..5).map(|_| player::next_reader_id(&ids)).collect();
    assert_eq!(a, (start..start + 5).collect::<Vec<_>>());
    let ids = AtomicU64::new(end - 2);
    let b: Vec<u64> = (0..4).map(|_| player::next_reader_id(&ids)).collect();
    assert_eq!(
        b,
        vec![end - 2, end - 1, start, start + 1],
        "dá a volta dentro da faixa"
    );
}

/// O perfil que um PH2D morto deixou varre-se; o de um processo vivo (este) e o que não é nosso ficam.
#[test]
fn stale_browser_profiles_of_dead_processes_are_swept() {
    let dir = std::env::temp_dir().join(format!("ph2d-video-sweep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Um pid acima do máximo do Linux (4 194 304) nunca existe.
    let dead = dir.join("ph2d-cdp-4294967-1");
    let alive = dir.join(format!("ph2d-cdp-{}-2", std::process::id()));
    let other = dir.join("outra-coisa");
    for d in [&dead, &alive, &other] {
        std::fs::create_dir_all(d).unwrap();
    }
    browser::sweep_stale_profiles(&dir);
    assert!(!dead.exists(), "o do morto fica");
    assert!(alive.exists() && other.exists());
    let _ = std::fs::remove_dir_all(&dir);
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
    println!(
        "miniatura: {} bytes, rede {:?}, disco {:?}",
        b.len(),
        rede,
        t1.elapsed()
    );
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
        Mouse::Down {
            x: 280.0,
            y: 157.0,
            button: Button::Left,
            clicks: 1,
        },
        Mouse::Up {
            x: 280.0,
            y: 157.0,
            button: Button::Left,
            clicks: 1,
        },
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
    println!(
        "quadros distintos {fps:.2}/s · clique → quadro {answer:?} · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    assert!(fps >= 24.0, "{fps:.2} q/s < 24");
    assert!(
        answer.is_some_and(|d| d <= Duration::from_millis(100)),
        "{answer:?}"
    );
}

/// Arranca o tocador e espera que toque (`secs` de vídeo).
fn playing(size: [u32; 2], secs: u64) -> Player {
    let p = Player::start(&bunny(), size);
    let t0 = Instant::now();
    while p.state() == PlayerState::Starting {
        assert!(t0.elapsed() < Duration::from_secs(20), "não arrancou");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(p.state(), PlayerState::Playing);
    std::thread::sleep(Duration::from_secs(secs));
    p
}

fn click(p: &Player, x: f64, y: f64) {
    p.mouse(Mouse::Move { x, y });
    std::thread::sleep(Duration::from_millis(400));
    for m in [
        Mouse::Down {
            x,
            y,
            button: Button::Left,
            clicks: 1,
        },
        Mouse::Up {
            x,
            y,
            button: Button::Left,
            clicks: 1,
        },
    ] {
        p.mouse(m);
    }
}

/// W19 (smoke do dono, defeito 1): o botão de ecrã inteiro DO TOCADOR (no canto de baixo à direita, onde o
/// tocador do YouTube o pinta a 560×315) não pode mudar a forma dos quadros que chegam.
#[test]
#[ignore = "navegador real + rede: corre à mão"]
fn the_player_s_fullscreen_corner_never_changes_the_frame_shape() {
    let mut p = playing([560, 315], 6);
    click(&p, 511.0, 210.0);
    let t = Instant::now();
    let mut sizes = std::collections::BTreeSet::new();
    let mut last = None;
    while t.elapsed() < Duration::from_secs(3) {
        if let Some(f) = p.take_frame() {
            sizes.insert(f.size);
            last = Some(f);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if let (Some(f), Some(dir)) = (last, std::env::var_os("PH2D_W19_FRAME")) {
        let mut ppm = format!("P6 {} {} 255\n", f.size[0], f.size[1]).into_bytes();
        ppm.extend(f.rgba.chunks_exact(4).flat_map(|c| [c[0], c[1], c[2]]));
        let _ = std::fs::write(std::path::Path::new(&dir).join("fullscreen.ppm"), ppm);
    }
    println!("tamanhos dos quadros depois do clique: {sizes:?}");
    assert_eq!(
        sizes.into_iter().collect::<Vec<_>>(),
        vec![[560, 315]],
        "o ecrã inteiro do tocador mudou a forma dos quadros"
    );
}

/// A luz média de um quadro (0 = preto).
fn luma(f: &Frame) -> f64 {
    let px = f.rgba.chunks_exact(4);
    let n = px.len().max(1) as f64;
    px.map(|c| 0.299 * f64::from(c[0]) + 0.587 * f64::from(c[1]) + 0.114 * f64::from(c[2]))
        .sum::<f64>()
        / n
}

/// ⭐⭐ W19 (smoke do dono, defeito 3): clicar no nome do CANAL dentro do tocador não o deixa preto — o
/// vídeo continua a tocar, com quadros novos e com luz.
#[test]
#[ignore = "navegador real + rede: corre à mão"]
fn clicking_the_channel_keeps_the_player_playing_and_lit() {
    let mut p = playing([560, 315], 6);
    // O nome do canal, por baixo do título (medido: `youtube_motor/saidas/w19b`, 78..380 × 38..51).
    click(&p, 100.0, 44.0);
    std::thread::sleep(Duration::from_millis(500));
    let _ = p.take_frame();
    let t = Instant::now();
    let (mut n, mut last, mut lit) = (0u32, Vec::new(), 0.0);
    while t.elapsed() < Duration::from_secs(3) {
        if let Some(f) = p.take_frame() {
            lit = luma(&f);
            if f.rgba != last {
                n += 1;
                last = f.rgba;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    println!("depois do clique no canal: {n} quadros distintos em 3 s, luz {lit:.1}");
    assert!(n >= 30, "o tocador parou: {n} quadros em 3 s");
    assert!(lit > 20.0, "o tocador ficou preto: luz {lit:.1}");
    let out = p.take_outside();
    assert!(
        out.len() == 1 && out[0].starts_with("https://www.youtube.com/channel/"),
        "o canal abre-se UMA vez no navegador do sistema: {out:?}"
    );
}

/// ⭐⭐ W19 (smoke do dono, defeito 2): um zoom contínuo 1×→2×→1× em 5 s com o vídeo a tocar — o tamanho
/// do tocador muda a cada quadro do app — não o pausa, e os quadros continuam a chegar.
#[test]
#[ignore = "navegador real + rede: corre à mão"]
fn a_continuous_zoom_never_pauses_the_player() {
    let mut p = playing([560, 315], 6);
    let t = Instant::now();
    let (mut n, mut last, mut paused) = (0u32, Vec::new(), 0u32);
    let mut was = p.playback().phase;
    while t.elapsed() < Duration::from_secs(5) {
        let k = 1.0 + (std::f64::consts::PI * t.elapsed().as_secs_f64() / 5.0).sin();
        p.resize([(560.0 * k).round() as u32, (315.0 * k).round() as u32]);
        if let Some(f) = p.take_frame()
            && f.rgba != last
        {
            n += 1;
            last = f.rgba;
        }
        let now = p.playback().phase;
        if now != was && !now.running() {
            paused += 1;
        }
        was = now;
        std::thread::sleep(Duration::from_millis(16));
    }
    println!("zoom de 5 s: {n} quadros distintos, {paused} pausas, fase {was:?}");
    assert_eq!(paused, 0, "o zoom pausou o vídeo");
    assert_eq!(was, Phase::Playing);
    assert!(n >= 120, "{n} quadros distintos em 5 s");
}

/// ⭐⭐ W19 — os NOSSOS controles: cada ordem chega ao tocador e volta no estado dele (a API oficial).
#[test]
#[ignore = "navegador real + rede: corre à mão"]
fn our_controls_reach_the_player_and_come_back_in_its_state() {
    let p = playing([560, 315], 6);
    let wait = |what: &str, ok: &dyn Fn(&Playback) -> bool| {
        let t = Instant::now();
        while !ok(&p.playback()) {
            assert!(
                t.elapsed() < Duration::from_secs(3),
                "{what}: {:?}",
                p.playback()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        println!("{what}: {:?}", t.elapsed());
    };
    wait("o estado chega", &|b| {
        b.duration > 600.0 && b.phase == Phase::Playing
    });
    // O que se ouve DEPOIS de `t` veio do tocador (as ordens escrevem o estado na hora, para o botão).
    let fresh = |t: Instant| move |b: &Playback| b.heard_at.is_some_and(|r| r > t);
    let t = Instant::now();
    p.set_muted(true);
    p.seek(120.0);
    p.set_volume(30);
    p.set_muted(true);
    wait("o tocador confirma o salto, o volume e o mudo", &|b| {
        fresh(t + Duration::from_millis(300))(b)
            && (119.0..125.0).contains(&b.time)
            && b.volume == 30
            && b.muted
    });
    let t = Instant::now();
    p.pause();
    assert_eq!(
        p.playback().shown(Instant::now()),
        Phase::Paused,
        "o botão muda na hora"
    );
    assert!(t.elapsed() < Duration::from_millis(100));
    wait("pausado", &|b| b.phase == Phase::Paused);
    p.play();
    wait("a tocar", &|b| b.phase == Phase::Playing);
}

/// O estado lê-se das mensagens do tocador (as formas medidas: `youtube_motor/saidas/w19b`), e uma ordem
/// manda no que se mostra até ele a confirmar.
#[test]
fn the_playback_state_reads_the_players_messages() {
    let t0 = Instant::now();
    let mut b = Playback::default();
    b.apply(
        &json!({"event":"infoDelivery","info":{"playerState":1,"currentTime":7.5,"duration":635,
            "volume":80,"muted":false,"playbackRate":1}}),
        t0,
    );
    assert_eq!(
        (b.phase, b.time, b.duration, b.volume),
        (Phase::Playing, 7.5, 635.0, 80)
    );
    let later = t0 + Duration::from_millis(500);
    assert!(
        (b.now(later) - 8.0).abs() < 1e-9,
        "o tempo anda entre as mensagens"
    );
    b.want = Some((Phase::Paused, t0));
    assert_eq!(b.shown(later), Phase::Paused, "a pausa pedida mostra-se já");
    assert!((b.now(later) - 7.5).abs() < 1e-9, "pausado, o tempo pára");
    b.apply(
        &json!({"event":"infoDelivery","info":{"playerState":1}}),
        later,
    );
    assert_eq!(
        b.shown(later),
        Phase::Paused,
        "uma mensagem velha não a desfaz"
    );
    b.apply(&json!({"event":"onStateChange","info":2}), later);
    assert_eq!((b.phase, b.want), (Phase::Paused, None), "confirmada");
    assert_eq!(b.shown(t0 + Duration::from_secs(5)), Phase::Paused);
    b.apply(
        &json!({"event":"infoDelivery","info":{"currentTime":700.0}}),
        later,
    );
    b.phase = Phase::Playing;
    assert_eq!(
        b.now(later + Duration::from_secs(9)),
        635.0,
        "nunca passa do fim"
    );
}
