//! **As leis da P6 que nenhuma fixtura cobre sozinha**: RotSprite a 90° é o giro exacto; espelhar
//! duas vezes devolve o documento; pintar fora da selecção não é passo; a selecção e a
//! transformação desfazem-se num passo; o formato não grava a selecção; os ops de tamanho voltam
//! ao byte.

use std::collections::BTreeMap;

use ph2d_pixel_core::affine::{Affine, Linear, Resample, transform_image};
use ph2d_pixel_core::canvas::{self, SizeMethod};
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::edit::{self, Rotation};
use ph2d_pixel_core::fill::Bucket;
use ph2d_pixel_core::selection::{self, Mask, SelectMode, Wand, wand};
use ph2d_pixel_core::transform::{Image, lift, rot90ccw, rot90cw};
use ph2d_pixel_core::{
    Background, ColorMode, History, NewSprite, Op, OpError, Paint, Patch, Rect, Sprite, Stroke,
    apply,
};

fn sprite(mode: ColorMode) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 12,
        height: 9,
        mode,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = Rect::new(1, 1, 10, 7);
    let bpp = mode.bpp();
    let mut pixels = Vec::new();
    for i in 0..r.area() as u32 {
        let v = match mode {
            ColorMode::Rgba => u32::from_le_bytes([(i * 37) as u8, (i * 11) as u8, 3, 255]),
            ColorMode::Gray => ((i * 29) % 256) | 0xff00,
            ColorMode::Indexed => 1 + (i * 7 + i * i / 3) % 20,
        };
        pixels.extend_from_slice(&v.to_le_bytes()[..bpp]);
    }
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

fn key(s: &Sprite) -> (ph2d_pixel_core::LayerId, ph2d_pixel_core::FrameId) {
    (s.layers()[0].id, s.frames()[0].id)
}

fn pixels(s: &Sprite) -> Vec<u8> {
    s.read_rect(key(s), s.bounds())
}

fn select(s: &mut Sprite, m: &Mask) {
    let op = selection::select(s, m, SelectMode::Replace).expect("select");
    apply(s, op).expect("mask");
}

#[test]
fn rotsprite_at_ninety_degrees_is_the_exact_quarter_turn() {
    let mut seed = 0x1234_5678u32;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };
    let mut cases = 0;
    for w in 1..9u32 {
        for h in 1..9u32 {
            let vals: Vec<u32> = (0..w * h).map(|_| next() % 4).collect();
            let src = Image::from_values(w, h, 1, &vals);
            for (deg, exact) in [(90.0, rot90cw(&src)), (-90.0, rot90ccw(&src))] {
                let t = Affine::compose(
                    Linear {
                        rot: deg,
                        sx: 1.0,
                        sy: 1.0,
                        skew: 0.0,
                    },
                    3.0,
                    2.0,
                );
                for m in [Resample::RotSprite, Resample::Fast] {
                    let (_, img) = transform_image(&src, &t, m, 0, None).expect("90");
                    assert_eq!(img, exact, "{w}x{h} {deg} {m:?}");
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 256);
}

#[test]
fn flipping_twice_gives_back_the_document_with_and_without_a_selection() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        for sel in [None, Some(Mask::marquee(12, 9, (2, 1), (8, 6)))] {
            for horizontal in [true, false] {
                let mut s = sprite(mode);
                if let Some(m) = &sel {
                    select(&mut s, m);
                }
                let (before, mask) = (pixels(&s), s.mask().cloned());
                for _ in 0..2 {
                    let op = edit::flip(&s, key(&s), horizontal, 0).expect("flip");
                    apply(&mut s, op).expect("flip");
                }
                assert_eq!(pixels(&s), before, "{mode:?} {horizontal}");
                assert_eq!(s.mask().cloned(), mask);
            }
        }
    }
}

#[test]
fn painting_fully_outside_the_selection_is_no_step_and_inside_is_clipped() {
    let mut s = sprite(ColorMode::Rgba);
    let k = key(&s);
    select(&mut s, &Mask::marquee(12, 9, (2, 2), (5, 5)));
    let red = u32::from_le_bytes([255, 0, 0, 255]);
    // Fora: lápis, linha e balde não dão passo.
    let mut st = Stroke::begin(&s, k.0, k.1, red, Paint::default(), (8, 7));
    st.to(&s, 11, 8);
    assert!(
        st.op(&s).is_none(),
        "lápis fora da selecção mudou o documento"
    );
    let mut b = Stroke::new(k.0, k.1, red, Paint::default());
    b.fill(&s, (9, 7), Bucket::default());
    assert!(
        b.op(&s).is_none(),
        "balde fora da selecção mudou o documento"
    );
    // Atravessando: só as células de dentro mudam.
    let mut st = Stroke::begin(&s, k.0, k.1, red, Paint::default(), (0, 3));
    st.to(&s, 11, 3);
    let before = s.clone();
    let mut h = History::default();
    let op = st.op(&s).expect("dentro");
    h.commit(&mut s, op).expect("commit");
    for x in 0..12 {
        let inside = (2..=5).contains(&x);
        let changed = s.value_at(k, x, 3) != before.value_at(k, x, 3);
        assert_eq!(
            changed,
            inside && before.value_at(k, x, 3) != red,
            "x = {x}"
        );
    }
    assert_eq!(h.undo_len(), 1);
}

#[test]
fn a_selection_and_a_transform_commit_are_one_undo_step_each() {
    let mut s = sprite(ColorMode::Indexed);
    let k = key(&s);
    let orig = s.clone();
    let mut h = History::default();
    let m = Mask::marquee(12, 9, (2, 1), (7, 4));
    let op = selection::select(&s, &m, SelectMode::Replace).expect("sel");
    h.commit(&mut s, op).expect("sel");
    let f = lift(&s, k, Some(&m)).expect("lift");
    let t = Affine::around_center(Linear::from_screen_ccw(30.0, 1.0, 1.0, 0.0), f.rect());
    let moved = edit::transform(&f, &t, Resample::RotSprite, s.transparent_value()).expect("t");
    let op = edit::commit(&s, k, &m, &moved, 0).expect("commit");
    assert!(matches!(op, Op::Batch(_)), "células e selecção num passo");
    let selected = s.clone();
    h.commit(&mut s, op).expect("commit");
    assert_ne!(s, selected);
    assert_eq!(h.undo_len(), 2);
    assert!(h.undo(&mut s));
    assert_eq!(s, selected, "o desfazer da transformação volta ao byte");
    assert!(h.undo(&mut s));
    assert_eq!(s, orig, "o desfazer da selecção tira-a");
    assert!(h.redo(&mut s) && h.redo(&mut s));
    assert!(s.mask().is_some());
}

#[test]
fn reselect_brings_back_the_mask_that_deselect_took() {
    let mut s = sprite(ColorMode::Rgba);
    let m = Mask::ellipse(12, 9, (1, 1), (9, 7));
    select(&mut s, &m);
    let op = selection::deselect(&s).expect("deselect");
    apply(&mut s, op).expect("deselect");
    assert!(s.mask().is_none());
    let op = selection::reselect(&s).expect("reselect");
    apply(&mut s, op).expect("reselect");
    assert_eq!(s.mask(), Some(&m));
}

#[test]
fn the_saved_format_never_carries_the_selection() {
    let mut s = sprite(ColorMode::Rgba);
    let plain = s.clone();
    select(&mut s, &Mask::marquee(12, 9, (0, 0), (3, 3)));
    let lib: BTreeMap<u64, Sprite> = [(7, s.clone())].into();
    let back = read_library(&write_library(&lib)).expect("read");
    assert_eq!(
        back[&7], plain,
        "a selecção não se grava; o resto volta ao byte"
    );
    let plain_lib: BTreeMap<u64, Sprite> = [(7, plain)].into();
    assert_eq!(write_library(&lib), write_library(&plain_lib));
}

#[test]
fn size_ops_undo_to_the_byte_and_move_every_cel() {
    let mut s = sprite(ColorMode::Rgba);
    let first = key(&s).1;
    let (op, _) = ph2d_pixel_core::frames::new_frame(
        &mut s,
        first,
        ph2d_pixel_core::frames::NewFrame::Linked,
    )
    .expect("frame");
    apply(&mut s, op).expect("frame");
    select(&mut s, &Mask::marquee(12, 9, (1, 1), (4, 4)));
    let orig = s.clone();
    let ops: Vec<Op> = vec![
        canvas::rotate_canvas(&s, Rotation::Cw90).expect("rot"),
        canvas::sprite_size(&s, 30, 5, SizeMethod::Bilinear).expect("size"),
        canvas::canvas_size(
            &s,
            canvas::Borders {
                left: -2,
                top: 3,
                right: 1,
                bottom: 0,
            },
            0,
        )
        .expect("canvas"),
        canvas::crop_to_selection(&s).expect("crop"),
        canvas::flip_canvas(&s, true).expect("flip"),
    ];
    for op in ops {
        let mut t = orig.clone();
        let inv = apply(&mut t, op).expect("op");
        assert!(t.mask().is_none(), "o tamanho novo tira a selecção");
        let f = t.frames().to_vec();
        assert_eq!(f, orig.frames(), "frames intactos");
        let n = t.entries().count();
        assert_eq!(n, orig.entries().count(), "um cel por cel");
        apply(&mut t, inv).expect("inv");
        assert_eq!(t, orig);
    }
}

#[test]
fn a_batch_that_fails_halfway_leaves_the_document_intact() {
    let mut s = sprite(ColorMode::Rgba);
    let orig = s.clone();
    let good = selection::select_all(&s).expect("all");
    let bad = Op::Mask(Box::new(ph2d_pixel_core::MaskSet {
        mask: Some(Mask::full(3, 3)),
        last: None,
    }));
    assert_eq!(
        apply(&mut s, Op::Batch(vec![good, bad])),
        Err(OpError::BadMask)
    );
    assert_eq!(s, orig);
}

#[test]
fn the_wand_on_indexed_compares_the_palette_colours() {
    let mut s = sprite(ColorMode::Indexed);
    let k = key(&s);
    let mut pal = s.palette().to_vec();
    pal[2] = pal[1]; // dois índices com a MESMA cor
    let transparent = s.transparent_index();
    apply(
        &mut s,
        Op::Palette {
            colors: pal,
            transparent,
        },
    )
    .expect("pal");
    let seed = (0..12)
        .flat_map(|x| (0..9).map(move |y| (x, y)))
        .find(|&(x, y)| s.value_at(k, x, y) == 1)
        .expect("índice 1");
    let opt = Wand {
        contiguous: false,
        ..Wand::default()
    };
    let m = wand(&s, k, seed, opt);
    for (x, y) in (0..12).flat_map(|x| (0..9).map(move |y| (x, y))) {
        let v = s.value_at(k, x, y);
        assert_eq!(m.get(x, y), v == 1 || v == 2, "({x},{y}) = {v}");
    }
}

#[test]
fn edit_rotate_ninety_with_a_selection_moves_the_mask_with_the_pixels() {
    let mut s = sprite(ColorMode::Rgba);
    let k = key(&s);
    select(&mut s, &Mask::marquee(12, 9, (2, 2), (7, 4)));
    let op = edit::rotate(&s, k, Rotation::Cw90, 0).expect("rot");
    apply(&mut s, op).expect("rot");
    let b = s.mask().and_then(Mask::bounds).expect("mask");
    assert_eq!((b.w, b.h), (3, 6));
    assert_eq!((b.x, b.y), edit::rot90_origin(Rect::new(2, 2, 6, 3)));
}
