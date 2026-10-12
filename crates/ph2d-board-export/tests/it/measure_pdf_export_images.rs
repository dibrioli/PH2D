//! ⏱ W17 — **a régua do PDF com a mesma fotografia em N frames**: antes, cada página tinha a sua
//! cache e descodificava a fotografia outra vez; depois, as páginas de um export partilham as imagens
//! ([`ExportCache`]). As duas variantes no MESMO processo, rodadas intercaladas, vale o MÍNIMO. Corre à
//! mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-export --release --test it measure_pdf_export_images -- --ignored --nocapture
//! ```

use std::sync::Arc;
use std::time::Instant;

use ph2d_board_export::{ExportCache, ExportTarget, paint_page, pdf, pdf_pages};
use ph2d_board_model::{Board, BoardOp, BoardSet, Element, Frame, Image, ImageFile, Rgba, Style};
use ph2d_board_render::{EXPORT_PAPER, Pixels};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;

const ROUNDS: usize = 5;
const FRAMES: u32 = 6;

fn install() {
    ph2d_board_render::install_decoder(Arc::new(|b: &[u8], _at: Option<[u32; 2]>| {
        let img = image::load_from_memory(b).ok()?.to_rgba8();
        Some(Pixels {
            size: [img.width(), img.height()],
            rgba: img.into_raw(),
        })
    }));
}

/// Uma fotografia 4000 × 3000 (ruído suave, JPEG 85).
fn photo() -> Vec<u8> {
    let img = image::RgbImage::from_fn(4000, 3000, |x, y| {
        image::Rgb([
            (x * 7 % 256) as u8,
            (y * 5 % 256) as u8,
            ((x ^ y) % 256) as u8,
        ])
    });
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 85)
        .encode_image(&img)
        .unwrap();
    out
}

/// [`FRAMES`] frames lado a lado, a mesma fotografia em cada.
fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("Moodboard".into());
    let b = set.get_mut(id).unwrap();
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    let file = b.put_file(ImageFile::new(photo(), [4000, 3000]));
    for k in 0..FRAMES {
        let x = f64::from(k) * 1000.0;
        let f = Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Frame {
                name: format!("F{k}"),
                style: st.clone(),
            },
            [x, 0.0, 900.0, 700.0],
        );
        let fid = f.id;
        BoardOp::Put(f).apply(&mut b.doc);
        let mut el = Element::new_image(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Image::new(file, st.clone()),
            [x + 50.0, 50.0, 800.0, 600.0],
        );
        el.frame = Some(fid);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    set.boards()[0].clone()
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_pdf_export_images() {
    install();
    let b = board();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let pages = pdf_pages(&b, &routes, &[]);
    assert_eq!(pages.len(), FRAMES as usize);
    let mut ts = TextSystem::without_system_fonts();
    let names = ["antes (uma cache por página)", "depois (uma por export)"];
    let mut t = [Vec::new(), Vec::new()];
    let mut len = [0usize; 2];
    for round in 0..ROUNDS {
        for k in 0..2 {
            let k = if round % 2 == 0 { k } else { 1 - k };
            let at = Instant::now();
            let mut shared = ExportCache::default();
            let recs: Vec<_> = pages
                .iter()
                .map(|p| {
                    let mut fresh = ExportCache::default();
                    let cache = if k == 0 { &mut fresh } else { &mut shared };
                    let (mut s, size) = paint_page(
                        &b,
                        p,
                        1.0,
                        Some(EXPORT_PAPER),
                        ExportTarget::Pdf,
                        cache,
                        &mut ts,
                    );
                    (s.take_record(), size)
                })
                .collect();
            let list: Vec<pdf::PdfPage<'_>> = recs
                .iter()
                .map(|(ops, size)| pdf::PdfPage {
                    ops,
                    size: *size,
                    links: Vec::new(),
                })
                .collect();
            let out = pdf::write(&list);
            t[k].push(at.elapsed().as_secs_f64() * 1e3);
            len[k] = out.len();
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\nloadavg: {} · {FRAMES} frames", load.trim());
    for (k, name) in names.iter().enumerate() {
        t[k].sort_by(f64::total_cmp);
        println!(
            "{name:>30}: {:9.1} ms mínimo [{:9.1} mediana] · PDF {} B",
            t[k][0],
            t[k][ROUNDS / 2],
            len[k]
        );
    }
    assert_eq!(len[0], len[1], "o mesmo PDF pelos dois caminhos");
}
