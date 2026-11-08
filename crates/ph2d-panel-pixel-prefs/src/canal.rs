//! O canal entre a família e a janela, e a lei pura de cada controlo sobre [`PixelPrefs`].

use std::cell::RefCell;

use ph2d_pixel_core::PixelPrefs;

/// A quem a janela escreve (Aseprite O18: `Active Document` / `New Documents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    Document,
    New,
}

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Caixa {
    GridVisible,
    GridAuto,
    PixelVisible,
    PixelAuto,
    CheckerZoom,
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
    }
}

/// Escreve um número (cortado à faixa do campo: tamanhos ≥ 1, opacidade 0–255).
pub fn poe_numero(p: &mut PixelPrefs, n: Numero, v: f64) {
    let r = v.round();
    let lado = r.clamp(1.0, f64::from(u16::MAX)) as u32;
    let op = r.clamp(0.0, 255.0) as u8;
    let pos = r.clamp(f64::from(i16::MIN), f64::from(i16::MAX)) as i32;
    match n {
        Numero::GridX => p.grid.x = pos,
        Numero::GridY => p.grid.y = pos,
        Numero::GridW => p.grid.w = lado,
        Numero::GridH => p.grid.h = lado,
        Numero::GridOpacity => p.grid.opacity = op,
        Numero::PixelOpacity => p.pixel_grid.opacity = op,
        Numero::CheckerW => p.checker.w = lado,
        Numero::CheckerH => p.checker.h = lado,
    }
}

fn caixa_mut(p: &mut PixelPrefs, c: Caixa) -> &mut bool {
    match c {
        Caixa::GridVisible => &mut p.grid.visible,
        Caixa::GridAuto => &mut p.grid.auto_opacity,
        Caixa::PixelVisible => &mut p.pixel_grid.visible,
        Caixa::PixelAuto => &mut p.pixel_grid.auto_opacity,
        Caixa::CheckerZoom => &mut p.checker.apply_zoom,
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

/// O que a janela mostra: o documento activo (`None` = nenhum Pixel Sprite em Draw) e os valores
/// para documentos novos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrefsView {
    pub doc: Option<PixelPrefs>,
    pub new: PixelPrefs,
}

impl PrefsView {
    /// As preferências que o âmbito `s` edita.
    #[must_use]
    pub fn of(&self, s: Scope) -> Option<PixelPrefs> {
        match s {
            Scope::Document => self.doc,
            Scope::New => Some(self.new),
        }
    }
}

/// Um gesto da janela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefsRequest {
    /// As preferências inteiras do âmbito, já mudadas.
    Set(Scope, PixelPrefs),
    /// `Set as Defaults`: as do documento passam a ser as dos novos.
    SetAsDefaults,
    /// `Reset`: o âmbito volta aos valores de fábrica.
    Reset(Scope),
}

#[derive(Default)]
struct Canal {
    view: PrefsView,
    requests: Vec<PrefsRequest>,
    abrir: bool,
    carregado: bool,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

/// Abre a janela no próximo quadro (Edit ▸ Grid & Background… ou o Run Command).
pub fn pede_abrir() {
    com(|c| c.abrir = true);
}

pub(crate) fn take_abrir() -> bool {
    com(|c| std::mem::take(&mut c.abrir))
}

#[must_use]
pub fn view() -> PrefsView {
    com(|c| c.view)
}

pub fn request(r: PrefsRequest) {
    com(|c| c.requests.push(r));
}

/// O laço da família, uma vez por quadro: publica as preferências do documento activo, resolve os
/// pedidos dos documentos novos (gravados em `ph2d_pixel_render::prefs_view::defaults_path`) e
/// devolve as novas preferências do documento, se mudaram — a família grava-as sem passo de desfazer.
pub fn troca(doc: Option<PixelPrefs>) -> Option<PixelPrefs> {
    troca_em(
        doc,
        ph2d_pixel_render::prefs_view::defaults_path().as_deref(),
    )
}

/// O [`troca`] com o ficheiro dos novos em `file` (`None` = sem ficheiro, só na memória).
pub fn troca_em(doc: Option<PixelPrefs>, file: Option<&std::path::Path>) -> Option<PixelPrefs> {
    use ph2d_pixel_render::prefs_view::{load_defaults_from, save_defaults_to};
    let (requests, mut new) = com(|c| {
        if !c.carregado {
            c.carregado = true;
            c.view.new = file.map_or_else(PixelPrefs::default, load_defaults_from);
        }
        (std::mem::take(&mut c.requests), c.view.new)
    });
    let (mut out, antes) = (doc, new);
    for r in requests {
        match r {
            PrefsRequest::Set(Scope::Document, p) => out = out.map(|_| p),
            PrefsRequest::Set(Scope::New, p) => new = p,
            PrefsRequest::SetAsDefaults => new = out.unwrap_or(new),
            PrefsRequest::Reset(Scope::Document) => out = out.map(|_| PixelPrefs::default()),
            PrefsRequest::Reset(Scope::New) => new = PixelPrefs::default(),
        }
    }
    if new != antes
        && let Some(f) = file
    {
        let _ = save_defaults_to(f, &new);
    }
    com(|c| c.view = PrefsView { doc: out, new });
    out.filter(|p| Some(*p) != doc)
}
