//! **A cena 9 — a selecção e a transformação que SE VÊEM** (P6). Um Pixel Sprite RGBA 128 × 64,
//! em Draw, com a ferramenta Rectangular Marquee na mão e UMA selecção activa feita de quatro
//! peças (as formigas marcham à volta de cada uma):
//!
//! - em cima, da esquerda para a direita: um **retângulo** de marquee atravessado por um **traço
//!   que só pintou dentro dele** (o traço foi de ponta a ponta com aquela selecção activa), uma
//!   **elipse**, uma **estrela de laço** e a **região da varinha** (o losango verde, um clique);
//!   à direita a mesma imagem 3 × 3 aumentada para 12 × 12 em **Nearest** e em **Bilinear**;
//! - em baixo: a **figura** original, a mesma figura **girada 30°** em **Fast** e em
//!   **RotSprite**, a figura **espelhada**, e um **ladrilho deslocado com volta** (Shift).

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::affine::{Affine, Linear, Resample};
use ph2d_pixel_core::selection::{self, Mask, SelectMode, Wand};
use ph2d_pixel_core::transform::{Floating, Image, flip_h, resize_bilinear, resize_nearest};
use ph2d_pixel_core::{ColorMode, NewSprite, Op, Paint, Patch, Rect, Stroke, edit};

use crate::PixelApp;

/// O tamanho da cena 9.
pub const SIZE: (u32, u32) = (128, 64);
/// A marquee retangular e o traço que a atravessa (linha `STROKE_Y`, de `0` a `STROKE_X1`).
pub const RECT: Rect = Rect::new(4, 4, 16, 12);
pub const STROKE_Y: i32 = 10;
pub const STROKE_X1: i32 = 24;
pub const STROKE: [u8; 4] = [235, 60, 60, 255];
/// A caixa da elipse (cantos inclusivos).
pub const ELLIPSE: ((i32, i32), (i32, i32)) = ((27, 4), (42, 19));
/// A estrela de laço (vértices em células).
pub const STAR: [(i32, i32); 10] = [
    (56, 2),
    (59, 9),
    (66, 9),
    (60, 13),
    (63, 20),
    (56, 16),
    (49, 20),
    (52, 13),
    (46, 9),
    (53, 9),
];
/// O losango da varinha: centro e raio (|dx| + |dy| ≤ r), e a cor dele.
pub const WAND: ((i32, i32), i32) = ((78, 12), 8);
pub const WAND_COLOR: [u8; 4] = [60, 180, 90, 255];
/// A imagem 3 × 3 aumentada: onde ficam a Nearest e a Bilinear (12 × 12).
pub const NEAREST_AT: (i32, i32) = (92, 4);
pub const BILINEAR_AT: (i32, i32) = (110, 4);
pub const SCALED: u32 = 12;
/// A figura (14 × 14) e onde ficam a original, as duas giradas, a espelhada e o ladrilho.
pub const FIGURE: u32 = 14;
pub const ORIGINAL_AT: (i32, i32) = (4, 42);
pub const FAST_AT: (i32, i32) = (26, 42);
pub const ROTSPRITE_AT: (i32, i32) = (50, 42);
pub const FLIP_AT: (i32, i32) = (74, 42);
pub const TILE_AT: (i32, i32) = (96, 42);
/// O giro das cópias (anti-horário, o campo do Aseprite) e o deslocamento do ladrilho.
pub const ANGLE: f64 = 30.0;
pub const SHIFT: (i32, i32) = (5, 4);

/// A imagem 3 × 3 de cores (RGBA) que a cena aumenta.
#[must_use]
pub fn small() -> Image {
    let c = [
        [230, 70, 70, 255],
        [240, 200, 60, 255],
        [70, 170, 90, 255],
        [70, 120, 230, 255],
        [250, 250, 250, 255],
        [150, 80, 200, 255],
        [40, 40, 40, 255],
        [200, 120, 60, 255],
        [90, 210, 220, 255],
    ];
    Image::map(3, 3, 4, |x, y| u32::from_le_bytes(c[(y * 3 + x) as usize]))
}

/// A figura: um «F» grosso de duas cores (sem simetria — o giro e o espelho leem-se nele).
#[must_use]
pub fn figure() -> Image {
    Image::map(FIGURE, FIGURE, 4, |x, y| {
        let haste = (2..5).contains(&x) && (1..13).contains(&y);
        let topo = (2..12).contains(&x) && (1..4).contains(&y);
        let meio = (2..9).contains(&x) && (6..8).contains(&y);
        let c: [u8; 4] = if topo {
            [240, 140, 40, 255]
        } else if haste || meio {
            [60, 90, 200, 255]
        } else {
            [0; 4]
        };
        u32::from_le_bytes(c)
    })
}

/// O ladrilho antes do Shift: um quadrante de cada cor (a volta lê-se nas costuras).
#[must_use]
pub fn tile() -> Image {
    let h = FIGURE / 2;
    Image::map(FIGURE, FIGURE, 4, |x, y| {
        let c: [u8; 4] = match (x < h, y < h) {
            (true, true) => [220, 60, 60, 255],
            (false, true) => [240, 210, 70, 255],
            (true, false) => [60, 170, 90, 255],
            (false, false) => [70, 110, 220, 255],
        };
        u32::from_le_bytes(c)
    })
}

/// A matriz de uma cópia girada `ANGLE` com o centro na caixa da figura em `at`.
#[must_use]
pub fn rotation_at(at: (i32, i32)) -> Affine {
    Affine::around_center(
        Linear::from_screen_ccw(ANGLE, 1.0, 1.0, 0.0),
        Rect::new(at.0, at.1, FIGURE, FIGURE),
    )
}

/// A figura girada pela amostragem `r`, como a cena a assenta.
#[must_use]
pub fn rotated(at: (i32, i32), r: Resample) -> Option<Floating> {
    edit::transform(&solta(0, 0, figure()), &rotation_at(at), r, 0)
}

/// O losango da varinha.
#[must_use]
pub fn in_wand_blob(x: i32, y: i32) -> bool {
    let ((cx, cy), r) = WAND;
    (x - cx).abs() + (y - cy).abs() <= r
}

fn solta(x: i32, y: i32, image: Image) -> Floating {
    let keep = vec![true; (image.w * image.h) as usize];
    Floating { x, y, image, keep }
}

/// A selecção que a cena deixa activa: as quatro peças somadas.
#[must_use]
pub fn mask(s: &ph2d_pixel_core::Sprite, key: ph2d_pixel_core::CelKey) -> Mask {
    let (w, h) = SIZE;
    let r = Mask::from_rect(w, h, RECT);
    let e = Mask::ellipse(w, h, ELLIPSE.0, ELLIPSE.1);
    let l = Mask::polygon(w, h, &STAR);
    let v = selection::wand(s, key, WAND.0, Wand::default());
    r.combine(&e, SelectMode::Add)
        .combine(&l, SelectMode::Add)
        .combine(&v, SelectMode::Add)
}

fn assenta(app: &mut PixelApp, key: u32, f: &Floating) -> Option<()> {
    let doc = app.docs.get(&key)?;
    let op = edit::paste(&doc.sprite, (doc.layer, doc.frame), f, 0)?;
    app.commit(key, op).ok()
}

/// Um passo (`None` = nada a mudar, como a selecção que o Paste já deixou).
fn passo(app: &mut PixelApp, key: u32, op: Option<Op>) -> Option<()> {
    match op {
        Some(op) => app.commit(key, op).ok(),
        None => Some(()),
    }
}

pub(crate) fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u32> {
    let (w, h) = SIZE;
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: ph2d_pixel_core::Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let (layer, frame) = {
        let d = app.docs.get(&key)?;
        (d.layer, d.frame)
    };
    // O losango que a varinha apanha.
    let ((cx, cy), r) = WAND;
    let caixa = Rect::new(cx - r, cy - r, (2 * r + 1) as u32, (2 * r + 1) as u32);
    let mut pixels = Vec::with_capacity(caixa.area() * 4);
    for y in caixa.y..caixa.y + caixa.h as i32 {
        for x in caixa.x..caixa.x + caixa.w as i32 {
            pixels.extend_from_slice(&if in_wand_blob(x, y) {
                WAND_COLOR
            } else {
                [0; 4]
            });
        }
    }
    let p = Patch {
        layer,
        frame,
        bbox: Some(caixa),
        rect: caixa,
        pixels,
    };
    app.commit(key, Op::Patch(p)).ok()?;
    // As cópias: aumentadas, a figura e as giradas, a espelhada, o ladrilho com volta.
    let s = small();
    assenta(
        app,
        key,
        &solta(
            NEAREST_AT.0,
            NEAREST_AT.1,
            resize_nearest(&s, SCALED, SCALED),
        ),
    )?;
    assenta(
        app,
        key,
        &solta(
            BILINEAR_AT.0,
            BILINEAR_AT.1,
            resize_bilinear(&s, SCALED, SCALED),
        ),
    )?;
    assenta(app, key, &solta(ORIGINAL_AT.0, ORIGINAL_AT.1, figure()))?;
    assenta(app, key, &rotated(FAST_AT, Resample::Fast)?)?;
    assenta(app, key, &rotated(ROTSPRITE_AT, Resample::RotSprite)?)?;
    assenta(app, key, &solta(FLIP_AT.0, FLIP_AT.1, flip_h(&figure())))?;
    assenta(app, key, &solta(TILE_AT.0, TILE_AT.1, tile()))?;
    // O Shift com volta, um degrau de cada vez, com o ladrilho seleccionado (como o artista faz).
    let t = Mask::from_rect(w, h, Rect::new(TILE_AT.0, TILE_AT.1, FIGURE, FIGURE));
    let sel = |app: &PixelApp, m: &Mask| {
        selection::select(&app.docs.get(&key)?.sprite, m, SelectMode::Replace)
    };
    passo(app, key, sel(app, &t))?;
    for (dx, dy) in std::iter::repeat_n((1, 0), SHIFT.0 as usize)
        .chain(std::iter::repeat_n((0, 1), SHIFT.1 as usize))
    {
        let doc = app.docs.get(&key)?;
        passo(
            app,
            key,
            edit::shift(&doc.sprite, (layer, frame), dx, dy, 0),
        )?;
    }
    // O traço de ponta a ponta com a marquee activa: só pinta dentro dela.
    passo(app, key, sel(app, &Mask::from_rect(w, h, RECT)))?;
    let doc = app.docs.get(&key)?;
    let s = &doc.sprite;
    let v = s.value_of_rgba(STROKE);
    let mut st = Stroke::begin(s, layer, frame, v, Paint::default(), (0, STROKE_Y));
    st.to(s, STROKE_X1, STROKE_Y);
    passo(app, key, st.op(s))?;
    // A selecção final: as quatro peças.
    let m = mask(&app.docs.get(&key)?.sprite, (layer, frame));
    passo(app, key, sel(app, &m))?;
    app.tool = crate::Tool::RectMarquee;
    Some(bits as u32)
}
