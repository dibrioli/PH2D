//! **As alças de uma slice**: onde o ponteiro agarra, e o que arrastar cada uma faz à forma.

use ph2d_pixel_core::{Rect, SliceShape};

/// Os lados presos a uma alça: um é uma borda, dois são um canto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Lados {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

impl Lados {
    const fn algum(self) -> bool {
        self.left || self.right || self.top || self.bottom
    }
}

/// O que um premir agarra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// Só o pivot: além do Aseprite (T4), pode sair da caixa.
    Pivot,
    /// As linhas e os cantos do 9-slice.
    Center(Lados),
    /// As bordas e os cantos da caixa.
    Bounds(Lados),
    /// O interior.
    Move,
}

/// O alcance das alças em píxeis de ECRÃ: o mesmo a qualquer zoom.
const GRAB_PX: f64 = 6.0; // LITERAL-PX-OK: alcance do ponteiro sobre uma alça, em píxeis de ecrã
/// Mas nunca menos que a célula que o ponteiro está a tocar (T4: o píxel da linha agarra-a).
const GRAB_MIN_CELLS: f64 = 0.5;

/// **O alcance das alças, em células**, a `px_per_cell` píxeis de ecrã por célula.
#[must_use]
pub fn alcance(px_per_cell: f64) -> f64 {
    (GRAB_PX / px_per_cell).max(GRAB_MIN_CELLS)
}

/// A fração do lado que uma alça pode ocupar por DENTRO: o miolo de qualquer caixa é sempre mover,
/// mesmo com a tolerância de um zoom baixo.
const FRACAO_POR_DENTRO: f64 = 0.2;

fn end(start: i32, len: u32) -> f64 {
    f64::from(start) + f64::from(len)
}

/// Agarra `lo` ou `hi` no eixo: a mais perto, se está ao alcance.
fn eixo(p: f64, lo: f64, hi: f64, tol: f64) -> (bool, bool) {
    let dentro = (tol.min((hi - lo) * FRACAO_POR_DENTRO)).max(0.0);
    let alcance = |borda: f64, fora: bool| {
        let d = p - borda;
        d.abs() <= if fora { tol } else { dentro }
    };
    let (a, b) = (alcance(lo, p < lo), alcance(hi, p > hi));
    let nearer_lo = (p - lo).abs() <= (p - hi).abs();
    (a && (!b || nearer_lo), b && (!a || !nearer_lo))
}

/// As linhas do 9-slice em `p`: cada uma cede a metade da margem à borda da caixa, para a borda
/// continuar ao alcance a qualquer zoom.
fn centro(s: &SliceShape, c: Rect, p: (f64, f64), tol: f64) -> Lados {
    let b = s.bounds;
    let linha = |p: f64, start: i32, c0: i32, clen: u32, blen: u32| {
        let lo = f64::from(start) + f64::from(c0);
        let hi = lo + f64::from(clen);
        let miolo = f64::from(clen) * FRACAO_POR_DENTRO;
        let (m_lo, m_hi) = (
            f64::from(c0).max(0.0) / 2.0,
            (f64::from(blen) - f64::from(c0) - f64::from(clen)).max(0.0) / 2.0,
        );
        let t_lo = tol.min(miolo).min(m_lo);
        let t_hi = tol.min(miolo).min(m_hi);
        let (a, z) = ((p - lo).abs(), (p - hi).abs());
        let on_lo = m_lo > 0.0 && a <= t_lo;
        let on_hi = m_hi > 0.0 && z <= t_hi;
        (on_lo && (!on_hi || a <= z), on_hi && (!on_lo || z < a))
    };
    let (left, right) = linha(p.0, b.x, c.x, c.w, b.w);
    let (top, bottom) = linha(p.1, b.y, c.y, c.h, b.h);
    Lados {
        left,
        right,
        top,
        bottom,
    }
}

/// **O que o premir em `p` agarra** numa slice SELECIONADA: o pivot, depois o 9-slice, depois a caixa,
/// depois o interior. `p` em células contínuas; `tol` também.
#[must_use]
pub fn hit(s: &SliceShape, p: (f64, f64), tol: f64) -> Option<Grab> {
    let b = s.bounds;
    if let Some((px, py)) = s.pivot {
        // O pivot é um PÍXEL (T4), agarrado também a zoom baixo.
        let cx = f64::from(b.x) + f64::from(px) + 0.5;
        let cy = f64::from(b.y) + f64::from(py) + 0.5;
        let r = tol.max(0.5);
        if (p.0 - cx).abs() <= r && (p.1 - cy).abs() <= r {
            return Some(Grab::Pivot);
        }
    }
    let (x1, y1) = (end(b.x, b.w), end(b.y, b.h));
    let (bx, by) = (f64::from(b.x), f64::from(b.y));
    let dentro = p.0 >= bx && p.0 <= x1 && p.1 >= by && p.1 <= y1;
    if dentro && let Some(c) = s.center {
        let l = centro(s, c, p, tol);
        if l.algum() {
            return Some(Grab::Center(l));
        }
    }
    let perto = p.0 >= bx - tol && p.0 <= x1 + tol && p.1 >= by - tol && p.1 <= y1 + tol;
    if perto {
        let (left, right) = eixo(p.0, bx, x1, tol);
        let (top, bottom) = eixo(p.1, by, y1, tol);
        let l = Lados {
            left,
            right,
            top,
            bottom,
        };
        if l.algum() {
            return Some(Grab::Bounds(l));
        }
    }
    dentro.then_some(Grab::Move)
}

/// Um eixo redimensionado: a borda presa anda; passar da oposta trava com tamanho 1 (T2, T4).
fn resize(start: i64, len: i64, lo: bool, hi: bool, d: i64) -> (i64, i64) {
    if lo {
        let fim = start + len;
        let novo = (start + d).min(fim - 1);
        (novo, fim - novo)
    } else if hi {
        (start, (len + d).max(1))
    } else {
        (start, len)
    }
}

/// O novo tamanho sai como `u32`, a posição como `i32`, saturados.
fn sat_i32(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn sat_u32(v: i64) -> u32 {
    v.clamp(1, i64::from(u32::MAX)) as u32
}

/// As bordas da caixa: `x,y` do centro ficam e a largura dele leva a diferença; o que não cabe
/// desliga o 9-slice (T4). O pivot nunca muda (G4).
fn com_caixa(s: SliceShape, l: Lados, dx: i32, dy: i32) -> SliceShape {
    let b = s.bounds;
    let (x, w) = resize(
        i64::from(b.x),
        i64::from(b.w),
        l.left,
        l.right,
        i64::from(dx),
    );
    let (y, h) = resize(
        i64::from(b.y),
        i64::from(b.h),
        l.top,
        l.bottom,
        i64::from(dy),
    );
    let center = s.center.and_then(|c| {
        let cw = i64::from(c.w) + w - i64::from(b.w);
        let ch = i64::from(c.h) + h - i64::from(b.h);
        (cw >= 1 && ch >= 1).then(|| Rect::new(c.x, c.y, sat_u32(cw), sat_u32(ch)))
    });
    SliceShape {
        bounds: Rect::new(sat_i32(x), sat_i32(y), sat_u32(w), sat_u32(h)),
        center,
        pivot: s.pivot,
    }
}

/// As linhas do 9-slice: travam com largura 1 e não saem da caixa.
fn com_centro(s: SliceShape, c: Rect, l: Lados, dx: i32, dy: i32) -> SliceShape {
    let eixo = |c0: i32, clen: u32, blen: u32, lo: bool, hi: bool, d: i32| {
        let (mut a, mut n) = resize(i64::from(c0), i64::from(clen), lo, hi, i64::from(d));
        if lo && a < 0 {
            n += a;
            a = 0;
        }
        if hi {
            n = n.min((i64::from(blen) - a).max(1));
        }
        (a, n)
    };
    let (x, w) = eixo(c.x, c.w, s.bounds.w, l.left, l.right, dx);
    let (y, h) = eixo(c.y, c.h, s.bounds.h, l.top, l.bottom, dy);
    SliceShape {
        center: Some(Rect::new(sat_i32(x), sat_i32(y), sat_u32(w), sat_u32(h))),
        ..s
    }
}

/// **A forma depois de arrastar `g` por `(dx, dy)` células**, a partir da forma de quando premiu.
#[must_use]
pub fn aplica(s: SliceShape, g: Grab, dx: i32, dy: i32) -> SliceShape {
    match g {
        Grab::Move => {
            let mut out = s;
            out.bounds.x = s.bounds.x.saturating_add(dx);
            out.bounds.y = s.bounds.y.saturating_add(dy);
            out
        }
        Grab::Pivot => {
            let mut out = s;
            out.pivot = s
                .pivot
                .map(|(x, y)| (x.saturating_add(dx), y.saturating_add(dy)));
            out
        }
        Grab::Bounds(l) => com_caixa(s, l, dx, dy),
        Grab::Center(l) => match s.center {
            Some(c) => com_centro(s, c, l, dx, dy),
            None => s,
        },
    }
}
