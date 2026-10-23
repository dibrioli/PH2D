//! **A gravação em curso**: o gravador, as trilhas abertas pelas famílias e o objecto que fica com a
//! `Macro` ao Parar.
//!
//! Duas armações (`docs/Macros/16_W18_plano.md` §2.2): **o objecto Macro** (W18) — a sessão arma-o vazio
//! e abre a trilha de uma família quando ela começa a trabalhar num objecto DENTRO dele (a imagem que o
//! Painter pinta, o Motion em edição), com a ficha de nascimento desse objecto; e **a família** (antes
//! da W18) — a sprite que o Painter pinta, ou o Motion em edição, é o próprio dono.

use crate::{Cena, Familias};
use ph2d_ecs::{Entity, SimWorld, StableId};
use ph2d_editor_core::macro_edits::{self as m, FamiliaDaMacro};
use ph2d_macro::{Gravadora, MacroRecorder, MacroTime, Nascimento, RefRect, TrackId};
use std::collections::BTreeMap;

/// O que o Gravar arma.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Armacao {
    /// O objecto Macro (bits): grava o que as famílias fazem dentro dele.
    Objeto(u64),
    /// Uma família sobre o seu próprio objecto.
    Familia(FamiliaDaMacro),
}

/// O que fazer com os passos de uma família neste quadro.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Passos {
    /// São desta gravação.
    Grava,
    /// Nada aberto: ficam na família.
    Espera,
    /// Foram feitos noutro objecto: deitam-se fora (a recusa diz porquê).
    Deita,
}

/// Uma trilha aberta: o objecto que ela grava (bits) e o id no gravador.
#[derive(Clone, Copy)]
struct Aberta {
    alvo: u64,
    id: TrackId,
}

pub struct Sessao {
    rec: MacroRecorder,
    /// Os bits do objecto que fica com a `Macro`.
    dono: u64,
    armacao: Armacao,
    /// Uma trilha por família (W18: uma imagem e um Motion por objecto Macro — o palco toca a 1.ª trilha
    /// de cada família).
    painter: Option<Aberta>,
    motion: Option<Aberta>,
    /// A recusa do quadro (a família trabalha num objecto que esta gravação não pode levar).
    recusa: Option<&'static str>,
    /// Os objectos que nasceram DENTRO do objecto Macro a gravar, e quando (µs do chamador).
    nascidos: BTreeMap<u64, u64>,
    armada_us: u64,
}

fn gravadora<'a>(f: &'a mut Familias<'_>, fam: FamiliaDaMacro) -> Option<&'a mut dyn Gravadora> {
    match fam {
        FamiliaDaMacro::Painter => f.painter.as_deref_mut().map(|p| p as &mut dyn Gravadora),
        FamiliaDaMacro::Motion => Some(&mut *f.motion),
    }
}

/// **O objecto Macro de `bits`**: ele mesmo ou o antepassado mais próximo com uma `Macro`. Uma sprite com
/// `Macro` (as gravações de antes da W18) é um objecto Macro que é a sua própria imagem.
#[must_use]
pub fn objeto_macro_de(sim: &SimWorld, bits: u64) -> Option<u64> {
    let w = sim.world();
    let mut e = Entity::try_from_bits(bits)?;
    loop {
        let ent = w.get_entity(e).ok()?;
        if ent.contains::<ph2d_ecs::Macro>() {
            return Some(e.to_bits());
        }
        e = ent.get::<ph2d_ecs::ChildOf>()?.0;
    }
}

/// `filho` é `pai` ou descende dele.
fn dentro_de(sim: &SimWorld, filho: u64, pai: u64) -> bool {
    let w = sim.world();
    let mut e = Entity::try_from_bits(filho);
    while let Some(x) = e {
        if x.to_bits() == pai {
            return true;
        }
        e = w.get::<ph2d_ecs::ChildOf>(x).map(|c| c.0);
    }
    false
}

/// A ficha de nascimento de `alvo` dentro de `dono` (filho DIRECTO; a transformada local é a dele).
fn ficha(sim: &SimWorld, alvo: u64, dono: u64, inicio_us: u64) -> Result<Nascimento, &'static str> {
    let w = sim.world();
    let e = Entity::from_bits(alvo);
    if w.get::<ph2d_ecs::ChildOf>(e).map(|c| c.0.to_bits()) != Some(dono) {
        return Err(m::MACRO_RECUSA_FUNDO_DEMAIS);
    }
    let t = w.get::<ph2d_ecs::Transform>(e).copied().unwrap_or_default();
    Ok(Nascimento {
        inicio: MacroTime(inicio_us),
        nome: w
            .get::<ph2d_ecs::Name>(e)
            .map(|n| n.0.clone())
            .unwrap_or_default(),
        local: [
            t.translation.x,
            t.translation.y,
            t.rotation,
            t.scale.x,
            t.scale.y,
            t.skew_x,
            t.skew_y,
        ],
        pai: None,
    })
}

impl Sessao {
    /// Arma `a`. O objecto Macro arma vazio; uma família abre já a trilha dela.
    ///
    /// # Errors
    /// A chave da frase.
    pub fn abre(
        a: Armacao,
        f: &mut Familias<'_>,
        sim: &mut SimWorld,
        agora_us: u64,
    ) -> Result<Self, &'static str> {
        let mut s = Self {
            rec: MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, agora_us),
            dono: 0,
            armacao: a,
            painter: None,
            motion: None,
            recusa: None,
            nascidos: BTreeMap::new(),
            armada_us: agora_us,
        };
        match a {
            Armacao::Objeto(bits) => s.dono = bits,
            Armacao::Familia(fam) => {
                let (dono, alvo) = match fam {
                    FamiliaDaMacro::Painter => {
                        let p = f.painter.as_deref().ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                        let b = p.bound_document().ok_or(m::MACRO_RECUSA_SEM_TELA)?;
                        (b, b)
                    }
                    FamiliaDaMacro::Motion => {
                        let id = f.motion_em_edicao.ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                        let e = ph2d_ecs::entity_of_stable_id(sim.world_mut(), StableId(id))
                            .ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
                        (e.to_bits(), e.to_bits())
                    }
                };
                s.dono = dono;
                s.abre_trilha(fam, alvo, f, sim, None)?;
            }
        }
        Ok(s)
    }

    /// Abre a trilha de `fam` sobre `alvo`; `ficha` = o objecto nasce dentro do objecto Macro.
    fn abre_trilha(
        &mut self,
        fam: FamiliaDaMacro,
        alvo: u64,
        f: &mut Familias<'_>,
        sim: &SimWorld,
        ficha: Option<Nascimento>,
    ) -> Result<(), &'static str> {
        let g = gravadora(f, fam).ok_or(m::MACRO_RECUSA_SEM_FAMILIA)?;
        let id = g.abre(&mut self.rec)?;
        if fam == FamiliaDaMacro::Painter
            && let Some(p) = f.painter.as_deref()
        {
            let (w, h) = p.source_size();
            #[expect(clippy::cast_precision_loss, reason = "o lado de uma tela cabe num f32")]
            let r = RefRect {
                w: w as f32,
                h: h as f32,
            };
            self.rec.doc_mut().reference = r;
        }
        if let Some(n) = ficha {
            let sid = sim
                .world()
                .get::<StableId>(Entity::from_bits(alvo))
                .map_or(0, |s| s.0);
            let t = &mut self.rec.doc_mut().tracks[id.0];
            t.target = sid;
            t.nasce = Some(n);
        }
        let aberta = Some(Aberta { alvo, id });
        match fam {
            FamiliaDaMacro::Painter => self.painter = aberta,
            FamiliaDaMacro::Motion => self.motion = aberta,
        }
        Ok(())
    }

    /// A família com que o painel diz «a gravar»: a da trilha aberta (o Painter primeiro).
    #[must_use]
    pub fn familia(&self) -> Option<FamiliaDaMacro> {
        if self.painter.is_some() {
            Some(FamiliaDaMacro::Painter)
        } else if self.motion.is_some() {
            Some(FamiliaDaMacro::Motion)
        } else {
            match self.armacao {
                Armacao::Familia(f) => Some(f),
                Armacao::Objeto(_) => None,
            }
        }
    }

    #[must_use]
    pub fn armacao(&self) -> Armacao {
        self.armacao
    }

    /// A recusa do último quadro (a família trabalha num objecto que esta gravação não leva).
    #[must_use]
    pub fn recusa(&self) -> Option<&'static str> {
        self.recusa
    }

    /// `true` = a família `fam` tem trilha aberta nesta gravação.
    #[must_use]
    pub fn grava(&self, fam: FamiliaDaMacro) -> bool {
        match fam {
            FamiliaDaMacro::Painter => self.painter.is_some(),
            FamiliaDaMacro::Motion => self.motion.is_some(),
        }
    }

    /// **Um objecto nasceu**: a gravar um objecto Macro, ele nasce DENTRO dele, na origem dele (a
    /// transformada local é a identidade: herda a posição, a rotação e a escala do objecto Macro), e o
    /// instante entra na ficha. Devolve `true` se o pendurou.
    pub fn nasceu(&mut self, sim: &mut SimWorld, bits: u64, agora_us: u64) -> bool {
        let Armacao::Objeto(dono) = self.armacao else {
            return false;
        };
        let (Some(c), Some(p)) = (Entity::try_from_bits(bits), Entity::try_from_bits(dono)) else {
            return false;
        };
        if bits == dono || sim.world().get_entity(p).is_err() {
            return false;
        }
        let Ok(mut e) = sim.world_mut().get_entity_mut(c) else {
            return false;
        };
        e.insert((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::ChildOf(p)));
        self.nascidos
            .insert(bits, agora_us.saturating_sub(self.armada_us));
        true
    }

    /// **O objecto em que `fam` trabalha agora** entra (ou não) nesta gravação: abre a trilha, ou recusa
    /// em voz alta. Devolve se os passos da família neste quadro são desta gravação.
    fn segue_o_alvo(
        &mut self,
        fam: FamiliaDaMacro,
        alvo: Option<u64>,
        f: &mut Familias<'_>,
        sim: &SimWorld,
    ) -> Passos {
        let aberta = match fam {
            FamiliaDaMacro::Painter => self.painter,
            FamiliaDaMacro::Motion => self.motion,
        };
        let Armacao::Objeto(dono) = self.armacao else {
            return if aberta.is_some() {
                Passos::Grava
            } else {
                Passos::Espera
            };
        };
        match (aberta, alvo) {
            (Some(_), None) => return Passos::Grava,
            (Some(a), Some(x)) if a.alvo == x => return Passos::Grava,
            (Some(_), Some(_)) => {
                self.recusa = Some(m::MACRO_RECUSA_UM_DE_CADA);
                return Passos::Deita;
            }
            (None, None) => return Passos::Espera,
            (None, Some(_)) => {}
        }
        let alvo = alvo.expect("acabou de casar");
        if !dentro_de(sim, alvo, dono) {
            self.recusa = Some(m::MACRO_RECUSA_FORA_DO_OBJETO);
            return Passos::Espera;
        }
        // O próprio objecto Macro (uma sprite com `Macro`): a trilha dele, sem ficha.
        let r = if alvo == dono {
            self.abre_trilha(fam, alvo, f, sim, None)
        } else {
            let inicio = self.nascidos.get(&alvo).copied().unwrap_or(0);
            ficha(sim, alvo, dono, inicio)
                .and_then(|n| self.abre_trilha(fam, alvo, f, sim, Some(n)))
        };
        self.recusa = r.err();
        if self.recusa.is_none() {
            Passos::Grava
        } else {
            Passos::Espera
        }
    }

    /// **O quadro**, depois de o Painter consumir a entrada: as trilhas que abrem (o Painter e o Motion,
    /// cujos passos vêm em [`Self::passos_do_motion`]) e os passos do Painter, no instante `agora_us`.
    pub fn quadro(&mut self, f: &mut Familias<'_>, sim: &mut SimWorld, agora_us: u64) {
        let motion = f
            .motion_em_edicao
            .and_then(|id| ph2d_ecs::entity_of_stable_id(sim.world_mut(), StableId(id)))
            .map(Entity::to_bits);
        if matches!(self.armacao, Armacao::Objeto(_)) {
            let _ = self.segue_o_alvo(FamiliaDaMacro::Motion, motion, f, sim);
        }
        let alvo = f
            .painter
            .as_deref()
            .filter(|_| f.painter_activo)
            .and_then(ph2d_tool_painter::PainterTool::bound_document);
        let destino = self.segue_o_alvo(FamiliaDaMacro::Painter, alvo, f, sim);
        // A gravadora do Painter chega sempre (o Parar alcança-a depois de trocar de ferramenta).
        let Some(g) = gravadora(f, FamiliaDaMacro::Painter) else {
            return;
        };
        match (destino, self.painter) {
            (Passos::Grava, Some(a)) => {
                for c in g.passos() {
                    let _ = self.rec.push(a.id, agora_us, c);
                }
            }
            (Passos::Deita, _) => drop(g.passos()),
            _ => {}
        }
    }

    /// **Os passos do Motion** neste quadro, depois de ele correr, no instante `agora_us`.
    pub fn passos_do_motion(&mut self, g: &mut dyn Gravadora, agora_us: u64) {
        if let Some(a) = self.motion {
            for c in g.passos() {
                let _ = self.rec.push(a.id, agora_us, c);
            }
        }
    }

    /// O objecto que fica com a `Macro`.
    #[must_use]
    pub fn dono(&self) -> u64 {
        self.dono
    }

    /// A imagem que o Painter grava (o palco dela cede ao traço vivo do editor).
    #[must_use]
    pub fn sprite_gravada(&self) -> Option<u64> {
        self.painter.map(|a| a.alvo)
    }

    /// **Parar**: os últimos passos, e o objecto fica com a `Macro` e o clipe.
    ///
    /// # Errors
    /// A chave da frase (nada gravado; o clipe não coube na animação).
    pub fn para(
        mut self,
        f: &mut Familias<'_>,
        c: &mut Cena<'_>,
        agora_us: u64,
    ) -> Result<usize, &'static str> {
        for (fam, a) in [
            (FamiliaDaMacro::Painter, self.painter),
            (FamiliaDaMacro::Motion, self.motion),
        ] {
            let (Some(a), Some(g)) = (a, gravadora(f, fam)) else {
                continue;
            };
            for p in g.para() {
                let _ = self.rec.push(a.id, agora_us, p);
            }
        }
        let doc = self.rec.finish();
        if doc.tracks.is_empty() {
            return Err(m::MACRO_RECUSA_NADA_GRAVADO);
        }
        let modo = if doc
            .tracks
            .iter()
            .any(|t| t.family == ph2d_tool_painter::macro_track::FAMILIA)
        {
            ph2d_ecs::ModoDaMacro::Fiel
        } else {
            ph2d_ecs::ModoDaMacro::Explicativo
        };
        crate::instala::instala(c, f.etapas, self.dono, &doc, modo)
    }
}
