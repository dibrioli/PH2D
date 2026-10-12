//! W15 — o preto e branco (outra pirâmide, os píxeis cinzentos e a origem marcada), o SVG rasterizado
//! ao tamanho do nível (nítido ao zoom) e a moldura (o traço do estilo, por cima, composta com a imagem).

use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_model::{
    Board, BoardOp, BoardSet, Camera, Element, Image, ImageFile, ImageFilter, Rgba, Style,
};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, VectorScene};

use super::*;
use crate::RenderCache;

/// Um raster de teste: `w`, `h` nos primeiros 8 bytes; cada píxel `[200, 40, 10, 255]`.
fn raster(w: u32, h: u32) -> ImageFile {
    let mut b = w.to_le_bytes().to_vec();
    b.extend(h.to_le_bytes());
    ImageFile::new(b, [w, h])
}

/// Um «SVG» de teste: começa por `<svg` (o MIME diz vector); o descodificador pinta-o ao `at`.
fn svg(w: u32, h: u32) -> ImageFile {
    ImageFile::new(format!("<svg w={w} h={h}/>").into_bytes(), [w, h])
}

fn decode() -> Decode {
    Arc::new(|b: &[u8], at: Option<[u32; 2]>| {
        let size = if b.starts_with(b"<svg") {
            at?
        } else {
            [
                u32::from_le_bytes(b.get(..4)?.try_into().ok()?),
                u32::from_le_bytes(b.get(4..8)?.try_into().ok()?),
            ]
        };
        let rgba = [200u8, 40, 10, 255].repeat(size[0] as usize * size[1] as usize);
        Some(Pixels { size, rgba })
    })
}

fn board(f: ImageFile, bx: [f64; 4], zoom: f64, edit: impl Fn(&mut Image)) -> Board {
    let mut set = BoardSet::default();
    let id = set.create("i".into());
    let b = set.get_mut(id).unwrap();
    b.camera = Camera {
        center_x: bx[0] + bx[2] / 2.0,
        center_y: bx[1] + bx[3] / 2.0,
        zoom,
    };
    let file = b.put_file(f);
    let mut img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
    edit(&mut img);
    let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, bx);
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    set.boards()[0].clone()
}

fn ops(b: &Board, cache: &mut RenderCache) -> Vec<DrawOp> {
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    crate::paint(
        b,
        [0.0, 0.0, 640.0, 480.0],
        &mut scene,
        Theme::Forge,
        &mut ts,
        cache,
        &routes,
        None,
    );
    scene.take_record()
}

fn image_ops(o: &[DrawOp]) -> Vec<&ph2d_vector::ImageOp> {
    o.iter()
        .filter_map(|o| match o {
            DrawOp::Image(i) => Some(&**i),
            _ => None,
        })
        .collect()
}

fn settled(b: &Board, cache: &mut RenderCache, want: impl Fn(&[DrawOp]) -> bool) -> Vec<DrawOp> {
    let t0 = Instant::now();
    loop {
        let o = ops(b, cache);
        if !cache.images.pending() && want(&o) {
            return o;
        }
        assert!(t0.elapsed() < Duration::from_secs(20), "não assentou");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// ⭐ O preto e branco: os píxeis desenhados são o cinzento canónico dos do ficheiro, a origem vai
/// marcada para o SVG, e a mesma fotografia a cores no mesmo quadro continua a cores (outra pirâmide).
#[test]
fn a_gray_image_draws_gray_pixels_and_marks_its_source() {
    let mut cache = RenderCache::default();
    cache.images.set_decoder(decode());
    let mut b = board(raster(8, 8), [0.0, 0.0, 80.0, 80.0], 1.0, |i| {
        i.filter = ImageFilter::Gray;
    });
    // A mesma fotografia a cores, ao lado.
    let file = b.put_file(raster(8, 8));
    let el = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255]))),
        [100.0, 0.0, 80.0, 80.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    let o = settled(&b, &mut cache, |o| image_ops(o).len() == 2);
    let imgs = image_ops(&o);
    let g = crate::gray::gray_of([200, 40, 10]);
    assert_eq!(&imgs[0].image.rgba()[..4], [g, g, g, 255]);
    assert_eq!(&imgs[1].image.rgba()[..4], [200, 40, 10, 255]);
    assert_eq!(cache.images.entries.len(), 2);
}

/// O filtro passa à origem que o SVG embebe (um ficheiro de MIME conhecido).
#[test]
fn the_source_of_a_gray_svg_file_is_marked_gray() {
    let mut cache = RenderCache::default();
    cache.images.set_decoder(decode());
    let b = board(svg(10, 10), [0.0, 0.0, 100.0, 100.0], 1.0, |i| {
        i.filter = ImageFilter::Gray;
    });
    let o = settled(&b, &mut cache, |o| !image_ops(o).is_empty());
    let s = image_ops(&o)[0].source.as_ref().expect("um SVG tem MIME");
    assert_eq!(s.mime, "image/svg+xml");
    assert!(s.gray);
}

/// ⭐ Um SVG é nítido a qualquer zoom: o nível desenhado é o do ECRÃ (≥ os píxeis da caixa), nunca a
/// miniatura ampliada — a 100 px de caixa um nível pequeno, a 4000 px um de ~4096 (rasterizado a
/// esse tamanho, não por metades de um natural de 10 px).
#[test]
fn an_svg_draws_a_level_rasterized_at_the_screen_size() {
    for (zoom, lo, hi) in [(1.0, 100, 1024), (40.0, 4000, 8192)] {
        let mut cache = RenderCache::default();
        cache.images.set_decoder(decode());
        let b = board(svg(10, 10), [0.0, 0.0, 100.0, 100.0], zoom, |_| {});
        let o = settled(&b, &mut cache, |o| {
            image_ops(o).first().is_some_and(|i| i.image.width() >= lo)
        });
        let w = image_ops(&o)[0].image.width();
        assert!((lo..=hi).contains(&w), "zoom {zoom}: nível de {w} px");
    }
}

/// A miniatura de um SVG também se pede (é o que fica quando o nível fino se esquece).
#[test]
fn an_svg_keeps_a_thumbnail() {
    let mut cache = RenderCache::default();
    cache.images.set_decoder(decode());
    let b = board(svg(10, 10), [0.0, 0.0, 100.0, 100.0], 4.0, |_| {});
    let _ = settled(&b, &mut cache, |o| !image_ops(o).is_empty());
    let e = cache.images.entries.values().next().unwrap();
    assert!(e.levels.last().unwrap().image.is_some());
    assert!(e.levels.last().unwrap().size[0] <= THUMB_SIDE);
}

/// ⭐ A moldura: o contorno da caixa no traço do estilo, DEPOIS da imagem; sem `stroke`, nenhum traço.
#[test]
fn the_frame_strokes_the_box_after_the_image() {
    let red = Rgba([224, 49, 49, 255]);
    for framed in [false, true] {
        let mut cache = RenderCache::default();
        cache.images.set_decoder(decode());
        let b = board(raster(8, 8), [0.0, 0.0, 80.0, 60.0], 1.0, |i| {
            if framed {
                i.style.stroke = Some(red);
                i.style.stroke_width = 4.0;
            }
        });
        let o = settled(&b, &mut cache, |o| !image_ops(o).is_empty());
        let at_img = o
            .iter()
            .position(|o| matches!(o, DrawOp::Image(_)))
            .unwrap();
        let strokes: Vec<(usize, f64)> = o
            .iter()
            .enumerate()
            .filter_map(|(k, o)| match o {
                DrawOp::Stroke { style, .. } => Some((k, style.width)),
                _ => None,
            })
            .collect();
        if framed {
            assert_eq!(strokes.len(), 1, "{strokes:?}");
            assert!(strokes[0].0 > at_img && strokes[0].1 == 4.0);
        } else {
            assert!(strokes.is_empty(), "{strokes:?}");
        }
    }
}

/// Moldura numa imagem a 50 %: UMA camada de opacidade à volta da imagem e da moldura (a lei das
/// formas), e a da imagem fica a 1.
#[test]
fn a_faded_framed_image_composes_with_its_frame() {
    let mut cache = RenderCache::default();
    cache.images.set_decoder(decode());
    let b = board(raster(8, 8), [0.0, 0.0, 80.0, 60.0], 1.0, |i| {
        i.style.stroke = Some(Rgba([0, 0, 0, 255]));
        i.style.stroke_width = 2.0;
        i.style.opacity = 50;
        i.style.round = true;
    });
    let o = settled(&b, &mut cache, |o| !image_ops(o).is_empty());
    let alphas: Vec<f32> = o
        .iter()
        .filter_map(|o| match o {
            DrawOp::Layer { alpha, .. } => Some(*alpha),
            _ => None,
        })
        .collect();
    // O recorte da página, a camada do grupo (0,5) e a do recorte redondo (1).
    assert_eq!(&alphas[alphas.len() - 2..], [0.5, 1.0], "{alphas:?}");
}

/// Auditoria W15 — no export, uma imagem cujos píxeis não chegam (bytes ilegíveis) não sai, mas a
/// MOLDURA dela sai (no ecrã viam-se as duas).
#[test]
fn an_unreadable_framed_image_exports_its_frame() {
    let mut cache = RenderCache::default();
    cache.images.set_decoder(decode());
    cache.export = Some(crate::ExportLook {
        paper: None,
        vector: true,
    });
    let b = board(
        ImageFile::new(vec![1, 2], [8, 8]),
        [0.0, 0.0, 80.0, 60.0],
        1.0,
        |i| {
            i.style.stroke = Some(Rgba([0, 0, 0, 255]));
            i.style.stroke_width = 3.0;
        },
    );
    let o = ops(&b, &mut cache);
    assert!(image_ops(&o).is_empty());
    assert!(
        o.iter()
            .any(|o| matches!(o, DrawOp::Stroke { style, .. } if style.width == 3.0))
    );
    assert!(
        !o.iter().any(|o| matches!(o, DrawOp::Fill { .. })),
        "o cinzento do «a chegar»"
    );
}
