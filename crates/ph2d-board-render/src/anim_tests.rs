//! W16 — um GIF toca no ecrã (um quadro de cada vez, pelo relógio) e o export leva o primeiro.

use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_model::{Board, BoardOp, BoardSet, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, VectorScene};

use super::*;
use crate::{ExportLook, ExportTarget, RenderCache};

const PYRAMID: [u8; 4] = [255, 0, 255, 255];
const PLAYED: [[u8; 4]; 3] = [[255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255]];

fn flat(c: [u8; 4]) -> Pixels {
    Pixels {
        size: [4, 2],
        rgba: c.repeat(8),
    }
}

/// Um «GIF» 4×2 (só os bytes mágicos: o descodificador e o animador são os do teste).
fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("g".into());
    let b = set.get_mut(id).unwrap();
    let mut bytes = b"GIF89a".to_vec();
    bytes.extend([0; 8]);
    let file = b.put_file(ImageFile::new(bytes, [4, 2]));
    let img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        img,
        [0.0, 0.0, 40.0, 20.0],
    );
    BoardOp::Put(el).apply(&mut b.doc);
    set.boards()[0].clone()
}

fn cache() -> RenderCache {
    let mut c = RenderCache::default();
    c.images
        .set_decoder(Arc::new(|_: &[u8], _| Some(flat(PYRAMID))));
    // Três quadros de 100 ms, e o terceiro com 5 ms no ficheiro (a lei: 100).
    c.images.set_animator(Arc::new(|_: &[u8]| {
        let it = PLAYED
            .into_iter()
            .zip([100u32, 100, 5])
            .map(|(c, ms)| (flat(c), ms));
        Some(Box::new(it) as Box<dyn Iterator<Item = (Pixels, u32)>>)
    }));
    c
}

/// A cor da imagem desenhada.
fn drawn(b: &Board, cache: &mut RenderCache) -> [u8; 4] {
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
    scene
        .take_record()
        .iter()
        .find_map(|o| match o {
            DrawOp::Image(i) => Some(i.image.rgba()[..4].try_into().unwrap()),
            _ => None,
        })
        .unwrap_or([0; 4])
}

/// Desenha em `at` até a cor ser `want` (o fio do tocador chega quando chega), no máximo 5 s.
fn until(b: &Board, cache: &mut RenderCache, at: Instant, want: [u8; 4]) -> bool {
    cache.images.set_clock(Some(at));
    let end = Instant::now() + Duration::from_secs(5);
    while Instant::now() < end {
        if drawn(b, cache) == want {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    false
}

/// ⭐⭐ No ecrã o GIF TOCA pelo relógio — cada quadro o tempo dele, e volta ao primeiro; o primeiro
/// desenho (antes de o fio dar um quadro) é o da pirâmide; o export leva o 1.º quadro da pirâmide e
/// não arranca tocador nenhum.
#[test]
fn a_gif_plays_on_screen_by_the_clock_and_exports_its_first_frame() {
    let b = board();
    let t0 = Instant::now();
    let mut c = cache();
    assert!(
        until(&b, &mut c, t0, PLAYED[0]),
        "o 1.º quadro do tocador não chegou"
    );
    assert!(until(&b, &mut c, t0 + Duration::from_millis(99), PLAYED[0]));
    assert!(until(
        &b,
        &mut c,
        t0 + Duration::from_millis(100),
        PLAYED[1]
    ));
    assert!(until(
        &b,
        &mut c,
        t0 + Duration::from_millis(200),
        PLAYED[2]
    ));
    assert!(
        until(&b, &mut c, t0 + Duration::from_millis(299), PLAYED[2]),
        "um quadro de 5 ms no ficheiro dura 100"
    );
    assert!(
        until(&b, &mut c, t0 + Duration::from_millis(300), PLAYED[0]),
        "no fim volta ao primeiro"
    );
    let mut export = cache();
    export.export = Some(ExportLook {
        paper: None,
        target: ExportTarget::Pdf,
    });
    assert_eq!(drawn(&b, &mut export), PYRAMID);
    assert!(!export.images.players.any(), "o export não toca");
}

/// ⭐ W18 — a APRESENTAÇÃO é um ecrã com o desenho do export: o GIF toca lá também (o dono pediu GIF
/// animado, W16); o PNG exportado continua com o 1.º quadro.
#[test]
fn a_gif_plays_in_the_presentation_and_not_in_a_png_export() {
    let b = board();
    let t0 = Instant::now();
    let mut show = cache();
    show.export = Some(ExportLook {
        paper: None,
        target: ExportTarget::Screen,
    });
    assert!(until(&b, &mut show, t0, PLAYED[0]), "a apresentação não toca");
    assert!(until(
        &b,
        &mut show,
        t0 + Duration::from_millis(100),
        PLAYED[1]
    ));
    let mut png = cache();
    png.export = Some(ExportLook {
        paper: None,
        target: ExportTarget::Png,
    });
    assert_eq!(drawn(&b, &mut png), PYRAMID);
    assert!(!png.images.players.any(), "o PNG não toca");
    assert!(!ExportTarget::Screen.vector(), "o detalhe da apresentação é o do PNG");
}

/// Um GIF de um quadro só não fica a tocar: o tocador sai e a pirâmide desenha-o.
#[test]
fn a_still_gif_stops_its_player() {
    let b = board();
    let mut c = cache();
    c.images.set_animator(Arc::new(|_: &[u8]| {
        Some(Box::new(std::iter::once((flat(PLAYED[1]), 100u32)))
            as Box<dyn Iterator<Item = (Pixels, u32)>>)
    }));
    let t0 = Instant::now();
    assert!(until(&b, &mut c, t0, PLAYED[1]), "o único quadro chega");
    // Passado o tempo dele, o fio já acabou: o tocador sai e a pirâmide desenha.
    assert!(until(&b, &mut c, t0 + Duration::from_millis(200), PYRAMID));
    assert!(!c.images.players.any(), "um quadro só não é animação");
}

#[test]
fn a_frame_of_ten_ms_or_less_lasts_a_tenth_of_a_second() {
    assert_eq!([0, 10, 11, 70].map(frame_ms), [100, 100, 11, 70]);
}

/// Um desenho que chega um pouco ATRASADO (menos de 1 s) não empurra o ritmo: o quadro seguinte
/// acaba no tempo dele contado do fim do anterior, não de agora.
#[test]
fn a_late_draw_does_not_drift_the_rhythm() {
    let b = board();
    let t0 = Instant::now();
    let mut c = cache();
    assert!(until(&b, &mut c, t0, PLAYED[0]));
    assert!(until(
        &b,
        &mut c,
        t0 + Duration::from_millis(150),
        PLAYED[1]
    ));
    // O 2.º acaba em t0 + 200 (não em 150 + 100 = 250).
    assert!(
        until(&b, &mut c, t0 + Duration::from_millis(210), PLAYED[2]),
        "o ritmo derivou"
    );
}
