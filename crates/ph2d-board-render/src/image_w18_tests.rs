//! W18 — uma imagem estragada de que se LÊEM as linhas de cima (um PNG/JPEG cortado): desenha-as, com a
//! marca do perigo num canto, e continua a ser dita como estragada (o aviso) — o export leva as linhas
//! lidas sem a marca (o Excalidraw exporta o que o navegador descodificou, `saidas/w17`).

use std::sync::Arc;
use std::time::{Duration, Instant};

use ph2d_board_model::{Board, BoardOp, BoardSet, Element, Image, ImageFile, Rgba, Style};
use ph2d_board_route::RouteCache;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Brush, DrawOp, VectorScene};

use super::*;
use crate::{ExportLook, ExportTarget, RenderCache};

fn board() -> Board {
    let mut set = BoardSet::default();
    let id = set.create("i".into());
    let b = set.get_mut(id).unwrap();
    let file = b.put_file(ImageFile::new(vec![1, 2, 3, 4], [8, 8]));
    let img = Image::new(file, Style::new(None, None, Rgba([0, 0, 0, 255])));
    let el = Element::new_image(
        b.doc.mint_id(),
        b.doc.z_on_top(),
        img,
        [0.0, 0.0, 80.0, 80.0],
    );
    let _ = BoardOp::Put(el).apply(&mut b.doc);
    set.boards()[0].clone()
}

/// O inteiro recusa; o salvador dá as 3 linhas de cima de 8 (vermelhas), o resto transparente.
fn cache() -> RenderCache {
    let mut c = RenderCache::default();
    c.images.set_decoder(Arc::new(|_: &[u8], _| None));
    c.images.set_salvager(Arc::new(|_: &[u8]| {
        let mut rgba = [255, 0, 0, 255].repeat(8 * 3);
        rgba.extend([0u8; 8 * 5 * 4]);
        Some(Pixels { size: [8, 8], rgba })
    }));
    c
}

fn ops(b: &Board, c: &mut RenderCache) -> Vec<DrawOp> {
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
        c,
        &routes,
        None,
    );
    scene.take_record()
}

fn has_image(ops: &[DrawOp]) -> bool {
    ops.iter().any(|o| matches!(o, DrawOp::Image(_)))
}

fn has_ink(ops: &[DrawOp], t: ColorToken) -> bool {
    let c = crate::token(t, Theme::Forge);
    ops.iter()
        .any(|o| matches!(o, DrawOp::Fill { brush: Brush::Solid(k), .. } if *k == c))
}

/// ⭐⭐ As linhas salvas desenham-se (não o cinzento riscado), com a marca do perigo, e o ficheiro é
/// dito estragado UMA vez; o export leva-as sem a marca.
#[test]
fn a_salvaged_picture_draws_what_it_read_marked_and_reported() {
    let b = board();
    let id = b.doc.live().find_map(|e| e.image()).unwrap().file;
    let mut c = cache();
    let t0 = Instant::now();
    let shown = loop {
        let o = ops(&b, &mut c);
        if !c.images.pending() && has_image(&o) {
            break o;
        }
        assert!(
            t0.elapsed() < Duration::from_secs(20),
            "as linhas salvas não chegaram"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(
        has_ink(&shown, ColorToken::Danger),
        "a estragada salva sem a marca"
    );
    assert_eq!(
        c.images.take_failed(),
        vec![id],
        "a salva continua a avisar"
    );
    let img = shown.iter().find_map(|o| match o {
        DrawOp::Image(i) => Some(i.image.rgba().to_vec()),
        _ => None,
    });
    let px = img.unwrap();
    assert_eq!(&px[..4], &[255, 0, 0, 255]);
    assert_eq!(px[(8 * 7) * 4 + 3], 0, "abaixo do que se leu, transparente");
    let mut export = cache();
    export.export = Some(ExportLook {
        paper: None,
        target: ExportTarget::Png,
    });
    let out = ops(&b, &mut export);
    assert!(has_image(&out), "o export leva as linhas lidas");
    assert!(
        !has_ink(&out, ColorToken::Danger),
        "a marca não sai no export"
    );
}
