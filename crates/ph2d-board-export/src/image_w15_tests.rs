//! W15 — o preto e branco na saída: o SVG embebe os bytes ORIGINAIS com o filtro `gray` (definido uma
//! vez) e um rasterizador de SVG (o `resvg`) desenha-o com o MESMO cinzento do ecrã; o PDF leva os
//! píxeis cinzentos (outro XObject que a mesma fotografia a cores) e um JPEG cinzento não entra como
//! chegou.

use std::collections::BTreeSet;

use super::*;
use ph2d_board_model::{BoardSet, Element, Image, ImageFile, ImageFilter, Rgba, Style};
use ph2d_vector::DrawOp;

/// Uma PNG `w × h` opaca de uma cor só.
fn png(w: u32, h: u32, c: [u8; 3]) -> Vec<u8> {
    let rgba: Vec<u8> = [c[0], c[1], c[2], 255].repeat((w * h) as usize);
    let mut out = Vec::new();
    let mut e = ::png::Encoder::new(&mut out, w, h);
    e.set_color(::png::ColorType::Rgba);
    e.set_depth(::png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&rgba).unwrap();
    out
}

const COLOR: [u8; 3] = [200, 40, 10];

/// A mesma fotografia a cinzento em `(0, 0)` e a cores em `(60, 0)`, `40 × 40` cada.
fn board(bytes: Vec<u8>, size: [u32; 2]) -> Board {
    super::image_tests::install();
    let mut set = BoardSet::default();
    let id = set.create("Cinza".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(bytes, size));
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for (x, f) in [(0.0, ImageFilter::Gray), (60.0, ImageFilter::Original)] {
        let mut img = Image::new(file, st.clone());
        img.filter = f;
        let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, [x, 0.0, 40.0, 40.0]);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    set.boards()[0].clone()
}

fn ops(b: &Board) -> (Vec<DrawOp>, [f64; 2]) {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let (mut s, size) = paint_page(
        b,
        &board_page(b, &routes).unwrap(),
        1.0,
        None,
        true,
        &BTreeSet::new(),
        &mut ts,
    );
    (s.take_record(), size)
}

/// ⭐ O SVG: o ficheiro uma vez (os bytes originais, a cores), o `<use>` cinzento com o filtro e o
/// colorido sem ele; e o `resvg` desenha o cinzento de `ph2d_board_render::gray_of` (±2).
///
/// A tolerância é MEDIDA (`fumo_w15_cinza.sh`, `saidas/w15/fumo_cinza.txt`, 11/10), sobre `200,40,10`
/// cujo cinzento do ecrã é 104: `rsvg-convert` 104 · Inkscape 103 · ImageMagick 103 · `resvg` 102 · o
/// PDF (`pdftoppm`, `gs`) 104 — os filtros guardam a luz linear em 8 bits e arredondam os escuros. A
/// mesma conta em sRGB (o filtro sem `linearRGB`) daria 72: ±2 ainda distingue a lei.
#[test]
fn the_svg_grays_with_a_filter_that_a_rasterizer_draws_like_the_screen() {
    let b = board(png(4, 4, COLOR), [4, 4]);
    let (o, size) = ops(&b);
    let svg = svg::write(&o, size);
    assert_eq!(svg.matches("<image id=").count(), 1, "{svg}");
    assert_eq!(svg.matches("<filter id=\"gray\"").count(), 1);
    assert_eq!(svg.matches("filter=\"url(#gray)\"").count(), 1);
    let tree = ph2d_imageio_svg::raster::parse(svg.as_bytes()).unwrap();
    let [w, h] = ph2d_imageio_svg::raster::natural_size(&tree);
    let px = ph2d_imageio_svg::raster::rasterize(&tree, [w, h]).unwrap();
    let at = |x: f64, y: f64| {
        let (x, y) = ((x + PAD) as usize, (y + PAD) as usize);
        px[(y * w as usize + x) * 4..][..4].to_vec()
    };
    let g = ph2d_board_render::gray_of(COLOR);
    let gray = at(20.0, 20.0);
    assert!(
        gray[..3].iter().all(|&c| c.abs_diff(g) <= 2) && gray[3] == 255,
        "{gray:?} ≠ {g}"
    );
    assert_eq!(at(80.0, 20.0), [COLOR[0], COLOR[1], COLOR[2], 255]);
}

/// O PDF: a mesma fotografia a cores e a cinzento são DOIS XObjects, e os píxeis gravados do cinzento
/// já são cinzentos (os que o PDF escreve).
#[test]
fn the_pdf_writes_the_gray_pixels_as_their_own_image() {
    let b = board(png(4, 4, COLOR), [4, 4]);
    let (o, size) = ops(&b);
    let imgs: Vec<_> = o
        .iter()
        .filter_map(|o| match o {
            DrawOp::Image(i) => Some(i),
            _ => None,
        })
        .collect();
    let g = ph2d_board_render::gray_of(COLOR);
    assert_eq!(&imgs[0].image.rgba()[..4], [g, g, g, 255]);
    assert!(imgs[0].source.as_ref().unwrap().gray);
    let bytes = pdf::write(&[pdf::PdfPage {
        ops: &o,
        size,
        links: Vec::new(),
    }]);
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(text.matches("/Subtype /Image").count(), 2, "a cores e a cinzento");
}

/// Um JPEG cinzento não entra como chegou (os bytes dele são a cores): vão os píxeis.
#[test]
fn a_gray_jpeg_does_not_go_into_the_pdf_as_it_arrived() {
    let mut j = Vec::new();
    let img = image::RgbImage::from_pixel(8, 8, image::Rgb(COLOR));
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut j, 90)
        .encode_image(&img)
        .unwrap();
    let b = board(j, [8, 8]);
    let (o, size) = ops(&b);
    let text = String::from_utf8_lossy(&pdf::write(&[pdf::PdfPage {
        ops: &o,
        size,
        links: Vec::new(),
    }]))
    .into_owned();
    assert_eq!(text.matches("/DCTDecode").count(), 1, "só o colorido");
}

/// Grava um SVG e um PDF com o cinzento e a moldura em `$PH2D_W15_OUT`, para os rasterizadores do
/// sistema (`docs/MiroClone/ferramentas/excalidraw_oracle/fumo_w15_cinza.sh`).
#[test]
#[ignore = "grava ficheiros para os rasterizadores do sistema"]
fn grava_svg_e_pdf_cinzentos() {
    let dir = std::env::var("PH2D_W15_OUT").expect("PH2D_W15_OUT");
    let mut b = board(png(4, 4, COLOR), [4, 4]);
    let ids: Vec<_> = b.doc.live().map(|e| e.id).collect();
    let mut a = b.doc.get(ids[1]).unwrap().clone();
    let st = &mut a.image_mut().unwrap().style;
    st.stroke = Some(Rgba([30, 30, 30, 255]));
    st.stroke_width = 4.0;
    BoardOp::Put(a).apply(&mut b.doc);
    let (o, size) = ops(&b);
    std::fs::write(format!("{dir}/w15.svg"), svg::write(&o, size)).unwrap();
    let pdf = pdf::write(&[pdf::PdfPage {
        ops: &o,
        size,
        links: Vec::new(),
    }]);
    std::fs::write(format!("{dir}/w15.pdf"), pdf).unwrap();
    println!("PAD={PAD} cinza={}", ph2d_board_render::gray_of(COLOR));
}

/// ⭐ Auditoria W15 — o MESMO SVG desenhado pequeno e grande vai ao PDF rasterizado a cada tamanho
/// (dois XObjects de larguras diferentes), não o pequeno esticado ao grande.
#[test]
fn an_svg_drawn_twice_goes_to_the_pdf_at_each_size() {
    super::image_tests::install();
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><circle cx="5" cy="5" r="5" fill="#00ff00"/></svg>"##;
    let mut set = BoardSet::default();
    let id = set.create("SVG".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(svg.to_vec(), [10, 10]));
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for bx in [[0.0, 0.0, 60.0, 60.0], [100.0, 0.0, 600.0, 600.0]] {
        let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), Image::new(file, st.clone()), bx);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let board = set.boards()[0].clone();
    let (o, size) = ops(&board);
    let text = String::from_utf8_lossy(&pdf::write(&[pdf::PdfPage {
        ops: &o,
        size,
        links: Vec::new(),
    }]))
    .into_owned();
    let widths: std::collections::BTreeSet<&str> = text
        .match_indices("/Width ")
        .map(|(i, _)| text[i + 7..].split_whitespace().next().unwrap())
        .collect();
    assert!(widths.len() >= 2, "um tamanho só: {widths:?}");
}
