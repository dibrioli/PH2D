//! **O tamanho do sprite** (onda P6): Rotate/Flip Canvas, Sprite Size, Canvas Size, Crop e Trim —
//! cada um é UM [`Op::Canvas`] (todos os cels de todos os frames e camadas, com inverso exacto). As
//! tags e os frames não mudam; a selecção sai.

use std::collections::BTreeMap;

use crate::affine::{Affine, Resample, transform_image};
use crate::compose::compose_rgba;
use crate::edit::Rotation;
use crate::op::{CanvasSet, Op};
use crate::sprite::{Cel, CelKey, Rect, Sprite};
use crate::transform::{
    Image, flip_h, flip_v, resize_bilinear, resize_nearest, rot90ccw, rot90cw, rot180,
};

/// O método de Sprite Size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeMethod {
    #[default]
    Nearest,
    Bilinear,
    RotSprite,
}

/// As bordas que Canvas Size acrescenta (negativas cortam).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Borders {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// A âncora do diálogo Canvas Size: coluna (0 esquerda · 1 centro · 2 direita) e linha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    pub col: u8,
    pub row: u8,
}

impl Anchor {
    pub const CENTER: Anchor = Anchor { col: 1, row: 1 };
}

/// As bordas de um Canvas Size `w × h` → `nw × nh` pela âncora. Centro: `left = trunc(dw/2)`,
/// `top = dh − trunc(dh/2)` (Trial: centro, ↖ e ↘ medidos; os outros seis por coluna/linha).
#[must_use]
pub fn anchor_borders(w: u32, h: u32, nw: u32, nh: u32, a: Anchor) -> Borders {
    let dw = i64::from(nw) - i64::from(w);
    let dh = i64::from(nh) - i64::from(h);
    let left = match a.col {
        0 => 0,
        1 => dw / 2,
        _ => dw,
    };
    let top = match a.row {
        0 => 0,
        1 => dh - dh / 2,
        _ => dh,
    };
    Borders {
        left: left as i32,
        top: top as i32,
        right: (dw - left) as i32,
        bottom: (dh - top) as i32,
    }
}

/// Toda imagem de dono (cel não ligado) como canvas inteiro → `f(canvas, é Background)` → o cel
/// novo com a caixa justa (Background: o canvas inteiro). O resto do sprite fica.
fn remap(s: &Sprite, nw: u32, nh: u32, f: impl Fn(&Image, bool) -> Image) -> Option<Op> {
    let bpp = s.mode().bpp();
    let t = s.transparent_value();
    let mut cels: BTreeMap<CelKey, Cel> = BTreeMap::new();
    for (&key, cel) in s.entries() {
        if cel.link.is_some() {
            cels.insert(key, cel.clone());
            continue;
        }
        let canvas = Image::map(s.width(), s.height(), bpp, |x, y| {
            s.value_at(key, x as i32, y as i32)
        });
        let bg = s.layer(key.0).is_some_and(|l| l.background);
        let out = f(&canvas, bg);
        debug_assert_eq!((out.w, out.h), (nw, nh));
        let tight = if bg {
            Rect::new(0, 0, nw, nh)
        } else {
            (0..nh)
                .flat_map(|y| (0..nw).map(move |x| (x, y)))
                .filter(|&(x, y)| out.get(x, y) != t)
                .fold(Rect::default(), |r, (x, y)| {
                    r.union(Rect::new(x as i32, y as i32, 1, 1))
                })
        };
        let held = s
            .entries()
            .any(|(k, c)| k.0 == key.0 && c.link == Some(key.1));
        if tight.is_empty() && !held {
            continue;
        }
        let mut data = Vec::with_capacity(tight.area() * bpp);
        for y in 0..tight.h {
            for x in 0..tight.w {
                let v = out.get(tight.x as u32 + x, tight.y as u32 + y);
                data.extend_from_slice(&v.to_le_bytes()[..bpp]);
            }
        }
        cels.insert(
            key,
            Cel {
                bbox: tight,
                data,
                ..cel.clone()
            },
        );
    }
    let same = nw == s.width() && nh == s.height() && s.entries().eq(cels.iter());
    (!same || s.mask().is_some()).then(|| {
        Op::Canvas(Box::new(CanvasSet {
            width: nw,
            height: nh,
            cels,
            mask: None,
            last: None,
        }))
    })
}

/// **Sprite ▸ Rotate Canvas** 90°/180°: `(x, y)` vai para `(H−1−y, x)` no horário, `(y, W−1−x)` no
/// anti-horário; nos de 90° largura e altura trocam.
#[must_use]
pub fn rotate_canvas(s: &Sprite, r: Rotation) -> Option<Op> {
    let (w, h) = (s.width(), s.height());
    match r {
        Rotation::Cw90 => remap(s, h, w, |c, _| rot90cw(c)),
        Rotation::Ccw90 => remap(s, h, w, |c, _| rot90ccw(c)),
        Rotation::R180 => remap(s, w, h, |c, _| rot180(c)),
    }
}

/// **Sprite ▸ Flip Canvas** Horizontal/Vertical.
#[must_use]
pub fn flip_canvas(s: &Sprite, horizontal: bool) -> Option<Op> {
    remap(s, s.width(), s.height(), |c, _| {
        if horizontal { flip_h(c) } else { flip_v(c) }
    })
}

fn to_rgba(s: &Sprite, c: &Image) -> Image {
    Image::map(c.w, c.h, 4, |x, y| {
        u32::from_le_bytes(s.rgba_of_value(c.get(x, y)))
    })
}

fn from_rgba(s: &Sprite, c: &Image) -> Image {
    Image::map(c.w, c.h, s.mode().bpp(), |x, y| {
        s.value_of_rgba(c.get(x, y).to_le_bytes())
    })
}

/// **Sprite ▸ Sprite Size**. Nearest: `floor(d·S/N)`. Bilinear: cantos alinhados em RGBA (de volta
/// ao modo pela cor mais próxima). RotSprite: em factores inteiros é o nearest (Trial a 2×); nos
/// outros, o RotSprite do MVP na escala (o Trial tem padrão próprio, não fixado).
#[must_use]
pub fn sprite_size(s: &Sprite, w: u32, h: u32, method: SizeMethod) -> Option<Op> {
    if w == 0 || h == 0 {
        return None;
    }
    let (sw, sh) = (s.width(), s.height());
    let t = s.transparent_value();
    remap(s, w, h, |c, _| match method {
        SizeMethod::Nearest => resize_nearest(c, w, h),
        SizeMethod::Bilinear => from_rgba(s, &resize_bilinear(&to_rgba(s, c), w, h)),
        SizeMethod::RotSprite if w.is_multiple_of(sw) && h.is_multiple_of(sh) => {
            resize_nearest(c, w, h)
        }
        SizeMethod::RotSprite => {
            let a = Affine {
                a: f64::from(w) / f64::from(sw),
                b: 0.0,
                c: 0.0,
                d: f64::from(h) / f64::from(sh),
                e: 0.0,
                f: 0.0,
            };
            let clip = Rect::new(0, 0, w, h);
            let mut out = Image::new(w, h, c.bpp, t);
            if let Some((r, img)) = transform_image(c, &a, Resample::RotSprite, t, Some(clip)) {
                for y in 0..r.h {
                    for x in 0..r.w {
                        out.set(r.x as u32 + x, r.y as u32 + y, img.get(x, y));
                    }
                }
            }
            out
        }
    })
}

/// **Sprite ▸ Canvas Size** pelas bordas: a imagem fica em `(left, top)`; a área nova é
/// transparente, ou `bg` numa camada Background (a cor de fundo da barra).
#[must_use]
pub fn canvas_size(s: &Sprite, b: Borders, bg: u32) -> Option<Op> {
    let nw = i64::from(s.width()) + i64::from(b.left) + i64::from(b.right);
    let nh = i64::from(s.height()) + i64::from(b.top) + i64::from(b.bottom);
    if nw < 1 || nh < 1 || nw > i64::from(crate::MAX_SIDE) || nh > i64::from(crate::MAX_SIDE) {
        return None;
    }
    let (nw, nh) = (nw as u32, nh as u32);
    let t = s.transparent_value();
    remap(s, nw, nh, |c, is_bg| {
        Image::map(nw, nh, c.bpp, |x, y| {
            let (ox, oy) = (x as i64 - i64::from(b.left), y as i64 - i64::from(b.top));
            if ox >= 0 && oy >= 0 && ox < i64::from(c.w) && oy < i64::from(c.h) {
                c.get(ox as u32, oy as u32)
            } else if is_bg {
                bg
            } else {
                t
            }
        })
    })
}

/// **Sprite ▸ Crop** ao retângulo `r` (cortado ao sprite).
#[must_use]
pub fn crop(s: &Sprite, r: Rect) -> Option<Op> {
    let r = r.intersect(s.bounds());
    if r.is_empty() {
        return None;
    }
    remap(s, r.w, r.h, |c, _| {
        Image::map(r.w, r.h, c.bpp, |x, y| {
            c.get(r.x as u32 + x, r.y as u32 + y)
        })
    })
}

/// Crop à CAIXA da selecção (o que fica fora de uma máscara não retangular fica — Trial `crop_ell`).
#[must_use]
pub fn crop_to_selection(s: &Sprite) -> Option<Op> {
    crop(s, s.mask()?.bounds()?)
}

/// **Sprite ▸ Trim**: corta a borda da cor do canto ↖ do primeiro frame composto — transparente
/// ou uma cor opaca uniforme, numa camada normal ou Background (Trial 3/3) — em todos os frames.
#[must_use]
pub fn trim(s: &Sprite) -> Option<Op> {
    let (w, h) = (s.width() as usize, s.height() as usize);
    let mut px = Vec::new();
    let mut keep = Rect::default();
    let mut reference: Option<[u8; 4]> = None;
    for f in s.frames() {
        compose_rgba(s, f.id, None, &mut px);
        let at = |i: usize| {
            let c = [px[4 * i], px[4 * i + 1], px[4 * i + 2], px[4 * i + 3]];
            if c[3] == 0 { [0; 4] } else { c }
        };
        let r = *reference.get_or_insert_with(|| at(0));
        for i in 0..w * h {
            if at(i) != r {
                keep = keep.union(Rect::new((i % w) as i32, (i / w) as i32, 1, 1));
            }
        }
    }
    if keep.is_empty() || keep == s.bounds() {
        return None;
    }
    crop(s, keep)
}
