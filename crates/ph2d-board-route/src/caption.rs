//! W23 B5(2) — **onde mora cada texto de uma seta**: o ponto da linha desenhada a uma fracção do
//! comprimento dela, e a direcção da linha ali ([doc 14](../../../docs/MiroClone/14_texto_ao_longo_da_linha.md)).
//! A MESMA conta para quem desenha, quem edita e quem arrasta.

use ph2d_board_model::TextTurn;
use ph2d_vector::{ParamCurve, ParamCurveArclen, ParamCurveNearest, PathSeg, Point};

use crate::{LABEL_WRAP, Routed};

/// A precisão do comprimento de arco (mundo), a do meio da rota.
const ACCURACY: f64 = 1e-2;

/// A folga do recorte à volta das letras de um texto, em fracção da letra.
pub const CAPTION_PAD: f64 = 0.25;

/// Meio passo (mundo) da diferença que dá a direcção da linha num ponto.
const TANGENT_STEP: f64 = 0.5;

/// O sítio de um texto: o centro do bloco (mundo) e o ângulo (rad) por que ele roda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptionFrame {
    pub center: [f64; 2],
    pub angle: f64,
}

impl CaptionFrame {
    /// Mundo → espaço do texto (origem no centro, `x` ao longo das letras).
    #[must_use]
    pub fn to_local(&self, p: [f64; 2]) -> [f64; 2] {
        let (s, c) = self.angle.sin_cos();
        let [dx, dy] = [p[0] - self.center[0], p[1] - self.center[1]];
        [dx * c + dy * s, -dx * s + dy * c]
    }

    /// Onde começa o bloco de altura `block_h`, no espaço do texto.
    #[must_use]
    pub fn block_origin(block_h: f64) -> [f64; 2] {
        [-LABEL_WRAP / 2.0, -block_h / 2.0]
    }

    /// `p` (mundo) cai no bloco de altura `block_h`?
    #[must_use]
    pub fn contains(&self, p: [f64; 2], block_h: f64) -> bool {
        let [x, y] = self.to_local(p);
        x.abs() <= LABEL_WRAP / 2.0 && y.abs() <= block_h / 2.0
    }

    /// `p` (mundo) cai em `knock` (o recorte, no espaço do bloco de altura `block_h`)?
    #[must_use]
    pub fn hits(&self, p: [f64; 2], block_h: f64, knock: [f64; 4]) -> bool {
        let [x, y] = self.to_local(p);
        let [ox, oy] = Self::block_origin(block_h);
        let [x, y] = [x - ox, y - oy];
        x >= knock[0] && x <= knock[2] && y >= knock[1] && y <= knock[3]
    }
}

/// O ângulo de uma direcção, de modo que as letras nunca fiquem de pernas para o ar: em
/// `[-π/2, π/2)` — uma linha vertical lê-se de baixo para cima, para onde quer que vá.
#[must_use]
pub fn upright(dir: [f64; 2]) -> f64 {
    use std::f64::consts::{FRAC_PI_2, PI};
    let a = dir[1].atan2(dir[0]);
    if a >= FRAC_PI_2 {
        a - PI
    } else if a < -FRAC_PI_2 {
        a + PI
    } else {
        a
    }
}

impl Routed {
    fn segs(&self) -> (Vec<PathSeg>, Vec<f64>) {
        let segs: Vec<PathSeg> = ph2d_vec_render::build_bezpath(&self.path)
            .segments()
            .collect();
        let lens = segs.iter().map(|s| s.arclen(ACCURACY)).collect();
        (segs, lens)
    }

    fn point_at_len(segs: &[PathSeg], lens: &[f64], len: f64) -> [f64; 2] {
        let mut left = len.max(0.0);
        for (s, l) in segs.iter().zip(lens) {
            if left <= *l {
                let p = s.eval(s.inv_arclen(left, ACCURACY));
                return [p.x, p.y];
            }
            left -= l;
        }
        segs.last().map_or([0.0, 0.0], |s| {
            let p = s.end();
            [p.x, p.y]
        })
    }

    /// O ponto da linha desenhada a `frac` do comprimento dela (`0` = início, `1` = fim), e a
    /// direcção da linha ali (unitária; `[1, 0]` numa seta sem comprimento).
    #[must_use]
    pub fn point_at(&self, frac: f64) -> ([f64; 2], [f64; 2]) {
        let (segs, lens) = self.segs();
        let total: f64 = lens.iter().sum();
        let len = frac.clamp(0.0, 1.0) * total;
        let p = Self::point_at_len(&segs, &lens, len);
        let a = Self::point_at_len(&segs, &lens, (len - TANGENT_STEP).max(0.0));
        let b = Self::point_at_len(&segs, &lens, (len + TANGENT_STEP).min(total));
        let d = [b[0] - a[0], b[1] - a[1]];
        let n = d[0].hypot(d[1]);
        let dir = if n > 1e-9 {
            [d[0] / n, d[1] / n]
        } else {
            [1.0, 0.0]
        };
        (p, dir)
    }

    /// A fracção do comprimento onde fica o ponto da linha mais perto de `p` — onde um texto
    /// arrastado ou nascido de um duplo clique assenta.
    #[must_use]
    pub fn frac_of(&self, p: [f64; 2]) -> f64 {
        let (segs, lens) = self.segs();
        let total: f64 = lens.iter().sum();
        if total <= 0.0 {
            return ph2d_board_model::Caption::MIDDLE;
        }
        let q = Point::new(p[0], p[1]);
        let mut best = (f64::INFINITY, 0.0);
        let mut before = 0.0;
        for (s, l) in segs.iter().zip(&lens) {
            let n = s.nearest(q, 1e-6);
            if n.distance_sq < best.0 {
                best = (
                    n.distance_sq,
                    before + s.subsegment(0.0..n.t).arclen(ACCURACY),
                );
            }
            before += l;
        }
        (best.1 / total).clamp(0.0, 1.0)
    }

    /// Onde fica um texto em `at` que se deita por `turn`.
    #[must_use]
    pub fn caption_frame(&self, at: f64, turn: TextTurn) -> CaptionFrame {
        let (center, dir) = self.point_at(at);
        let angle = match turn {
            TextTurn::Along => upright(dir),
            TextTurn::Level => 0.0,
        };
        CaptionFrame { center, angle }
    }
}
