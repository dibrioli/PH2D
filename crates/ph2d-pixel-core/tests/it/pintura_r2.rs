//! P13 R2: o Shading anda na rampa do DOCUMENTO, o pincel de dither fica preso ao canvas e o
//! Smudge ganha o Push (`docs/PixelArt/P13_plano.md` §4, §7).

use ph2d_pixel_core::dither::{Dither, PincelDither};
use ph2d_pixel_core::filter::smudge::{Kind, Smudge};
use ph2d_pixel_core::ink::{Ink, shade};
use ph2d_pixel_core::{
    Background, ColorMode, DocOp, FrameId, LayerId, NewSprite, Op, Paint, Sprite, Stroke, apply,
};

fn sprite(mode: ColorMode, w: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: w,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn ids(s: &Sprite) -> (LayerId, FrameId) {
    (s.layers()[0].id, s.frames()[0].id)
}

fn rampa(s: &mut Sprite, indices: Vec<u8>) {
    apply(
        s,
        Op::Doc(Box::new(DocOp::AddRamp {
            name: "r".into(),
            indices,
        })),
    )
    .expect("rampa");
}

/// Põe `v` nas células dadas (tinta Simple).
fn pinta(s: &mut Sprite, v: u32, cells: &[(i32, i32)]) {
    let (l, f) = ids(s);
    let mut st = Stroke::new(l, f, v, Paint::default());
    st.set_shape(s, &[], cells);
    if let Some(op) = st.op(s) {
        apply(s, op).expect("pinta");
    }
}

/// O valor que um carimbo de Shading põe sobre `dest`, com a selecção `sel` e o sentido `dir`.
fn sombra(s: &Sprite, sel: &[u32], dir: i8, at: (i32, i32)) -> u32 {
    let (l, f) = ids(s);
    let paint = Paint {
        ink: Ink::Shading,
        ..Paint::default()
    };
    let st = Stroke::begin(s, l, f, 0, paint, at).with_shading(sel.to_vec(), dir);
    st.value_at(s, at.0, at.1).expect("tocou")
}

#[test]
fn o_shading_anda_na_rampa_do_documento() {
    let mut s = sprite(ColorMode::Indexed, 8);
    // Sem rampa no documento: a lei de antes (a selecção), byte a byte.
    let sel = [5u32, 4, 3, 20, 21];
    for v in [3u32, 4, 5, 20, 21, 30] {
        pinta(&mut s, v, &[(0, 0)]);
        for dir in [1, -1] {
            assert_eq!(
                sombra(&s, &sel, dir, (0, 0)),
                shade(&sel, v, dir),
                "{v} {dir}"
            );
        }
    }
    rampa(&mut s, vec![1, 2, 3, 4, 5]);
    rampa(&mut s, vec![7]);
    let caso = |s: &mut Sprite, v: u32, dir: i8| {
        pinta(s, v, &[(0, 0)]);
        sombra(s, &sel, dir, (0, 0))
    };
    // Dentro da rampa do documento: um degrau nela, não na selecção (que daria o contrário).
    assert_eq!(caso(&mut s, 3, 1), 4);
    assert_eq!(caso(&mut s, 3, -1), 2);
    assert_eq!(caso(&mut s, 4, 1), 5);
    // As pontas param.
    assert_eq!(caso(&mut s, 5, 1), 5);
    assert_eq!(caso(&mut s, 1, -1), 1);
    // Fora de toda rampa do documento: a selecção de hoje.
    assert_eq!(caso(&mut s, 20, 1), 21);
    assert_eq!(caso(&mut s, 30, 1), 30);
    // A rampa de um degrau não tem para onde andar (e manda mais que a selecção).
    let sel7 = [7u32, 8];
    pinta(&mut s, 7, &[(0, 0)]);
    assert_eq!(sombra(&s, &sel7, 1, (0, 0)), 7);

    // RGB: a rampa são as cores da paleta.
    let mut r = sprite(ColorMode::Rgba, 8);
    rampa(&mut r, vec![1, 2, 3]);
    let cor = |r: &Sprite, i: usize| r.value_of_rgba(r.palette()[i]);
    let (c2, c3) = (cor(&r, 2), cor(&r, 3));
    pinta(&mut r, c2, &[(0, 0)]);
    assert_eq!(sombra(&r, &[], 1, (0, 0)), c3);
}

fn dither_paint(s: &Sprite, prim: u32, sec: u32, d: PincelDither, path: &[(i32, i32)]) -> Stroke {
    let (l, f) = ids(s);
    let mut st = Stroke::new(l, f, prim, Paint::default()).with_dither(sec, d);
    st.start(s, path[0].0, path[0].1);
    for &(x, y) in &path[1..] {
        st.to(s, x, y);
    }
    st
}

#[test]
fn o_dither_fica_preso_ao_canvas() {
    let s = sprite(ColorMode::Indexed, 16);
    let (p, q) = (3u32, 9u32);
    let all: Vec<(i32, i32)> = (0..16).flat_map(|y| (0..16).map(move |x| (x, y))).collect();
    for m in [Dither::Bayer2, Dither::Bayer4, Dither::Bayer8] {
        let n2 = m.side() * m.side();
        for nivel in [0u8, 16, 32, 40, 64] {
            let d = PincelDither { nivel, matriz: m };
            let a = dither_paint(&s, p, q, d, &[(0, 0), (15, 0), (15, 15), (0, 15), (0, 0)]);
            let mut b = Stroke::new(ids(&s).0, ids(&s).1, p, Paint::default()).with_dither(q, d);
            b.set_shape(&s, &[], &all);
            for &(x, y) in &all {
                let want = if m.level(x, y) * (64 / n2) < u32::from(nivel) {
                    p
                } else {
                    q
                };
                for st in [&a, &b] {
                    if let Some(v) = st.value_at(&s, x, y) {
                        assert_eq!(v, want, "{m:?} {nivel} ({x},{y})");
                    }
                }
            }
            match nivel {
                0 => assert!(all.iter().all(|&(x, y)| b.value_at(&s, x, y) == Some(q))),
                64 => assert!(all.iter().all(|&(x, y)| b.value_at(&s, x, y) == Some(p))),
                _ => {}
            }
        }
    }
    // Dois traços sobrepostos que começam em sítios diferentes dão o MESMO padrão.
    let d = PincelDither {
        nivel: 24,
        matriz: Dither::Bayer4,
    };
    let a = dither_paint(&s, p, q, d, &[(1, 5), (14, 5)]);
    let b = dither_paint(&s, p, q, d, &[(14, 5), (2, 5)]);
    for x in 2..=14 {
        assert_eq!(a.value_at(&s, x, 5), b.value_at(&s, x, 5), "x {x}");
    }
    // Um cel deslocado (tinta já em (9,9)): o padrão no canvas não muda.
    let mut t = s.clone();
    pinta(&mut t, 12, &[(9, 9), (11, 12)]);
    let c = dither_paint(&t, p, q, d, &[(1, 5), (14, 5)]);
    for x in 1..=14 {
        assert_eq!(
            c.value_at(&t, x, 5),
            a.value_at(&s, x, 5),
            "cel deslocado x {x}"
        );
    }
}

#[test]
fn push_arrasta_a_cor_e_fica_na_paleta() {
    let mut s = sprite(ColorMode::Indexed, 12);
    let key = ids(&s);
    let (vermelho, azul) = (
        s.value_of_rgba([0xac, 0x32, 0x32, 0xff]),
        s.value_of_rgba([0x5b, 0x6e, 0xe1, 0xff]),
    );
    let esq: Vec<_> = (0..12).flat_map(|y| (0..6).map(move |x| (x, y))).collect();
    let dir: Vec<_> = (0..12).flat_map(|y| (6..12).map(move |x| (x, y))).collect();
    pinta(&mut s, vermelho, &esq);
    pinta(&mut s, azul, &dir);
    let pal: Vec<[u8; 4]> = s.palette().to_vec();
    for forca in [128u8, 255] {
        let mut g = Smudge::new(&s, key, Kind::Push { forca }, 0);
        for x in 3..=9 {
            g.arrasta(&s, (x, 6), &[(x, 6)]);
        }
        // A cor anda no sentido do traço: o azul logo depois da fronteira ficou avermelhado.
        let c = g.color_at(6, 6).unwrap();
        assert_ne!(
            c,
            s.rgba_of_value(azul),
            "força {forca}: (6,6) não recebeu cor"
        );
        assert!(c[0] > 0x5b, "força {forca}: (6,6) = {c:?}");
        // Atrás do primeiro carimbo nada muda; fora da linha também não.
        assert_eq!(g.color_at(2, 6).unwrap(), s.rgba_of_value(vermelho));
        assert_eq!(g.color_at(7, 5).unwrap(), s.rgba_of_value(azul));
        // Em Indexed toda cor de trabalho é uma cor da paleta.
        for y in 0..12 {
            for x in 0..12 {
                let c = g.color_at(x, y).unwrap();
                assert!(
                    c[3] == 0 || pal.contains(&c),
                    "({x},{y}) = {c:?} fora da paleta"
                );
            }
        }
        assert!(!g.values(&s).is_empty());
    }
    // Força 0 não arrasta nada.
    let mut g = Smudge::new(&s, key, Kind::Push { forca: 0 }, 0);
    for x in 3..=9 {
        g.arrasta(&s, (x, 6), &[(x, 6)]);
    }
    assert_eq!(g.color_at(6, 6).unwrap(), s.rgba_of_value(azul));
}
