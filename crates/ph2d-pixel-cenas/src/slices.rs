//! A cena 24 (P11, `docs/PixelArt/P11_plano.md` §8): um botão de jogo e um boneco a andar, em 6 quadros.
//! O miolo do 9-slice cai na fronteira dos cantos com o enchimento; o pivot cai no pé do boneco.

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{
    LayerId, Op, Patch, Rect, Slice, SliceId, SliceKey, SliceShape, Sprite, apply, layers,
};

/// 72 de largura: `FIT_FRACTION` abre a 800 % numa janela de 1930 × 1040.
pub const SIZE: (u32, u32) = (72, 48);
pub const FRAMES: u32 = 6;

/// O botão no sprite: moldura de [`BUTTON_BORDER`] píxeis à volta do enchimento.
pub const BUTTON: Rect = Rect::new(4, 4, 36, 14);
pub const BUTTON_BORDER: i32 = 4;
/// O miolo do 9-slice, relativo à caixa: exatamente o enchimento.
pub const BUTTON_CENTER: Rect = Rect::new(
    BUTTON_BORDER,
    BUTTON_BORDER,
    BUTTON.w - 2 * BUTTON_BORDER as u32,
    BUTTON.h - 2 * BUTTON_BORDER as u32,
);

/// O boneco: a caixa, a 1.ª linha e o pivot relativo à caixa (o píxel do pé que está sempre no chão).
pub const HERO_SIZE: (u32, u32) = (10, 14);
pub const HERO_Y: i32 = 28;
pub const HERO_PIVOT: (i32, i32) = (5, 13);
const HERO_STEP: i32 = 10;
const HERO_START: i32 = 6;
/// A primeira linha do chão: o pé assenta na anterior.
pub const GROUND_Y: i32 = HERO_Y + HERO_SIZE.1 as i32;

pub const COLOR_BUTTON: [u8; 4] = [0, 140, 70, 255];
pub const COLOR_HERO: [u8; 4] = [230, 30, 170, 255];

const TRANSPARENT: [u8; 4] = [0; 4];
const OUTLINE: [u8; 4] = [46, 28, 20, 255];
const LIGHT: [u8; 4] = [255, 224, 150, 255];
const DARK: [u8; 4] = [176, 92, 34, 255];
pub const FILL: [u8; 4] = [236, 150, 54, 255];

const SKY: [u8; 4] = [188, 224, 244, 255];
const CLOUD: [u8; 4] = [250, 252, 255, 255];
/// As nuvens: linha, primeira coluna e largura de cada faixa.
const CLOUDS: [(i32, i32, i32); 6] = [
    (8, 46, 8),
    (9, 44, 13),
    (10, 45, 10),
    (22, 56, 7),
    (23, 54, 11),
    (24, 55, 9),
];
const GRASS: [u8; 4] = [96, 176, 72, 255];
const DIRT: [u8; 4] = [104, 72, 48, 255];

const HAIR: [u8; 4] = [80, 44, 28, 255];
const SKIN: [u8; 4] = [244, 196, 152, 255];
const EYE: [u8; 4] = [20, 20, 30, 255];
const SHIRT: [u8; 4] = [228, 56, 56, 255];
const PANTS: [u8; 4] = [52, 96, 200, 255];
pub const BOOT: [u8; 4] = [64, 40, 28, 255];

const HERO_TOP: [&[u8; 10]; 10] = [
    b"...HHHH...",
    b"..HHHHHH..",
    b"..HSSSEH..",
    b"..HSSSSS..",
    b"...SSSS...",
    b"...RRRR...",
    b"..RRRRRR..",
    b"..SRRRRS..",
    b"...RRRR...",
    b"...PPPP...",
];
/// As pernas a passar uma pela outra.
const LEGS_PASS: [&[u8; 10]; 4] = [b"....PPP...", b"....PPP...", b"....PP....", b"....BBB..."];
/// As pernas abertas: a de trás ainda toca o chão, a da frente planta no pivot.
const LEGS_STRIDE: [&[u8; 10]; 4] = [b"...PPPP...", b"..PP.PP...", b".PP..PP...", b".B...BBB.."];

/// A caixa do boneco no quadro `k`.
#[must_use]
pub const fn hero_box(k: u32) -> Rect {
    Rect::new(
        HERO_START + k as i32 * HERO_STEP,
        HERO_Y,
        HERO_SIZE.0,
        HERO_SIZE.1,
    )
}

fn hero_cell(k: u32, x: i32, y: i32) -> [u8; 4] {
    let row = match y {
        0..10 => HERO_TOP[y as usize],
        10..14 if k.is_multiple_of(2) => LEGS_PASS[y as usize - 10],
        10..14 => LEGS_STRIDE[y as usize - 10],
        _ => return TRANSPARENT,
    };
    match row.get(x as usize) {
        Some(b'H') => HAIR,
        Some(b'S') => SKIN,
        Some(b'E') => EYE,
        Some(b'R') => SHIRT,
        Some(b'P') => PANTS,
        Some(b'B') => BOOT,
        _ => TRANSPARENT,
    }
}

fn button_cell(x: i32, y: i32) -> [u8; 4] {
    let (w, h) = (BUTTON.w as i32, BUTTON.h as i32);
    let (cx, cy) = (x.min(w - 1 - x), y.min(h - 1 - y));
    let to_edge = cx.min(cy);
    match (cx + cy, to_edge) {
        (0..2, _) => TRANSPARENT,
        (2, _) | (_, 0) => OUTLINE,
        (_, d) if d >= BUTTON_BORDER => FILL,
        _ if y < BUTTON_BORDER || (x < BUTTON_BORDER && y < h - BUTTON_BORDER) => LIGHT,
        _ => DARK,
    }
}

/// A camada de cima no quadro `k`: o botão, parado, e o boneco.
#[must_use]
pub fn art_rgba(k: u32, x: i32, y: i32) -> [u8; 4] {
    if BUTTON.contains(x, y) {
        return button_cell(x - BUTTON.x, y - BUTTON.y);
    }
    let b = hero_box(k);
    if b.contains(x, y) {
        return hero_cell(k, x - b.x, y - b.y);
    }
    TRANSPARENT
}

/// A camada de baixo: céu com nuvens, relva e terra. Igual em todos os quadros.
#[must_use]
pub fn background_rgba(x: i32, y: i32) -> [u8; 4] {
    match y - GROUND_Y {
        d if d >= 2 => DIRT,
        d if d >= 0 => GRASS,
        _ if CLOUDS
            .iter()
            .any(|&(r, x0, w)| y == r && (x0..x0 + w).contains(&x)) =>
        {
            CLOUD
        }
        _ => SKY,
    }
}

fn named(id: SliceId, name: &str, color: [u8; 4], keys: Vec<SliceKey>) -> Slice {
    Slice {
        id,
        name: name.to_owned(),
        user: ph2d_pixel_core::UserData {
            color,
            ..Default::default()
        },
        keys,
    }
}

fn patch(s: &mut Sprite, layer: LayerId, k: u32, f: impl Fn(i32, i32) -> [u8; 4]) -> Option<()> {
    let rect = s.bounds();
    let frame = s.frames().get(k as usize)?.id;
    let mut pixels = Vec::with_capacity(rect.area() * 4);
    for y in 0..rect.h as i32 {
        for x in 0..rect.w as i32 {
            pixels.extend_from_slice(&f(x, y));
        }
    }
    let op = Op::Patch(Patch {
        layer,
        frame,
        bbox: Some(rect),
        rect,
        pixels,
    });
    apply(s, op).ok().map(|_| ())
}

fn slices(s: &mut Sprite) -> SliceId {
    let (button, hero) = (SliceId(s.reserve_id()), SliceId(s.reserve_id()));
    let forma = SliceShape {
        bounds: BUTTON,
        center: Some(BUTTON_CENTER),
        pivot: None,
    };
    let passos = (0..FRAMES)
        .map(|frame| SliceKey {
            frame,
            shape: SliceShape {
                bounds: hero_box(frame),
                center: None,
                pivot: Some(HERO_PIVOT),
            },
        })
        .collect();
    let lista = vec![
        named(
            button,
            "button",
            COLOR_BUTTON,
            vec![SliceKey {
                frame: 0,
                shape: forma,
            }],
        ),
        named(hero, "hero", COLOR_HERO, passos),
    ];
    let _ = apply(s, Op::Slices(lista));
    hero
}

/// **Monta a cena sobre um sprite novo**: fundo ligado, arte por quadro, as duas slices. Fora do
/// desfazer: o `Ctrl+Z` do smoke nunca desmonta a cena. Devolve a camada da arte e o id de `hero`.
pub fn monta(s: &mut Sprite) -> Option<(LayerId, SliceId)> {
    let fundo = s.layers().first()?.id;
    let op = layers::edit(s, fundo, |l| {
        "Background".clone_into(&mut l.name);
        l.continuous = true;
    })?;
    apply(s, op).ok()?;
    patch(s, fundo, 0, background_rgba)?;
    let (op, arte) = layers::new_layer(s, fundo)?;
    apply(s, op).ok()?;
    let op = layers::edit(s, arte, |l| "Art".clone_into(&mut l.name))?;
    apply(s, op).ok()?;
    for _ in 1..FRAMES {
        let ultimo = s.frames().last()?.id;
        let (op, _) = frames::new_frame(s, ultimo, NewFrame::Copy)?;
        apply(s, op).ok()?;
    }
    for k in 0..FRAMES {
        patch(s, arte, k, |x, y| art_rgba(k, x, y))?;
    }
    Some((arte, slices(s)))
}
