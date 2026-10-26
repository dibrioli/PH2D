//! **P6 contra o MVP web** — a fixtura `mvp_selection.json`
//! (`docs/PixelArt/ferramentas/extrai_selecao_mvp.mjs`): as famílias em que o MVP CONCORDA com o
//! Trial — retângulo/varinha exacta/inverter/caixa/extrair, crescer/encolher em 8 (= o pincel
//! quadrado), espelhos, giros de 90°, ampliação inteira e o Shift com volta.
//!
//! ⛔ Recusas MEDIDAS (não se portam): `morph.border` do MVP é crescer − encolher, 0/60 contra o
//! Trial (que é a faixa INTERIOR, máscara − contrair); `morph` em conectividade 4 (Manhattan) e
//! `distance` não são do Aseprite (o pincel de Modify é quadrado ou círculo); `rotate`/`rotated_size`
//! são o RotSprite do CANTO e o giro centrado do MVP, que o `xform` (variante do CENTRO) substitui.

use ph2d_pixel_core::selection::{Mask, ModifyBrush, SelectMode, Wand, wand};
use ph2d_pixel_core::transform::{
    Image, flip_h, flip_v, lift, offset_wrap, rot90ccw, rot90cw, rot180, scale_int,
};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, Sprite, apply};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_selection.json")).expect("json")
}

fn n(v: &Value) -> i64 {
    v.as_i64().expect("int")
}

fn arr(v: &Value) -> Vec<u32> {
    v.as_array()
        .expect("arr")
        .iter()
        .map(|x| x.as_u64().expect("u") as u32)
        .collect()
}

fn rows(m: &Mask) -> Vec<String> {
    (0..m.height() as i32)
        .map(|y| {
            (0..m.width() as i32)
                .map(|x| if m.get(x, y) { '1' } else { '0' })
                .collect()
        })
        .collect()
}

fn mask_of(v: &Value, w: u32, h: u32) -> Mask {
    let mut m = Mask::empty(w, h);
    for (y, r) in v.as_array().expect("mask").iter().enumerate() {
        for (x, ch) in r.as_str().expect("r").chars().enumerate() {
            m.set(x as i32, y as i32, ch == '1');
        }
    }
    m
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .expect("rows")
        .iter()
        .map(|r| r.as_str().expect("r").to_owned())
        .collect()
}

/// Um índice do MVP como cor opaca distinta (a varinha exacta em índices = em RGBA opaco).
fn colour(i: u32) -> [u8; 4] {
    [(i * 40 + 10) as u8, 7, 9, 255]
}

fn indexed_as_rgba(w: u32, h: u32, idx: &[u32]) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = Rect::new(0, 0, w, h);
    let pixels = idx.iter().flat_map(|&i| colour(i)).collect();
    apply(
        &mut s,
        Op::Patch(Patch {
            layer: l,
            frame: f,
            bbox: Some(r),
            rect: r,
            pixels,
        }),
    )
    .expect("patch");
    s
}

fn mode(v: &Value) -> SelectMode {
    match v.as_str().expect("mode") {
        "add" => SelectMode::Add,
        "subtract" => SelectMode::Subtract,
        _ => SelectMode::Replace,
    }
}

#[test]
fn mask_sequences_bounds_and_extract_match_the_mvp() {
    let f = fx();
    let seqs = f["mask_seq"].as_array().expect("seq");
    let mut steps = 0;
    for (k, c) in seqs.iter().enumerate() {
        let (w, h) = (n(&c["w"]) as u32, n(&c["h"]) as u32);
        let idx = arr(&c["indices"]);
        let s = indexed_as_rgba(w, h, &idx);
        let key = (s.layers()[0].id, s.frames()[0].id);
        let mut m = Mask::empty(w, h);
        for st in c["steps"].as_array().expect("steps") {
            m = match st["op"].as_str().expect("op") {
                "rect" => {
                    let r = &st["rect"];
                    let rect = Rect::new(
                        n(&r["x"]) as i32,
                        n(&r["y"]) as i32,
                        n(&r["w"]) as u32,
                        n(&r["h"]) as u32,
                    );
                    m.combine(&Mask::from_rect(w, h, rect), mode(&st["mode"]))
                }
                "wand" => {
                    let (x, y) = (n(&st["x"]) as i32, n(&st["y"]) as i32);
                    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                        m // o MVP não toca a máscara com a semente fora
                    } else {
                        let opt = Wand {
                            eight: n(&st["conn"]) == 8,
                            ..Wand::default()
                        };
                        m.combine(&wand(&s, key, (x, y), opt), mode(&st["mode"]))
                    }
                }
                _ => m.inverse(),
            };
            assert_eq!(rows(&m), strs(&st["mask"]), "seq {k}");
            assert_eq!(!m.is_empty(), st["any"].as_bool().expect("any"), "seq {k}");
            steps += 1;
        }
        let b = m.bounds();
        match &c["bounds"] {
            Value::Null => assert!(b.is_none(), "seq {k}"),
            v => assert_eq!(
                b,
                Some(Rect::new(
                    n(&v["x"]) as i32,
                    n(&v["y"]) as i32,
                    n(&v["w"]) as u32,
                    n(&v["h"]) as u32
                )),
                "seq {k}"
            ),
        }
        let ex = lift(&s, key, Some(&m));
        match &c["extract"] {
            Value::Null => assert!(ex.is_none(), "seq {k}"),
            v => {
                let ex = ex.expect("extract");
                assert_eq!((ex.x, ex.y), (n(&v["x"]) as i32, n(&v["y"]) as i32));
                let back: Vec<u32> = ex
                    .image
                    .values()
                    .iter()
                    .map(|&c| {
                        (0..8)
                            .find(|&i| u32::from_le_bytes(colour(i)) == c)
                            .unwrap_or(0)
                    })
                    .collect();
                assert_eq!(back, arr(&v["data"]), "seq {k}");
                let keep: Vec<u32> = ex.keep.iter().map(|&b| u32::from(b)).collect();
                assert_eq!(keep, arr(&v["keep"]), "seq {k}");
            }
        }
    }
    assert_eq!(seqs.len(), 60);
    assert!(steps > 150, "{steps} passos");
}

#[test]
fn grow_and_shrink_in_eight_are_the_square_brush_of_modify() {
    let f = fx();
    let cs = f["morph"].as_array().expect("morph");
    let mut checked = 0;
    for (k, c) in cs.iter().enumerate() {
        let (w, h) = (n(&c["w"]) as u32, n(&c["h"]) as u32);
        let m = mask_of(&c["mask"], w, h);
        for o in c["out"].as_array().expect("out") {
            if o.get("grow").is_none() || n(&o["conn"]) != 8 {
                continue; // ⛔ conectividade 4 e a distância: não são do Aseprite
            }
            let r = n(&o["n"]) as u32;
            assert_eq!(
                rows(&m.expand(r, ModifyBrush::Square)),
                strs(&o["grow"]),
                "{k} n{r}"
            );
            assert_eq!(
                rows(&m.contract(r, ModifyBrush::Square)),
                strs(&o["shrink"]),
                "{k} n{r}"
            );
            checked += 1;
        }
    }
    assert_eq!((cs.len(), checked), (40, 160));
}

fn img(w: i64, h: i64, data: &Value) -> Image {
    Image::from_values(w as u32, h as u32, 1, &arr(data))
}

fn sized(v: &Value) -> Image {
    img(n(&v["w"]), n(&v["h"]), &v["data"])
}

#[test]
fn flips_quarter_turns_and_integer_scale_are_the_mvp_permutations() {
    let f = fx();
    let cs = f["perm"].as_array().expect("perm");
    for (k, c) in cs.iter().enumerate() {
        let src = img(n(&c["w"]), n(&c["h"]), &c["data"]);
        assert_eq!(flip_h(&src).values(), arr(&c["flip_h"]), "{k}");
        assert_eq!(flip_v(&src).values(), arr(&c["flip_v"]), "{k}");
        assert_eq!(rot90cw(&src), sized(&c["rot90cw"]), "{k}");
        assert_eq!(rot90ccw(&src), sized(&c["rot90ccw"]), "{k}");
        assert_eq!(rot180(&src), sized(&c["rot180"]), "{k}");
        for sc in c["scale"].as_array().expect("scale") {
            assert_eq!(scale_int(&src, n(&sc["n"]) as u32), sized(sc), "{k}");
        }
    }
    assert_eq!(cs.len(), 20);
}

#[test]
fn edit_shift_wraps_like_the_mvp_offset() {
    let f = fx();
    let cs = f["offset_wrap"].as_array().expect("ow");
    for (k, c) in cs.iter().enumerate() {
        let src = img(n(&c["w"]), n(&c["h"]), &c["data"]);
        let out = offset_wrap(&src, n(&c["dx"]) as i32, n(&c["dy"]) as i32);
        assert_eq!(out.values(), arr(&c["out"]), "{k}");
    }
    assert_eq!(cs.len(), 30);
}
