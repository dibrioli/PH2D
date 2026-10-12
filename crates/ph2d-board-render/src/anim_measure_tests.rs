//! ⏱ W18 — **a régua dos GIF à vista** (o aberto (d) da W16: um fio por GIF, sem tecto). Duas variantes no
//! MESMO processo, intercaladas em rodadas, vale o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`):
//! `pool` = cada quadro é um passo no grupo de fios das imagens (W18); `fio_por_gif` = um fio dedicado por
//! GIF (o W16, reproduzido por `Images::gif_thread_each`). Para 1, 10, 50 e 200 GIFs: fios vivos, CPU,
//! RSS, o tempo do desenho e os quadros mostrados por GIF por segundo. Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-render --release --lib anim::measure -- --ignored --nocapture
//! ```

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_model::{Board, BoardOp, BoardSet, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::RenderCache;
use crate::image::Pixels;

const SIDE: [u16; 2] = [160, 120];
const FRAMES: usize = 24;
/// 40 ms por quadro (25 q/s, um GIF comum).
const DELAY_CS: u16 = 4;
const SECONDS: f64 = 3.0;
const ROUNDS: usize = 3;

/// Um GIF de [`FRAMES`] quadros inteiros, diferente para cada `seed` (o ficheiro é outro).
fn gif(seed: usize) -> Vec<u8> {
    let palette: Vec<u8> = (0..=255u8).flat_map(|i| [i, i.wrapping_mul(3), i.wrapping_mul(7)]).collect();
    let mut out = Vec::new();
    {
        let mut e = gif::Encoder::new(&mut out, SIDE[0], SIDE[1], &palette).unwrap();
        for k in 0..FRAMES {
            let buf: Vec<u8> = (0..usize::from(SIDE[0]) * usize::from(SIDE[1]))
                .map(|i| ((i / 7 + k * 11 + seed * 29) % 256) as u8)
                .collect();
            e.write_frame(&gif::Frame {
                width: SIDE[0],
                height: SIDE[1],
                buffer: Cow::Owned(buf),
                delay: DELAY_CS,
                ..gif::Frame::default()
            })
            .unwrap();
        }
    }
    out
}

/// Os quadros pela crate `gif` (o custo de descodificar de verdade; quadros inteiros, sem composição).
fn animator() -> crate::anim::Animate {
    Arc::new(|b: &[u8]| {
        let mut o = gif::DecodeOptions::new();
        o.set_color_output(gif::ColorOutput::RGBA);
        let mut d = o.read_info(std::io::Cursor::new(b.to_vec())).ok()?;
        let size = [u32::from(d.width()), u32::from(d.height())];
        Some(Box::new(std::iter::from_fn(move || {
            let f = d.read_next_frame().ok()??;
            Some((Pixels { size, rgba: f.buffer.to_vec() }, u32::from(f.delay) * 10))
        })) as Box<dyn Iterator<Item = (Pixels, u32)> + Send>)
    })
}

/// `n` GIFs numa grelha que cabe INTEIRA na vista de 1280×800 (a câmara no meio dela).
fn board(n: usize) -> Board {
    let mut set = BoardSet::default();
    let id = set.create("gifs".into());
    let b = set.get_mut(id).unwrap();
    let cols = ((n as f64) * 1.6).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    let (cw, ch) = (1280.0 / cols as f64, 800.0 / rows as f64);
    for k in 0..n {
        let file = b.put_file(ImageFile::new(gif(k), [u32::from(SIDE[0]), u32::from(SIDE[1])]));
        let img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
        let (x, y) = ((k % cols) as f64 * cw, (k / cols) as f64 * ch);
        let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, [x, y, cw * 0.95, ch * 0.9]);
        let _ = BoardOp::Put(el).apply(&mut b.doc);
    }
    let mut out = set.boards()[0].clone();
    out.camera = ph2d_board_model::Camera { center_x: 640.0, center_y: 400.0, zoom: 1.0 };
    out
}

fn proc_threads() -> usize {
    let s = std::fs::read_to_string("/proc/self/status").unwrap();
    s.lines().find_map(|l| l.strip_prefix("Threads:")).unwrap().trim().parse().unwrap()
}
fn proc_rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap();
    let kb: f64 = s.lines().find_map(|l| l.strip_prefix("VmRSS:")).unwrap().trim().trim_end_matches(" kB").trim().parse().unwrap();
    kb / 1024.0
}
fn proc_cpu_s() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap();
    let f: Vec<&str> = s.rsplit_once(") ").unwrap().1.split(' ').collect();
    (f[11].parse::<f64>().unwrap() + f[12].parse::<f64>().unwrap()) / 100.0
}

struct Run {
    threads: usize,
    cpu: f64,
    rss: f64,
    paint_med_ms: f64,
    /// Os GIF à vista (a tocar) — a câmara do quadro não mostra a grelha inteira.
    playing: usize,
    shown_per_gif_s: f64,
}

fn run(each: bool, b: &Board) -> Run {
    let mut c = RenderCache::default();
    c.images.set_decoder(Arc::new(|_: &[u8], _| Some(Pixels { size: [160, 120], rgba: vec![128; 160 * 120 * 4] })));
    c.images.set_animator(animator());
    if each {
        c.images.gif_thread_each = Some(BTreeMap::new());
    }
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let (t0, cpu0, rss0) = (Instant::now(), proc_cpu_s(), proc_rss_mb());
    let mut paints = Vec::new();
    let mut peak_threads = 0;
    while t0.elapsed().as_secs_f64() < SECONDS {
        let tick = Instant::now();
        let mut scene = VectorScene::new();
        crate::paint(b, [0.0, 0.0, 1280.0, 800.0], &mut scene, Theme::Forge, &mut ts, &mut c, &routes, None);
        paints.push(tick.elapsed().as_secs_f64() * 1e3);
        peak_threads = peak_threads.max(proc_threads());
        std::thread::sleep(Duration::from_millis(16).saturating_sub(tick.elapsed()));
    }
    let wall = t0.elapsed().as_secs_f64();
    paints.sort_by(f64::total_cmp);
    let r = Run {
        threads: peak_threads,
        cpu: (proc_cpu_s() - cpu0) / wall,
        rss: proc_rss_mb() - rss0,
        paint_med_ms: paints[paints.len() / 2],
        playing: c.images.players.playing_count(),
        shown_per_gif_s: c.images.players.shown_total() as f64
            / c.images.players.playing_count().max(1) as f64
            / wall,
    };
    drop(c);
    std::thread::sleep(Duration::from_millis(300));
    r
}

#[test]
#[ignore = "régua: corre à mão (cabeçalho)"]
fn measure_gif_players_pool_against_a_thread_each() {
    let base_threads = proc_threads();
    println!("load {:?} · fios no começo {base_threads} · {FRAMES} quadros {}×{} a 25 q/s", std::fs::read_to_string("/proc/loadavg").unwrap().trim(), SIDE[0], SIDE[1]);
    println!("{:>4} {:>8} {:>12} {:>6} {:>7} {:>8} {:>9} {:>11}", "GIFs", "a tocar", "variante", "fios", "CPU", "RSS MB", "desenho", "q/s por GIF");
    for n in [1, 10, 50, 200] {
        let b = board(n);
        let mut best: BTreeMap<&str, Run> = BTreeMap::new();
        for round in 0..ROUNDS {
            let order: [bool; 2] = if round % 2 == 0 { [false, true] } else { [true, false] };
            for each in order {
                let r = run(each, &b);
                let k = if each { "fio_por_gif" } else { "pool" };
                let e = best.entry(k).or_insert(Run { threads: usize::MAX, cpu: f64::MAX, rss: f64::MAX, paint_med_ms: f64::MAX, playing: 0, shown_per_gif_s: 0.0 });
                e.threads = e.threads.min(r.threads);
                e.cpu = e.cpu.min(r.cpu);
                e.rss = e.rss.min(r.rss);
                e.paint_med_ms = e.paint_med_ms.min(r.paint_med_ms);
                e.shown_per_gif_s = e.shown_per_gif_s.max(r.shown_per_gif_s);
                e.playing = e.playing.max(r.playing);
            }
        }
        for (k, r) in &best {
            println!("{n:>4} {:>8} {k:>12} {:>6} {:>6.2}n {:>8.1} {:>7.2}ms {:>11.1}", r.playing, r.threads, r.cpu, r.rss, r.paint_med_ms, r.shown_per_gif_s);
        }
    }
}

