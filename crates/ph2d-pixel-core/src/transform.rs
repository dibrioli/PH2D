//! **Transformar células** (onda P6): uma imagem solta no modo do sprite ([`Image`]), as
//! permutações exactas, as reamostragens medidas, e o levantar/assentar de uma selecção
//! ([`Floating`], [`lift`], [`stamp`]). Nenhuma função aqui inventa cor, salvo a bilinear.

use crate::op::{MaskSet, Op, Patch};
use crate::selection::Mask;
use crate::sprite::{CelKey, Rect, Sprite};

/// Uma imagem `w × h` com `bpp` bytes por célula (os valores do modo do sprite), linha a linha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub w: u32,
    pub h: u32,
    pub bpp: usize,
    pub data: Vec<u8>,
}

impl Image {
    /// Toda célula com o valor `fill`.
    #[must_use]
    pub fn new(w: u32, h: u32, bpp: usize, fill: u32) -> Self {
        let b = fill.to_le_bytes();
        let mut data = Vec::with_capacity(w as usize * h as usize * bpp);
        for _ in 0..w as usize * h as usize {
            data.extend_from_slice(&b[..bpp]);
        }
        Self { w, h, bpp, data }
    }

    /// Uma imagem de valores (cada um com `bpp` bytes).
    #[must_use]
    pub fn from_values(w: u32, h: u32, bpp: usize, values: &[u32]) -> Self {
        let mut data = Vec::with_capacity(values.len() * bpp);
        for v in values {
            data.extend_from_slice(&v.to_le_bytes()[..bpp]);
        }
        Self { w, h, bpp, data }
    }

    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> u32 {
        let i = (y as usize * self.w as usize + x as usize) * self.bpp;
        let mut b = [0u8; 4];
        b[..self.bpp].copy_from_slice(&self.data[i..i + self.bpp]);
        u32::from_le_bytes(b)
    }

    pub fn set(&mut self, x: u32, y: u32, v: u32) {
        let i = (y as usize * self.w as usize + x as usize) * self.bpp;
        self.data[i..i + self.bpp].copy_from_slice(&v.to_le_bytes()[..self.bpp]);
    }

    /// Os valores, linha a linha.
    #[must_use]
    pub fn values(&self) -> Vec<u32> {
        (0..self.h)
            .flat_map(|y| (0..self.w).map(move |x| (x, y)))
            .map(|(x, y)| self.get(x, y))
            .collect()
    }

    /// Cada célula de `w × h` por `f(x, y)`.
    #[must_use]
    pub fn map(w: u32, h: u32, bpp: usize, f: impl Fn(u32, u32) -> u32) -> Self {
        let mut out = Image::new(w, h, bpp, 0);
        for y in 0..h {
            for x in 0..w {
                out.set(x, y, f(x, y));
            }
        }
        out
    }
}

// ── permutações exactas ───────────────────────────────────────────────────────

#[must_use]
pub fn flip_h(i: &Image) -> Image {
    Image::map(i.w, i.h, i.bpp, |x, y| i.get(i.w - 1 - x, y))
}

#[must_use]
pub fn flip_v(i: &Image) -> Image {
    Image::map(i.w, i.h, i.bpp, |x, y| i.get(x, i.h - 1 - y))
}

/// 90° horário: `w × h` → `h × w`, `(x, y)` vai para `(h − 1 − y, x)`.
#[must_use]
pub fn rot90cw(i: &Image) -> Image {
    Image::map(i.h, i.w, i.bpp, |x, y| i.get(y, i.h - 1 - x))
}

/// 90° anti-horário: `(x, y)` vai para `(y, w − 1 − x)`.
#[must_use]
pub fn rot90ccw(i: &Image) -> Image {
    Image::map(i.h, i.w, i.bpp, |x, y| i.get(i.w - 1 - y, x))
}

#[must_use]
pub fn rot180(i: &Image) -> Image {
    Image::map(i.w, i.h, i.bpp, |x, y| i.get(i.w - 1 - x, i.h - 1 - y))
}

/// Ampliação inteira por `n` (cada célula vira um bloco `n × n`).
#[must_use]
pub fn scale_int(i: &Image, n: u32) -> Image {
    let n = n.max(1);
    Image::map(i.w * n, i.h * n, i.bpp, |x, y| i.get(x / n, y / n))
}

/// Desloca com volta (Edit ▸ Shift, o `offsetWrap` do MVP).
#[must_use]
pub fn offset_wrap(i: &Image, dx: i32, dy: i32) -> Image {
    let (w, h) = (i.w as i32, i.h as i32);
    let (mx, my) = (dx.rem_euclid(w), dy.rem_euclid(h));
    Image::map(i.w, i.h, i.bpp, |x, y| {
        i.get(
            (x as i32 - mx).rem_euclid(w) as u32,
            (y as i32 - my).rem_euclid(h) as u32,
        )
    })
}

// ── reamostragens ─────────────────────────────────────────────────────────────

/// **Sprite Size ▸ Nearest-neighbor**: a fonte é `floor(d · S / N)` (Trial).
#[must_use]
pub fn resize_nearest(i: &Image, w: u32, h: u32) -> Image {
    let at = |d: u32, s: u32, n: u32| (u64::from(d) * u64::from(s) / u64::from(n)) as u32;
    Image::map(w, h, i.bpp, |x, y| i.get(at(x, i.w, w), at(y, i.h, h)))
}

/// **A escala da transformação** (campos w/h, Trial): a fonte é `floor((d + ½) · S / N)`.
#[must_use]
pub fn resize_nearest_centred(i: &Image, w: u32, h: u32) -> Image {
    let at = |d: u32, s: u32, n: u32| {
        ((2 * u64::from(d) + 1) * u64::from(s) / (2 * u64::from(n))) as u32
    };
    Image::map(w, h, i.bpp, |x, y| i.get(at(x, i.w, w), at(y, i.h, h)))
}

/// **Sprite Size ▸ Bilinear** sobre RGBA (`bpp == 4`): cantos alinhados, `fx = d·(S−1)/(N−1)`, a
/// soma em `f64` truncada. O Trial bate canal a canal em 99,2 % e nunca a mais de 1 (fixtura
/// `size_bilinear_*`); a lei inteira dele não foi achada.
#[must_use]
pub fn resize_bilinear(i: &Image, w: u32, h: u32) -> Image {
    let coord = |d: u32, s: u32, n: u32| -> (u32, u32, f64) {
        if n <= 1 || s <= 1 {
            return (0, 0, 0.0);
        }
        let f = f64::from(d) * f64::from(s - 1) / f64::from(n - 1);
        let a = (f.floor() as u32).min(s - 1);
        (a, (a + 1).min(s - 1), f - f64::from(a))
    };
    Image::map(w, h, 4, |x, y| {
        let (x0, x1, ax) = coord(x, i.w, w);
        let (y0, y1, ay) = coord(y, i.h, h);
        let px = |x, y| i.get(x, y).to_le_bytes();
        let (p00, p10, p01, p11) = (px(x0, y0), px(x1, y0), px(x0, y1), px(x1, y1));
        let mut out = [0u8; 4];
        for c in 0..4 {
            let top = f64::from(p00[c]) * (1.0 - ax) + f64::from(p10[c]) * ax;
            let bot = f64::from(p01[c]) * (1.0 - ax) + f64::from(p11[c]) * ax;
            out[c] = (top * (1.0 - ay) + bot * ay) as u8;
        }
        u32::from_le_bytes(out)
    })
}

/// O EPX (Scale2x) uma vez: igualdade de cor = igualdade de valor.
#[must_use]
pub fn epx2(i: &Image) -> Image {
    let mut out = Image::new(i.w * 2, i.h * 2, i.bpp, 0);
    for y in 0..i.h {
        for x in 0..i.w {
            let p = i.get(x, y);
            let a = if y > 0 { i.get(x, y - 1) } else { p };
            let b = if x + 1 < i.w { i.get(x + 1, y) } else { p };
            let c = if x > 0 { i.get(x - 1, y) } else { p };
            let d = if y + 1 < i.h { i.get(x, y + 1) } else { p };
            let tl = if c == a && c != d && a != b { a } else { p };
            let tr = if a == b && a != c && b != d { b } else { p };
            let bl = if d == c && d != b && c != a { c } else { p };
            let br = if b == d && b != a && d != c { d } else { p };
            out.set(2 * x, 2 * y, tl);
            out.set(2 * x + 1, 2 * y, tr);
            out.set(2 * x, 2 * y + 1, bl);
            out.set(2 * x + 1, 2 * y + 1, br);
        }
    }
    out
}

/// O RotSprite amplia ×8 (três EPX).
pub const ROTSPRITE_SCALE: u32 = 8;

/// A fonte ampliada ×8 pelo EPX (o que o RotSprite lê).
#[must_use]
pub fn epx8(i: &Image) -> Image {
    epx2(&epx2(&epx2(i)))
}

// ── levantar e assentar ───────────────────────────────────────────────────────

/// Células levantadas: a imagem em `(x, y)` do sprite e quais delas são da selecção (o resto é
/// transparente e não se assenta).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Floating {
    pub x: i32,
    pub y: i32,
    pub image: Image,
    pub keep: Vec<bool>,
}

impl Floating {
    #[must_use]
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.image.w, self.image.h)
    }

    /// A máscara (do tamanho `w × h`) que as células guardadas cobrem.
    #[must_use]
    pub fn mask(&self, w: u32, h: u32) -> Mask {
        let mut m = Mask::empty(w, h);
        for y in 0..self.image.h {
            for x in 0..self.image.w {
                if self.keep[(y * self.image.w + x) as usize] {
                    m.set(self.x + x as i32, self.y + y as i32, true);
                }
            }
        }
        m
    }

    /// A mesma colocação, imagem e guarda transformadas pela permutação `f` (aplicada às duas).
    #[must_use]
    pub fn permuted(&self, f: impl Fn(&Image) -> Image) -> (Image, Vec<bool>) {
        let keep = Image::from_values(
            self.image.w,
            self.image.h,
            1,
            &self.keep.iter().map(|&k| u32::from(k)).collect::<Vec<_>>(),
        );
        (
            f(&self.image),
            f(&keep).values().iter().map(|&v| v != 0).collect(),
        )
    }
}

/// **Levantar** as células do cel `key` sob `mask` (sem máscara: o sprite inteiro). Fora da
/// máscara a imagem é transparente. `None` = máscara vazia.
#[must_use]
pub fn lift(s: &Sprite, key: CelKey, mask: Option<&Mask>) -> Option<Floating> {
    let full = Mask::full(s.width(), s.height());
    let m = mask.unwrap_or(&full);
    let b = m.bounds()?;
    let t = s.transparent_value();
    let mut image = Image::new(b.w, b.h, s.mode().bpp(), t);
    let mut keep = vec![false; b.area()];
    for y in 0..b.h {
        for x in 0..b.w {
            let (cx, cy) = (b.x + x as i32, b.y + y as i32);
            if m.get(cx, cy) {
                image.set(x, y, s.value_at(key, cx, cy));
                keep[(y * b.w + x) as usize] = true;
            }
        }
    }
    Some(Floating {
        x: b.x,
        y: b.y,
        image,
        keep,
    })
}

/// **Assentar**: limpa as células de `clear` (para o transparente — ou o fundo, numa camada
/// Background) e escreve por cima as guardadas NÃO transparentes de `f` (cortadas ao sprite); com
/// `mask`, a selecção passa a ser essa. UM passo; `None` se nada muda.
#[must_use]
pub fn stamp(
    s: &Sprite,
    key: CelKey,
    f: &Floating,
    clear: Option<&Mask>,
    mask: Option<Option<Mask>>,
    background: u32,
) -> Option<Op> {
    let bounds = s.bounds();
    let fr = f.rect().intersect(bounds);
    let cr = clear.and_then(Mask::bounds).unwrap_or_default();
    let rect = fr.union(cr);
    let mut ops = Vec::new();
    if !rect.is_empty() {
        let bpp = s.mode().bpp();
        let before = s.read_rect(key, rect);
        let mut pixels = before.clone();
        let is_bg = s.layer(key.0).is_some_and(|l| l.background);
        let empty = if is_bg {
            background
        } else {
            s.transparent_value()
        };
        let mut put = |x: i32, y: i32, v: u32| {
            let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
            pixels[i..i + bpp].copy_from_slice(&v.to_le_bytes()[..bpp]);
        };
        if let Some(c) = clear {
            for (x, y) in c.cells() {
                put(x, y, empty);
            }
        }
        let t = s.transparent_value();
        for y in 0..f.image.h {
            for x in 0..f.image.w {
                let (cx, cy) = (f.x + x as i32, f.y + y as i32);
                let v = f.image.get(x, y);
                if f.keep[(y * f.image.w + x) as usize] && v != t && bounds.contains(cx, cy) {
                    put(cx, cy, v);
                }
            }
        }
        if pixels != before {
            let bbox = s.cel(key).map_or(rect, |c| c.bbox.union(rect));
            ops.push(Op::Patch(Patch {
                layer: key.0,
                frame: key.1,
                bbox: Some(bbox),
                rect,
                pixels,
            }));
        }
    }
    if let Some(m) = mask {
        let m = m.filter(|m| !m.is_empty());
        if m.as_ref() != s.mask() {
            ops.push(Op::Mask(Box::new(MaskSet {
                mask: m,
                last: s.last_mask().cloned(),
            })));
        }
    }
    match ops.len() {
        0 => None,
        1 => ops.pop(),
        _ => Some(Op::Batch(ops)),
    }
}
