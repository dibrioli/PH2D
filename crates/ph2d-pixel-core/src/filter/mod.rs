//! **Edit ▸ Invert, Adjustments, FX** (onda P8, `docs/PixelArt/P8_plano.md` §2.2): um [`Filter`] lê
//! a imagem do cel e escreve uma CÓPIA — nunca se alimenta do que já escreveu. Age só dentro da
//! selecção (sem selecção: o sprite inteiro) e só nos [`Channels`] ligados. Todos os cels pedidos
//! viram UM passo do desfazer.

pub mod adjust;
pub mod convolution;
mod kernels;
pub mod median;
pub mod outline;
pub mod smudge;

use crate::op::{Op, Patch};
use crate::sprite::{CelKey, Rect, Sprite};

/// A imagem de um cel em RGBA (alfa reto), do tamanho do sprite — o que um filtro lê.
pub struct Source {
    w: u32,
    h: u32,
    rgba: Vec<[u8; 4]>,
}

impl Source {
    /// O cel `key` de `s` em RGBA.
    #[must_use]
    pub fn of(s: &Sprite, key: CelKey) -> Self {
        let (w, h) = (s.width(), s.height());
        let mut rgba = Vec::with_capacity(w as usize * h as usize);
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                rgba.push(s.rgba_of_value(s.value_at(key, x, y)));
            }
        }
        Self { w, h, rgba }
    }

    /// Uma imagem solta `w × h` (testes, preview de uma região).
    #[must_use]
    pub fn from_rgba(w: u32, h: u32, rgba: Vec<[u8; 4]>) -> Self {
        assert_eq!(
            rgba.len(),
            w as usize * h as usize,
            "Source: {w}×{h} pede {} células",
            w * h
        );
        Self { w, h, rgba }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.w
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.h
    }

    /// A célula `(x, y)`; `None` fora da imagem — a regra da borda é do filtro.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        (x >= 0 && y >= 0 && (x as u32) < self.w && (y as u32) < self.h)
            .then(|| self.rgba[y as usize * self.w as usize + x as usize])
    }

    pub(crate) fn set_index(&mut self, i: usize, c: [u8; 4]) {
        self.rgba[i] = c;
    }

    /// A célula `(x, y)` com a borda `edge` (a vizinhança dos filtros de janela).
    #[must_use]
    pub fn at_edge(&self, x: i32, y: i32, edge: Edge) -> [u8; 4] {
        let (w, h) = (self.w as i32, self.h as i32);
        let (x, y) = match edge {
            Edge::Clamp => (x.clamp(0, w - 1), y.clamp(0, h - 1)),
            Edge::Wrap => (x.rem_euclid(w), y.rem_euclid(h)),
        };
        self.rgba[y as usize * self.w as usize + x as usize]
    }
}

/// O que a janela de um filtro lê fora da imagem (Trial: *Tiled* desligado = `Clamp`, ligado =
/// `Wrap`; `oracle_trial_despeckle.json`, `oracle_trial_convolution.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Edge {
    #[default]
    Clamp,
    Wrap,
}

/// Um filtro de imagem: a cor nova da célula `(x, y)` lida de `src`.
pub trait Filter {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4];
}

/// Os canais que o filtro pode mudar (o *Target* do diálogo); os desligados ficam como estavam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channels {
    pub r: bool,
    pub g: bool,
    pub b: bool,
    pub a: bool,
}

impl Channels {
    pub const ALL: Self = Self {
        r: true,
        g: true,
        b: true,
        a: true,
    };

    /// `new` nos canais ligados, `old` nos outros.
    #[must_use]
    pub fn merge(self, old: [u8; 4], new: [u8; 4]) -> [u8; 4] {
        let on = [self.r, self.g, self.b, self.a];
        std::array::from_fn(|k| if on[k] { new[k] } else { old[k] })
    }
}

impl Default for Channels {
    fn default() -> Self {
        Self::ALL
    }
}

/// O que `f` faz ao cel `key`: `None` se nada muda.
#[must_use]
pub fn patch(s: &Sprite, key: CelKey, channels: Channels, f: &dyn Filter) -> Option<Op> {
    let key = s.owner(key);
    s.layer(key.0)?;
    let rect = reach(s)?;
    let src = Source::of(s, key);
    let bpp = s.mode().bpp();
    let before = s.read_rect(key, rect);
    let mut pixels = before.clone();
    for y in rect.y..rect.y + rect.h as i32 {
        for x in rect.x..rect.x + rect.w as i32 {
            if s.mask().is_some_and(|m| !m.get(x, y)) {
                continue;
            }
            let old = src.get(x, y).unwrap_or_default();
            let v = s.value_of_rgba(channels.merge(old, f.at(&src, x, y)));
            let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
            pixels[i..i + bpp].copy_from_slice(&v.to_le_bytes()[..bpp]);
        }
    }
    (pixels != before).then(|| {
        Op::Patch(Patch {
            layer: key.0,
            frame: key.1,
            bbox: Some(s.cel(key).map_or(rect, |c| c.bbox.union(rect))),
            rect,
            pixels,
        })
    })
}

/// O filtro sobre vários cels (o cel activo, ou *all frames/layers*) como UM passo; os cels
/// ligados contam uma vez.
#[must_use]
pub fn apply(s: &Sprite, keys: &[CelKey], channels: Channels, f: &dyn Filter) -> Option<Op> {
    let mut owners: Vec<CelKey> = keys.iter().map(|&k| s.owner(k)).collect();
    owners.sort_unstable();
    owners.dedup();
    let mut ops: Vec<Op> = owners
        .into_iter()
        .filter_map(|k| patch(s, k, channels, f))
        .collect();
    match ops.len() {
        0 => None,
        1 => ops.pop(),
        _ => Some(Op::Batch(ops)),
    }
}

/// A caixa onde [`patch`] pode escrever (a selecção, ou o sprite).
#[must_use]
pub fn reach(s: &Sprite) -> Option<Rect> {
    s.mask().map_or(Some(s.bounds()), |m| m.bounds())
}
