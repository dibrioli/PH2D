//! **O canal das preferências do Pixel** (P12 P, Aseprite O18/O19): a vista que a janela GRID &
//! BACKGROUND e a linha do tempo (os colchetes do onion) lêem, os pedidos que devolvem e o laço da
//! família que os resolve. Mora aqui, ao lado de [`crate::prefs_view`], e não num painel: dois
//! painéis não dependem um do outro (`architecture_cycle_prevention`).

use std::cell::RefCell;

use ph2d_pixel_core::PixelPrefs;

/// A quem a janela escreve (Aseprite O18: `Active Document` / `New Documents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    Document,
    New,
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
    onion: bool,
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

/// Abre a janela no cartão Onion Skin, sobre o documento (o botão direito no botão do onion, R4).
pub fn pede_abrir_onion() {
    com(|c| (c.abrir, c.onion) = (true, true));
}

/// O que a janela consome no quadro: `(abrir, no cartão Onion Skin)`.
pub fn take_abrir() -> (bool, bool) {
    com(|c| (std::mem::take(&mut c.abrir), std::mem::take(&mut c.onion)))
}

#[must_use]
pub fn view() -> PrefsView {
    com(|c| c.view)
}

pub fn request(r: PrefsRequest) {
    com(|c| c.requests.push(r));
}

/// O laço da família, uma vez por quadro: publica as preferências do documento activo, resolve os
/// pedidos dos documentos novos (gravados em [`crate::prefs_view::defaults_path`]) e
/// devolve as novas preferências do documento, se mudaram — a família grava-as sem passo de desfazer.
pub fn troca(doc: Option<PixelPrefs>) -> Option<PixelPrefs> {
    troca_em(doc, crate::prefs_view::defaults_path().as_deref())
}

/// O [`troca`] com o ficheiro dos novos em `file` (`None` = sem ficheiro, só na memória).
pub fn troca_em(doc: Option<PixelPrefs>, file: Option<&std::path::Path>) -> Option<PixelPrefs> {
    use crate::prefs_view::{load_defaults_from, save_defaults_to};
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
