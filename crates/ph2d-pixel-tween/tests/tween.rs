//! Os gates do bloco T «Quadros do meio» (P13 §7). Gabarito próprio: a lei é aritmética (posição
//! inteira arredondada da curva) e a rotação compara-se com o RotSprite do núcleo chamado aqui.

use ph2d_pixel_core::affine::{Affine, Linear, Resample, transform_image};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::transform::Image;
use ph2d_pixel_core::{
    Cel, CelKey, FrameId, LayerId, NewSprite, Op, Patch, Rect, Sprite, apply, layers,
};
use ph2d_pixel_tween::{Facilidade, tween};

const A: Rect = Rect::new(2, 3, 5, 4);
const B: Rect = Rect::new(12, 13, 5, 4);

fn pixels(r: Rect, semente: u8) -> Vec<u8> {
    (0..r.w * r.h)
        .flat_map(|i| {
            let v = (i as u8).wrapping_mul(37).wrapping_add(semente);
            [v, v ^ 0x5a, semente, if i % 3 == 0 { 0 } else { 255 }]
        })
        .collect()
}

fn poe(s: &mut Sprite, layer: LayerId, frame: FrameId, r: Rect, semente: u8) {
    let p = Patch {
        layer,
        frame,
        bbox: Some(r),
        rect: r,
        pixels: pixels(r, semente),
    };
    apply(s, Op::Patch(p)).expect("patch");
}

/// Duas camadas (a de cima é a do tween), quadros A e B; um cel em cada.
fn cena() -> (Sprite, LayerId, LayerId, FrameId, FrameId) {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    let fundo = s.layers()[0].id;
    let (op, l) = layers::new_layer(&mut s, fundo).expect("camada");
    apply(&mut s, op).expect("camada");
    let fa = s.frames()[0].id;
    let (op, fb) = frames::new_frame(&mut s, fa, NewFrame::Empty).expect("quadro");
    apply(&mut s, op).expect("quadro");
    poe(&mut s, l, fa, A, 7);
    poe(&mut s, l, fb, B, 99);
    poe(&mut s, fundo, fa, Rect::new(0, 0, 3, 3), 1);
    (s, l, fundo, fa, fb)
}

fn gerados(s: &Sprite, fa: FrameId, n: usize) -> Vec<FrameId> {
    let i = s.frame_index(fa).unwrap();
    s.frames()[i + 1..=i + n].iter().map(|f| f.id).collect()
}

fn instantaneo(s: &Sprite) -> (Vec<ph2d_pixel_core::Frame>, Vec<(CelKey, Cel)>) {
    let cels = s.entries().map(|(k, c)| (*k, c.clone())).collect();
    (s.frames().to_vec(), cels)
}

#[test]
fn o_tween_chega_ao_cel_b() {
    for (ease, n) in [(Facilidade::Linear, 3u32), (Facilidade::In, 4)] {
        let (mut s, l, _, fa, fb) = cena();
        let b_antes = s.cel((l, fb)).cloned();
        let op = tween(&mut s, l, fa, fb, n, &|t| ease.em(t), 0.0).expect("tween");
        assert!(matches!(op, Op::Batch(_)), "um Batch");
        apply(&mut s, op).expect("aplica");
        assert_eq!(s.frames().len(), 2 + n as usize);
        let ids = gerados(&s, fa, n as usize);
        for (k, &f) in ids.iter().enumerate() {
            let e = f64::from(ease.em((k + 1) as f32 / n as f32));
            let esperado = |a: i32, b: i32| a + (f64::from(b - a) * e).round() as i32;
            let c = s.cel((l, f)).expect("cel gerado");
            assert_eq!(
                (c.bbox.x, c.bbox.y, c.bbox.w, c.bbox.h),
                (esperado(A.x, B.x), esperado(A.y, B.y), A.w, A.h),
                "{ease:?} quadro {k}"
            );
            assert_eq!(c.data, pixels(A, 7), "a imagem é a de A");
        }
        let ultimo = s.cel((l, *ids.last().unwrap())).unwrap();
        assert_eq!(
            (ultimo.bbox.x, ultimo.bbox.y),
            (B.x, B.y),
            "o último chega a B"
        );
        assert_eq!(s.cel((l, fb)).cloned(), b_antes, "B intacto");
        assert_eq!(s.frame_index(fb), Some(1 + n as usize), "B logo depois");
    }
    // Linear, n = 3, Δ = 10: 3,33 → 3 e 6,67 → 7 (arredondar, não truncar).
    let (mut s, l, _, fa, fb) = cena();
    let op = tween(&mut s, l, fa, fb, 3, &|t| t, 0.0).unwrap();
    apply(&mut s, op).unwrap();
    let xs: Vec<i32> = gerados(&s, fa, 3)
        .iter()
        .map(|&f| s.cel((l, f)).unwrap().bbox.x)
        .collect();
    assert_eq!(xs, [5, 9, 12]);
    // A curva entra: o In cúbico no meio fica atrás do linear.
    let (mut s, l, _, fa, fb) = cena();
    let op = tween(&mut s, l, fa, fb, 4, &|t| Facilidade::In.em(t), 0.0).unwrap();
    apply(&mut s, op).unwrap();
    let meio = s.cel((l, gerados(&s, fa, 4)[1])).unwrap().bbox.x;
    assert_eq!(meio, A.x + 1, "In cúbico em t = 0,5 = 0,125 · 10 → 1");
}

#[test]
fn o_tween_e_um_passo() {
    let (mut s, l, _, fa, fb) = cena();
    let antes = instantaneo(&s);
    let op = tween(&mut s, l, fa, fb, 5, &|t| Facilidade::InOut.em(t), 45.0).unwrap();
    let inverso = apply(&mut s, op).unwrap();
    assert_eq!(s.frames().len(), 7);
    apply(&mut s, inverso).unwrap();
    assert_eq!(instantaneo(&s), antes, "um desfazer tira todos os quadros");
}

#[test]
fn os_quadros_do_meio_retocam_se() {
    let (mut s, l, fundo, fa, fb) = cena();
    let op = tween(&mut s, l, fa, fb, 2, &|t| t, 0.0).unwrap();
    apply(&mut s, op).unwrap();
    let f = gerados(&s, fa, 2)[0];
    assert_eq!(s.entry((l, f)).unwrap().link, None, "o cel do tween é dono");
    assert_eq!(
        s.entry((fundo, f)).unwrap().link,
        Some(fa),
        "o resto liga a A"
    );
    let (ca, cb) = (s.cel((l, fa)).cloned(), s.cel((l, fb)).cloned());
    let r = s.cel((l, f)).unwrap().bbox;
    poe(&mut s, l, f, r, 200);
    assert_eq!(s.cel((l, fa)).cloned(), ca, "A não muda");
    assert_eq!(s.cel((l, fb)).cloned(), cb, "B não muda");
    assert_eq!(s.cel((l, f)).unwrap().data, pixels(r, 200));
}

#[test]
fn com_rotacao_e_o_rotsprite_do_nucleo() {
    let (mut s, l, _, fa, fb) = cena();
    let op = tween(&mut s, l, fa, fb, 2, &|t| t, 30.0).unwrap();
    apply(&mut s, op).unwrap();
    let ids = gerados(&s, fa, 2);
    let src = Image {
        w: A.w,
        h: A.h,
        bpp: 4,
        data: pixels(A, 7),
    };
    let t = s.transparent_value();
    for (k, graus) in [(0usize, 15.0), (1, 30.0)] {
        let alvo = if k == 1 { B } else { Rect::new(7, 8, A.w, A.h) };
        let m = Affine::around_center(Linear::from_screen_ccw(graus, 1.0, 1.0, 0.0), alvo);
        let (r, img) = transform_image(&src, &m, Resample::RotSprite, t, None).unwrap();
        let c = s.cel((l, ids[k])).unwrap();
        assert_eq!((c.bbox, &c.data), (r, &img.data), "quadro {k} = RotSprite");
        let (_, fast) = transform_image(&src, &m, Resample::Fast, t, None).unwrap();
        assert_ne!(c.data, fast.data, "não é a Fast Rotation");
    }
}

#[test]
fn recusa_o_que_nao_se_aplica() {
    let (mut s, l, _, fa, fb) = cena();
    assert!(
        tween(&mut s, l, fb, fa, 2, &|t| t, 0.0).is_none(),
        "B antes de A"
    );
    assert!(
        tween(&mut s, l, fa, fb, 0, &|t| t, 0.0).is_none(),
        "zero quadros"
    );
}
