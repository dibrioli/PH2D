//! **O erro que chega ao artista** (plano 01 §5.6): cada erro das famílias numa chave do i18n
//! (`panel.macros.error.*`) e o detalhe que a frase cita.

use ph2d_app_motion::macro_track::{etapas::ErroDeEdicaoDoMotion, tocar::ErroDaMacro as Mo};
use ph2d_editor_core::macro_edits::ErroDaMacroView;
use ph2d_macro::{CodecError, ErroDeEdicao};
use ph2d_tool_painter::macro_track::{etapas::ErroDeEdicaoDoPainter, tocar::ErroDaMacro as Pi};

pub const VERSAO_NOVA: &str = "panel.macros.error.newer_version";
pub const ESTRAGADA: &str = "panel.macros.error.damaged";
pub const INICIAL: &str = "panel.macros.error.initial_state";
pub const TEXTURAS: &str = "panel.macros.error.missing_textures";
pub const PORTA: &str = "panel.macros.error.refused_door";
pub const PASSO_NOVO: &str = "panel.macros.error.newer_step";
pub const DIVERGIU: &str = "panel.macros.error.diverged";
pub const PRESET: &str = "panel.macros.error.missing_preset";
pub const NAO_CABE: &str = "panel.macros.error.edit_does_not_fit";
pub const ESPERA: &str = "panel.macros.error.wait_longer_than_gap";
pub const NAO_PINCELADA: &str = "panel.macros.error.not_a_stroke";
pub const NAO_VALOR: &str = "panel.macros.error.not_a_value";
pub const SEM_ETAPA: &str = "panel.macros.error.no_such_step";
pub const NOME: &str = "panel.macros.error.unknown_name";
pub const ALVO: &str = "panel.macros.error.target";
pub const DOCUMENTO: &str = "panel.macros.error.document";
pub const SEM_GRAFO: &str = "panel.macros.error.no_graph";

/// Todas as chaves (o censo da tabela as confere).
pub const CHAVES: &[&str] = &[
    VERSAO_NOVA,
    ESTRAGADA,
    INICIAL,
    TEXTURAS,
    PORTA,
    PASSO_NOVO,
    DIVERGIU,
    PRESET,
    NAO_CABE,
    ESPERA,
    NAO_PINCELADA,
    NAO_VALOR,
    SEM_ETAPA,
    NOME,
    ALVO,
    DOCUMENTO,
    SEM_GRAFO,
];

fn v(chave: &'static str) -> ErroDaMacroView {
    ErroDaMacroView {
        chave,
        detalhe: None,
    }
}

fn com(chave: &'static str, d: impl Into<String>) -> ErroDaMacroView {
    ErroDaMacroView {
        chave,
        detalhe: Some(d.into()),
    }
}

#[must_use]
pub fn sem_etapa() -> ErroDaMacroView {
    v(SEM_ETAPA)
}

#[must_use]
pub fn do_ficheiro(e: &CodecError) -> ErroDaMacroView {
    match e {
        CodecError::MajorMismatch { .. } | CodecError::Exige(_) => v(VERSAO_NOVA),
        _ => v(ESTRAGADA),
    }
}

#[must_use]
pub fn de_edicao(e: &ErroDeEdicao) -> ErroDaMacroView {
    match e {
        ErroDeEdicao::EsperaMaiorQueOIntervalo(_) => v(ESPERA),
        _ => v(NAO_CABE),
    }
}

#[must_use]
pub fn do_painter(e: &Pi) -> ErroDaMacroView {
    use ph2d_tool_painter::macro_track::inicial::ErroDoInicial as I;
    match e {
        Pi::Ficheiro(c) => do_ficheiro(c),
        Pi::Inicial(I::VersaoFutura(_)) => v(VERSAO_NOVA),
        Pi::Inicial(I::Texturas(t)) => com(TEXTURAS, t.join(", ")),
        Pi::Inicial(_) => v(INICIAL),
        Pi::Passo(_) => v(ESTRAGADA),
        Pi::PortaRecusada(p) => com(PORTA, p.clone()),
        Pi::PassoDesconhecido(_) => v(PASSO_NOVO),
        Pi::Divergiu(_) => v(DIVERGIU),
        Pi::Preset(p) => com(PRESET, p.clone()),
        Pi::Edicao(x) => de_edicao(x),
    }
}

#[must_use]
pub fn do_motion(e: &Mo) -> ErroDaMacroView {
    match e {
        Mo::Ficheiro(c) => do_ficheiro(c),
        Mo::Inicial(_) => v(INICIAL),
        Mo::Passo(_) => v(ESTRAGADA),
        Mo::PortaRecusada(p) => com(PORTA, p.clone()),
        Mo::PassoDesconhecido(_) | Mo::IntencaoDesconhecida(_) => v(PASSO_NOVO),
        Mo::Nome(n) => com(NOME, n.clone()),
        Mo::Alvo => v(ALVO),
        Mo::Documento(d) => com(DOCUMENTO, d.clone()),
        Mo::ArestasDoDocumento => v(DOCUMENTO),
        Mo::Divergiu => v(DIVERGIU),
        Mo::SemGrafo => v(SEM_GRAFO),
        Mo::Edicao(x) => de_edicao(x),
    }
}

#[must_use]
pub fn de_edicao_do_painter(e: &ErroDeEdicaoDoPainter) -> ErroDaMacroView {
    match e {
        ErroDeEdicaoDoPainter::SemEtapa(_) => v(SEM_ETAPA),
        ErroDeEdicaoDoPainter::NaoEPincelada(_) => v(NAO_PINCELADA),
        ErroDeEdicaoDoPainter::Macro(m) => do_painter(m),
    }
}

#[must_use]
pub fn de_edicao_do_motion(e: &ErroDeEdicaoDoMotion) -> ErroDaMacroView {
    match e {
        ErroDeEdicaoDoMotion::SemEtapa(_) => v(SEM_ETAPA),
        ErroDeEdicaoDoMotion::NaoEParam(_) => v(NAO_VALOR),
    }
}
