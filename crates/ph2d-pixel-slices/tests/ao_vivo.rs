//! **`Transform` ao vivo** (bug do smoke, cena 24): a meio do arrasto a arte segue a caixa. O que
//! se vê a meio do gesto (o documento composto com o traço da pré-visualização) é, píxel a píxel, o
//! documento depois de largar — mover e mudar o tamanho.

use ph2d_pixel_core::slice::create;
use ph2d_pixel_core::{NewSprite, Rect, SliceId, Sprite, Stroke, apply, compose_rgba};
use ph2d_pixel_slices::Gesto;
use ph2d_pixel_slices::transforma::{caixas_que_mudam, com_pixels, previa};

const TOL: f64 = 0.5;

/// Um sprite 32×24 com arte não uniforme por todo o lado e a slice `hero` em (4,4) 6×5.
fn cena() -> (Sprite, SliceId) {
    let mut s = Sprite::new(NewSprite {
        width: 32,
        height: 24,
        ..NewSprite::default()
    })
    .expect("sprite");
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let v: Vec<_> = (0..24)
        .flat_map(|y| (0..32).map(move |x| (x, y)))
        .filter(|(x, y)| (x + y) % 3 != 0)
        .map(|(x, y)| {
            let c = [(x * 7) as u8, (y * 11) as u8, ((x ^ y) * 5) as u8, 255];
            ((x, y), u32::from_le_bytes(c))
        })
        .collect();
    let op = Stroke::with_values(layer, frame, v).op(&s).expect("pinta");
    apply(&mut s, op).expect("aplica");
    let (op, id) = create(&mut s, 0, Rect::new(4, 4, 6, 5)).expect("cria");
    apply(&mut s, op).expect("aplica");
    (s, id)
}

fn imagem(s: &Sprite, st: Option<&Stroke>) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, s.frames()[0].id, st, &mut out);
    out
}

/// Prime em `de`, arrasta a `para`; devolve a imagem a meio e a imagem depois de largar.
fn gesto(de: (f64, f64), para: (f64, f64)) -> (Vec<u8>, Vec<u8>, Vec<u8>, Rect) {
    let (mut s, id) = cena();
    let antes = imagem(&s, None);
    let key = (s.layers()[0].id, s.frames()[0].id);
    let mut sel = vec![id];
    let mut g = Gesto::press(&s, 0, &mut sel, de, TOL, false);
    g.drag(para);
    let st = previa(&s, key, &g, 0).expect("a caixa muda");
    let meio = imagem(&s, Some(&st));
    let caixas = caixas_que_mudam(&s, 0, &g.previa());
    let op = g.release(&mut s, 0, &mut sel).expect("muda");
    let op = com_pixels(&s, op, key, &caixas, 0);
    apply(&mut s, op).expect("aplica");
    let bounds = s.slice(id).unwrap().shape_at(0).unwrap().bounds;
    (antes, meio, imagem(&s, None), bounds)
}

#[test]
fn mover_a_slice_mostra_a_arte_no_sitio_onde_o_largar_a_poe() {
    let (antes, meio, depois, b) = gesto((6.5, 6.5), (11.5, 9.5));
    assert_eq!(b, Rect::new(9, 7, 6, 5), "o gesto é um mover");
    assert_ne!(meio, antes, "a meio do arrasto a arte já saiu do sítio");
    assert!(meio == depois, "a pré-visualização é o passo");
}

#[test]
fn mudar_o_tamanho_mostra_a_arte_escalada_como_o_largar_a_grava() {
    let (antes, meio, depois, b) = gesto((10.0, 9.0), (14.0, 12.0));
    assert_eq!(
        b,
        Rect::new(4, 4, 10, 8),
        "o gesto é um mudar de tamanho pelo canto"
    );
    assert_ne!(meio, antes, "a meio do arrasto a arte já escalou");
    assert!(meio == depois, "a pré-visualização é o passo");
}
