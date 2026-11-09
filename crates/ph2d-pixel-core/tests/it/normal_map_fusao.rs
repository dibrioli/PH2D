//! **P14c-pixel 1b — a camada Normal Map nas fusões e no modo de cor**: o Merge Down entre normal e
//! cor recusa-se (nos dois sentidos), duas Normal Map fundem-se numa Normal Map, o Flatten guarda a
//! normal numa Normal Map à parte, e a normal só existe num sprite RGB.

use ph2d_pixel_core::convert::{Convert, convert, normal_needs_rgb};
use ph2d_pixel_core::layers;
use ph2d_pixel_core::{
    Background, ColorMode, FrameId, LayerId, NewSprite, Op, Patch, Rect, Sprite, apply,
    compose_normal, compose_rgba,
};

fn frame(s: &Sprite) -> FrameId {
    s.frames()[0].id
}

fn fill(s: &mut Sprite, l: LayerId, r: Rect, rgba: [u8; 4]) {
    let v = s.value_of_rgba(rgba).to_le_bytes();
    let bpp = s.mode().bpp();
    let pixels = (0..r.area()).flat_map(|_| v[..bpp].to_vec()).collect();
    let bbox = s.cel((l, frame(s))).map_or(r, |c| c.bbox.union(r));
    let op = Op::Patch(Patch {
        layer: l,
        frame: frame(s),
        bbox: Some(bbox),
        rect: r,
        pixels,
    });
    apply(s, op).expect("patch");
}

fn cor(s: &Sprite) -> Vec<u8> {
    let mut out = Vec::new();
    compose_rgba(s, frame(s), None, &mut out);
    out
}

fn sprite(mode: ColorMode) -> Sprite {
    Sprite::new(NewSprite {
        width: 6,
        height: 4,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn nova(s: &mut Sprite, acima: LayerId, normal: bool) -> LayerId {
    let (op, id) = layers::new_layer(s, acima).expect("camada");
    apply(s, op).expect("camada");
    if normal {
        let op = layers::set_normal(s, id, true).expect("normal");
        apply(s, op).expect("normal");
    }
    id
}

/// «arte» vermelha em (0,0,4,3) e, por cima, «n» Normal Map pintada em (1,0,2,1).
fn cena() -> (Sprite, LayerId, LayerId) {
    let mut s = sprite(ColorMode::Rgba);
    let arte = s.layers()[0].id;
    fill(&mut s, arte, Rect::new(0, 0, 4, 3), [200, 30, 40, 255]);
    let n = nova(&mut s, arte, true);
    fill(&mut s, n, Rect::new(1, 0, 2, 1), [255, 128, 128, 255]);
    (s, arte, n)
}

#[test]
fn merge_down_entre_normal_e_cor_recusa_nos_dois_sentidos() {
    let (mut s, _, n) = cena();
    assert!(layers::merge_down_mixes_normal(&s, n));
    assert!(layers::merge_down(&s, n).is_none(), "normal sobre cor");
    let c = nova(&mut s, n, false);
    fill(&mut s, c, Rect::new(0, 0, 1, 1), [0, 200, 0, 255]);
    assert!(layers::merge_down_mixes_normal(&s, c));
    assert!(layers::merge_down(&s, c).is_none(), "cor sobre normal");
    // cor sobre cor continua a fundir
    let mut t = sprite(ColorMode::Rgba);
    let a = t.layers()[0].id;
    let b = nova(&mut t, a, false);
    assert!(!layers::merge_down_mixes_normal(&t, b));
    assert!(layers::merge_down(&t, b).is_some());
}

#[test]
fn merge_down_de_duas_normal_funde_e_fica_normal() {
    let (mut s, _, n) = cena();
    let n2 = nova(&mut s, n, true);
    fill(&mut s, n2, Rect::new(2, 0, 1, 1), [0, 128, 128, 255]);
    let antes = cor(&s);
    assert!(!layers::merge_down_mixes_normal(&s, n2));
    let op = layers::merge_down(&s, n2).expect("duas Normal Map fundem-se");
    apply(&mut s, op).expect("merge");
    assert_eq!(s.layers().len(), 2);
    assert!(s.layer(n).is_some_and(|l| l.normal), "continua Normal Map");
    assert_eq!(cor(&s), antes, "a cor não mudou");
    let px = compose_normal(&s, frame(&s));
    let at = |x: usize| px[x * 4..x * 4 + 4].to_vec(); // linha 0
    assert_eq!(at(1), [255, 128, 128, 255]);
    assert_eq!(at(2), [0, 128, 128, 255]);
}

#[test]
fn flatten_achata_a_cor_numa_e_a_normal_noutra() {
    for visible_only in [false, true] {
        let (mut s, _, _) = cena();
        let (antes_cor, antes_n) = (cor(&s), compose_normal(&s, frame(&s)));
        let op = layers::flatten(&mut s, visible_only).expect("flatten");
        apply(&mut s, op).expect("flatten");
        let kinds: Vec<bool> = s.layers().iter().map(|l| l.normal).collect();
        assert_eq!(kinds, [false, true], "cor em baixo, Normal Map por cima");
        assert_eq!(cor(&s), antes_cor);
        assert_eq!(compose_normal(&s, frame(&s)), antes_n, "a normal não saiu");
    }
    // sem Normal Map, uma camada só (como antes)
    let mut t = sprite(ColorMode::Rgba);
    let a = t.layers()[0].id;
    fill(&mut t, a, Rect::new(0, 0, 1, 1), [9, 9, 9, 255]);
    let _ = nova(&mut t, a, false);
    let op = layers::flatten(&mut t, false).expect("flatten");
    apply(&mut t, op).expect("flatten");
    assert_eq!(t.layers().len(), 1);
}

#[test]
fn a_normal_so_existe_num_sprite_rgb() {
    for mode in [ColorMode::Indexed, ColorMode::Gray] {
        let mut s = sprite(mode);
        let l = s.layers()[0].id;
        let op = layers::set_normal(&s, l, true).expect("op");
        assert!(apply(&mut s, op).is_err(), "{mode:?}: SetLayerNormal recusado");
        assert!(s.layer(l).is_some_and(|l| !l.normal));
    }
    let (s, _, _) = cena();
    for mode in [ColorMode::Indexed, ColorMode::Gray] {
        assert!(normal_needs_rgb(&s, mode));
        assert!(convert(&s, mode, Convert::default()).is_none(), "{mode:?}");
    }
    let mut t = sprite(ColorMode::Rgba);
    assert!(!normal_needs_rgb(&t, ColorMode::Indexed));
    assert!(convert(&t, ColorMode::Indexed, Convert::default()).is_some());
    // desligar a normal continua possível em qualquer modo (o inverso do passo)
    let l = t.layers()[0].id;
    let op = layers::set_normal(&t, l, true).expect("op");
    apply(&mut t, op).expect("RGB aceita");
    let op = layers::set_normal(&t, l, false).expect("op");
    apply(&mut t, op).expect("desliga");
}
