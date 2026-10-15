//! **Uma macro do Painter em cena** (ADR-0186, W4 item 5.5 D): a trilha `"painter"` de um
//! `ph2d_ecs::Macro`, num [`TocadorDoPainter`] próprio, a seguir o instante que a Timeline escreveu
//! (`ph2d_ecs::MacroTempo`). Para a frente despacha os passos até lá; para trás recomeça do estado
//! inicial gravado e avança (a lei do `MacroPlayer::seek`, à mão, como a do Motion).
//!
//! Um passo que falha PÁRA a macro nesse passo e o erro fica guardado ([`MacroEmCena::erro`]); recuar
//! para antes dele recomeça e limpa-o.

use super::FAMILIA;
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use ph2d_macro::{MacroDoc, MacroPlayer, MacroTime};
use std::sync::Arc;

pub struct MacroEmCena {
    /// Só a trilha do Painter: as outras famílias tocam a sua lendo o mesmo componente.
    doc: MacroDoc,
    tocador: TocadorDoPainter,
    player: MacroPlayer,
    erro: Option<ErroDaMacro>,
}

fn tocador_de(doc: &MacroDoc) -> Result<TocadorDoPainter, ErroDaMacro> {
    let t = &doc.tracks[0];
    TocadorDoPainter::novo(&t.initial, t.events.iter().map(|e| e.payload.as_slice()))
}

impl MacroEmCena {
    /// A macro de um `Macro.doc`, no estado inicial gravado. `Ok(None)` = sem trilha do Painter.
    ///
    /// # Errors
    /// [`ErroDaMacro::Ficheiro`] (o `doc` não se lê) e os do [`TocadorDoPainter::novo`].
    pub fn de(bytes: &[u8]) -> Result<Option<Self>, ErroDaMacro> {
        let mut doc = ph2d_macro::codec::read(bytes).map_err(ErroDaMacro::Ficheiro)?;
        doc.tracks.retain(|t| t.family == FAMILIA);
        doc.tracks.truncate(1);
        if doc.tracks.is_empty() {
            return Ok(None);
        }
        doc.tracks[0] = doc.tracks[0]
            .tocavel(&super::etapas::LeitorDoPainter)
            .map_err(ErroDaMacro::Edicao)?;
        let tocador = tocador_de(&doc)?;
        let player = MacroPlayer::new(&doc);
        Ok(Some(Self {
            doc,
            tocador,
            player,
            erro: None,
        }))
    }

    /// **Segue o relógio da macro até `segundos`.**
    ///
    /// # Errors
    /// O [`ErroDaMacro`] do passo que parou a macro (também guardado em [`Self::erro`]).
    pub fn segue(&mut self, segundos: f64) -> Result<(), ErroDaMacro> {
        let to = MacroTime::from_secs_f64(segundos);
        let recua = to < self.player.time();
        if !recua && let Some(e) = &self.erro {
            return Err(e.clone());
        }
        if recua {
            self.erro = None;
            match tocador_de(&self.doc) {
                Ok(t) => self.tocador = t,
                Err(e) => {
                    self.erro = Some(e.clone());
                    return Err(e);
                }
            }
            self.player = MacroPlayer::new(&self.doc);
        }
        let Self {
            doc,
            tocador,
            player,
            ..
        } = self;
        let feito = player
            .advance_to(doc, to, |_, _, ev| tocador.aplica(&ev.payload))
            .map(|_| ());
        if let Err(e) = &feito {
            self.erro = Some(e.clone());
        }
        feito
    }

    /// A tela para a cena, se mudou desde a última leitura ([`TocadorDoPainter::toma_a_tela`]).
    pub fn toma_a_tela(&mut self) -> Option<(Arc<Vec<u8>>, u32, u32)> {
        self.tocador.toma_a_tela()
    }

    #[must_use]
    pub fn tocador(&self) -> &TocadorDoPainter {
        &self.tocador
    }

    /// A sprite que a macro pinta (o `target` da trilha, `StableId`; `0` = a do próprio `Macro`).
    #[must_use]
    pub fn alvo(&self) -> u64 {
        self.doc.tracks[0].target
    }

    /// O tempo até onde a macro já tocou.
    #[must_use]
    pub fn tempo(&self) -> MacroTime {
        self.player.time()
    }

    /// O passo que parou a macro, se algum.
    #[must_use]
    pub fn erro(&self) -> Option<&ErroDaMacro> {
        self.erro.as_ref()
    }
}
