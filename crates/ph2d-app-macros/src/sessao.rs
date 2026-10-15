//! **A gravação em curso**: o gravador, a trilha aberta pela família e o objecto que fica com a
//! `Macro` ao Parar.

use crate::{Cena, Familias};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::macro_edits::{self as m, FamiliaDaMacro};
use ph2d_macro::{Gravadora, MacroRecorder, RefRect, TrackId};

pub struct Sessao {
    rec: MacroRecorder,
    trilha: TrackId,
    familia: FamiliaDaMacro,
    /// Os bits do objecto que fica com a `Macro`.
    dono: u64,
}

fn gravadora<'a>(f: &'a mut Familias<'_>, fam: FamiliaDaMacro) -> Option<&'a mut dyn Gravadora> {
    match fam {
        FamiliaDaMacro::Painter => f.painter.as_deref_mut().map(|p| p as &mut dyn Gravadora),
        FamiliaDaMacro::Motion => Some(f.motion as &mut dyn Gravadora),
    }
}

impl Sessao {
    /// Arma `fam`: o dono é a sprite que o Painter pinta, ou o objecto Motion em edição.
    ///
    /// # Errors
    /// A chave da frase.
    pub fn abre(
        fam: FamiliaDaMacro,
        f: &mut Familias<'_>,
        sim: &mut SimWorld,
        agora_us: u64,
    ) -> Result<Self, &'static str> {
        let (dono, referencia) = match fam {
            FamiliaDaMacro::Painter => {
                let p = f.painter.as_deref().ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                let (w, h) = p.source_size();
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "o lado de uma tela cabe num f32"
                )]
                let r = RefRect {
                    w: w as f32,
                    h: h as f32,
                };
                (p.bound_document().ok_or(m::MACRO_RECUSA_SEM_TELA)?, r)
            }
            FamiliaDaMacro::Motion => {
                let id = f.motion.objeto.editing.ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                let e = ph2d_ecs::entity_of_stable_id(sim.world_mut(), ph2d_ecs::StableId(id))
                    .ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                (e.to_bits(), RefRect { w: 1.0, h: 1.0 })
            }
        };
        let mut rec = MacroRecorder::new(referencia, agora_us);
        let g = gravadora(f, fam).ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
        let trilha = g.abre(&mut rec)?;
        Ok(Self {
            rec,
            trilha,
            familia: fam,
            dono,
        })
    }

    #[must_use]
    pub fn familia(&self) -> FamiliaDaMacro {
        self.familia
    }

    /// Os passos que a família gravou neste quadro, no instante `agora_us`.
    pub fn quadro(&mut self, f: &mut Familias<'_>, agora_us: u64) {
        let Some(g) = gravadora(f, self.familia) else {
            return; // o Painter deixou de ser a ferramenta activa: os passos esperam por ele
        };
        self.passos(g, agora_us);
    }

    /// Os passos de `g` (a família que grava), no instante `agora_us`.
    pub fn passos(&mut self, g: &mut dyn Gravadora, agora_us: u64) {
        for c in g.passos() {
            let _ = self.rec.push(self.trilha, agora_us, c);
        }
    }

    /// O objecto que fica com a `Macro`.
    #[must_use]
    pub fn dono(&self) -> u64 {
        self.dono
    }

    /// **Parar**: os últimos passos, e o objecto fica com a `Macro` e o clipe.
    ///
    /// # Errors
    /// A chave da frase (o clipe não coube na animação).
    pub fn para(
        mut self,
        f: &mut Familias<'_>,
        c: &mut Cena<'_>,
        agora_us: u64,
    ) -> Result<usize, &'static str> {
        if let Some(g) = gravadora(f, self.familia) {
            for p in g.para() {
                let _ = self.rec.push(self.trilha, agora_us, p);
            }
        }
        let doc = self.rec.finish();
        let modo = match self.familia {
            FamiliaDaMacro::Painter => ph2d_ecs::ModoDaMacro::Fiel,
            FamiliaDaMacro::Motion => ph2d_ecs::ModoDaMacro::Explicativo,
        };
        crate::instala::instala(c, self.dono, &doc, modo)
    }
}
