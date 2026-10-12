//! W14 — as imagens na saída: no SVG cada ficheiro uma vez (os bytes como chegaram) e cada imagem um
//! `<use>` no sítio da caixa; no PDF os píxeis uma vez, com o alfa à parte.

use std::sync::Arc;

use super::*;
use ph2d_board_model::{BoardSet, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_render::Pixels;
use ph2d_vector::{DrawOp, Point};

/// Uma PNG 3×2 de verdade (o píxel do meio de cima transparente), escrita aqui pelo `png`.
fn png_3x2() -> Vec<u8> {
    let mut rgba = vec![200u8; 3 * 2 * 4];
    rgba[4 + 3] = 0;
    let mut out = Vec::new();
    let mut e = ::png::Encoder::new(&mut out, 3, 2);
    e.set_color(::png::ColorType::Rgba);
    e.set_depth(::png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&rgba).unwrap();
    out
}

/// As descodificações feitas, pela impressão digital dos bytes (os testes correm em paralelo: cada um
/// conta só os ficheiros dele).
static DECODED: std::sync::Mutex<Vec<u64>> = std::sync::Mutex::new(Vec::new());

fn print_of(b: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    b.hash(&mut h);
    h.finish()
}

/// Quantas vezes `b` foi descodificado.
fn decodes_of(b: &[u8]) -> usize {
    let p = print_of(b);
    DECODED.lock().unwrap().iter().filter(|&&q| q == p).count()
}

/// O descodificador do app, para estes testes: só PNG.
pub(crate) fn install() {
    ph2d_board_render::install_decoder(Arc::new(|b: &[u8], at: Option<[u32; 2]>| {
        DECODED.lock().unwrap().push(print_of(b));
        // W15: um SVG pelo rasterizador do app (o mesmo da porta de entrada).
        if b.starts_with(b"<svg") {
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

/// Um JPEG RGB `w × h` (ruído suave: comprime como uma fotografia).
fn jpeg(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbImage::from_fn(w, h, |x, y| {
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

/// ⭐ Auditoria W14 — a MESMA fotografia em duas páginas (dois frames) vai UMA vez no PDF, e um JPEG
/// entra como chegou (`DCTDecode`, os bytes dele): o PDF não cresce para os píxeis brutos.
#[test]
fn a_jpeg_goes_into_the_pdf_once_and_as_it_arrived() {
    install();
    let bytes = jpeg(400, 300);
    let mut set = BoardSet::default();
    let id = set.create("Fotos".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(bytes.clone(), [400, 300]));
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for k in 0..2 {
        let x = f64::from(k) * 1000.0;
        let f = Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            ph2d_board_model::Frame {
                name: format!("F{k}"),
                style: st.clone(),
            },
            [x, 0.0, 500.0, 400.0],
        );
        let fid = f.id;
        BoardOp::Put(f).apply(&mut b.doc);
        let mut el = Element::new_image(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Image::new(file, st.clone()),
            [x + 50.0, 50.0, 400.0, 300.0],
        );
        el.frame = Some(fid);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let b = set.boards()[0].clone();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let pages = pdf_pages(&b, &routes, &[]);
    assert_eq!(pages.len(), 2);
    let recs: Vec<_> = pages
        .iter()
        .map(|p| {
            let (mut s, size) = paint_page(
                &b,
                p,
                1.0,
                None,
                ExportTarget::Pdf,
                &mut ExportCache::default(),
                &mut ts,
            );
            (s.take_record(), size)
        })
        .collect();
    let pdf_pages: Vec<pdf::PdfPage<'_>> = recs
        .iter()
        .map(|(ops, size)| pdf::PdfPage {
            ops,
            size: *size,
            links: Vec::new(),
        })
        .collect();
    let out = pdf::write(&pdf_pages);
    let text = String::from_utf8_lossy(&out);
    assert_eq!(
        text.matches("/Subtype /Image").count(),
        1,
        "uma vez para as duas páginas"
    );
    assert!(text.contains("/DCTDecode"));
    assert!(
        out.windows(bytes.len()).any(|w| w == &bytes[..]),
        "os bytes do JPEG não estão no PDF"
    );
    assert!(
        out.len() < bytes.len() + 20_000,
        "{} B para um JPEG de {} B",
        out.len(),
        bytes.len()
    );
}

/// ⭐ W17 — a MESMA fotografia em três frames: o PDF descodifica-a UMA vez para as três páginas (o
/// export partilha as imagens; antes, uma por página).
#[test]
fn one_export_decodes_a_photo_once_for_all_its_pages() {
    install();
    let bytes = jpeg(404, 303);
    let mut set = BoardSet::default();
    let id = set.create("Fotos".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(bytes.clone(), [404, 303]));
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for k in 0..3 {
        let x = f64::from(k) * 1000.0;
        let f = Element::new_frame(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            ph2d_board_model::Frame {
                name: format!("F{k}"),
                style: st.clone(),
            },
            [x, 0.0, 500.0, 400.0],
        );
        let fid = f.id;
        BoardOp::Put(f).apply(&mut b.doc);
        let mut el = Element::new_image(
            b.doc.mint_id(),
            b.doc.z_on_top(),
            Image::new(file, st.clone()),
            [x + 50.0, 50.0, 404.0, 303.0],
        );
        el.frame = Some(fid);
        BoardOp::Put(el).apply(&mut b.doc);
    }
    let b = set.boards()[0].clone();
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let pages = pdf_pages(&b, &routes, &[]);
    assert_eq!(pages.len(), 3);
    let (mut drawn, mut shared) = (0, ExportCache::default());
    for p in &pages {
        let (mut s, _) = paint_page(&b, p, 1.0, None, ExportTarget::Pdf, &mut shared, &mut ts);
        drawn += s
            .take_record()
            .iter()
            .filter(|o| matches!(o, DrawOp::Image(_)))
            .count();
    }
    assert_eq!(drawn, 3, "cada página com a sua imagem");
    assert_eq!(
        decodes_of(&bytes),
        1,
        "a mesma foto descodificada por página"
    );
}

/// Duas imagens do MESMO ficheiro, uma em `(0, 0)` de `30 × 20` e outra em `(100, 0)` de `60 × 40`.
fn board() -> Board {
    install();
    let mut set = BoardSet::default();
    let id = set.create("Fotos".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(png_3x2(), [3, 2]));
    let st = Style::new(None, None, Rgba([0, 0, 0, 255]));
    for bx in [[0.0, 0.0, 30.0, 20.0], [100.0, 0.0, 60.0, 40.0]] {
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

fn ops(b: &Board, target: ExportTarget) -> (Vec<DrawOp>, [f64; 2]) {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut ts = TextSystem::without_system_fonts();
    let (mut s, size) = paint_page(
        b,
        &board_page(b, &routes).unwrap(),
        1.0,
        None,
        target,
        &mut ExportCache::default(),
        &mut ts,
    );
    (s.take_record(), size)
}

/// ⭐ O SVG define o ficheiro UMA vez com os bytes PNG como chegaram e põe cada imagem com um `<use>`
/// cujo afim leva o píxel natural ao canto da caixa na página (a página começa em `-PAD`) — e W16:
/// sem descodificar nenhuma (a gravação do SVG não tem píxeis; a do PDF tem).
#[test]
fn the_svg_defines_each_file_once_and_places_each_image() {
    let b = board();
    let (ops, size) = ops(&b, ExportTarget::Svg);
    let pixels = |ops: &[DrawOp]| ops.iter().filter(|o| matches!(o, DrawOp::Image(_))).count();
    assert_eq!(pixels(&ops), 0, "o SVG descodificou uma imagem que não usa");
    assert_eq!(pixels(&self::ops(&b, ExportTarget::Pdf).0), 2);
    let imgs: Vec<_> = ops
        .iter()
        .filter_map(|o| match o {
            DrawOp::Embed(i) => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(imgs.len(), 2);
    let src = &imgs[1].source;
    assert_eq!((src.mime, src.size), ("image/png", [3, 2]));
    // A 2.ª imagem: o píxel natural (3, 2) cai no canto de baixo à direita da caixa.
    let corner = imgs[1].transform * Point::new(3.0, 2.0);
    assert!(
        (corner - Point::new(160.0 + PAD, 40.0 + PAD)).hypot() < 1e-9,
        "{corner:?}"
    );
    let svg = svg::write(&ops, size);
    assert_eq!(svg.matches("<image id=").count(), 1, "{svg}");
    assert_eq!(svg.matches("<use xlink:href=\"#i0\"").count(), 2);
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_3x2());
    assert!(svg.contains(&format!("width=\"3\" height=\"2\" preserveAspectRatio=\"none\" xlink:href=\"data:image/png;base64,{b64}\"")));
}

/// O PDF escreve os píxeis UMA vez, com o alfa numa `SMask` (há um píxel transparente) e suavizados.
#[test]
fn the_pdf_writes_the_pixels_once_with_their_alpha() {
    let b = board();
    let (ops, size) = ops(&b, ExportTarget::Pdf);
    let bytes = pdf::write(&[pdf::PdfPage {
        ops: &ops,
        size,
        links: Vec::new(),
    }]);
    let text = String::from_utf8_lossy(&bytes);
    assert_eq!(
        text.matches("/Subtype /Image").count(),
        2,
        "o RGB e a máscara"
    );
    assert_eq!(text.matches("/SMask").count(), 1);
    assert_eq!(text.matches("/Width 3").count(), 2);
    assert!(text.contains("/Interpolate true"));
    assert_eq!(
        text.matches("/I0 Do").count(),
        0,
        "o conteúdo vai comprimido"
    );
}

/// O PNG (pela placa) também desenha a imagem: a gravação do modo imagem leva-a, ao nível do ecrã.
#[test]
fn the_png_scene_draws_the_image_too() {
    let b = board();
    let (ops, _) = ops(&b, ExportTarget::Png);
    assert_eq!(
        ops.iter().filter(|o| matches!(o, DrawOp::Image(_))).count(),
        2
    );
}

/// Grava um SVG e um PDF com imagens (recorte + espelho, redonda + meio opaca + rodada) em
/// `$PH2D_W14_OUT`, para os rasterizadores do sistema (`rsvg-convert`, `resvg`, `pdftoppm`).
#[test]
#[ignore = "grava ficheiros para olhar (à mão)"]
fn grava_svg_e_pdf_com_imagens() {
    let dir = std::env::var("PH2D_W14_OUT").expect("PH2D_W14_OUT");
    let mut b = board();
    let ids: Vec<_> = b.doc.live().map(|e| e.id).collect();
    let mut a = b.doc.get(ids[0]).unwrap().clone();
    let i = a.image_mut().unwrap();
    i.crop = Some(ph2d_board_model::Crop {
        x: 1.0,
        y: 0.0,
        w: 2.0,
        h: 2.0,
    });
    i.flip = [true, false];
    BoardOp::Put(a).apply(&mut b.doc);
    let mut c = b.doc.get(ids[1]).unwrap().clone();
    c.angle = 0.5;
    let i = c.image_mut().unwrap();
    i.style.round = true;
    i.style.opacity = 50;
    BoardOp::Put(c).apply(&mut b.doc);
    let (svg_ops, size) = ops(&b, ExportTarget::Svg);
    std::fs::write(format!("{dir}/w14.svg"), svg::write(&svg_ops, size)).unwrap();
    let (ops, size) = ops(&b, ExportTarget::Pdf);
    let pdf = pdf::write(&[pdf::PdfPage {
        ops: &ops,
        size,
        links: Vec::new(),
    }]);
    std::fs::write(format!("{dir}/w14.pdf"), pdf).unwrap();
}
