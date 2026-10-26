//! **A selecção** (onda P6): a máscara do sprite e as leis que a constroem, medidas no Aseprite Trial
//! (`tests/fixtures/oracle_trial_selection.json`) e no MVP (`mvp_selection.json`).
//!
//! A selecção é estado do DOCUMENTO com desfazer ([`crate::Op::Mask`]), nunca gravado. Enquanto há
//! selecção, toda ferramenta só escreve dentro dela ([`crate::Sprite::in_selection`], lido pelo
//! [`crate::Stroke`]).

use crate::compose::compose_rgba;
use crate::fill::{BucketSample, flood};
use crate::op::{MaskSet, Op};
use crate::raster::{Cell, polygon_cells};
use crate::sprite::{CelKey, Rect, Sprite};

/// Uma máscara `w × h` (uma célula = seleccionada ou não).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    w: u32,
    h: u32,
    bits: Vec<bool>,
}

/// Como uma selecção nova se junta à que há (Shift = somar, Alt+Shift = tirar, Ctrl+Shift = cruzar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectMode {
    #[default]
    Replace,
    Add,
    Subtract,
    Intersect,
}

/// O pincel de Select ▸ Modify.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModifyBrush {
    /// O quadrado `(2n+1)²`.
    #[default]
    Square,
    /// A elipse da marquee numa caixa `(2n+1)²`.
    Circle,
}

/// O que o Select ▸ Modify faz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modify {
    Expand,
    Contract,
    /// A faixa INTERIOR de largura `n`: máscara − contrair (Trial; o MVP fazia crescer − encolher).
    Border,
}

impl Mask {
    #[must_use]
    pub fn empty(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            bits: vec![false; w as usize * h as usize],
        }
    }

    #[must_use]
    pub fn full(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            bits: vec![true; w as usize * h as usize],
        }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.w
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.h
    }

    fn at(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as u32) < self.w && (y as u32) < self.h)
            .then(|| y as usize * self.w as usize + x as usize)
    }

    /// Seleccionada? Fora da máscara: não.
    #[must_use]
    pub fn get(&self, x: i32, y: i32) -> bool {
        self.at(x, y).is_some_and(|i| self.bits[i])
    }

    /// Marca a célula (fora da máscara: nada).
    pub fn set(&mut self, x: i32, y: i32, on: bool) {
        if let Some(i) = self.at(x, y) {
            self.bits[i] = on;
        }
    }

    /// Nenhuma célula seleccionada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.bits.contains(&true)
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.bits.iter().filter(|b| **b).count()
    }

    /// As células seleccionadas, linha a linha.
    pub fn cells(&self) -> impl Iterator<Item = Cell> + '_ {
        let w = self.w as usize;
        self.bits
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(move |(i, _)| ((i % w) as i32, (i / w) as i32))
    }

    /// A caixa justa das células seleccionadas.
    #[must_use]
    pub fn bounds(&self) -> Option<Rect> {
        self.cells().fold(None, |r: Option<Rect>, (x, y)| {
            Some(r.unwrap_or_default().union(Rect::new(x, y, 1, 1)))
        })
    }

    /// O retângulo `r` (cortado à máscara).
    #[must_use]
    pub fn from_rect(w: u32, h: u32, r: Rect) -> Self {
        let mut m = Self::empty(w, h);
        let c = r.intersect(Rect::new(0, 0, w, h));
        for y in c.y..c.y + c.h as i32 {
            for x in c.x..c.x + c.w as i32 {
                m.set(x, y, true);
            }
        }
        m
    }

    /// **A marquee retangular**: de `a` a `b` INCLUSIVO, qualquer sentido (Trial 10/10).
    #[must_use]
    pub fn marquee(w: u32, h: u32, a: Cell, b: Cell) -> Self {
        let (x0, x1) = (a.0.min(b.0), a.0.max(b.0));
        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
        Self::from_rect(
            w,
            h,
            Rect::new(x0, y0, (x1 - x0 + 1) as u32, (y1 - y0 + 1) as u32),
        )
    }

    /// **A marquee elíptica** na caixa inclusiva `a..b`: as linhas da elipse de Zingl (Trial 106/112).
    #[must_use]
    pub fn ellipse(w: u32, h: u32, a: Cell, b: Cell) -> Self {
        let mut m = Self::empty(w, h);
        ellipse_rows(a, b, |x0, x1, y| {
            for x in x0..=x1 {
                m.set(x, y, true);
            }
        });
        m
    }

    /// **O laço poligonal**: interior par-ímpar nos centros ∪ o Bresenham de cada aresta, DO vértice
    /// `i` PARA o `i + 1`, a de fecho incluída (Trial 6/6).
    #[must_use]
    pub fn polygon(w: u32, h: u32, pts: &[Cell]) -> Self {
        let mut m = Self::empty(w, h);
        if pts.is_empty() {
            return m;
        }
        for (x, y) in polygon_cells(pts).1 {
            m.set(x, y, true);
        }
        for i in 0..pts.len() {
            directed_line(pts[i], pts[(i + 1) % pts.len()], |x, y| m.set(x, y, true));
        }
        m
    }

    /// **O laço livre**: o polígono das amostras do rato (repetidas seguidas não contam).
    #[must_use]
    pub fn lasso(w: u32, h: u32, samples: &[Cell]) -> Self {
        let mut pts: Vec<Cell> = Vec::with_capacity(samples.len());
        for &p in samples {
            if pts.last() != Some(&p) {
                pts.push(p);
            }
        }
        if pts.len() > 1 && pts.last() == pts.first() {
            pts.pop();
        }
        Self::polygon(w, h, &pts)
    }

    /// Junta `o` a esta pelo modo (as duas do mesmo tamanho).
    #[must_use]
    pub fn combine(&self, o: &Mask, mode: SelectMode) -> Mask {
        let bits = self
            .bits
            .iter()
            .zip(&o.bits)
            .map(|(&a, &b)| match mode {
                SelectMode::Replace => b,
                SelectMode::Add => a || b,
                SelectMode::Subtract => a && !b,
                SelectMode::Intersect => a && b,
            })
            .collect();
        Mask { bits, ..*self }
    }

    /// O complemento.
    #[must_use]
    pub fn inverse(&self) -> Mask {
        Mask {
            bits: self.bits.iter().map(|b| !b).collect(),
            ..*self
        }
    }

    /// Dilatação pelo pincel (cortada à máscara).
    #[must_use]
    pub fn expand(&self, n: u32, brush: ModifyBrush) -> Mask {
        let offs = brush_offsets(n, brush);
        let mut out = Mask::empty(self.w, self.h);
        for (x, y) in self.cells() {
            for &(dx, dy) in &offs {
                out.set(x + dx, y + dy, true);
            }
        }
        out
    }

    /// Erosão pelo pincel; a borda do sprite conta como FORA.
    #[must_use]
    pub fn contract(&self, n: u32, brush: ModifyBrush) -> Mask {
        let offs = brush_offsets(n, brush);
        let mut out = Mask::empty(self.w, self.h);
        for (x, y) in self.cells() {
            if offs.iter().all(|&(dx, dy)| self.get(x + dx, y + dy)) {
                out.set(x, y, true);
            }
        }
        out
    }

    #[must_use]
    pub fn border(&self, n: u32, brush: ModifyBrush) -> Mask {
        self.combine(&self.contract(n, brush), SelectMode::Subtract)
    }

    #[must_use]
    pub fn modify(&self, kind: Modify, n: u32, brush: ModifyBrush) -> Mask {
        match kind {
            Modify::Expand => self.expand(n, brush),
            Modify::Contract => self.contract(n, brush),
            Modify::Border => self.border(n, brush),
        }
    }

    /// A máscara deslocada (o que sai fica de fora).
    #[must_use]
    pub fn offset(&self, dx: i32, dy: i32) -> Mask {
        let mut out = Mask::empty(self.w, self.h);
        for (x, y) in self.cells() {
            out.set(x + dx, y + dy, true);
        }
        out
    }
}

/// Os deslocamentos do pincel de Modify de raio `n`.
#[must_use]
pub fn brush_offsets(n: u32, brush: ModifyBrush) -> Vec<Cell> {
    let n = n as i32;
    match brush {
        ModifyBrush::Square => (-n..=n)
            .flat_map(|y| (-n..=n).map(move |x| (x, y)))
            .collect(),
        ModifyBrush::Circle => {
            let mut v = Vec::new();
            ellipse_rows((-n, -n), (n, n), |x0, x1, y| {
                v.extend((x0..=x1).map(|x| (x, y)))
            });
            v
        }
    }
}

/// As linhas cheias `(x0, x1, y)` da elipse inscrita na caixa inclusiva `a..b` — o contorno de
/// Zingl (`plotEllipseRect`, com a ponta das elipses achatadas acabada até `y0 - y1 <= b`).
pub fn ellipse_rows(a: Cell, b: Cell, mut row: impl FnMut(i32, i32, i32)) {
    let (mut x0, mut x1) = (i64::from(a.0.min(b.0)), i64::from(a.0.max(b.0)));
    let (ya, yb) = (i64::from(a.1.min(b.1)), i64::from(a.1.max(b.1)));
    let (top, height) = (ya, (yb - ya + 1) as usize);
    let mut lo = vec![i64::MAX; height];
    let mut hi = vec![i64::MIN; height];
    let mut put = |x: i64, y: i64| {
        let r = (y - top) as usize;
        if r < height {
            lo[r] = lo[r].min(x);
            hi[r] = hi[r].max(x);
        }
    };
    let aa = x1 - x0;
    let bb = yb - ya;
    let mut b1 = bb & 1;
    let mut dx = 4 * (1 - aa) * bb * bb;
    let mut dy = 4 * (b1 + 1) * aa * aa;
    let mut err = dx + dy + b1 * aa * aa;
    let mut y0 = ya + (bb + 1) / 2;
    let mut y1 = y0 - b1;
    let a8 = 8 * aa * aa;
    b1 = 8 * bb * bb;
    loop {
        put(x1, y0);
        put(x0, y0);
        put(x0, y1);
        put(x1, y1);
        let e2 = 2 * err;
        if e2 <= dy {
            y0 += 1;
            y1 -= 1;
            dy += a8;
            err += dy;
        }
        if e2 >= dx || 2 * err > dy {
            x0 += 1;
            x1 -= 1;
            dx += b1;
            err += dx;
        }
        if x0 > x1 {
            break;
        }
    }
    while y0 - y1 <= bb {
        put(x0 - 1, y0);
        put(x1 + 1, y0);
        y0 += 1;
        put(x0 - 1, y1);
        put(x1 + 1, y1);
        y1 -= 1;
    }
    for r in 0..height {
        if lo[r] <= hi[r] {
            row(lo[r] as i32, hi[r] as i32, (top + r as i64) as i32);
        }
    }
}

/// Bresenham DE `a` PARA `b` (sem canonicalizar: o laço do Trial desempata pela direcção).
pub fn directed_line(a: Cell, b: Cell, mut cb: impl FnMut(i32, i32)) {
    let dx = i64::from(b.0 - a.0).abs();
    let dy = -i64::from(b.1 - a.1).abs();
    let sx = if a.0 < b.0 { 1 } else { -1 };
    let sy = if a.1 < b.1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = a;
    loop {
        cb(x, y);
        if (x, y) == b {
            return;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// ── a varinha e o Color Range ─────────────────────────────────────────────────

/// As opções da varinha (a barra de contexto do Aseprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wand {
    pub tolerance: u8,
    pub contiguous: bool,
    pub eight: bool,
    pub sample: BucketSample,
}

impl Default for Wand {
    fn default() -> Self {
        Self {
            tolerance: 0,
            contiguous: true,
            eight: false,
            sample: BucketSample::Layer,
        }
    }
}

/// «A mesma cor» da varinha: `max |Δ|` por canal RGBA CRU `<= tol`; clicado transparente
/// (A=0), todo A=0 entra seja qual for o RGB (Trial 41+7; `wand_a1_tol1`: um A=1 clicado NÃO
/// normaliza os A=0 vizinhos).
#[must_use]
pub fn rgba_matches(c: [u8; 4], target: [u8; 4], tol: u8) -> bool {
    (target[3] == 0 && c[3] == 0) || (0..4).all(|i| c[i].abs_diff(target[i]) <= tol)
}

/// As cores RGBA (alfa reto) do que a varinha vê, linha a linha. Indexed e Gray passam pela COR
/// (a da paleta / o cinza): a lei medida é sobre RGBA.
fn sampled(s: &Sprite, key: CelKey, sample: BucketSample) -> Vec<[u8; 4]> {
    let (w, h) = (s.width() as i32, s.height() as i32);
    match sample {
        BucketSample::Layer => (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| s.rgba_of_value(s.value_at(key, x, y)))
            .collect(),
        BucketSample::Visible => {
            let mut out = Vec::new();
            compose_rgba(s, key.1, None, &mut out);
            out.as_chunks::<4>().0.to_vec()
        }
    }
}

/// **A varinha** sobre uma grelha RGBA `w × h`.
#[must_use]
pub fn wand_rgba(w: u32, h: u32, rgba: &[[u8; 4]], seed: Cell, opt: Wand) -> Mask {
    let mut m = Mask::empty(w, h);
    if m.at(seed.0, seed.1).is_none() {
        return m;
    }
    let at = |x: i32, y: i32| rgba[y as usize * w as usize + x as usize];
    let target = at(seed.0, seed.1);
    let same = |x: i32, y: i32| rgba_matches(at(x, y), target, opt.tolerance);
    if opt.contiguous {
        let mut hit = Vec::new();
        flood(w, h, seed, opt.eight, same, |x, y| hit.push((x, y)));
        for (x, y) in hit {
            m.set(x, y, true);
        }
    } else {
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                if same(x, y) {
                    m.set(x, y, true);
                }
            }
        }
    }
    m
}

/// A varinha no cel `key` do sprite.
#[must_use]
pub fn wand(s: &Sprite, key: CelKey, seed: Cell, opt: Wand) -> Mask {
    wand_rgba(
        s.width(),
        s.height(),
        &sampled(s, key, opt.sample),
        seed,
        opt,
    )
}

/// **Select ▸ Color Range**: a varinha não-contígua com a cor do diálogo (Trial 7/7).
#[must_use]
pub fn color_range(
    s: &Sprite,
    key: CelKey,
    color: [u8; 4],
    tolerance: u8,
    sample: BucketSample,
) -> Mask {
    let rgba = sampled(s, key, sample);
    let mut m = Mask::empty(s.width(), s.height());
    for (i, c) in rgba.iter().enumerate() {
        if rgba_matches(*c, color, tolerance) {
            let w = s.width() as usize;
            m.set((i % w) as i32, (i / w) as i32, true);
        }
    }
    m
}

// ── os passos do documento ────────────────────────────────────────────────────

fn mask_op(s: &Sprite, mask: Option<Mask>, last: Option<Mask>) -> Option<Op> {
    let mask = mask.filter(|m| !m.is_empty());
    (mask.as_ref() != s.mask() || last.as_ref() != s.last_mask())
        .then(|| Op::Mask(Box::new(MaskSet { mask, last })))
}

/// A selecção nova `m` junta à de agora pelo `mode`; vazia = Deselect.
#[must_use]
pub fn select(s: &Sprite, m: &Mask, mode: SelectMode) -> Option<Op> {
    let cur = s
        .mask()
        .cloned()
        .unwrap_or_else(|| Mask::empty(s.width(), s.height()));
    let next = cur.combine(m, mode);
    if next.is_empty() {
        return deselect(s);
    }
    mask_op(s, Some(next), s.last_mask().cloned())
}

/// Select ▸ Deselect (`Esc`, um clique sem arrastar): a de agora vai para o Reselect.
#[must_use]
pub fn deselect(s: &Sprite) -> Option<Op> {
    let cur = s.mask()?.clone();
    mask_op(s, None, Some(cur))
}

/// Select ▸ Reselect.
#[must_use]
pub fn reselect(s: &Sprite) -> Option<Op> {
    let last = s.last_mask()?.clone();
    mask_op(s, Some(last), s.last_mask().cloned())
}

/// Select ▸ All.
#[must_use]
pub fn select_all(s: &Sprite) -> Option<Op> {
    mask_op(
        s,
        Some(Mask::full(s.width(), s.height())),
        s.last_mask().cloned(),
    )
}

/// Select ▸ Inverse (de nada = tudo).
#[must_use]
pub fn invert(s: &Sprite) -> Option<Op> {
    let inv = s
        .mask()
        .map_or_else(|| Mask::full(s.width(), s.height()), Mask::inverse);
    if inv.is_empty() {
        return deselect(s);
    }
    mask_op(s, Some(inv), s.last_mask().cloned())
}

/// Select ▸ Modify ▸ Expand/Contract/Border.
#[must_use]
pub fn modify(s: &Sprite, kind: Modify, n: u32, brush: ModifyBrush) -> Option<Op> {
    let m = s.mask()?.modify(kind, n, brush);
    if m.is_empty() {
        return deselect(s);
    }
    mask_op(s, Some(m), s.last_mask().cloned())
}
