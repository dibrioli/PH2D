//! **O recorte do grid** — a metade REGIONAL do [`super::snapshot_grid`], para quem precisa desfazer
//! uma escrita LOCAL sem clonar a folha inteira.
//!
//! ## Por que ele existe
//!
//! O host autora uma figura (Line / Curve / Ellipse / Polygon / Free Hand) re-carimbando a figura
//! INTEIRA a cada quadro sobre o estado pristino — a dança *save → stamp → restore* que o canvas já
//! faz. Para que o rascunho seja a **própria água** e não um esboço digital, o mesmo gesto precisa
//! acontecer no grid; e um snapshot de folha inteira não serve, porque a 4096² ele são centenas de
//! megabytes por QUADRO.
//!
//! ## O que ele carrega, e o que ele deliberadamente NÃO carrega
//!
//! Carrega os oito planos que um depósito escreve (`film`, `susp`, `susp_rgb`, `sett`, `sett_rgb`,
//! `wet`, `active`, `bloom`) mais os **dois persistentes da grade de FLUXO** (`vel_x`/`vel_y`), cujo
//! retângulo é o mesmo projetado por `rf`.
//!
//! NÃO carrega:
//!
//! - **`flow_x`/`flow_y`** — transientes, reconstruídos do campo persistente a cada passe (o
//!   [`super::restore_grid`] de folha inteira os ZERA pelo mesmo motivo);
//! - **`paper`** — identidade da folha; um depósito não a toca;
//! - **a bbox e a faixa viva** (`bx0..`, `row_lo`/`row_hi`, `has_fluid`) — elas são um **SUPERCONJUNTO
//!   DECLARADO** (ver [`super::Grid::row_lo`]), e restaurar uma região só pode ENCOLHER o que está
//!   vivo. Deixar o superconjunto mais largo é correto por construção (`active ⊆ faixa` continua
//!   valendo, e todo passe já faz early-out por-célula); o próximo `rebuild_active_region` o reaperta.
//!   ⚠️ O inverso — tentar reapertá-los aqui — é que seria errado: este recorte não sabe nada sobre a
//!   água que vive FORA dele.

use super::Grid;

/// Um recorte retangular do grid, em células FINAS de INTERIOR (`1..=w`), já clampado.
pub struct GridPatch {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    film: Vec<f32>,
    susp: Vec<f32>,
    susp_rgb: Vec<[f32; 3]>,
    sett: Vec<f32>,
    sett_rgb: Vec<[f32; 3]>,
    wet: Vec<u8>,
    active: Vec<u8>,
    bloom: Vec<u8>,
    /// O MESMO retângulo projetado na grade de fluxo (em células de fluxo).
    fx0: i32,
    fy0: i32,
    fx1: i32,
    fy1: i32,
    vel_x: Vec<f32>,
    vel_y: Vec<f32>,
}

impl GridPatch {
    /// O retângulo FINO que este recorte descreve (interior, inclusivo).
    #[must_use]
    pub fn rect(&self) -> (i32, i32, i32, i32) {
        (self.x0, self.y0, self.x1, self.y1)
    }

    /// Quantas células finas ele guarda — o número que precifica a dança de autoria.
    #[must_use]
    pub fn cells(&self) -> usize {
        self.film.len()
    }
}

/// Clampa `[a0, a1]` ao interior `1..=n` e devolve `None` quando não sobra nada.
fn clamp_span(a0: i32, a1: i32, n: usize) -> Option<(i32, i32)> {
    let lo = a0.max(1);
    let hi = a1.min(n as i32);
    (lo <= hi).then_some((lo, hi))
}

/// O retângulo de fluxo que COBRE o retângulo fino — a projeção por `rf`, dilatada de um por segurança
/// (a amostragem bilinear do `advect` lê o vizinho, e um recorte que não o contivesse restauraria meio
/// campo).
fn flow_span(a0: i32, a1: i32, rf: usize, n: usize) -> Option<(i32, i32)> {
    let lo = ((a0 - 1) / rf as i32 + 1 - 1).max(1);
    let hi = (((a1 - 1) / rf as i32) + 1 + 1).min(n as i32);
    (lo <= hi).then_some((lo, hi))
}

/// Copia `[x0..=x1] × [y0..=y1]` de `src` (stride `s`) para um vetor linha-a-linha.
fn take_rows<T: Copy>(src: &[T], s: usize, x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<T> {
    let w = (x1 - x0 + 1) as usize;
    let mut out = Vec::with_capacity(w * (y1 - y0 + 1) as usize);
    for y in y0..=y1 {
        let row = y as usize * s + x0 as usize;
        out.extend_from_slice(&src[row..row + w]);
    }
    out
}

/// A inversa exata de [`take_rows`].
fn put_rows<T: Copy>(dst: &mut [T], s: usize, x0: i32, y0: i32, x1: i32, y1: i32, src: &[T]) {
    let w = (x1 - x0 + 1) as usize;
    for (r, y) in (y0..=y1).enumerate() {
        let row = y as usize * s + x0 as usize;
        let from = r * w;
        if from + w > src.len() {
            return;
        }
        dst[row..row + w].copy_from_slice(&src[from..from + w]);
    }
}

/// Recorta o grid em `[x0..=x1] × [y0..=y1]` (células finas). `None` quando o retângulo não intersecta
/// o interior.
#[must_use]
pub fn snapshot_grid_region(g: &Grid, x0: i32, y0: i32, x1: i32, y1: i32) -> Option<GridPatch> {
    let (x0, x1) = clamp_span(x0, x1, g.w)?;
    let (y0, y1) = clamp_span(y0, y1, g.h)?;
    let (fx0, fx1) = flow_span(x0, x1, g.flow.rf, g.flow.w)?;
    let (fy0, fy1) = flow_span(y0, y1, g.flow.rf, g.flow.h)?;
    let (s, fs) = (g.s, g.flow.s);
    Some(GridPatch {
        x0,
        y0,
        x1,
        y1,
        film: take_rows(&g.film, s, x0, y0, x1, y1),
        susp: take_rows(&g.susp, s, x0, y0, x1, y1),
        susp_rgb: take_rows(&g.susp_rgb, s, x0, y0, x1, y1),
        sett: take_rows(&g.sett, s, x0, y0, x1, y1),
        sett_rgb: take_rows(&g.sett_rgb, s, x0, y0, x1, y1),
        wet: take_rows(&g.wet, s, x0, y0, x1, y1),
        active: take_rows(&g.active, s, x0, y0, x1, y1),
        bloom: take_rows(&g.bloom, s, x0, y0, x1, y1),
        fx0,
        fy0,
        fx1,
        fy1,
        vel_x: take_rows(&g.vel_x, fs, fx0, fy0, fx1, fy1),
        vel_y: take_rows(&g.vel_y, fs, fx0, fy0, fx1, fy1),
    })
}

/// Devolve o recorte ao grid. Os transientes de fluxo são ZERADOS na janela restaurada, pelo mesmo
/// motivo que o [`super::restore_grid`] os zera na folha inteira: eles são derivados do campo
/// persistente e o próximo `build_flow_field` os reconstrói.
pub fn restore_grid_region(g: &mut Grid, p: &GridPatch) {
    let (s, fs) = (g.s, g.flow.s);
    let (x0, y0, x1, y1) = (p.x0, p.y0, p.x1, p.y1);
    put_rows(&mut g.film, s, x0, y0, x1, y1, &p.film);
    put_rows(&mut g.susp, s, x0, y0, x1, y1, &p.susp);
    put_rows(&mut g.susp_rgb, s, x0, y0, x1, y1, &p.susp_rgb);
    put_rows(&mut g.sett, s, x0, y0, x1, y1, &p.sett);
    put_rows(&mut g.sett_rgb, s, x0, y0, x1, y1, &p.sett_rgb);
    put_rows(&mut g.wet, s, x0, y0, x1, y1, &p.wet);
    put_rows(&mut g.active, s, x0, y0, x1, y1, &p.active);
    put_rows(&mut g.bloom, s, x0, y0, x1, y1, &p.bloom);
    let (fx0, fy0, fx1, fy1) = (p.fx0, p.fy0, p.fx1, p.fy1);
    put_rows(&mut g.vel_x, fs, fx0, fy0, fx1, fy1, &p.vel_x);
    put_rows(&mut g.vel_y, fs, fx0, fy0, fx1, fy1, &p.vel_y);
    let fw = (fx1 - fx0 + 1) as usize;
    for fy in fy0..=fy1 {
        let row = fy as usize * fs + fx0 as usize;
        g.flow_x[row..row + fw].fill(0.0);
        g.flow_y[row..row + fw].fill(0.0);
    }
}

#[cfg(test)]
#[path = "patch_tests.rs"]
mod tests;
