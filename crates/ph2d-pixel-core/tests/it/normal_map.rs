//! **P14 — a camada Normal Map**: a cor composta exclui-a, `compose_normal` compõe só ela sobre a
//! normal de frente, `SetLayerNormal` é UM passo com inverso, e o blob leva a bandeira.

use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::layers;
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, History, LayerId, NORMAL_FLAT, NewSprite, Op, Patch, Rect,
    Sprite, apply, compose_normal, compose_rgba,
};
use std::collections::BTreeMap;

fn frame(s: &Sprite) -> FrameId {
    s.frames()[0].id
}

fn fill(s: &mut Sprite, l: LayerId, r: Rect, rgba: [u8; 4]) {
    let v = s.value_of_rgba(rgba).to_le_bytes();
    let bpp = s.mode().bpp();
    let pixels = (0..r.area()).flat_map(|_| v[..bpp].to_vec()).collect();
    let bbox = s.cel((l, frame(s))).map_or(r, |c| c.bbox.union(r));
    apply(
        s,
        Op::Patch(Patch {
            layer: l,
            frame: frame(s),
            bbox: Some(bbox),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
}

fn cor(s: &Sprite) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, frame(s), None, &mut out);
    out
}

/// 6 × 4 transparente: «arte» vermelha em (0,0,4,3) e, por cima, «n» marcada Normal Map.
fn cena() -> (Sprite, LayerId, LayerId) {
    let mut s = Sprite::new(NewSprite {
        width: 6,
        height: 4,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let arte = s.layers()[0].id;
    fill(&mut s, arte, Rect::new(0, 0, 4, 3), [200, 30, 40, 255]);
    let (op, n) = layers::new_layer(&mut s, arte).expect("n");
    apply(&mut s, op).expect("n");
    let op = layers::set_normal(&s, n, true).expect("normal");
    apply(&mut s, op).expect("normal");
    (s, arte, n)
}

#[test]
fn pintar_na_camada_normal_nao_muda_a_cor_composta() {
    let (mut s, _, n) = cena();
    let antes = cor(&s);
    fill(&mut s, n, Rect::new(1, 0, 5, 4), [255, 128, 128, 255]);
    assert_eq!(cor(&s), antes, "a camada Normal Map entrou na cor");
}

#[test]
fn compose_normal_devolve_so_as_camadas_normal() {
    let (mut s, _, n) = cena();
    fill(&mut s, n, Rect::new(1, 0, 2, 1), [255, 128, 128, 255]);
    let px = compose_normal(&s, frame(&s));
    let w = s.width() as usize;
    let at = |x: usize, y: usize| -> [u8; 4] {
        let i = (y * w + x) * 4;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    // Pintada: a normal da camada; o alfa é o da cor (a arte cobre (1,0)).
    assert_eq!(at(1, 0), [255, 128, 128, 255]);
    // Arte sem normal pintada: a de frente, com o alfa da cor — nunca o vermelho da arte.
    assert_eq!(
        at(0, 0),
        [NORMAL_FLAT[0], NORMAL_FLAT[1], NORMAL_FLAT[2], 255]
    );
    // Fora da arte: a de frente, alfa 0 (a banda não cobre o vizinho).
    assert_eq!(
        at(5, 3),
        [NORMAL_FLAT[0], NORMAL_FLAT[1], NORMAL_FLAT[2], 0]
    );
    assert_eq!(at(2, 0), [255, 128, 128, 255]);
    // Normal pintada fora da arte: RGB dela, alfa 0.
    fill(&mut s, n, Rect::new(5, 3, 1, 1), [0, 128, 128, 255]);
    let px = compose_normal(&s, frame(&s));
    let i = (3 * w + 5) * 4;
    assert_eq!(&px[i..i + 4], &[0, 128, 128, 0]);
}

#[test]
fn set_layer_normal_desfaz_num_passo() {
    let (mut s, arte, _) = cena();
    let mut h = History::default();
    let antes = s.clone();
    let op = layers::set_normal(&s, arte, true).expect("op");
    h.commit(&mut s, op).expect("commit");
    assert!(s.layer(arte).is_some_and(|l| l.normal));
    assert_eq!(h.undo_len(), 1, "um passo");
    assert!(h.undo(&mut s));
    assert_eq!(s, antes, "o Ctrl+Z repõe o documento inteiro");
    assert!(h.redo(&mut s));
    assert!(s.layer(arte).is_some_and(|l| l.normal));
    assert!(
        layers::set_normal(&s, arte, true).is_none(),
        "sem mudança, sem passo"
    );
}

#[test]
fn o_blob_leva_a_bandeira_normal() {
    let (s, arte, n) = cena();
    let b = write_library(&BTreeMap::from([(1, s.clone())]));
    let back = read_library(&b).expect("lê");
    let l = &back[&1];
    assert!(l.layer(n).is_some_and(|l| l.normal));
    assert!(l.layer(arte).is_some_and(|l| !l.normal));
    assert_eq!(l, &s);
}
