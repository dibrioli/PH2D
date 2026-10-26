//! **A moldura da transformação** (o `affine.js`/`distort.js` do MVP, paridade em
//! `mvp_selection.json`): UMA matriz afim, um pivô, alças e números.
//!
//! Coordenadas: a fonte `w × h` com `(u, v)` nos CANTOS das células; a matriz leva-a ao documento,
//! `x = a·u + c·v + e`, `y = b·u + d·v + f`, y para baixo. ⚠️ Nesta matemática o giro positivo é
//! HORÁRIO na tela (a do MVP); o campo de ângulo do Aseprite é ANTI-horário — quem vem do campo
//! usa [`Linear::from_screen_ccw`]. Nada aqui é kernel inteiro: a amostragem é que só COPIA valores.

use crate::sprite::Rect;
use crate::transform::{Image, ROTSPRITE_SCALE, epx8, rot90ccw, rot90cw, rot180};

const RAD: f64 = std::f64::consts::PI / 180.0;
/// O passo do giro com Shift (graus absolutos).
pub const ROT_SNAP_DEG: f64 = 15.0;
/// O tecto da inclinação: a 90° a tangente não existe (a moldura vira uma reta).
pub const MAX_SKEW_DEG: f64 = 89.0;

/// A matriz `x = a·u + c·v + e`, `y = b·u + d·v + f`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

/// A parte linear em GIRO · INCLINAÇÃO · ESCALA (graus; giro horário na tela).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Linear {
    pub rot: f64,
    pub sx: f64,
    pub sy: f64,
    pub skew: f64,
}

impl Linear {
    /// O ângulo do campo do Aseprite (positivo = ANTI-horário na tela) nesta convenção.
    #[must_use]
    pub fn from_screen_ccw(deg_ccw: f64, sx: f64, sy: f64, skew: f64) -> Self {
        Self {
            rot: -deg_ccw,
            sx,
            sy,
            skew,
        }
    }
}

/// Um ponto da fonte, em `(u, v)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pivot {
    pub u: f64,
    pub v: f64,
}

/// Um ponto do documento, em células.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// A amostragem de uma transformação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Resample {
    /// A célula sob o centro do pixel de saída (Aseprite *Fast Rotation*).
    #[default]
    Fast,
    /// O ×8 do EPX lido no centro do pixel de saída (o RotSprite do MVP, variante do CENTRO).
    RotSprite,
}

/// Uma alça da moldura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Handle {
    Move,
    Pivot,
    /// Girar (as zonas fora dos cantos).
    Rotate,
    /// Escalar por um canto/lado: a fracção `(fu, fv)` da fonte em meios (`0`, `1`, `2`).
    Scale(u8, u8),
    /// Inclinar por um lado (fracção em meios, como em `Scale`).
    Skew(u8, u8),
}

/// As opções de um arrasto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragOpts {
    pub keep_ratio: bool,
    pub from_center: bool,
    pub snap: bool,
}

/// Os números da barra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fields {
    pub x: i32,
    pub y: i32,
    pub w: f64,
    pub h: f64,
    pub rot: f64,
    pub skew: f64,
}

/// Os números a trocar (`None` = fica).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FieldPatch {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub w: Option<f64>,
    pub h: Option<f64>,
    pub rot: Option<f64>,
    pub skew: Option<f64>,
}

/// Ângulo em `(−180, 180]`.
#[must_use]
pub fn wrap_deg(deg: f64) -> f64 {
    let r = deg.rem_euclid(360.0);
    if r > 180.0 { r - 360.0 } else { r }
}

/// O `Math.round` do JS (meio → +∞).
fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

impl Affine {
    /// A parte linear `lin` aplicada a uma fonte que está na caixa `r`, com o CENTRO dela parado
    /// (o pivô por omissão da transformação do Aseprite).
    #[must_use]
    pub fn around_center(lin: Linear, r: Rect) -> Self {
        let m = Affine::compose(lin, 0.0, 0.0);
        let (u, v) = (f64::from(r.w) / 2.0, f64::from(r.h) / 2.0);
        let (cx, cy) = (f64::from(r.x) + u, f64::from(r.y) + v);
        Affine {
            e: cx - (m.a * u + m.c * v),
            f: cy - (m.b * u + m.d * v),
            ..m
        }
    }

    /// A fonte no lugar: o canto ↖ em `(x, y)`.
    #[must_use]
    pub fn at(x: f64, y: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    #[must_use]
    pub fn apply(&self, u: f64, v: f64) -> Point {
        Point {
            x: self.a * u + self.c * v + self.e,
            y: self.b * u + self.d * v + self.f,
        }
    }

    #[must_use]
    pub fn det(&self) -> f64 {
        self.a * self.d - self.b * self.c
    }

    /// Um ponto do documento de volta na fonte (`None` se a matriz degenerou).
    #[must_use]
    pub fn unapply(&self, x: f64, y: f64) -> Option<Pivot> {
        let det = self.det();
        if det.abs() <= 1e-12 {
            return None;
        }
        let (dx, dy) = (x - self.e, y - self.f);
        Some(Pivot {
            u: (self.d * dx - self.c * dy) / det,
            v: (-self.b * dx + self.a * dy) / det,
        })
    }

    /// Os cantos da fonte no documento, ↖ ↗ ↘ ↙.
    #[must_use]
    pub fn quad(&self, w: f64, h: f64) -> [Point; 4] {
        [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(u, v)| self.apply(u, v))
    }

    /// GIRO · INCLINAÇÃO · ESCALA (exacta para determinante positivo; inversa de [`Self::compose`]).
    #[must_use]
    pub fn decompose(&self) -> Linear {
        let sx = self.a.hypot(self.b);
        let r = self.b.atan2(self.a);
        let (cs, sn) = (r.cos(), r.sin());
        let p = self.c * cs + self.d * sn;
        let q = -self.c * sn + self.d * cs;
        Linear {
            rot: r / RAD,
            sx,
            sy: q,
            skew: (-p).atan2(q) / RAD,
        }
    }

    /// A matriz de `lin` com a origem em `(e, f)`.
    #[must_use]
    pub fn compose(lin: Linear, e: f64, f: f64) -> Self {
        let r = lin.rot * RAD;
        let (cs, sn, tk) = (r.cos(), r.sin(), (lin.skew * RAD).tan());
        Self {
            a: lin.sx * cs,
            b: lin.sx * sn,
            c: lin.sy * (-tk * cs - sn),
            d: lin.sy * (-tk * sn + cs),
            e,
            f,
        }
    }

    /// A moldura ASSENTADA: a origem arredondada para a célula.
    #[must_use]
    pub fn settle(&self) -> Self {
        Self {
            e: js_round(self.e),
            f: js_round(self.f),
            ..*self
        }
    }

    /// A parte linear de `lin` com a origem escolhida para `(u, v)` ficar onde estava em `self`.
    fn keep_point(&self, lin: &Affine, u: f64, v: f64) -> Affine {
        let p = self.apply(u, v);
        Affine {
            e: p.x - (lin.a * u + lin.c * v),
            f: p.y - (lin.b * u + lin.d * v),
            ..*lin
        }
    }

    /// A permutação exacta que a parte linear é (identidade e giros de 90°).
    fn axis(&self) -> Option<Axis> {
        let is = |v: f64, k: f64| (v - k).abs() < 1e-9;
        let (a, b, c, d) = (self.a, self.b, self.c, self.d);
        if is(a, 1.0) && is(b, 0.0) && is(c, 0.0) && is(d, 1.0) {
            Some(Axis::Id)
        } else if is(a, 0.0) && is(b, 1.0) && is(c, -1.0) && is(d, 0.0) {
            Some(Axis::Cw)
        } else if is(a, -1.0) && is(b, 0.0) && is(c, 0.0) && is(d, -1.0) {
            Some(Axis::R180)
        } else if is(a, 0.0) && is(b, -1.0) && is(c, 1.0) && is(d, 0.0) {
            Some(Axis::Ccw)
        } else {
            None
        }
    }

    /// O canto ↖ da caixa onde a arte da moldura ASSENTADA cai (a mesma conta do assentar).
    fn bake_origin(&self, w: f64, h: f64) -> (i32, i32) {
        let q = self.quad(w, h);
        let mx = q.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
        let my = q.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
        if self.axis().is_some() {
            (js_round(mx) as i32, js_round(my) as i32)
        } else {
            (mx.floor() as i32, my.floor() as i32)
        }
    }

    /// Os números da barra para uma fonte `w × h`.
    #[must_use]
    pub fn fields(&self, w: u32, h: u32) -> Fields {
        let dec = self.decompose();
        let (x, y) = self.settle().bake_origin(f64::from(w), f64::from(h));
        Fields {
            x,
            y,
            w: dec.sx * f64::from(w),
            h: dec.sy * f64::from(h),
            rot: dec.rot,
            skew: dec.skew,
        }
    }

    /// A matriz com alguns números trocados: tamanho, ângulo e inclinação em volta do PIVÔ;
    /// `x`/`y` deslocam a moldura. Nenhum lado abaixo de uma célula; inclinação em ±89°.
    #[must_use]
    pub fn with_fields(
        &self,
        pivot: Pivot,
        w: u32,
        h: u32,
        p: FieldPatch,
        keep_ratio: bool,
    ) -> Self {
        let (wf, hf) = (f64::from(w), f64::from(h));
        let mut out = *self;
        if p.w.is_some() || p.h.is_some() || p.rot.is_some() || p.skew.is_some() {
            let Linear {
                mut rot,
                mut sx,
                mut sy,
                mut skew,
            } = self.decompose();
            if let Some(nw) = p.w.filter(|v| v.is_finite()) {
                let nsx = nw.max(1.0) / wf;
                if keep_ratio {
                    sy *= nsx / sx;
                }
                sx = nsx;
            }
            if let Some(nh) = p.h.filter(|v| v.is_finite()) {
                let nsy = nh.max(1.0) / hf;
                if keep_ratio {
                    sx *= nsy / sy;
                }
                sy = nsy;
            }
            sx = sx.max(1.0 / wf);
            sy = sy.max(1.0 / hf);
            if let Some(r) = p.rot.filter(|v| v.is_finite()) {
                rot = wrap_deg(r);
            }
            if let Some(k) = p.skew.filter(|v| v.is_finite()) {
                skew = k.clamp(-MAX_SKEW_DEG, MAX_SKEW_DEG);
            }
            let lin = Affine::compose(Linear { rot, sx, sy, skew }, 0.0, 0.0);
            out = self.keep_point(&lin, pivot.u, pivot.v);
        }
        let px = p.x.filter(|v| v.is_finite());
        let py = p.y.filter(|v| v.is_finite());
        if px.is_some() || py.is_some() {
            let f = out.fields(w, h);
            out.e += px.map_or(0.0, |x| js_round(x) - f64::from(f.x));
            out.f += py.map_or(0.0, |y| js_round(y) - f64::from(f.y));
        }
        out
    }

    /// **Um arrasto** de `from` a `to`: tudo é DELTA desde a matriz e o pivô de PARTIDA. Devolve
    /// `(matriz, pivô)`.
    #[must_use]
    pub fn drag(
        &self,
        pivot: Pivot,
        (w, h): (u32, u32),
        handle: Handle,
        (from, to): (Point, Point),
        opt: DragOpts,
    ) -> (Affine, Pivot) {
        let same = (*self, pivot);
        let (wf, hf) = (f64::from(w), f64::from(h));
        match handle {
            Handle::Move => {
                let (dx, dy) = (js_round(to.x - from.x), js_round(to.y - from.y));
                (
                    Affine {
                        e: self.e + dx,
                        f: self.f + dy,
                        ..*self
                    },
                    pivot,
                )
            }
            Handle::Pivot => {
                let p0 = self.apply(pivot.u, pivot.v);
                match self.unapply(p0.x + to.x - from.x, p0.y + to.y - from.y) {
                    Some(q) => (
                        *self,
                        Pivot {
                            u: js_round(q.u * 2.0) / 2.0,
                            v: js_round(q.v * 2.0) / 2.0,
                        },
                    ),
                    None => same,
                }
            }
            Handle::Rotate => {
                let p = self.apply(pivot.u, pivot.v);
                let a0 = (from.y - p.y).atan2(from.x - p.x);
                let a1 = (to.y - p.y).atan2(to.x - p.x);
                let dec = self.decompose();
                let mut rot = dec.rot + (a1 - a0) / RAD;
                if opt.snap {
                    rot = js_round(rot / ROT_SNAP_DEG) * ROT_SNAP_DEG;
                }
                let lin = Affine::compose(
                    Linear {
                        rot: wrap_deg(rot),
                        ..dec
                    },
                    0.0,
                    0.0,
                );
                (self.keep_point(&lin, pivot.u, pivot.v), pivot)
            }
            Handle::Skew(hu, hv) => {
                let det = self.det();
                if det <= 1e-12 {
                    return same;
                }
                let (fu, fv) = (f64::from(hu) / 2.0, f64::from(hv) / 2.0);
                let (dx, dy) = (to.x - from.x, to.y - from.y);
                let lu = (self.d * dx - self.c * dy) / det;
                let lv = (-self.b * dx + self.a * dy) / det;
                let lin = if hu == 1 {
                    let arm = fv * hf - pivot.v;
                    if arm.abs() < 1e-9 {
                        return same;
                    }
                    let s = js_round(lu) / arm;
                    Affine {
                        c: self.c + s * self.a,
                        d: self.d + s * self.b,
                        ..*self
                    }
                } else {
                    let arm = fu * wf - pivot.u;
                    if arm.abs() < 1e-9 {
                        return same;
                    }
                    let s = js_round(lv) / arm;
                    Affine {
                        a: self.a + s * self.c,
                        b: self.b + s * self.d,
                        ..*self
                    }
                };
                let out = self.keep_point(&lin, pivot.u, pivot.v);
                if out.decompose().skew.abs() > MAX_SKEW_DEG {
                    same
                } else {
                    (out, pivot)
                }
            }
            Handle::Scale(hu, hv) => {
                let det = self.det();
                if det <= 1e-12 {
                    return same;
                }
                let (fu, fv) = (f64::from(hu) / 2.0, f64::from(hv) / 2.0);
                let (rx, ry) = (js_round(to.x - from.x), js_round(to.y - from.y));
                let du = (self.d * rx - self.c * ry) / det;
                let dv = (-self.b * rx + self.a * ry) / det;
                let (au, av) = if opt.from_center {
                    (0.5, 0.5)
                } else {
                    (1.0 - fu, 1.0 - fv)
                };
                let mut kx = if hu == 1 {
                    1.0
                } else {
                    1.0 + du / ((fu - au) * wf)
                };
                let mut ky = if hv == 1 {
                    1.0
                } else {
                    1.0 + dv / ((fv - av) * hf)
                };
                if opt.keep_ratio {
                    if hu != 1 && hv != 1 {
                        let (bu, bv) = ((fu - au) * wf, (fv - av) * hf);
                        kx = 1.0 + (du * bu + dv * bv) / (bu * bu + bv * bv);
                        ky = kx;
                    } else if hu != 1 {
                        ky = kx;
                    } else {
                        kx = ky;
                    }
                }
                let col0 = self.a.hypot(self.b);
                let height = self.decompose().sy;
                let (min_kx, min_ky) = (1.0 / (col0 * wf), 1.0 / (height * hf));
                if opt.keep_ratio {
                    let k = kx.max(min_kx).max(min_ky);
                    kx = k;
                    ky = k;
                } else {
                    kx = kx.max(min_kx);
                    ky = ky.max(min_ky);
                }
                let lin = Affine {
                    a: self.a * kx,
                    b: self.b * kx,
                    c: self.c * ky,
                    d: self.d * ky,
                    ..*self
                };
                (self.keep_point(&lin, au * wf, av * hf), pivot)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    Id,
    Cw,
    R180,
    Ccw,
}

/// A homografia (3×3 por linhas) que leva `[0,w]×[0,h]` ao quadrilátero `q` (Heckbert 1989).
fn quad_homography(w: f64, h: f64, q: &[Point; 4]) -> [f64; 9] {
    let [p0, p1, p2, p3] = *q;
    let (dx1, dx2, dx3) = (p1.x - p2.x, p3.x - p2.x, p0.x - p1.x + p2.x - p3.x);
    let (dy1, dy2, dy3) = (p1.y - p2.y, p3.y - p2.y, p0.y - p1.y + p2.y - p3.y);
    let den = dx1 * dy2 - dx2 * dy1;
    let g = (dx3 * dy2 - dx2 * dy3) / den;
    let k = (dx1 * dy3 - dx3 * dy1) / den;
    [
        (p1.x - p0.x + g * p1.x) / w,
        (p3.x - p0.x + k * p3.x) / h,
        p0.x,
        (p1.y - p0.y + g * p1.y) / w,
        (p3.y - p0.y + k * p3.y) / h,
        p0.y,
        g / w,
        k / h,
        1.0,
    ]
}

fn invert3(m: &[f64; 9]) -> [f64; 9] {
    let [a, b, c, d, e, f, g, h, i] = *m;
    let (aa, bb, cc) = (e * i - f * h, -(d * i - f * g), d * h - e * g);
    let det = a * aa + b * bb + c * cc;
    [
        aa / det,
        -(b * i - c * h) / det,
        (b * f - c * e) / det,
        bb / det,
        (a * i - c * g) / det,
        -(a * f - c * d) / det,
        cc / det,
        -(a * h - b * g) / det,
        (a * e - b * d) / det,
    ]
}

/// Convexo, sem gravata e na orientação do retângulo.
fn is_convex(q: &[Point; 4]) -> bool {
    (0..4).all(|i| {
        let (a, b, c) = (q[i], q[(i + 1) % 4], q[(i + 2) % 4]);
        (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x) > 1e-9
    })
}

fn crop(rect: Rect, img: Image, clip: Option<Rect>) -> (Rect, Image) {
    let Some(clip) = clip else {
        return (rect, img);
    };
    let c = rect.intersect(clip);
    if c.is_empty() {
        let x = rect.x.max(clip.x);
        let y = rect.y.max(clip.y);
        return (Rect::new(x, y, 0, 0), Image::new(0, 0, img.bpp, 0));
    }
    let out = Image::map(c.w, c.h, img.bpp, |x, y| {
        img.get((c.x - rect.x) as u32 + x, (c.y - rect.y) as u32 + y)
    });
    (c, out)
}

/// **A fonte transformada por `t`** (a moldura ASSENTADA): a caixa em células do documento e a
/// imagem (transparente fora do paralelogramo). Identidade e giros de 90° saem pela PERMUTAÇÃO nas
/// duas amostragens; `None` se a matriz espelha ou degenerou. `clip` corta a caixa (o desenho).
#[must_use]
pub fn transform_image(
    src: &Image,
    t: &Affine,
    method: Resample,
    transparent: u32,
    clip: Option<Rect>,
) -> Option<(Rect, Image)> {
    let s = t.settle();
    if s.det() <= 1e-12 {
        return None;
    }
    let (w, h) = (f64::from(src.w), f64::from(src.h));
    if let Some(ax) = s.axis() {
        let out = match ax {
            Axis::Id => src.clone(),
            Axis::Cw => rot90cw(src),
            Axis::R180 => rot180(src),
            Axis::Ccw => rot90ccw(src),
        };
        let (x, y) = s.bake_origin(w, h);
        return Some(crop(Rect::new(x, y, out.w, out.h), out, clip));
    }
    let q = s.quad(w, h);
    if !is_convex(&q) {
        return None;
    }
    let m = invert3(&quad_homography(w, h, &q));
    let mx0 = q.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let my0 = q.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let mx1 = q.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let my1 = q.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let (mut x0, mut y0) = (mx0.floor() as i64, my0.floor() as i64);
    let mut bw = (mx1.ceil() as i64 - x0).max(1);
    let mut bh = (my1.ceil() as i64 - y0).max(1);
    if let Some(c) = clip {
        let (cx0, cy0) = (x0.max(i64::from(c.x)), y0.max(i64::from(c.y)));
        let cx1 = (x0 + bw).min(i64::from(c.x) + i64::from(c.w));
        let cy1 = (y0 + bh).min(i64::from(c.y) + i64::from(c.h));
        if cx1 <= cx0 || cy1 <= cy0 {
            return Some((
                Rect::new(cx0 as i32, cy0 as i32, 0, 0),
                Image::new(0, 0, src.bpp, 0),
            ));
        }
        (x0, y0, bw, bh) = (cx0, cy0, cx1 - cx0, cy1 - cy0);
    }
    let up = (method == Resample::RotSprite).then(|| epx8(src));
    let k = f64::from(ROTSPRITE_SCALE);
    let mut out = Image::new(bw as u32, bh as u32, src.bpp, transparent);
    for y in 0..bh {
        let oy = (y0 + y) as f64 + 0.5;
        for x in 0..bw {
            let ox = (x0 + x) as f64 + 0.5;
            let z = m[6] * ox + m[7] * oy + m[8];
            let sx = (m[0] * ox + m[1] * oy + m[2]) / z;
            let sy = (m[3] * ox + m[4] * oy + m[5]) / z;
            if sx >= 0.0 && sy >= 0.0 && sx < w && sy < h {
                let v = match &up {
                    Some(up) => up.get((sx * k).floor() as u32, (sy * k).floor() as u32),
                    None => src.get(sx.floor() as u32, sy.floor() as u32),
                };
                out.set(x as u32, y as u32, v);
            }
        }
    }
    Some((Rect::new(x0 as i32, y0 as i32, bw as u32, bh as u32), out))
}
