//! O estado do painel MACROS: o do GESTO (a etapa escolhida, o renomear) na `State` tipada; o do
//! DOCUMENTO chega por thread-local, publicado pela shell antes de o painel pintar.

use ph2d_editor_core::macro_edits::{EtapaView, MacroPanelInfo};

/// A etapa escolhida: o objecto e o endereço dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Escolhida {
    pub entity_bits: u64,
    pub trilha: usize,
    pub k: usize,
}

#[derive(Clone, Debug, Default)]
pub struct MacrosPanelState {
    pub escolhida: Option<Escolhida>,
    /// O campo de renomear está aberto sobre a linha escolhida.
    pub renomeando: bool,
}

thread_local! {
    static CURRENT: std::cell::RefCell<MacroPanelInfo> =
        std::cell::RefCell::new(MacroPanelInfo::default());
    static LAST_CONTENT_H: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// **A porta da shell** — uma vez por quadro.
pub fn set_current_macros(info: MacroPanelInfo) {
    CURRENT.with(|c| *c.borrow_mut() = info);
}

/// O instantâneo como o painel o leu.
#[must_use]
pub fn current_macros() -> MacroPanelInfo {
    CURRENT.with(|c| c.borrow().clone())
}

pub(crate) fn with_current<R>(f: impl FnOnce(&MacroPanelInfo) -> R) -> R {
    CURRENT.with(|c| f(&c.borrow()))
}

/// A etapa `e` no instantâneo actual.
pub(crate) fn etapa(e: Escolhida) -> Option<EtapaView> {
    with_current(|i| {
        let o = i
            .objeto
            .as_ref()
            .filter(|o| o.entity_bits == e.entity_bits)?;
        o.etapas
            .iter()
            .find(|x| x.trilha == e.trilha && x.k == e.k)
            .cloned()
    })
}

pub(crate) fn set_last_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}

#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(std::cell::Cell::get)
}
