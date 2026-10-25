//! O traço em curso: acumula células entre o premir e o largar e vira UM [`Op::Patch`].

use std::collections::BTreeMap;

use crate::op::{Op, Patch};
use crate::raster::line_cells;
use crate::sprite::{FrameId, LayerId, Rect, Sprite};

/// Um traço de lápis/borracha de 1 px. Não toca o sprite até [`Stroke::op`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stroke {
    pub layer: LayerId,
    pub frame: FrameId,
    value: u32,
    cells: BTreeMap<(i32, i32), u32>,
    last: (i32, i32),
}

impl Stroke {
    /// Começa em `(x, y)` com o valor `value` (no modo do sprite).
    #[must_use]
    pub fn begin(
        sprite: &Sprite,
        layer: LayerId,
        frame: FrameId,
        value: u32,
        x: i32,
        y: i32,
    ) -> Self {
        let mut s = Self {
            layer,
            frame,
            value,
            cells: BTreeMap::new(),
            last: (x, y),
        };
        s.put(sprite, x, y);
        s
    }

    /// Continua até `(x, y)` pela linha canonicalizada desde o último ponto.
    pub fn to(&mut self, sprite: &Sprite, x: i32, y: i32) {
        let (lx, ly) = self.last;
        let mut pts = Vec::new();
        line_cells(lx, ly, x, y, |cx, cy| pts.push((cx, cy)));
        for (cx, cy) in pts {
            self.put(sprite, cx, cy);
        }
        self.last = (x, y);
    }

    fn put(&mut self, sprite: &Sprite, x: i32, y: i32) {
        if sprite.bounds().contains(x, y) {
            self.cells.insert((x, y), self.value);
        }
    }

    /// O valor que o traço põe em `(x, y)`, se a tocou.
    #[must_use]
    pub fn value_at(&self, x: i32, y: i32) -> Option<u32> {
        self.cells.get(&(x, y)).copied()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// O retângulo das células tocadas.
    #[must_use]
    pub fn rect(&self) -> Rect {
        self.cells
            .keys()
            .fold(Rect::default(), |r, &(x, y)| r.union(Rect::new(x, y, 1, 1)))
    }

    /// O op do traço inteiro; `None` se não mudou nenhuma célula (um clique na mesma cor não
    /// gasta um passo de desfazer).
    #[must_use]
    pub fn op(&self, sprite: &Sprite) -> Option<Op> {
        let key = (self.layer, self.frame);
        let changed = self
            .cells
            .iter()
            .any(|(&(x, y), &v)| sprite.value_at(key, x, y) != v);
        if !changed {
            return None;
        }
        let rect = self.rect();
        let bpp = sprite.mode().bpp();
        let mut pixels = sprite.read_rect(key, rect);
        for (&(x, y), &v) in &self.cells {
            let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
            pixels[i..i + bpp].copy_from_slice(&v.to_le_bytes()[..bpp]);
        }
        let bbox = sprite.cel(key).map_or(rect, |c| c.bbox.union(rect));
        Some(Op::Patch(Patch {
            layer: self.layer,
            frame: self.frame,
            bbox: Some(bbox),
            rect,
            pixels,
        }))
    }
}
