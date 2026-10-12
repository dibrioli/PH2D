//! ⏱ **A RÉGUA da saída** (W8, `docs/MiroClone/06_equipa_e_saida.md` §4.2), antes de qualquer tecto:
//! num quadro de 10 mil e de 100 mil notas (as 16 cores, uma frase cada), o quadro inteiro —
//!
//! - **gravar** a página no modo vector (o `paint` com a gravação ligada) e escrever o **SVG** e o **PDF**;
//! - o **PNG** pela placa a 4 · 16 · 49 MP (o nível de detalhe do ecrã à escala, em ladrilhos do tecto
//!   do dispositivo);
//! - escrever e ler o **`.excalidraw`**;
//! - **guardar uma versão** pelo caminho real (`Board::save_version`) e lê-la.
//!
//! Variantes no MESMO processo, rodadas intercaladas, vale o MÍNIMO (mediana ao lado). Corre à mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-board-export --release --test it measure_export -- --ignored --nocapture
//! ```

use std::time::Instant;

use ph2d_board_export::{
    ExportTarget, PNG_MEGAPIXELS, board_page, paint_page, pdf, png, png_scale, svg,
};
use ph2d_board_model::{Board, BoardSet, Rgba, STICKY_COLORS, ShapeType};
use ph2d_board_render::EXPORT_PAPER;
use ph2d_board_route::RouteCache;
use ph2d_gpu::{GpuContext, TextureFormat};
use ph2d_render::VelloPass;
use ph2d_text::TextSystem;

use crate::sample::{put, shape};

const ROUNDS: usize = 3;
const PHRASES: [&str; 4] = [
    "Faster builds for the team",
    "Coffee machine upstairs",
    "Slow reviews block the release",
    "Pairing on the hard bugs",
];

fn notes(n: usize) -> Board {
    let mut set = BoardSet::default();
    let a = set.create(format!("{n}"));
    let b = set.get_mut(a).unwrap();
    let side = (n as f64).sqrt().ceil() as usize;
    for i in 0..n {
        let (r, c) = ((i / side) as f64, (i % side) as f64);
        let mut el = shape(
            b,
            ShapeType::Sticky,
            PHRASES[i % 4],
            [c * 230.0, r * 230.0, 199.0, 199.0],
        );
        if let Some(s) = el.shape_mut() {
            s.style.fill = Some(Rgba(STICKY_COLORS[i % 16]));
        }
        put(b, el);
    }
    b.clone()
}

fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn time<T>(f: impl FnOnce() -> T) -> (f64, T) {
    let t = Instant::now();
    let r = f();
    (t.elapsed().as_secs_f64() * 1e3, r)
}

#[test]
#[ignore = "régua: corre-se à mão em --release com a placa (ver o cabeçalho)"]
fn measure_export_cost() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador");
    let mut pass = VelloPass::new(&gpu, TextureFormat::Rgba8Unorm, (64, 64)).expect("VelloPass");
    let tile = png::tile_side(&gpu);
    println!("\ntecto de textura do dispositivo: {tile} px");
    let mut ts = TextSystem::without_system_fonts();
    for n in [10_000usize, 100_000] {
        let board = notes(n);
        let mut routes = RouteCache::default();
        routes.sync(&board.doc);
        let page = board_page(&board, &routes).unwrap();
        let names = [
            "gravar", "svg", "pdf", "png 4", "png 16", "png 49", "exc w", "exc r", "versão",
            "ler v",
        ];
        let mut t: Vec<Vec<f64>> = vec![Vec::new(); names.len()];
        let mut bytes = vec![0usize; names.len()];
        for _ in 0..ROUNDS {
            let (ms, (mut scene, size)) = time(|| {
                paint_page(
                    &board,
                    &page,
                    1.0,
                    Some(EXPORT_PAPER),
                    ExportTarget::Pdf,
                    &std::collections::BTreeSet::new(),
                    &mut ts,
                )
            });
            t[0].push(ms);
            let ops = scene.take_record();
            bytes[0] = ops.len();
            let (ms, s) = time(|| svg::write(&ops, size));
            t[1].push(ms);
            bytes[1] = s.len();
            let (ms, p) = time(|| {
                pdf::write(&[pdf::PdfPage {
                    ops: &ops,
                    size,
                    links: Vec::new(),
                }])
            });
            t[2].push(ms);
            bytes[2] = p.len();
            for (k, mp) in PNG_MEGAPIXELS.into_iter().enumerate() {
                let sc = png_scale(&page, mp);
                let (ms, img) = time(|| {
                    png::render(
                        &gpu,
                        &mut pass,
                        &board,
                        &page,
                        sc,
                        Some(EXPORT_PAPER),
                        tile,
                        &std::collections::BTreeSet::new(),
                        &mut ts,
                    )
                    .unwrap()
                });
                t[3 + k].push(ms);
                bytes[3 + k] = (img.width * img.height) as usize;
            }
            let (ms, x) = time(|| ph2d_board_excalidraw::write(&board));
            t[6].push(ms);
            bytes[6] = x.len();
            let (ms, back) = time(|| ph2d_board_excalidraw::read(&x).unwrap());
            t[7].push(ms);
            bytes[7] = back.doc.live_len();
            let mut b2 = board.clone();
            let (ms, ()) = time(|| b2.save_version("v".into(), 0));
            t[8].push(ms);
            bytes[8] = b2.team.versions[0].bytes.len();
            let (ms, _) = time(|| b2.team.versions[0].read().unwrap());
            t[9].push(ms);
        }
        println!("== {n} notas (ms mínimo [mediana] · tamanho) ==");
        for (k, name) in names.iter().enumerate() {
            let (m, d) = min_med(&mut t[k]);
            println!("{name:>8}: {m:9.1} [{d:9.1}] · {}", bytes[k]);
        }
    }
}
