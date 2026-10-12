//! ⏱ W16 — **a régua do export SVG com imagens**: antes, a gravação do SVG era a do PDF (descodifica
//! cada imagem — um SVG rasterizado aos 300 ppp — e o SVG só usava o tamanho do nível); depois, o
//! `ExportTarget::Svg` grava os bytes sem descodificar. As duas variantes no MESMO processo, rodadas
//! intercaladas, vale o MÍNIMO. Corre à mão:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-board-export --release --test it measure_svg_export_images -- --ignored --nocapture
//! ```

use std::sync::Arc;
use std::time::Instant;

use ph2d_board_export::{ExportTarget, board_page, paint_page, svg};
use ph2d_board_model::{Board, BoardOp, BoardSet, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_render::{EXPORT_PAPER, Pixels};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;

const ROUNDS: usize = 5;

fn install() {
    ph2d_board_render::install_decoder(Arc::new(|b: &[u8], at: Option<[u32; 2]>| {
        if ph2d_board_model::mime_of(b) == Some("image/svg+xml") {
            let tree = ph2d_imageio_svg::raster::parse(b).ok()?;
            let size = at.unwrap_or_else(|| ph2d_imageio_svg::raster::natural_size(&tree));
            let rgba = ph2d_imageio_svg::raster::rasterize(&tree, size)?;
            return Some(Pixels { size, rgba });
        }
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

/// Um desenho 1000 × 1000 com 2 000 círculos.
fn drawing() -> Vec<u8> {
    let mut s = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1000\" height=\"1000\" viewBox=\"0 0 1000 1000\">",
    );
    for i in 0..2000u32 {
        let (x, y) = ((i * 37) % 1000, (i * 91) % 1000);
        s.push_str(&format!(
            "<circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"#{:06x}\"/>",
            5 + i % 40,
            i * 2654435761 % 0xff_ffff
        ));
    }
    s.push_str("</svg>");
    s.into_bytes()
}

fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("Imagens".into());
    let b = set.get_mut(id).unwrap();
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for (bytes, size, bx) in [
        (photo(), [4000, 3000], [0.0, 0.0, 800.0, 600.0]),
        (drawing(), [1000, 1000], [900.0, 0.0, 400.0, 400.0]),
    ] {
        let file = b.put_file(ImageFile::new(bytes, size));
        let el = Element::new_image(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Image::new(file, st.clone()),
            bx,
        );
        BoardOp::Put(el).apply(&mut b.doc);
    }
    set.boards()[0].clone()
}

#[test]
#[ignore = "régua: corre-se à mão em --release (ver o cabeçalho)"]
fn measure_svg_export_images() {
    install();
    let b = board();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let page = board_page(&b, &routes).unwrap();
    let mut ts = TextSystem::without_system_fonts();
    let variants = [("antes (a gravação do PDF)", ExportTarget::Pdf), ("depois", ExportTarget::Svg)];
    let mut t = [Vec::new(), Vec::new()];
    let mut len = [0usize; 2];
    for round in 0..ROUNDS {
        for k in 0..2 {
            let k = if round % 2 == 0 { k } else { 1 - k };
            let at = Instant::now();
            let (mut s, size) = paint_page(
                &b,
                &page,
                1.0,
                Some(EXPORT_PAPER),
                variants[k].1,
                &std::collections::BTreeSet::new(),
                &mut ts,
            );
            let out = svg::write(&s.take_record(), size);
            t[k].push(at.elapsed().as_secs_f64() * 1e3);
            len[k] = out.len();
        }
    }
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\nloadavg: {}", load.trim());
    for (k, (name, _)) in variants.iter().enumerate() {
        t[k].sort_by(f64::total_cmp);
        println!(
            "{name:>26}: {:9.1} ms mínimo [{:9.1} mediana] · SVG {} B",
            t[k][0],
            t[k][ROUNDS / 2],
            len[k]
        );
    }
    assert_eq!(len[0], len[1], "o mesmo SVG pelos dois caminhos");
}
