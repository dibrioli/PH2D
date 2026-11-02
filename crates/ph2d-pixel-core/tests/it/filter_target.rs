//! **Onde um filtro age** (P8_plano §2.2): fora da selecção fica intocado, um canal desligado fica
//! como estava, o filtro lê o original (não se alimenta), nada a mudar não é passo, e um passo do
//! desfazer devolve o cel ao byte.

use ph2d_pixel_core::filter::{self, Channels, Filter, Source};
use ph2d_pixel_core::selection::{Mask, SelectMode, select};
use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Op, Patch, Rect, Sprite, apply};

fn sprite() -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 6,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let r = s.bounds();
    let pixels = (0..r.area() as u32)
        .flat_map(|i| [(i * 37) as u8, (i * 11) as u8, (i * 5) as u8, 255])
        .collect();
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

fn rgba(s: &Sprite, x: i32, y: i32) -> [u8; 4] {
    s.rgba_of_value(s.value_at(key(s), x, y))
}

struct Negate;
impl Filter for Negate {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let c = src.get(x, y).unwrap_or_default();
        [255 - c[0], 255 - c[1], 255 - c[2], c[3]]
    }
}

/// Copia o vizinho da esquerda: se o filtro lesse o que já escreveu, a linha inteira viraria a 1.ª.
struct FromLeft;
impl Filter for FromLeft {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        src.get(x - 1, y)
            .or_else(|| src.get(x, y))
            .unwrap_or_default()
    }
}

struct Same;
impl Filter for Same {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        src.get(x, y).unwrap_or_default()
    }
}

#[test]
fn outside_the_selection_nothing_changes() {
    let mut s = sprite();
    let m = Mask::from_rect(8, 6, Rect::new(2, 1, 3, 2));
    let op = select(&s, &m, SelectMode::Replace).expect("select");
    apply(&mut s, op).expect("select");
    let before = s.clone();
    let op = filter::apply(&s, &[key(&s)], Channels::ALL, &Negate).expect("muda");
    apply(&mut s, op).expect("filtro");
    for y in 0..6 {
        for x in 0..8 {
            let (a, b) = (rgba(&before, x, y), rgba(&s, x, y));
            if m.get(x, y) {
                assert_eq!(
                    b,
                    [255 - a[0], 255 - a[1], 255 - a[2], a[3]],
                    "({x},{y}) dentro"
                );
            } else {
                assert_eq!(a, b, "({x},{y}) fora da selecção mudou");
            }
        }
    }
}

#[test]
fn a_channel_turned_off_keeps_its_value() {
    let mut s = sprite();
    let before = s.clone();
    let ch = Channels {
        r: true,
        g: false,
        b: true,
        a: true,
    };
    let op = filter::apply(&s, &[key(&s)], ch, &Negate).expect("muda");
    apply(&mut s, op).expect("filtro");
    for (x, y) in [(0, 0), (3, 2), (7, 5)] {
        let (a, b) = (rgba(&before, x, y), rgba(&s, x, y));
        assert_eq!(b, [255 - a[0], a[1], 255 - a[2], a[3]], "({x},{y})");
    }
}

#[test]
fn the_filter_reads_the_original_not_what_it_wrote() {
    let mut s = sprite();
    let before = s.clone();
    let op = filter::apply(&s, &[key(&s)], Channels::ALL, &FromLeft).expect("muda");
    apply(&mut s, op).expect("filtro");
    for x in 1..8 {
        assert_eq!(rgba(&s, x, 3), rgba(&before, x - 1, 3), "x={x}");
    }
}

#[test]
fn nothing_to_change_is_not_a_step() {
    let s = sprite();
    assert!(filter::apply(&s, &[key(&s)], Channels::ALL, &Same).is_none());
    let none = Channels {
        r: false,
        g: false,
        b: false,
        a: false,
    };
    assert!(filter::apply(&s, &[key(&s)], none, &Negate).is_none());
}

#[test]
fn one_undo_gives_the_cel_back_to_the_byte() {
    let mut s = sprite();
    let before = s.clone();
    let mut h = History::default();
    let k = key(&s);
    let op = filter::apply(&s, &[k, k], Channels::ALL, &Negate).expect("muda");
    assert!(
        matches!(op, Op::Patch(_)),
        "o mesmo cel pedido 2× conta uma vez"
    );
    h.commit(&mut s, op).expect("commit");
    assert_ne!(s, before);
    assert!(h.undo(&mut s));
    assert_eq!(s, before);
}
