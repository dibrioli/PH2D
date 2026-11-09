//! A lei pura de cada controlo da janela sobre [`PixelPrefs`]. O canal entre a família, a janela e a
//! linha do tempo mora em `ph2d_pixel_render::prefs_canal` (dois painéis não se vêem).

use ph2d_pixel_core::PixelPrefs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Numero {
    GridX,
    GridY,
    GridW,
    GridH,
    GridOpacity,
    PixelOpacity,
    CheckerW,
    CheckerH,
    OnionPrev,
    OnionNext,
    /// Em % como o Trial (O21: 27 % = 68).
    OnionOpacity,
    OnionStep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caixa {
    GridVisible,
    GridAuto,
    PixelVisible,
    PixelAuto,
    CheckerZoom,
    OnionLoopTag,
    OnionCurrentLayer,
    /// `In Front` (ligada) / `Behind` (desligada).
    OnionFront,
    /// `Red/Blue Tint` (ligada) / `Merge` (desligada).
    OnionTint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cor {
    Grid,
    PixelGrid,
    CheckerA,
    CheckerB,
}

/// O valor de um número. A opacidade mostra-se 0–255, como no Trial.
#[must_use]
pub fn numero(p: &PixelPrefs, n: Numero) -> i64 {
    match n {
        Numero::GridX => i64::from(p.grid.x),
        Numero::GridY => i64::from(p.grid.y),
        Numero::GridW => i64::from(p.grid.w),
        Numero::GridH => i64::from(p.grid.h),
        Numero::GridOpacity => i64::from(p.grid.opacity),
        Numero::PixelOpacity => i64::from(p.pixel_grid.opacity),
        Numero::CheckerW => i64::from(p.checker.w),
        Numero::CheckerH => i64::from(p.checker.h),
        Numero::OnionPrev => i64::from(p.onion.prev),
        Numero::OnionNext => i64::from(p.onion.next),
        Numero::OnionOpacity => pct(p.onion.base),
        Numero::OnionStep => pct(p.onion.step),
    }
}

/// O topo de uma opacidade em bytes (0–255).
pub(crate) const BYTE_MAX: f64 = u8::MAX as f64;
/// O topo de uma percentagem.
pub(crate) const PCT_MAX: f64 = 100.0; // LITERAL-PX-OK: os 100 % de uma percentagem, não uma medida de UI

/// 0–255 → % (o Trial mostra 27 % para 68 e 11 % para 28).
fn pct(v: u8) -> i64 {
    (f64::from(v) * PCT_MAX / BYTE_MAX).round() as i64
}

/// % → 0–255, o inverso exacto de [`pct`] (`floor`: 27 % → 68, 11 % → 28).
fn de_pct(p: f64) -> u8 {
    (p.clamp(0.0, PCT_MAX) * BYTE_MAX / PCT_MAX).floor() as u8
}

/// Escreve um número (cortado à faixa do campo: tamanhos ≥ 1, opacidade 0–255).
pub fn poe_numero(p: &mut PixelPrefs, n: Numero, v: f64) {
    let r = v.round();
    let faixa = |lo: f64, hi: f64| ph2d_editor_core::math::safe_clamp_f64(r, lo, hi);
    let lado = faixa(1.0, f64::from(u16::MAX)) as u32;
    let op = faixa(0.0, BYTE_MAX) as u8;
    let pos = faixa(f64::from(i16::MIN), f64::from(i16::MAX)) as i32;
    let alcance = faixa(0.0, f64::from(u16::MAX)) as u32;
    match n {
        Numero::GridX => p.grid.x = pos,
        Numero::GridY => p.grid.y = pos,
        Numero::GridW => p.grid.w = lado,
        Numero::GridH => p.grid.h = lado,
        Numero::GridOpacity => p.grid.opacity = op,
        Numero::PixelOpacity => p.pixel_grid.opacity = op,
        Numero::CheckerW => p.checker.w = lado,
        Numero::CheckerH => p.checker.h = lado,
        Numero::OnionPrev => p.onion.prev = alcance,
        Numero::OnionNext => p.onion.next = alcance,
        Numero::OnionOpacity => p.onion.base = de_pct(r),
        Numero::OnionStep => p.onion.step = de_pct(r),
    }
}

fn caixa_mut(p: &mut PixelPrefs, c: Caixa) -> &mut bool {
    match c {
        Caixa::GridVisible => &mut p.grid.visible,
        Caixa::GridAuto => &mut p.grid.auto_opacity,
        Caixa::PixelVisible => &mut p.pixel_grid.visible,
        Caixa::PixelAuto => &mut p.pixel_grid.auto_opacity,
        Caixa::CheckerZoom => &mut p.checker.apply_zoom,
        Caixa::OnionLoopTag => &mut p.onion.loop_tag,
        Caixa::OnionCurrentLayer => &mut p.onion.current_layer_only,
        Caixa::OnionFront => &mut p.onion.front,
        Caixa::OnionTint => &mut p.onion.tint,
    }
}

#[must_use]
pub fn caixa(p: &PixelPrefs, c: Caixa) -> bool {
    let mut q = *p;
    *caixa_mut(&mut q, c)
}

pub fn alterna(p: &mut PixelPrefs, c: Caixa) {
    let v = caixa_mut(p, c);
    *v = !*v;
}

fn cor_mut(p: &mut PixelPrefs, c: Cor) -> &mut [u8; 4] {
    match c {
        Cor::Grid => &mut p.grid.color,
        Cor::PixelGrid => &mut p.pixel_grid.color,
        Cor::CheckerA => &mut p.checker.color_a,
        Cor::CheckerB => &mut p.checker.color_b,
    }
}

#[must_use]
pub fn cor(p: &PixelPrefs, c: Cor) -> [u8; 4] {
    let mut q = *p;
    *cor_mut(&mut q, c)
}

pub fn poe_cor(p: &mut PixelPrefs, c: Cor, rgba: [u8; 4]) {
    *cor_mut(p, c) = rgba;
}
