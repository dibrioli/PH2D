//! **P2 contra o MVP web** — a fixtura `mvp_color.json` (`docs/PixelArt/ferramentas/extrai_cor_mvp.mjs`):
//! permutações da paleta, duplicadas, hex e a lei do remap. As famílias da P13 (OKLCH, rampas,
//! quantização por rampa, cor da anotação, ciclo de cores) ficam na fixtura para a onda delas.

use ph2d_pixel_core::palette::{
    find_duplicates, invert_permutation, move_permutation, parse_hex, to_hex,
};
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, NewSprite, Op, Patch, Rect, Sprite, apply, compose_rgba,
};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/mvp_color.json")).expect("json")
}

fn list<'a>(v: &'a Value, k: &str, min: usize) -> &'a Vec<Value> {
    let a = v[k].as_array().expect(k);
    assert!(a.len() >= min, "a fixtura `{k}` encolheu: {}", a.len());
    a
}

fn n(v: &Value) -> usize {
    v.as_u64().expect("int") as usize
}

fn rgba(v: &Value) -> [u8; 4] {
    let a = v.as_array().expect("rgb");
    [n(&a[0]) as u8, n(&a[1]) as u8, n(&a[2]) as u8, 255]
}

#[test]
fn move_and_invert_permutation_match_the_mvp() {
    let f = fx();
    for c in list(&f["palette"], "move_permutation", 9) {
        let p = move_permutation(n(&c["size"]), n(&c["from"]), n(&c["to"]));
        let want: Vec<usize> = c["perm"].as_array().expect("perm").iter().map(n).collect();
        assert_eq!(p, want, "{c}");
        let inv: Vec<usize> = c["inverse"]
            .as_array()
            .expect("inv")
            .iter()
            .map(n)
            .collect();
        assert_eq!(invert_permutation(&p), inv, "{c}");
        assert_eq!(invert_permutation(&inv), p, "inverter 2× é a identidade");
    }
}

#[test]
fn duplicates_point_to_the_first_and_skip_the_transparent() {
    let f = fx();
    for c in list(&f["palette"], "duplicates", 5) {
        let colors: Vec<[u8; 4]> = c["colors"]
            .as_array()
            .expect("c")
            .iter()
            .map(rgba)
            .collect();
        let got = find_duplicates(&colors, Some(n(&c["transparent"])));
        let want: Vec<(usize, usize)> = c["dup"]
            .as_array()
            .expect("dup")
            .iter()
            .map(|p| (n(&p[0]), n(&p[1])))
            .collect();
        assert_eq!(got, want, "{c}");
    }
}

#[test]
fn hex_parses_and_formats_like_the_mvp() {
    let f = fx();
    for c in list(&f["palette"], "hex", 13) {
        let got = parse_hex(c["text"].as_str().expect("t"));
        let want = (!c["rgb"].is_null()).then(|| rgba(&c["rgb"]));
        assert_eq!(got, want, "{c}");
    }
    for c in list(&f["palette"], "to_hex", 5) {
        assert_eq!(to_hex(rgba(&c["rgb"])), c["hex"].as_str().expect("hex"));
    }
}

/// ★ A lei do remap (o teste do MVP com o NOSSO documento): reordenar a paleta deixa o RGBA
/// composto byte-igual, e desfazer devolve os NÚMEROS.
#[test]
fn remap_keeps_the_composed_image_byte_identical_and_undo_restores_the_indices() {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
    let pixels: Vec<u8> = (0..64).map(|i| (i % 17) as u8).collect();
    apply(
        &mut s,
        Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(Rect::new(0, 0, 8, 8)),
            rect: Rect::new(0, 0, 8, 8),
            pixels: pixels.clone(),
        }),
    )
    .expect("patch");
    let before = rgba_of(&s, frame);
    let perm = move_permutation(s.palette().len(), 2, 11);
    let inv = apply(&mut s, Op::Remap { perm: perm.clone() }).expect("remap");
    assert_eq!(
        rgba_of(&s, frame),
        before,
        "o remap tem de deixar a imagem byte-igual"
    );
    assert_ne!(
        s.read_rect((layer, frame), Rect::new(0, 0, 8, 8)),
        pixels,
        "controlo: os ÍNDICES mudaram (senão o teste não prova nada)"
    );
    apply(&mut s, inv).expect("undo");
    assert_eq!(s.read_rect((layer, frame), Rect::new(0, 0, 8, 8)), pixels);
}

fn rgba_of(s: &Sprite, frame: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, frame, None, &mut out);
    out
}

/// RGB → cinza contra o Aseprite 1.3.18.6 OBSERVADO (`oracle_trial_color_mode.json`): os três
/// métodos, 216 cores cada, exatos.
#[test]
fn to_gray_matches_the_aseprite_trial_for_luminance_hsv_and_hsl() {
    use ph2d_pixel_core::color::ToGray;
    let f: Value = serde_json::from_str(include_str!("../fixtures/oracle_trial_color_mode.json"))
        .expect("json");
    let input = list(&f["input"], "rgb", 216);
    for (key, method) in [
        ("luminance", ToGray::Luminance),
        ("hsv", ToGray::Hsv),
        ("hsl", ToGray::Hsl),
    ] {
        let want = list(&f["gray"], key, 216);
        for (c, w) in input.iter().zip(want) {
            assert_eq!(usize::from(method.of(rgba(c))), n(w), "{key} {c}");
        }
    }
}
