//! **O erro que chega ao artista** (plano 01 §5.6): os do Painter numa chave do i18n
//! (`panel.macros.error.*`). As chaves e os erros comuns vivem na folha
//! (`ph2d_editor_core::macro_etapas::erros`); os do Motion, na família dele.

use ph2d_editor_core::macro_edits::ErroDaMacroView;
pub use ph2d_editor_core::macro_etapas::erros::*;
use ph2d_tool_painter::macro_track::{etapas::ErroDeEdicaoDoPainter, tocar::ErroDaMacro as Pi};

#[must_use]
pub fn do_painter(e: &Pi) -> ErroDaMacroView {
    use ph2d_tool_painter::macro_track::inicial::ErroDoInicial as I;
    match e {
        Pi::Ficheiro(c) => do_ficheiro(c),
        Pi::Inicial(I::VersaoFutura(_)) => erro(VERSAO_NOVA),
        Pi::Inicial(I::Texturas(t)) => erro_com(TEXTURAS, t.join(", ")),
        Pi::Inicial(_) => erro(INICIAL),
        Pi::Passo(_) | Pi::FioCaiu(_) => erro(ESTRAGADA),
        Pi::PortaRecusada(p) => erro_com(PORTA, p.clone()),
        Pi::PassoDesconhecido(_) => erro(PASSO_NOVO),
        Pi::Divergiu(_) => erro(DIVERGIU),
        Pi::Preset(p) => erro_com(PRESET, p.clone()),
        Pi::Edicao(x) => de_edicao(x),
    }
}

#[must_use]
pub fn de_edicao_do_painter(e: &ErroDeEdicaoDoPainter) -> ErroDaMacroView {
    match e {
        ErroDeEdicaoDoPainter::SemEtapa(_) => erro(SEM_ETAPA),
        ErroDeEdicaoDoPainter::NaoEPincelada(_) => erro(NAO_PINCELADA),
        ErroDeEdicaoDoPainter::Macro(m) => do_painter(m),
    }
}
