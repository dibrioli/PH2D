//! **As etapas de UMA família de macros, vistas de fora dela** (ADR-0186, W5e): a porta pela qual a
//! família MACROS lê e edita as trilhas de uma família que ela não conhece. A composição injecta a
//! implementação (o Motion), e nenhuma família depende de outra
//! (`architecture_no_dependency_climbs_a_layer`). E as chaves do erro que chega ao artista
//! (`panel.macros.error.*`), comuns a todas.

use crate::macro_edits::{CampoDaEtapa, ErroDaMacroView, EtapaView, FamiliaDaMacro};
use ph2d_macro::{LeitorDaFamilia, MacroTime, Track};

/// O que uma família diz das suas trilhas a quem mostra e edita as etapas.
pub trait EtapasDaFamilia {
    fn familia(&self) -> FamiliaDaMacro;
    /// O `Track::family` das trilhas dela.
    fn chave(&self) -> &'static str;
    fn leitor(&self) -> &'static dyn LeitorDaFamilia;
    /// As etapas da trilha `trilha` (o índice no `MacroDoc`), como o painel as mostra.
    ///
    /// # Errors
    /// O porquê, para o painel o dizer.
    fn vistas(&self, trilha: usize, track: &Track) -> Result<Vec<EtapaView>, ErroDaMacroView>;
    /// Edita `campo` na etapa `k`. O nome é de todas as famílias e não chega aqui.
    ///
    /// # Errors
    /// O porquê, para o painel o dizer.
    fn edita(
        &self,
        track: &mut Track,
        k: usize,
        campo: &CampoDaEtapa,
    ) -> Result<(), ErroDaMacroView>;
}

/// Um instante da macro, em segundos.
#[must_use]
pub fn segundos(t: MacroTime) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "µs de uma macro cabem num f64")]
    let v = t.0 as f64 / 1e6;
    v
}

/// A espera editada antes da etapa ancorada em `ancora`, em segundos.
#[must_use]
pub fn espera_s(track: &Track, ancora: usize) -> f64 {
    #[expect(clippy::cast_precision_loss, reason = "µs de uma espera cabem num f64")]
    track
        .edicoes
        .iter()
        .find(|e| e.ancora == ancora)
        .map_or(0.0, |e| e.espera_us as f64 / 1e6)
}

/// Segundos de uma espera, em µs.
#[must_use]
pub fn us(segundos: f64) -> i64 {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "segundos de uma espera cabem num i64 de µs"
    )]
    let v = (segundos * 1e6).round() as i64;
    v
}

/// **O erro que chega ao artista** (plano 01 §5.6): a chave do i18n e o detalhe que a frase cita.
pub mod erros {
    use crate::macro_edits::ErroDaMacroView;
    use ph2d_macro::{CodecError, ErroDeEdicao};

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
    pub const ANTES_DO_ZERO: &str = "panel.macros.error.before_start";
    pub const DESFAZER: &str = "panel.macros.error.changes_what_undo_takes_back";
    pub const NAO_PINCELADA: &str = "panel.macros.error.not_a_stroke";
    pub const NAO_VALOR: &str = "panel.macros.error.not_a_value";
    pub const SEM_ETAPA: &str = "panel.macros.error.no_such_step";
    pub const NOME: &str = "panel.macros.error.unknown_name";
    pub const ALVO: &str = "panel.macros.error.target";
    pub const DOCUMENTO: &str = "panel.macros.error.document";
    pub const SEM_GRAFO: &str = "panel.macros.error.no_graph";
    pub const OUTRO_TAMANHO: &str = "panel.macros.error.other_size";

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
        ANTES_DO_ZERO,
        DESFAZER,
        NAO_PINCELADA,
        NAO_VALOR,
        SEM_ETAPA,
        NOME,
        ALVO,
        DOCUMENTO,
        SEM_GRAFO,
        OUTRO_TAMANHO,
    ];

    /// O erro sem detalhe.
    #[must_use]
    pub fn erro(chave: &'static str) -> ErroDaMacroView {
        ErroDaMacroView {
            chave,
            detalhe: None,
        }
    }

    /// O erro com o detalhe que a frase cita.
    #[must_use]
    pub fn erro_com(chave: &'static str, d: impl Into<String>) -> ErroDaMacroView {
        ErroDaMacroView {
            chave,
            detalhe: Some(d.into()),
        }
    }

    #[must_use]
    pub fn sem_etapa() -> ErroDaMacroView {
        erro(SEM_ETAPA)
    }

    #[must_use]
    pub fn do_ficheiro(e: &CodecError) -> ErroDaMacroView {
        match e {
            CodecError::MajorMismatch { .. } | CodecError::Exige(_) => erro(VERSAO_NOVA),
            _ => erro(ESTRAGADA),
        }
    }

    #[must_use]
    pub fn de_edicao(e: &ErroDeEdicao) -> ErroDaMacroView {
        match e {
            ErroDeEdicao::EsperaMaiorQueOIntervalo(_) => erro(ESPERA),
            ErroDeEdicao::ComecaAntesDoZero(_) => erro(ANTES_DO_ZERO),
            ErroDeEdicao::MudaOQueODesfazerDesfaz(_) => erro(DESFAZER),
            _ => erro(NAO_CABE),
        }
    }
}
