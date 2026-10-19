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

/// A tolerância (mundo) com que a linha curva se aplana na [`Ruler`].
const FLAT: f64 = 0.02;

/// O sítio de um texto: o centro do bloco (mundo) e o ângulo (rad) das letras ali. Ao longo de uma
/// linha CURVA o texto dobra com ela, letra a letra (o `aligned` do Miro, smoke do dono 19/10): o
/// espaço do texto mede-se então pela própria linha — `x` ao longo dela, `y` na normal.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionFrame {
    pub center: [f64; 2],
    pub angle: f64,
    bend: Option<Bend>,
}

/// O texto que dobra: a régua da linha, onde fica o centro e o sentido da leitura (`-1` = contra a
/// linha, para as letras não ficarem de pernas para o ar).
#[derive(Clone, Debug, PartialEq)]
struct Bend {
    ruler: Ruler,
    s: f64,
    sign: f64,
}

impl CaptionFrame {
    /// Um texto RÍGIDO em `center`, rodado por `angle`.
    #[must_use]
    pub fn rigid(center: [f64; 2], angle: f64) -> Self {
        Self {
            center,
            angle,
            bend: None,
        }
    }

    /// O texto dobra com a linha?
    #[must_use]
    pub fn bends(&self) -> bool {
        self.bend.is_some()
    }

    /// Mundo → espaço do texto (origem no centro, `x` ao longo das letras).
    #[must_use]
    pub fn to_local(&self, p: [f64; 2]) -> [f64; 2] {
        match &self.bend {
            None => {
                let (s, c) = self.angle.sin_cos();
                let [dx, dy] = [p[0] - self.center[0], p[1] - self.center[1]];
                [dx * c + dy * s, -dx * s + dy * c]
            }
            Some(b) => {
                let (s, d) = b.ruler.nearest(p);
                [(s - b.s) * b.sign, d * b.sign]
            }
        }
    }

    /// Espaço do texto → mundo, e o ângulo das letras nesse ponto.
    #[must_use]
    pub fn to_world(&self, [u, v]: [f64; 2]) -> ([f64; 2], f64) {
        match &self.bend {
            None => {
                let (s, c) = self.angle.sin_cos();
                let [cx, cy] = self.center;
                ([cx + u * c - v * s, cy + u * s + v * c], self.angle)
            }
            Some(b) => {
                let (p, t) = b.ruler.at(b.s + b.sign * u);
                let x = [t[0] * b.sign, t[1] * b.sign];
                ([p[0] - v * x[1], p[1] + v * x[0]], x[1].atan2(x[0]))
            }
        }
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

/// Um trecho RECTO: uma recta, ou uma curva com as pegas sobre a corda (o canto do `VecVertex` sai
/// em cúbica de pegas nulas).
fn straight(seg: PathSeg) -> bool {
    let off = |a: Point, b: Point, p: Point| {
        let l = (b - a).hypot();
        if l < 1e-12 {
            (p - a).hypot()
        } else {
            ((b - a).cross(p - a) / l).abs()
        }
    };
    match seg {
        PathSeg::Line(_) => true,
        PathSeg::Quad(q) => off(q.p0, q.p2, q.p1) < 1e-9,
        PathSeg::Cubic(c) => off(c.p0, c.p3, c.p1) < 1e-9 && off(c.p0, c.p3, c.p2) < 1e-9,
    }
}

/// ⭐ A linha desenhada aplanada, com o comprimento acumulado — a régua por onde as letras de um
/// texto que dobra se medem. Para lá das pontas continua RECTA (um texto mais comprido que a linha
/// não se amontoa na ponta).
#[derive(Clone, Debug, PartialEq)]
pub struct Ruler {
    pts: Vec<[f64; 2]>,
    cum: Vec<f64>,
}

impl Ruler {
    fn of(path: &ph2d_vector::BezPath) -> Self {
        let mut pts: Vec<[f64; 2]> = Vec::new();
        ph2d_vector::flatten(path, FLAT, |el| {
            if let ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) = el
                && pts
                    .last()
                    .is_none_or(|q| (q[0] - p.x).hypot(q[1] - p.y) > 1e-9)
            {
                pts.push([p.x, p.y]);
            }
        });
        let mut cum = vec![0.0];
        for w in pts.windows(2) {
            cum.push(cum[cum.len() - 1] + (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]));
        }
        Self { pts, cum }
    }

    /// O comprimento.
    #[must_use]
    pub fn len(&self) -> f64 {
        self.cum.last().copied().unwrap_or(0.0)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pts.len() < 2
    }

    /// O trecho onde cai `s` (o 1.º e o último servem também para lá das pontas).
    fn leg(&self, s: f64) -> usize {
        let n = self.pts.len() - 1;
        self.cum[1..n].partition_point(|c| *c < s)
    }

    fn pos(&self, s: f64) -> [f64; 2] {
        if self.is_empty() {
            return self.pts.first().copied().unwrap_or([0.0, 0.0]);
        }
        let i = self.leg(s);
        let (a, b) = (self.pts[i], self.pts[i + 1]);
        let l = self.cum[i + 1] - self.cum[i];
        let t = (s - self.cum[i]) / l;
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
    }

    /// O ponto a `s` do início e a direcção ali (unitária).
    #[must_use]
    pub fn at(&self, s: f64) -> ([f64; 2], [f64; 2]) {
        let a = self.pos(s - TANGENT_STEP);
        let b = self.pos(s + TANGENT_STEP);
        let d = [b[0] - a[0], b[1] - a[1]];
        let n = d[0].hypot(d[1]);
        let dir = if n > 1e-9 {
            [d[0] / n, d[1] / n]
        } else {
            [1.0, 0.0]
        };
        (self.pos(s), dir)
    }

    /// O ponto da régua mais perto de `p`: a distância ao longo dela e a distância (com sinal, para
    /// a ESQUERDA da direcção `(-ty, tx)`) até lá.
    #[must_use]
    pub fn nearest(&self, p: [f64; 2]) -> (f64, f64) {
        if self.is_empty() {
            return (0.0, 0.0);
        }
        let last = self.pts.len() - 2;
        let mut best = (f64::INFINITY, 0.0, 0.0);
        for i in 0..=last {
            let (a, b) = (self.pts[i], self.pts[i + 1]);
            let d = [b[0] - a[0], b[1] - a[1]];
            let l = self.cum[i + 1] - self.cum[i];
            let mut t = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / (l * l);
            if i > 0 {
                t = t.max(0.0);
            }
            if i < last {
                t = t.min(1.0);
            }
            let q = [a[0] + d[0] * t, a[1] + d[1] * t];
            let dist = (p[0] - q[0]).hypot(p[1] - q[1]);
            if dist < best.0 {
                let side = (d[0] * (p[1] - a[1]) - d[1] * (p[0] - a[0])) / l;
                best = (dist, self.cum[i] + t * l, side);
            }
        }
        (best.1, best.2)
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

    /// Onde fica um texto em `at` que se deita por `turn`: na horizontal; ao longo de uma linha de
    /// rectas, rodado pela direcção dela ali; ao longo de uma CURVA, dobrado com ela.
    #[must_use]
    pub fn caption_frame(&self, at: f64, turn: TextTurn) -> CaptionFrame {
        let (center, dir) = self.point_at(at);
        if turn == TextTurn::Level {
            return CaptionFrame {
                center,
                angle: 0.0,
                bend: None,
            };
        }
        let path = ph2d_vec_render::build_bezpath(&self.path);
        if path.segments().all(straight) {
            return CaptionFrame {
                center,
                angle: upright(dir),
                bend: None,
            };
        }
        let ruler = Ruler::of(&path);
        let s = at.clamp(0.0, 1.0) * ruler.len();
        let (center, t) = ruler.at(s);
        let angle = upright(t);
        let sign = if (angle - t[1].atan2(t[0])).abs() < 1e-9 {
            1.0
        } else {
            -1.0
        };
        CaptionFrame {
            center,
            angle,
            bend: Some(Bend { ruler, s, sign }),
        }
    }
}
