//! **O pincel de padrão** (Aseprite *Edit › New Brush*, Ctrl+B): a selecção vira um carimbo com as
//! cores da imagem. Lei MEDIDA no Trial 1.3.18.6 (`tests/fixtures/oracle_trial_brush_pattern.json`):
//! o carimbo é o rectângulo da selecção (nada se apara), a âncora `(⌊w/2⌋, ⌊h/2⌋)`; só as células
//! da máscara pintam, e uma cor transparente nunca pinta — o destino fica.

use crate::raster::Cell;
use crate::selection::Mask;
use crate::sprite::{CelKey, ColorMode, Sprite};

/// Onde o padrão está quando o pincel carimba (o dropdown da barra de contexto do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PatternAlign {
    /// O padrão fica preso ao sítio de onde foi copiado: `cor(x, y) = padrão[(x − sx) mod w, (y − sy) mod h]`.
    #[default]
    Source,
    /// Preso à origem do PRIMEIRO carimbo do traço (cada clique é um traço novo).
    Destination,
    /// O pincel viaja inteiro com o cursor; o último carimbo cobre os anteriores.
    Paint,
}

impl PatternAlign {
    pub const ALL: [Self; 3] = [Self::Source, Self::Destination, Self::Paint];
}

/// O carimbo copiado: `w × h` células, a máscara da selecção e o valor de cada uma (no modo do
/// sprite de origem; `None` = transparente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternBrush {
    w: u32,
    h: u32,
    /// As células da máscara `(i, j)`, em linhas: a ordem de [`Self::offsets`].
    foot: Vec<Cell>,
    values: Vec<Option<u32>>,
    /// O canto de cima-esquerda de onde foi copiado (o `Source` alinha-se aqui).
    pub origin: Cell,
}

impl PatternBrush {
    /// O pincel da selecção `mask` sobre o cel `key` de `sprite`; `None` se a selecção está vazia.
    #[must_use]
    pub fn from_selection(sprite: &Sprite, key: CelKey, mask: &Mask) -> Option<Self> {
        let r = mask.bounds()?;
        let mut foot = Vec::new();
        let mut values = Vec::with_capacity((r.w * r.h) as usize);
        for y in r.y..r.y + r.h as i32 {
            for x in r.x..r.x + r.w as i32 {
                if mask.get(x, y) {
                    foot.push((x - r.x, y - r.y));
                }
                let v = sprite.value_at(key, x, y);
                values.push((!is_clear(sprite, v)).then_some(v));
            }
        }
        Some(Self {
            w: r.w,
            h: r.h,
            foot,
            values,
            origin: (r.x, r.y),
        })
    }

    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.w, self.h)
    }

    /// A célula do carimbo que cai sob o cursor (Trial: 3×2, 4×2, 4×3 e a elipse 5×4).
    #[must_use]
    pub fn anchor(&self) -> Cell {
        ((self.w / 2) as i32, (self.h / 2) as i32)
    }

    /// As células do carimbo (as da máscara), relativas ao cursor, na ordem de [`Self::value`].
    #[must_use]
    pub fn offsets(&self) -> Vec<Cell> {
        let (ax, ay) = self.anchor();
        self.foot.iter().map(|&(i, j)| (i - ax, j - ay)).collect()
    }

    /// O valor que a célula `k` de [`Self::offsets`] põe em `at` com o padrão alinhado por `align`;
    /// `start` = o cursor do 1.º carimbo do traço. `None` = não pinta (transparente).
    #[must_use]
    pub fn value(&self, align: PatternAlign, k: usize, at: Cell, start: Cell) -> Option<u32> {
        let (ax, ay) = self.anchor();
        let (ox, oy) = match align {
            PatternAlign::Paint => {
                let &(i, j) = self.foot.get(k)?;
                return self.values[self.idx(i, j)];
            }
            PatternAlign::Source => self.origin,
            PatternAlign::Destination => (start.0 - ax, start.1 - ay),
        };
        let i = (at.0 - ox).rem_euclid(self.w as i32);
        let j = (at.1 - oy).rem_euclid(self.h as i32);
        self.values[self.idx(i, j)]
    }

    fn idx(&self, i: i32, j: i32) -> usize {
        j as usize * self.w as usize + i as usize
    }
}

/// Transparente no modo do sprite: alfa 0 (RGBA/Gray) ou o índice transparente (Indexed).
fn is_clear(sprite: &Sprite, v: u32) -> bool {
    match sprite.mode() {
        ColorMode::Indexed => v == u32::from(sprite.transparent_index()),
        _ => sprite.rgba_of_value(v)[3] == 0,
    }
}
