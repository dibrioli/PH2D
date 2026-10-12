//! W14 — a imagem pelo que o desenho EMITE (a gravação da cena, a mesma do SVG/PDF): o cinzento
//! enquanto não há píxeis, o nível com a resolução do ecrã, o recorte e o espelho, a camada da
//! opacidade e dos cantos, e o export que descodifica já e não pinta o cinzento.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_model::{Board, BoardOp, BoardSet, Crop, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, Point, VectorScene};

use super::*;
use crate::{ExportLook, RenderCache};

/// Os «bytes» de teste: um PNG de verdade não é preciso — o descodificador de teste lê `w` e `h` dos
/// primeiros 8 bytes e pinta cada píxel com a sua coluna (para ler o espelho).
fn file(w: u32, h: u32) -> ImageFile {
    let mut b = w.to_le_bytes().to_vec();
    b.extend(h.to_le_bytes());
    ImageFile::new(b, [w, h])
}

fn test_decode() -> Decode {
    Arc::new(|b: &[u8], _at: Option<[u32; 2]>| {
        let w = u32::from_le_bytes(b.get(..4)?.try_into().ok()?);
        let h = u32::from_le_bytes(b.get(4..8)?.try_into().ok()?);
        let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
        for _ in 0..h {
            for x in 0..w {
                rgba.extend([(x % 256) as u8, 0, 0, 255]);
            }
        }
        Some(Pixels { size: [w, h], rgba })
    })
}

fn board_with(f: ImageFile, bx: [f64; 4], edit: impl Fn(&mut Image)) -> Board {
    let mut set = BoardSet::default();
    let id = set.create("i".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(f);
    let mut img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
    edit(&mut img);
    let el = Element::new_image(b.doc.mint_id(), b.doc.z_on_top(), img, bx);
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    set.boards()[0].clone()
}

fn paint_ops(board: &Board, cache: &mut RenderCache) -> Vec<DrawOp> {
    let mut routes = RouteCache::default();
    routes.sync(&board.doc);
    let mut scene = VectorScene::new();
    scene.record();
    let mut ts = TextSystem::without_system_fonts();
    crate::paint(
        board,
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

fn images(ops: &[DrawOp]) -> Vec<&ph2d_vector::ImageOp> {
    ops.iter()
        .filter_map(|o| match o {
            DrawOp::Image(i) => Some(&**i),
            _ => None,
        })
        .collect()
}

/// Pinta até a descodificação acabar (os fios dela são de verdade).
fn settled(board: &Board, cache: &mut RenderCache) -> Vec<DrawOp> {
    let t0 = Instant::now();
    loop {
        let ops = paint_ops(board, cache);
        if !cache.images.pending() && !images(&ops).is_empty() {
            return ops;
        }
        assert!(
            t0.elapsed() < Duration::from_secs(20),
            "a descodificação não acabou"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// A pirâmide: o 1.º cabe no atlas do Vello, cada um é metade do anterior, o último cabe na miniatura.
#[test]
fn the_pyramid_fits_the_atlas_and_halves_down_to_the_thumbnail() {
    let s = level_sizes([20_000, 100], false);
    assert_eq!(s[0], [5000, 25]);
    assert!(s.windows(2).all(|p| p[1] == p[0].map(|v| v.div_ceil(2))));
    assert!(s.last().unwrap()[0] <= THUMB_SIDE && s[s.len() - 2][0] > THUMB_SIDE);
    assert_eq!(level_sizes([3, 2], false), [[3, 2]]);
}

/// Metade pesada pelo alfa: um píxel vermelho entre três transparentes dá vermelho a ¼ — nunca um
/// vermelho escurecido pelo preto dos transparentes.
#[test]
fn halving_weighs_by_alpha() {
    let src = [255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(halve(&src, [2, 2], [1, 1]), [255, 0, 0, 64]);
}

/// O afim leva o recorte à caixa e o espelho vira à volta do centro (a lei do oráculo, `img3` e
/// `img2` de `w14_imagens`).
#[test]
fn the_crop_stretches_to_the_box_and_the_flip_turns_about_the_centre() {
    let b = board_with(file(4, 2), [0.0, 0.0, 80.0, 80.0], |i| {
        i.crop = Some(Crop {
            x: 2.0,
            y: 0.0,
            w: 2.0,
            h: 2.0,
        });
    });
    let el = b.doc.live().next().unwrap().clone();
    let img = el.image().unwrap().clone();
    let m = level_to_box(&el, &img, [4, 2], [4, 2]);
    assert_eq!(m * Point::new(2.0, 0.0), Point::new(0.0, 0.0));
    assert_eq!(m * Point::new(4.0, 2.0), Point::new(80.0, 80.0));
    let mut flipped = img.clone();
    flipped.flip = [true, false];
    let m = level_to_box(&el, &flipped, [4, 2], [4, 2]);
    assert_eq!(m * Point::new(2.0, 0.0), Point::new(80.0, 0.0));
    // Um nível de metade cobre a mesma caixa.
    let m = level_to_box(&el, &img, [4, 2], [2, 1]);
    assert_eq!(m * Point::new(2.0, 1.0), Point::new(80.0, 80.0));
}

/// ⭐ No ecrã: sem descodificador fica o cinzento (e nenhuma imagem); com ele, o 1.º quadro é o
/// cinzento e, acabada a descodificação fora do fio, a imagem — com os bytes de origem para o SVG.
#[test]
fn the_screen_shows_the_placeholder_until_the_pixels_arrive() {
    let b = board_with(file(64, 32), [10.0, 10.0, 128.0, 64.0], |_| {});
    let mut cache = RenderCache::default();
    let ops = paint_ops(&b, &mut cache);
    assert!(images(&ops).is_empty());
    assert!(
        !cache.images.pending(),
        "sem descodificador nada fica pendente"
    );
    let mut cache = RenderCache::default();
    cache.images.set_decoder(test_decode());
    let first = paint_ops(&b, &mut cache);
    assert!(images(&first).is_empty(), "o 1.º quadro não espera");
    let ops = settled(&b, &mut cache);
    let img = images(&ops)[0];
    assert_eq!([img.image.width(), img.image.height()], [64, 32]);
    assert!(
        img.source.is_none(),
        "estes bytes não são de um MIME de imagem"
    );
}

/// ⭐ O nível tem a resolução do ECRÃ: uma imagem de 4096² mostrada a 100 px desenha um nível de
/// 128 px (o mais pequeno com ≥ 100), não os 4096.
#[test]
fn far_away_a_big_image_draws_a_small_level() {
    let b = board_with(file(4096, 4096), [0.0, 0.0, 100.0, 100.0], |_| {});
    let mut cache = RenderCache::default();
    cache.images.set_decoder(test_decode());
    let ops = settled(&b, &mut cache);
    let img = images(&ops)[0];
    assert_eq!([img.image.width(), img.image.height()], [128, 128]);
    // O afim leva o nível à caixa inteira no ecrã (zoom 1, centro na origem do quadro).
    let corner = img.transform * Point::new(128.0, 128.0);
    let origin = img.transform * Point::new(0.0, 0.0);
    assert!(((corner - origin).x - 100.0).abs() < 1e-9);
}

/// A opacidade e os cantos são UMA camada (recorte da caixa redonda, alfa da opacidade) à volta da
/// imagem; sem eles, nenhuma.
#[test]
fn opacity_and_round_corners_are_one_layer() {
    let layers = |round: bool, opacity: u8| {
        let b = board_with(file(8, 8), [0.0, 0.0, 80.0, 80.0], |i| {
            i.style.round = round;
            i.style.opacity = opacity;
        });
        let mut cache = RenderCache::default();
        cache.images.set_decoder(test_decode());
        let ops = settled(&b, &mut cache);
        let at = ops
            .iter()
            .position(|o| matches!(o, DrawOp::Image(_)))
            .unwrap();
        match &ops[at - 1] {
            DrawOp::Layer { alpha, .. } => Some(*alpha),
            _ => None,
        }
    };
    assert_eq!(layers(false, 100), None);
    assert_eq!(layers(false, 50), Some(0.5));
    assert_eq!(layers(true, 100), Some(1.0));
}

/// ⭐ W16 — o export SVG não descodifica NADA: grava os bytes com o afim do píxel natural ao ecrã (o
/// espelho incluído). O PDF descodifica (precisa dos píxeis).
#[test]
fn the_svg_export_decodes_nothing_and_places_the_natural_pixels() {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let decode: Decode = Arc::new(move |_: &[u8], _: Option<[u32; 2]>| {
        seen.fetch_add(1, Relaxed);
        Some(Pixels {
            size: [4, 2],
            rgba: vec![255; 4 * 2 * 4],
        })
    });
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend([0; 8]);
    let b = board_with(ImageFile::new(png, [4, 2]), [10.0, 0.0, 40.0, 20.0], |i| {
        i.flip = [true, false]
    });
    for (target, decoded) in [(crate::ExportTarget::Svg, 0), (crate::ExportTarget::Pdf, 1)] {
        calls.store(0, Relaxed);
        let mut cache = RenderCache::default();
        cache.images.set_decoder(decode.clone());
        cache.export = Some(ExportLook {
            paper: None,
            target,
        });
        let ops = paint_ops(&b, &mut cache);
        assert_eq!(calls.load(Relaxed), decoded, "{target:?}");
        let embeds: Vec<_> = ops
            .iter()
            .filter_map(|o| match o {
                DrawOp::Embed(e) => Some(e),
                _ => None,
            })
            .collect();
        if decoded > 0 {
            assert!(embeds.is_empty());
            assert_eq!(images(&ops).len(), 1);
            continue;
        }
        assert!(images(&ops).is_empty());
        assert_eq!(embeds.len(), 1);
        assert_eq!(embeds[0].source.size, [4, 2]);
        // Espelhada: o píxel natural (0, 0) cai no canto de cima à DIREITA da caixa.
        let v = crate::view(&b.camera, [0.0, 0.0, 640.0, 480.0]);
        let want = v * Point::new(50.0, 0.0);
        assert!((embeds[0].transform * Point::ORIGIN - want).hypot() < 1e-9);
    }
}

/// O export descodifica JÁ (não há próximo quadro) e desenha o nível inteiro no vector; sem ficheiro,
/// não pinta nada (nem o cinzento — o SVG do oráculo omite-a).
#[test]
fn the_export_decodes_at_once_and_skips_a_missing_file() {
    let b = board_with(file(300, 200), [0.0, 0.0, 30.0, 20.0], |_| {});
    let mut cache = RenderCache::default();
    cache.images.set_decoder(test_decode());
    cache.export = Some(ExportLook {
        paper: None,
        target: crate::ExportTarget::Pdf,
    });
    let ops = paint_ops(&b, &mut cache);
    let img = images(&ops)[0];
    assert_eq!([img.image.width(), img.image.height()], [300, 200]);
    let mut gone = b.clone();
    gone.files.clear();
    let before = paint_ops(
        &gone,
        &mut RenderCache {
            export: cache.export,
            ..RenderCache::default()
        },
    );
    assert!(images(&before).is_empty());
    assert!(
        !before.iter().any(|o| matches!(o, DrawOp::Fill { .. })),
        "nem o cinzento: {before:?}"
    );
}

/// ⭐ Auditoria W14 — uma imagem vista PEQUENA não guarda os níveis finos que a descodificação fez de
/// passagem (só os do pedido para baixo e a miniatura).
#[test]
fn a_small_view_keeps_only_the_levels_it_asked_for() {
    let b = board_with(file(4096, 4096), [0.0, 0.0, 100.0, 100.0], |_| {});
    let mut cache = RenderCache::default();
    cache.images.set_decoder(test_decode());
    let _ = settled(&b, &mut cache);
    let e = cache.images.entries.values().next().unwrap();
    let kept: Vec<u32> = e
        .levels
        .iter()
        .filter(|l| l.image.is_some())
        .map(|l| l.size[0])
        .collect();
    assert_eq!(kept, [128], "{kept:?}");
}

/// ⭐ Auditoria W14 — um descodificador que entra em PÂNICO não deixa a imagem pendente para sempre
/// (fica no cinzento, sem ficar «a descodificar»), e o fio continua a servir a seguinte.
#[test]
fn a_panicking_decoder_fails_the_image_and_the_pool_survives() {
    let b = board_with(file(8, 8), [0.0, 0.0, 80.0, 80.0], |_| {});
    let good = test_decode();
    let mut cache = RenderCache::default();
    cache.images.set_decoder(Arc::new(move |bytes: &[u8], at| {
        assert!(bytes[0] != 8, "pânico de teste");
        good(bytes, at)
    }));
    let t0 = Instant::now();
    loop {
        let ops = paint_ops(&b, &mut cache);
        if !cache.images.pending() && t0.elapsed() > Duration::from_millis(20) {
            assert!(images(&ops).is_empty());
            break;
        }
        assert!(t0.elapsed() < Duration::from_secs(20), "ficou pendente");
        std::thread::sleep(Duration::from_millis(5));
    }
    let other = board_with(file(16, 16), [0.0, 0.0, 80.0, 80.0], |_| {});
    let ops = settled(&other, &mut cache);
    assert_eq!(images(&ops).len(), 1, "o fio morreu com o pânico");
}

/// Sem o nível pedido, desenha o mais FINO mais perto (nítido), não a miniatura.
#[test]
fn without_the_wanted_level_the_nearest_finer_one_draws() {
    let f = file(512, 512);
    let mut imgs = Images::default();
    imgs.set_decoder(test_decode());
    // A pirâmide inteira por um pedido síncrono do nível 0, e depois esquece-se o nível 1.
    let _ = imgs
        .level(f.id(), ImageFilter::Original, &f, [512.0, 512.0], true)
        .unwrap();
    let e = imgs
        .entries
        .get_mut(&(f.id(), ImageFilter::Original))
        .unwrap();
    assert!(e.levels.len() >= 3);
    e.levels[1].image = None;
    let (_, size) = imgs
        .level(f.id(), ImageFilter::Original, &f, [200.0, 200.0], false)
        .unwrap();
    assert_eq!(size, [512, 512], "foi à miniatura em vez do fino");
}

/// ⭐ W16 — no recorte a imagem desenha-se também INTEIRA, esbatida (α = 27/255, a do Excalidraw),
/// por baixo da janela; fora do recorte, não; e o export nunca.
#[test]
fn a_cropping_picture_shows_its_whole_self_faded_under_the_window() {
    let b = board_with(file(40, 20), [0.0, 0.0, 20.0, 20.0], |i| {
        i.crop = Some(Crop {
            x: 10.0,
            y: 0.0,
            w: 20.0,
            h: 20.0,
        });
    });
    let id = b.doc.live().next().unwrap().id;
    let mut cache = RenderCache::default();
    cache.images.set_decoder(test_decode());
    let until_drawn = |cache: &mut RenderCache| {
        for _ in 0..500 {
            let ops = paint_ops(&b, cache);
            if !images(&ops).is_empty() {
                return ops;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("a imagem não chegou");
    };
    assert_eq!(images(&until_drawn(&mut cache)).len(), 1);
    cache.cropping = Some(id);
    let ops = until_drawn(&mut cache);
    assert_eq!(images(&ops).len(), 2, "a inteira esbatida e a janela");
    let faded = ops
        .iter()
        .any(|o| matches!(o, DrawOp::Layer { alpha, .. } if (*alpha - 27.0 / 255.0).abs() < 1e-6));
    assert!(faded, "{ops:?}");
}
