//! **O gesto em curso**: acumula células entre o premir e o largar e vira UM [`Op::Patch`]. Lápis,
//! borracha, spray, formas e balde passam todos aqui — e pelo MESMO funil ([`expand`]): carimbo,
//! simetria e ladrilho não têm uma ferramenta onde deixam de funcionar.

use std::collections::BTreeMap;

use crate::brush::{Brush, SplitMix32, spray_seed};
use crate::dither::{Dither, PincelDither};
use crate::dynamics::Dab;
use crate::fill::{Bucket, bucket_cells};
use crate::ink::{self, Ink};
use crate::op::{Op, Patch};
use crate::raster::{Cell, PixelPerfect, line_path};
use crate::sprite::{FrameId, LayerId, Rect, Sprite};
use crate::symmetry::{Symmetry, Tiling, expand, symmetry_images};

/// Como o gesto deposita: o pincel, a simetria, o ladrilho, o pixel-perfect (só com pincel de 1)
/// e a tinta.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
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
    /// Valores PRONTOS por célula (a pré-visualização de uma moldura solta, [`Self::with_values`]):
    /// a tinta não entra e a selecção não corta — o valor é o que o assentar vai escrever.
    fixed: BTreeMap<Cell, u32>,
    /// O valor de cada célula que um pincel de PADRÃO pôs (o último carimbo cobre os anteriores).
    src: BTreeMap<Cell, u32>,
    /// O cursor do 1.º carimbo (o `Destination` do padrão alinha-se aqui).
    start: Cell,
    /// O Gradient da dinâmica: o valor de fundo, a matriz e os degraus dela que o carimbo actual
    /// acende na cor de frente ([`Self::set_dab`]).
    grad: Option<(u32, Dither, u32)>,
    /// O pincel de dither: a secundária e o padrão ([`Self::with_dither`]).
    dith: Option<(u32, PincelDither)>,
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
            fixed: BTreeMap::new(),
            src: BTreeMap::new(),
            start: (0, 0),
            grad: None,
            dith: None,
        }
    }

    /// Um gesto com o valor de CADA célula já decidido (a pré-visualização de uma moldura solta:
    /// os valores que o op de assentar escreve). O [`Self::op`] dele escreve-os tal e qual.
    #[must_use]
    pub fn with_values(
        layer: LayerId,
        frame: FrameId,
        values: impl IntoIterator<Item = (Cell, u32)>,
    ) -> Self {
        let mut s = Self::new(layer, frame, 0, Paint::default());
        s.fixed = values.into_iter().collect();
        s.cells = s.fixed.keys().map(|&c| (c, 1)).collect();
        s
    }

    /// [`Self::with_values`] com os píxeis que os `Patch` de `op` escrevem (num `Batch`, o último
    /// ganha): a pré-visualização É o op de assentar.
    #[must_use]
    pub fn of_patches(layer: LayerId, frame: FrameId, op: &Op, bpp: usize) -> Self {
        fn junta(op: &Op, bpp: usize, v: &mut Vec<(Cell, u32)>) {
            match op {
                Op::Patch(p) => {
                    for (i, px) in p.pixels.chunks_exact(bpp).enumerate() {
                        let (x, y) = (i as u32 % p.rect.w, i as u32 / p.rect.w);
                        let mut b = [0u8; 4];
                        b[..bpp].copy_from_slice(px);
                        v.push((
                            (p.rect.x + x as i32, p.rect.y + y as i32),
                            u32::from_le_bytes(b),
                        ));
                    }
                }
                Op::Batch(ops) => ops.iter().for_each(|o| junta(o, bpp, v)),
                _ => {}
            }
        }
        let mut v = Vec::new();
        junta(op, bpp, &mut v);
        Self::with_values(layer, frame, v)
    }

    /// A rampa e o sentido do Shading (botão esquerdo = +1, direito = −1).
    #[must_use]
    pub fn with_shading(mut self, ramp: Vec<u32>, dir: i8) -> Self {
        self.ramp = ramp;
        self.dir = dir;
        self
    }

    /// [`Self::begin_replacing`] por partes: só troca as células que valem `from` (o premir vem
    /// depois, com [`Self::start`]).
    #[must_use]
    pub fn replacing(mut self, from: u32) -> Self {
        self.only = Some(from);
        self
    }

    /// [`Self::set_dab`] antes do premir.
    #[must_use]
    pub fn with_dab(mut self, d: Dab) -> Self {
        self.set_dab(d);
        self
    }

    /// O Gradient da dinâmica: o fundo `bg` e a matriz (o carimbo diz a fracção, [`Self::set_dab`]).
    #[must_use]
    pub fn with_gradient(mut self, bg: u32, dither: Dither) -> Self {
        self.grad = Some((bg, dither, dither.lit(1.0)));
        self
    }

    /// **O ink Dither**: cada célula tocada leva a cor do gesto ou `secondary` pelo padrão preso ao
    /// canvas ([`PincelDither::primaria`]) — lápis, formas, spray e balde, pelo mesmo funil.
    #[must_use]
    pub fn with_dither(mut self, secondary: u32, d: PincelDither) -> Self {
        self.dith = Some((secondary, d));
        self
    }

    /// **O carimbo dos pontos seguintes** (a dinâmica): o tamanho e o ângulo trocam a forma (o
    /// padrão não muda), o gradiente a fracção. O pixel-perfect só vale com o carimbo de 1.
    pub fn set_dab(&mut self, d: Dab) {
        let b = &mut self.paint.brush;
        if b.pattern.is_none() && (d.size.is_some() || d.angle.is_some()) {
            let (size, angle) = (d.size.unwrap_or(b.size), d.angle.unwrap_or(b.angle));
            if (size, angle) != (b.size, b.angle) {
                b.size = size;
                b.angle = angle;
                self.offs = b.offsets();
                if b.diameter() != 1 {
                    self.pp = None;
                }
            }
        }
        if let (Some(g), Some(t)) = (self.grad.as_mut(), d.gradient) {
            g.2 = g.1.lit(t);
        }
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

    /// O 1.º ponto de um traço livre feito com [`Self::new`] — depois de [`Self::set_dab`], para a
    /// dinâmica dar o carimbo do premir.
    pub fn start(&mut self, sprite: &Sprite, x: i32, y: i32) {
        self.last = (x, y);
        self.start = (x, y);
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

    /// O funil de toda célula que o gesto toca: a selecção e a troca de cor (borracha direita).
    fn accepts(&self, sprite: &Sprite, x: i32, y: i32) -> bool {
        sprite.in_selection(x, y)
            && self
                .only
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
            |x, y, k| hit.push((x, y, k)),
        );
        let b = &self.paint.brush;
        let (pattern, align, image) = (
            b.pattern.clone().filter(|_| !plain),
            b.pattern_align,
            b.image_color,
        );
        for (x, y, k) in hit {
            if !self.accepts(sprite, x, y) {
                continue;
            }
            if delta > 0 {
                if let Some(p) = &pattern {
                    let Some(v) = p.value(align, k, (x, y), self.start) else {
                        continue;
                    };
                    if image {
                        self.src.insert((x, y), v);
                    }
                }
                if let Some((bg, m, lit)) = self.grad
                    && (pattern.is_none() || !image)
                {
                    let v = if m.level(x, y) < lit { self.value } else { bg };
                    self.src.insert((x, y), v);
                }
                *self.cells.entry((x, y)).or_insert(0) += 1;
            } else if let Some(n) = self.cells.get_mut(&(x, y)) {
                *n -= 1;
                if *n == 0 {
                    self.cells.remove(&(x, y));
                    self.src.remove(&(x, y));
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
        if let Some(&v) = self.fixed.get(&(x, y)) {
            return v;
        }
        let dest = sprite.value_at((self.layer, self.frame), x, y);
        let value = match self.dith {
            Some((sec, d)) if !d.primaria(x, y) => sec,
            Some(_) => self.value,
            None => self.src.get(&(x, y)).copied().unwrap_or(self.value),
        };
        ink::apply(sprite, self.paint.ink, value, dest, &self.ramp, self.dir)
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

    /// O que o gesto escreve: cada célula tocada com o valor final, depois da tinta.
    #[must_use]
    pub fn writes(&self, sprite: &Sprite) -> Vec<(Cell, u32)> {
        self.cells
            .keys()
            .map(|&(x, y)| ((x, y), self.ink_value(sprite, x, y)))
            .collect()
    }

    /// O op do gesto inteiro; `None` se não mudou nenhuma célula (um clique na mesma cor não
    /// gasta um passo de desfazer).
    ///
    /// Numa camada de tilemap não há op de píxeis: o traço passa por [`Self::writes`] e pelo destino
    /// do modo (`ph2d-pixel-tiles`).
    #[must_use]
    pub fn op(&self, sprite: &Sprite) -> Option<Op> {
        if sprite.layer(self.layer)?.tileset().is_some() {
            return None;
        }
        let key = (self.layer, self.frame);
        let values = self.writes(sprite);
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
