//! Fontes de contorno (TTF/OTF do sistema): contorno pelo `skrifa`, rasterizador próprio por célula.
//!
//! - **Mono**: a célula acende se o CENTRO dela está dentro do contorno (regra não-zero) — a lei
//!   publicada do rasterizador mono da FreeType; conferida contra o oráculo
//!   (`tests/fixtures/oracle_freetype_mono.json`). Sem controle de dropout nem hinting (ver o handoff).
//! - **AA**: alfa = área coberta da célula (acumulação de área assinada por aresta).

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider};

/// Um glifo rasterizado, na convenção da FreeType: `left` da caneta, `top` acima da linha de base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphCells {
    pub left: i32,
    pub top: i32,
    pub w: u32,
    pub h: u32,
    /// `w·h`, 0..=255 (Mono: 0 ou 255).
    pub cover: Vec<u8>,
    /// Avanço em células (o do contorno, arredondado).
    pub adv: i32,
}

/// Métricas verticais da face no tamanho pedido, em células.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaceMetrics {
    pub ascent: i32,
    pub descent: i32,
}

const FLATTEN_STEPS: usize = 16;

#[derive(Default)]
struct Flatten {
    segs: Vec<[f32; 4]>,
    start: (f32, f32),
    cur: (f32, f32),
}

impl Flatten {
    fn line(&mut self, x: f32, y: f32) {
        if (x, y) != self.cur {
            self.segs.push([self.cur.0, self.cur.1, x, y]);
        }
        self.cur = (x, y);
    }
}

impl OutlinePen for Flatten {
    fn move_to(&mut self, x: f32, y: f32) {
        self.close();
        self.start = (x, y);
        self.cur = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line(x, y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (x0, y0) = self.cur;
        for i in 1..=FLATTEN_STEPS {
            let t = i as f32 / FLATTEN_STEPS as f32;
            let u = 1.0 - t;
            self.line(
                u * u * x0 + 2.0 * u * t * cx + t * t * x,
                u * u * y0 + 2.0 * u * t * cy + t * t * y,
            );
        }
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (x0, y0) = self.cur;
        for i in 1..=FLATTEN_STEPS {
            let t = i as f32 / FLATTEN_STEPS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            self.line(
                a * x0 + b * c0x + c * c1x + d * x,
                a * y0 + b * c0y + c * c1y + d * y,
            );
        }
    }
    fn close(&mut self) {
        let (sx, sy) = self.start;
        self.line(sx, sy);
    }
}

/// Abre a face `index` de `data`.
pub fn face(data: &[u8], index: u32) -> Result<FontRef<'_>, String> {
    FontRef::from_index(data, index).map_err(|e| format!("fonte: {e}"))
}

pub fn face_metrics(font: &FontRef<'_>, px: u32) -> FaceMetrics {
    let m = font.metrics(Size::new(px as f32), LocationRef::default());
    FaceMetrics {
        ascent: m.ascent.ceil() as i32,
        descent: (-m.descent).ceil() as i32,
    }
}

/// Rasteriza o glifo de `ch` a `px` píxeis por em. `None` se a face não tem o caractere.
pub fn rasterize_glyph(
    font: &FontRef<'_>,
    ch: char,
    px: u32,
    antialias: bool,
) -> Option<GlyphCells> {
    let gid = font.charmap().map(ch)?;
    let size = Size::new(px as f32);
    let adv = font
        .glyph_metrics(size, LocationRef::default())
        .advance_width(gid)
        .unwrap_or(0.0)
        .round() as i32;
    let glyph = font.outline_glyphs().get(gid)?;
    let mut pen = Flatten::default();
    glyph
        .draw(
            DrawSettings::unhinted(size, LocationRef::default()),
            &mut pen,
        )
        .ok()?;
    pen.close();
    let empty = || GlyphCells {
        left: 0,
        top: 0,
        w: 0,
        h: 0,
        cover: Vec::new(),
        adv,
    };
    if pen.segs.is_empty() {
        return Some(empty());
    }
    let (mut xmin, mut ymin, mut xmax, mut ymax) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for s in &pen.segs {
        xmin = xmin.min(s[0]).min(s[2]);
        xmax = xmax.max(s[0]).max(s[2]);
        ymin = ymin.min(s[1]).min(s[3]);
        ymax = ymax.max(s[1]).max(s[3]);
    }
    let (x0, y1) = (xmin.floor() as i32, ymax.ceil() as i32);
    let (w, h) = (
        (xmax.ceil() as i32 - x0).max(0) as u32,
        (y1 - ymin.floor() as i32).max(0) as u32,
    );
    // Para o espaço da caixa, y para baixo.
    let segs: Vec<[f32; 4]> = pen
        .segs
        .iter()
        .map(|s| {
            [
                s[0] - x0 as f32,
                y1 as f32 - s[1],
                s[2] - x0 as f32,
                y1 as f32 - s[3],
            ]
        })
        .collect();
    let cover = if antialias {
        area_cover(&segs, w, h)
    } else {
        center_cover(&segs, w, h)
    };
    let g = GlyphCells {
        left: x0,
        top: y1,
        w,
        h,
        cover,
        adv,
    };
    Some(if g.cover.iter().all(|&c| c == 0) {
        empty()
    } else {
        g
    })
}

/// Os vãos «dentro» (não-zero) da linha de varrimento `c` (eixo 0 = linhas, 1 = colunas).
fn spans(
    segs: &[[f32; 4]],
    axis: usize,
    c: f32,
    xs: &mut Vec<(f32, i32)>,
    out: &mut Vec<(f32, f32)>,
) {
    xs.clear();
    out.clear();
    for s in segs {
        let (u0, v0, u1, v1) = if axis == 0 {
            (s[0], s[1], s[2], s[3])
        } else {
            (s[1], s[0], s[3], s[2])
        };
        let dir = if v0 < v1 { 1 } else { -1 };
        let (lo, hi) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        if lo <= c && c < hi {
            xs.push((u0 + (c - v0) / (v1 - v0) * (u1 - u0), dir));
        }
    }
    xs.sort_by(|p, q| p.0.total_cmp(&q.0));
    let mut wind = 0;
    for k in 0..xs.len() {
        wind += xs[k].1;
        if wind != 0 && k + 1 < xs.len() {
            out.push((xs[k].0, xs[k + 1].0));
        }
    }
}

/// Mono: o centro da célula dentro (não-zero); e o vão que não cobre centro nenhum acende uma célula
/// (controlo de «dropout», nos dois sentidos — o traço fino não some).
fn center_cover(segs: &[[f32; 4]], w: u32, h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (w * h) as usize];
    let (mut xs, mut sp) = (Vec::new(), Vec::new());
    let mut drops: Vec<(u32, u32)> = Vec::new();
    for axis in 0..2 {
        let (lines, len) = if axis == 0 { (h, w) } else { (w, h) };
        for line in 0..lines {
            spans(segs, axis, line as f32 + 0.5, &mut xs, &mut sp);
            for &(ua, ub) in &sp {
                let (e1, e2) = ((ua - 0.5).ceil(), (ub - 0.5).floor());
                if e1 <= e2 {
                    if axis == 0 {
                        let (first, last) =
                            (e1.max(0.0) as u32, (e2 as u32).min(len.saturating_sub(1)));
                        for col in first..=last {
                            out[(line * len + col) as usize] = 255;
                        }
                    }
                    continue;
                }
                // Dropout SIMPLES (medido contra o oráculo: 40 células diferentes; o «smart», pelo meio
                // do vão, deu 98). Na FreeType y sobe: a «de baixo» nas colunas é a `e1` em y para baixo.
                let pick = if axis == 0 { e2 } else { e1 };
                let other = if pick == e1 { e2 } else { e1 };
                let pick = if pick < 0.0 {
                    e1
                } else if pick >= len as f32 {
                    e2
                } else {
                    pick
                };
                if pick < 0.0 || pick >= len as f32 {
                    continue;
                }
                let at = |p: f32| {
                    if axis == 0 {
                        (p as u32, line)
                    } else {
                        (line, p as u32)
                    }
                };
                if other >= 0.0 && other < len as f32 {
                    let (ox, oy) = at(other);
                    if out[(oy * w + ox) as usize] != 0 {
                        continue;
                    }
                }
                drops.push(at(pick));
            }
            for &(x, y) in &drops {
                out[(y * w + x) as usize] = 255;
            }
            drops.clear();
        }
    }
    out
}

/// AA: área assinada acumulada por linha (cobertura exata de polígono, não-zero aproximado por |soma|).
fn area_cover(segs: &[[f32; 4]], w: u32, h: u32) -> Vec<u8> {
    let stride = w as usize + 2;
    let mut acc = vec![0f32; stride * h as usize];
    for s in segs {
        accumulate_line(&mut acc, stride, h as usize, (s[0], s[1]), (s[2], s[3]));
    }
    let mut out = vec![0u8; (w * h) as usize];
    for row in 0..h as usize {
        let mut sum = 0f32;
        for col in 0..w as usize {
            sum += acc[row * stride + col];
            out[row * w as usize + col] = (sum.abs().min(1.0) * 255.0).round() as u8;
        }
    }
    out
}

fn accumulate_line(acc: &mut [f32], stride: usize, h: usize, p0: (f32, f32), p1: (f32, f32)) {
    if p0.1 == p1.1 {
        return;
    }
    let (dir, p0, p1) = if p0.1 < p1.1 {
        (1.0, p0, p1)
    } else {
        (-1.0, p1, p0)
    };
    let dxdy = (p1.0 - p0.0) / (p1.1 - p0.1);
    let mut x = p0.0;
    let y_start = p0.1.max(0.0) as usize;
    if p0.1 < 0.0 {
        x -= p0.1 * dxdy;
    }
    let y_end = (p1.1.ceil() as usize).min(h);
    let cell = |acc: &mut [f32], row: usize, col: i64, v: f32| {
        let c = col.clamp(0, stride as i64 - 1) as usize;
        acc[row * stride + c] += v;
    };
    for y in y_start..y_end {
        let dy = ((y + 1) as f32).min(p1.1) - (y as f32).max(p0.1);
        let xnext = x + dxdy * dy;
        let d = dy * dir;
        let (xa, xb) = if x < xnext { (x, xnext) } else { (xnext, x) };
        let xa_floor = xa.floor();
        let (ia, ib) = (xa_floor as i64, xb.ceil() as i64);
        if ib <= ia + 1 {
            let xmf = 0.5 * (x + xnext) - xa_floor;
            cell(acc, y, ia, d - d * xmf);
            cell(acc, y, ia + 1, d * xmf);
        } else {
            let s = (xb - xa).recip();
            let xaf = xa - xa_floor;
            let a0 = 0.5 * s * (1.0 - xaf) * (1.0 - xaf);
            let xbf = xb - xb.ceil() + 1.0;
            let am = 0.5 * s * xbf * xbf;
            cell(acc, y, ia, d * a0);
            if ib == ia + 2 {
                cell(acc, y, ia + 1, d * (1.0 - a0 - am));
            } else {
                let a1 = s * (1.5 - xaf);
                cell(acc, y, ia + 1, d * (a1 - a0));
                for xi in ia + 2..ib - 1 {
                    cell(acc, y, xi, d * s);
                }
                let a2 = a1 + (ib - ia - 3) as f32 * s;
                cell(acc, y, ib - 1, d * (1.0 - a2 - am));
            }
            cell(acc, y, ib, d * am);
        }
        x = xnext;
    }
}
