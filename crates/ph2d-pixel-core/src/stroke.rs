//! **O gesto em curso**: acumula células entre o premir e o largar e vira UM [`Op::Patch`]. Lápis,
//! borracha, spray, formas e balde passam todos aqui — e pelo MESMO funil ([`expand`]): carimbo,
//! simetria e ladrilho não têm uma ferramenta onde deixam de funcionar.

use std::collections::BTreeMap;

use crate::brush::{Brush, SplitMix32, spray_seed};
use crate::fill::{Bucket, bucket_cells};
use crate::ink::{self, Ink};
use crate::op::{Op, Patch};
use crate::raster::{Cell, PixelPerfect, line_path};
use crate::sprite::{FrameId, LayerId, Rect, Sprite};
use crate::symmetry::{Symmetry, Tiling, expand, symmetry_images};

/// Como o gesto deposita: o pincel, a simetria, o ladrilho, o pixel-perfect (só com pincel de 1)
/// e a tinta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Paint {
    pub brush: Brush,
    pub symmetry: Symmetry,
    pub tiling: Tiling,
    pub pixel_perfect: bool,
    pub ink: Ink,
}

/// O gesto. Não toca o sprite até [`Stroke::op`]; a pré-visualização lê [`Stroke::value_at`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stroke {
    pub layer: LayerId,
    pub frame: FrameId,
    value: u32,
    /// `Some(v)`: só as células que valem `v` no documento mudam (a borracha com o botão direito do
    /// Aseprite troca a cor da frente pela de fundo).
    only: Option<u32>,
    paint: Paint,
    offs: Vec<Cell>,
    pp: Option<PixelPerfect>,
    /// Quantas vezes o gesto depositou em cada célula: o pixel-perfect RETRAI um ponto já mostrado
    /// sem apagar o que outro ponto pôs na mesma célula.
    cells: BTreeMap<Cell, u32>,
    last: Cell,
    rng: SplitMix32,
    /// A rampa do Shading (valores no modo do sprite) e o sentido (+1 / −1).
    ramp: Vec<u32>,
    dir: i8,
}

impl Stroke {
    /// Um gesto vazio com o valor `value` (no modo do sprite) — formas e balde enchem-no depois.
    #[must_use]
    pub fn new(layer: LayerId, frame: FrameId, value: u32, paint: Paint) -> Self {
        let pp = (paint.pixel_perfect && paint.brush.diameter() == 1).then(PixelPerfect::default);
        Self {
            layer,
            frame,
            value,
            only: None,
            offs: paint.brush.offsets(),
            paint,
            pp,
            cells: BTreeMap::new(),
            last: (0, 0),
            rng: SplitMix32::new(0),
            ramp: Vec::new(),
            dir: 1,
        }
    }

    /// A rampa e o sentido do Shading (botão esquerdo = +1, direito = −1).
    #[must_use]
    pub fn with_shading(mut self, ramp: Vec<u32>, dir: i8) -> Self {
        self.ramp = ramp;
        self.dir = dir;
        self
    }

    /// Começa um traço livre em `(x, y)`.
    #[must_use]
    pub fn begin(
        sprite: &Sprite,
        layer: LayerId,
        frame: FrameId,
        value: u32,
        paint: Paint,
        (x, y): Cell,
    ) -> Self {
        let mut s = Self::new(layer, frame, value, paint);
        s.start(sprite, x, y);
        s
    }

    /// Como [`Self::begin`], mas só troca as células que valem `from`.
    #[must_use]
    pub fn begin_replacing(
        sprite: &Sprite,
        layer: LayerId,
        frame: FrameId,
        (from, to): (u32, u32),
        paint: Paint,
        (x, y): Cell,
    ) -> Self {
        let mut s = Self::new(layer, frame, to, paint);
        s.only = Some(from);
        s.start(sprite, x, y);
        s
    }

    fn start(&mut self, sprite: &Sprite, x: i32, y: i32) {
        self.last = (x, y);
        self.rng = SplitMix32::new(spray_seed(0, x, y));
        self.point(sprite, x, y);
    }

    /// O último ponto do traço livre (o `Shift`+clique do Aseprite parte daqui).
    #[must_use]
    pub fn last(&self) -> Cell {
        self.last
    }

    /// Continua o traço livre até `(x, y)` pela linha canonicalizada desde o último ponto.
    pub fn to(&mut self, sprite: &Sprite, x: i32, y: i32) {
        for (cx, cy) in line_path(self.last, (x, y)).into_iter().skip(1) {
            self.point(sprite, cx, cy);
        }
        self.last = (x, y);
    }

    fn point(&mut self, sprite: &Sprite, x: i32, y: i32) {
        let Some(pp) = self.pp.as_mut() else {
            self.deposit(sprite, (x, y), false, 1);
            return;
        };
        let step = pp.push(x, y);
        if let Some(c) = step.paint {
            self.deposit(sprite, c, false, 1);
        }
        if let Some(c) = step.retract {
            self.deposit(sprite, c, false, -1);
        }
    }

    /// **A forma inteira de novo** (o arrasto de uma forma redesenha-a a cada movimento): o contorno
    /// leva o pincel, o interior é cheio célula a célula.
    pub fn set_shape(&mut self, sprite: &Sprite, outline: &[Cell], interior: &[Cell]) {
        self.cells.clear();
        for &c in outline {
            self.deposit(sprite, c, false, 1);
        }
        for &c in interior {
            self.deposit(sprite, c, true, 1);
        }
    }

    /// **O balde** a partir de `seed` e de cada imagem dela pela simetria.
    pub fn fill(&mut self, sprite: &Sprite, seed: Cell, opt: Bucket) {
        let key = (self.layer, self.frame);
        for (img, _, _) in symmetry_images(self.paint.symmetry, seed) {
            for (x, y) in bucket_cells(sprite, key, img, opt) {
                if self.accepts(sprite, x, y) {
                    *self.cells.entry((x, y)).or_insert(0) += 1;
                }
            }
        }
    }

    /// **Um jato do spray** à volta de `at`: `count` pontos uniformes no disco de raio `radius`
    /// (sorteados pela semente do gesto), cada um com o pincel.
    pub fn spray(&mut self, sprite: &Sprite, at: Cell, radius: u32, count: u32) {
        let r = f64::from(radius);
        for _ in 0..count {
            let a = self.rng.next_f64() * std::f64::consts::TAU;
            let d = self.rng.next_f64().sqrt() * r;
            let c = (
                at.0 + (d * a.cos()).round() as i32,
                at.1 + (d * a.sin()).round() as i32,
            );
            self.deposit(sprite, c, false, 1);
        }
        self.last = at;
    }

    fn accepts(&self, sprite: &Sprite, x: i32, y: i32) -> bool {
        self.only
            .is_none_or(|from| sprite.value_at((self.layer, self.frame), x, y) == from)
    }

    fn deposit(&mut self, sprite: &Sprite, c: Cell, plain: bool, delta: i32) {
        const ONE: [Cell; 1] = [(0, 0)];
        let offs = if plain { &ONE[..] } else { &self.offs[..] };
        let mut hit = Vec::with_capacity(offs.len());
        expand(
            c,
            offs,
            self.paint.symmetry,
            self.paint.tiling,
            (sprite.width(), sprite.height()),
            |x, y, _| hit.push((x, y)),
        );
        for (x, y) in hit {
            if !self.accepts(sprite, x, y) {
                continue;
            }
            if delta > 0 {
                *self.cells.entry((x, y)).or_insert(0) += 1;
            } else if let Some(n) = self.cells.get_mut(&(x, y)) {
                *n -= 1;
                if *n == 0 {
                    self.cells.remove(&(x, y));
                }
            }
        }
    }

    /// O valor que o gesto põe em `(x, y)`, se a tocou: a tinta aplicada UMA vez sobre o que o
    /// documento lá tem (o Shading anda um degrau por traço, por mais amostras que passem).
    #[must_use]
    pub fn value_at(&self, sprite: &Sprite, x: i32, y: i32) -> Option<u32> {
        self.cells
            .contains_key(&(x, y))
            .then(|| self.ink_value(sprite, x, y))
    }

    fn ink_value(&self, sprite: &Sprite, x: i32, y: i32) -> u32 {
        let dest = sprite.value_at((self.layer, self.frame), x, y);
        ink::apply(
            sprite,
            self.paint.ink,
            self.value,
            dest,
            &self.ramp,
            self.dir,
        )
    }

    /// As células tocadas, em ordem.
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        self.cells.keys().copied()
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

    /// O op do gesto inteiro; `None` se não mudou nenhuma célula (um clique na mesma cor não
    /// gasta um passo de desfazer).
    #[must_use]
    pub fn op(&self, sprite: &Sprite) -> Option<Op> {
        let key = (self.layer, self.frame);
        let values: Vec<(Cell, u32)> = self
            .cells
            .keys()
            .map(|&(x, y)| ((x, y), self.ink_value(sprite, x, y)))
            .collect();
        if values
            .iter()
            .all(|&((x, y), v)| sprite.value_at(key, x, y) == v)
        {
            return None;
        }
        let rect = self.rect();
        let bpp = sprite.mode().bpp();
        let mut pixels = sprite.read_rect(key, rect);
        for ((x, y), v) in values {
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
