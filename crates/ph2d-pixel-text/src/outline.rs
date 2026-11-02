//! Fontes de contorno (TTF/OTF do sistema): contorno pelo `skrifa`, rasterizador próprio por célula.
//!
//! - **Mono**: a célula acende se o CENTRO dela está dentro do contorno (regra não-zero), com o
//!   controlo de dropout do `SCANTYPE` ([`Dropout`]: o que a fonte pede no `prep` hintado, a omissão do
//!   `FT_Outline` sem hinting); conferido contra o oráculo (`tests/fixtures/oracle_freetype_mono.json`).
//! - **AA**: alfa = área coberta da célula (acumulação de área assinada por aresta).
//! - **Hinting** (omissão, como no Aseprite): o contorno passa pelo interpretador TrueType do `skrifa`
//!   (alvo Mono no Mono, Smooth Normal no AA — os alvos da FreeType); o avanço é o ajustado por ele.

use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, HintingInstance, HintingOptions, OutlinePen, Target};
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
    /// Índice do primeiro segmento de cada contorno (o controlo de stubs anda pelo contorno).
    contours: Vec<usize>,
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
        if self.contours.last() != Some(&self.segs.len()) {
            self.contours.push(self.segs.len());
        }
        self.start = (x, y);
        self.cur = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line(x, y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let (x0, y0) = self.cur;
        let steps = FLATTEN_STEPS;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let u = 1.0 - t;
            self.line(
                u * u * x0 + 2.0 * u * t * cx + t * t * x,
                u * u * y0 + 2.0 * u * t * cy + t * t * y,
            );
        }
    }
    fn curve_to(&mut self, c0x: f32, c0y: f32, c1x: f32, c1y: f32, x: f32, y: f32) {
        let (x0, y0) = self.cur;
        let steps = FLATTEN_STEPS;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
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
        // Arredondado, não tecto: a linha de base do Aseprite (DejaVu 8 px: 7,42 → 7; gate trial_text).
        ascent: m.ascent.round() as i32,
        descent: (-m.descent).ceil() as i32,
    }
}

/// A instância de hinting de `font` a `px` para o modo de `antialias` (uma por texto: corre o `prep`).
/// `None` se a face não se deixa hintar — então o glifo sai sem hinting.
pub fn hinter(font: &FontRef<'_>, px: u32, antialias: bool) -> Option<HintingInstance> {
    let target = if antialias {
        Target::default()
    } else {
        Target::Mono
    };
    HintingInstance::new(
        &font.outline_glyphs(),
        Size::new(px as f32),
        LocationRef::default(),
        HintingOptions::from(target),
    )
    .ok()
}

/// Rasteriza o glifo de `ch` a `px` píxeis por em, hintado por `hint` se houver (ver [`hinter`]).
/// `None` se a face não tem o caractere.
pub fn rasterize_glyph(
    font: &FontRef<'_>,
    ch: char,
    px: u32,
    antialias: bool,
    hint: Option<&HintingInstance>,
) -> Option<GlyphCells> {
    let gid = font.charmap().map(ch)?;
    let size = Size::new(px as f32);
    let linear = font
        .glyph_metrics(size, LocationRef::default())
        .advance_width(gid)
        .unwrap_or(0.0);
    let glyph = font.outline_glyphs().get(gid)?;
    let mut pen = Flatten::default();
    let settings = match hint {
        Some(h) => DrawSettings::hinted(h, false),
        None => DrawSettings::unhinted(size, LocationRef::default()),
    };
    let adjusted = glyph.draw(settings, &mut pen).ok()?;
    let adv = adjusted.advance_width.unwrap_or(linear).round() as i32;
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
        let mode = if hint.is_some() {
            font_dropout(font, px)
        } else {
            Dropout::FREETYPE_DEFAULT
        };
        center_cover(&segs, &pen.contours, w, h, mode)
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

/// Controlo de «dropout» do Mono, nos termos da instrução TrueType `SCANTYPE` (Apple TrueType
/// Reference Manual / OpenType spec): o vão entre duas transições que não cobre centro nenhum acende
/// uma célula — a de baixo/esquerda (simples) ou a mais perto do meio do vão (smart) —, e os modos
/// «sem stubs» não o fazem quando as duas arestas não seguem até às linhas de varrimento vizinhas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dropout {
    Off,
    Simple { stubs: bool },
    Smart { stubs: bool },
}

impl Dropout {
    /// Sem hinting a fonte não corre o `prep`: a omissão documentada do `FT_Outline` (sem
    /// `FT_OUTLINE_SMART_DROPOUTS` nem `FT_OUTLINE_INCLUDE_STUBS`) = simples, sem stubs.
    pub const FREETYPE_DEFAULT: Dropout = Dropout::Simple { stubs: false };

    /// `SCANTYPE n` (os valores da especificação; 3 e os desconhecidos = sem controlo).
    pub fn from_scantype(n: i32) -> Dropout {
        match n {
            0 => Dropout::Simple { stubs: true },
            1 => Dropout::Simple { stubs: false },
            4 => Dropout::Smart { stubs: true },
            5 => Dropout::Smart { stubs: false },
            _ => Dropout::Off,
        }
    }
}

/// `SCANCTRL` (especificação): liga o controlo para `ppem` (sem rotação nem esticamento).
fn scanctrl_on(flags: i32, ppem: u32) -> bool {
    let threshold = flags & 0xFF;
    let small = threshold == 0xFF || ppem as i32 <= threshold;
    if threshold == 0 {
        return false;
    }
    let mut on = flags & 0x100 != 0 && small;
    if flags & 0x800 != 0 && !small {
        on = false;
    }
    on
}

/// O modo de dropout que a fonte escolhe no `prep` hintado a `ppem`. Lê o PRÓLOGO do `prep` — a
/// sequência inicial só de empilhamentos, `SCANCTRL` e `SCANTYPE`, a forma que as DejaVu usam (`PUSHW
/// 356; SCANCTRL; SCANTYPE` com o 1 empilhado antes: ligado até 100 ppem, simples sem stubs). Fora
/// do prólogo o estado gráfico não se sabe sem o interpretador: fica a omissão da especificação
/// (`scan_control` falso = sem controlo).
pub fn font_dropout(font: &FontRef<'_>, ppem: u32) -> Dropout {
    let Some(prep) = font.table_data(skrifa::Tag::new(b"prep")) else {
        return Dropout::Off;
    };
    let code = prep.as_bytes();
    let (mut stack, mut pc) = (Vec::<i32>::new(), 0usize);
    let (mut control, mut kind) = (false, 0i32);
    let byte = |i: usize| code.get(i).copied();
    while let Some(op) = byte(pc) {
        let (n, words, start) = match op {
            0x40 => (byte(pc + 1).map_or(0, usize::from), false, pc + 2),
            0x41 => (byte(pc + 1).map_or(0, usize::from), true, pc + 2),
            0xB0..=0xB7 => (usize::from(op - 0xAF), false, pc + 1),
            0xB8..=0xBF => (usize::from(op - 0xB7), true, pc + 1),
            0x85 | 0x8D => {
                let Some(v) = stack.pop() else { break };
                if op == 0x85 {
                    control = scanctrl_on(v, ppem);
                } else {
                    kind = v;
                }
                pc += 1;
                continue;
            }
            _ => break,
        };
        let size = if words { 2 } else { 1 };
        if code.len() < start + n * size {
            break;
        }
        for k in 0..n {
            let at = start + k * size;
            stack.push(if words {
                i32::from(i16::from_be_bytes([code[at], code[at + 1]]))
            } else {
                i32::from(code[at])
            });
        }
        pc = start + n * size;
    }
    if control {
        Dropout::from_scantype(kind)
    } else {
        Dropout::Off
    }
}

/// Os vãos «dentro» (não-zero) da linha de varrimento `c` (eixo 0 = linhas, 1 = colunas), com os
/// segmentos das duas transições.
fn spans(
    segs: &[[f32; 4]],
    axis: usize,
    c: f32,
    xs: &mut Vec<(f32, i32, usize)>,
    out: &mut Vec<(f32, f32, usize, usize)>,
) {
    xs.clear();
    out.clear();
    for (i, s) in segs.iter().enumerate() {
        let (u0, v0, u1, v1) = axis_seg(s, axis);
        let dir = if v0 < v1 { 1 } else { -1 };
        let (lo, hi) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
        if lo <= c && c < hi {
            xs.push((u0 + (c - v0) / (v1 - v0) * (u1 - u0), dir, i));
        }
    }
    xs.sort_by(|p, q| p.0.total_cmp(&q.0));
    let mut wind = 0;
    for k in 0..xs.len() {
        wind += xs[k].1;
        if wind != 0 && k + 1 < xs.len() {
            out.push((xs[k].0, xs[k + 1].0, xs[k].2, xs[k + 1].2));
        }
    }
}

fn axis_seg(s: &[f32; 4], axis: usize) -> (f32, f32, f32, f32) {
    if axis == 0 {
        (s[0], s[1], s[2], s[3])
    } else {
        (s[1], s[0], s[3], s[2])
    }
}

/// O vão da linha `c` entre as transições nos segmentos `i` e `j` é um «stub» se o contorno vai de uma
/// à outra sem cruzar nenhuma linha de varrimento vizinha (`c ± 1`) nem outra vez a própria `c`: as
/// arestas não «continuam a cruzar outras linhas nos dois sentidos» (regra 4 do `SCANTYPE`).
fn is_stub(segs: &[[f32; 4]], contours: &[usize], axis: usize, c: f32, i: usize, j: usize) -> bool {
    let k0 = contours.partition_point(|&s| s <= i) - 1;
    let (first, end) = (
        contours[k0],
        contours.get(k0 + 1).copied().unwrap_or(segs.len()),
    );
    let crosses = |lo: f32, hi: f32, l: f32| lo <= l && l < hi;
    let neighbour = |a: f32, b: f32| {
        let (lo, hi) = (a.min(b), a.max(b));
        crosses(lo, hi, c - 1.0) || crosses(lo, hi, c + 1.0)
    };
    let (_, v0, _, v1) = axis_seg(&segs[i], axis);
    for forward in [true, false] {
        // O resto do segmento `i`, da transição até à ponta, no sentido do passeio.
        if neighbour(c, if forward { v1 } else { v0 }) {
            continue;
        }
        let mut k = i;
        loop {
            k = if forward {
                if k + 1 == end { first } else { k + 1 }
            } else if k == first {
                end - 1
            } else {
                k - 1
            };
            if k == i {
                break;
            }
            let (_, a, _, b) = axis_seg(&segs[k], axis);
            let (lo, hi) = (a.min(b), a.max(b));
            if crosses(lo, hi, c) {
                if k == j {
                    return true;
                }
                break;
            }
            if neighbour(a, b) {
                break;
            }
        }
    }
    false
}

/// Mono: o centro da célula dentro (não-zero; regras 1 e 2), e o dropout de `mode` nos dois sentidos.
fn center_cover(segs: &[[f32; 4]], contours: &[usize], w: u32, h: u32, mode: Dropout) -> Vec<u8> {
    let mut out = vec![0u8; (w * h) as usize];
    let (mut xs, mut sp) = (Vec::new(), Vec::new());
    let mut drops: Vec<(u32, u32)> = Vec::new();
    for axis in 0..2 {
        let (lines, len) = if axis == 0 { (h, w) } else { (w, h) };
        for line in 0..lines {
            let c = line as f32 + 0.5;
            spans(segs, axis, c, &mut xs, &mut sp);
            for &(ua, ub, sa, sb) in &sp {
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
                let (smart, stubs) = match mode {
                    Dropout::Off => continue,
                    Dropout::Simple { stubs } => (false, stubs),
                    Dropout::Smart { stubs } => (true, stubs),
                };
                if !stubs && is_stub(segs, contours, axis, c, sa, sb) {
                    continue;
                }
                // Simples: na FreeType y sobe — a «de baixo» nas colunas é a `e1` em y para baixo.
                let pick = if smart {
                    (0.5 * (ua + ub)).floor()
                } else if axis == 0 {
                    e2
                } else {
                    e1
                };
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
