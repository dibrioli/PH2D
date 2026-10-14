//! **Uma macro em cena** (ADR-0186, item 5 da W4): a trilha do Motion de um `ph2d_ecs::Macro`, num
//! palco próprio, a seguir o instante que a Timeline escreveu (`ph2d_ecs::MacroTempo`). Para a frente
//! despacha os passos até lá; para trás recomeça do estado inicial gravado e avança (o `seek` do
//! núcleo) — o mesmo retrato que tocar do zero até lá.
//!
//! Um passo que falha PÁRA a macro nesse passo e o erro fica guardado ([`MacroEmCena::erro`]): para a
//! frente ela não tenta de novo a cada quadro; recuar para antes dele recomeça e limpa-o.

use super::FAMILIA;
use super::tocar::{ErroDaMacro, Modo, TocadorDoMotion};
use crate::motion_mundo::MundoDaCena;
use crate::motion_state::MotionState;
use ph2d_macro::{MacroDoc, MacroPlayer, MacroTime};

pub struct MacroEmCena {
    /// Só a trilha do Motion: as outras famílias tocam a sua lendo o mesmo componente.
    doc: MacroDoc,
    modo: Modo,
    tocador: TocadorDoMotion,
    player: MacroPlayer,
    erro: Option<ErroDaMacro>,
}

/// Segundos do relógio da macro → o tempo do núcleo (µs). Antes do início é o início.
#[must_use]
pub fn instante(segundos: f64) -> MacroTime {
    MacroTime((segundos.max(0.0) * 1e6).round() as u64)
}

impl MacroEmCena {
    /// A macro de um `Macro.doc`, no estado inicial gravado. `Ok(None)` = a gravação não tem trilha
    /// do Motion.
    ///
    /// # Errors
    /// [`ErroDaMacro::Ficheiro`] (o `doc` não se lê) e [`ErroDaMacro::Inicial`].
    pub fn de(bytes: &[u8], modo: Modo) -> Result<Option<Self>, ErroDaMacro> {
        let mut doc = ph2d_macro::codec::read(bytes).map_err(ErroDaMacro::Ficheiro)?;
        // Uma gravação abre UMA trilha do Motion (a do objecto em edição, `gravacao_abre`).
        doc.tracks.retain(|t| t.family == FAMILIA);
        doc.tracks.truncate(1);
        let Some(trilha) = doc.tracks.first() else {
            return Ok(None);
        };
        let tocador = TocadorDoMotion::da_trilha(trilha, modo)?;
        let player = MacroPlayer::new(&doc);
        Ok(Some(Self {
            doc,
            modo,
            tocador,
            player,
            erro: None,
        }))
    }

    /// **Segue o relógio da macro até `segundos`**, com o mundo que a cena publicou neste quadro.
    ///
    /// # Errors
    /// O [`ErroDaMacro`] do passo que parou a macro (também guardado em [`Self::erro`]).
    pub fn segue(
        &mut self,
        segundos: f64,
        mundo: &MundoDaCena,
        placa: Option<&ph2d_gpu::GpuContext>,
    ) -> Result<(), ErroDaMacro> {
        let to = instante(segundos);
        let recua = to < self.player.time();
        if !recua && let Some(e) = &self.erro {
            return Err(e.clone());
        }
        // Recuar = a lei do `MacroPlayer::seek` (estado inicial + avançar), à mão: o `reset` e o
        // `aplica` dele seriam dois empréstimos do mesmo palco.
        let feito = self.recomeca_se(recua).and_then(|()| {
            let Self {
                doc,
                tocador,
                player,
                ..
            } = self;
            player
                .advance_to(doc, to, |_, _, ev| {
                    tocador.recebe_o_mundo(mundo);
                    tocador.aplica(&ev.payload, placa)
                })
                .map(|_| ())
        });
        if let Err(e) = &feito {
            self.erro = Some(e.clone());
        }
        feito
    }

    fn recomeca_se(&mut self, recua: bool) -> Result<(), ErroDaMacro> {
        if recua {
            self.erro = None;
            self.tocador = TocadorDoMotion::da_trilha(&self.doc.tracks[0], self.modo)?;
            self.player = MacroPlayer::new(&self.doc);
        }
        Ok(())
    }

    #[must_use]
    pub fn palco(&self) -> &MotionState {
        self.tocador.palco()
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
