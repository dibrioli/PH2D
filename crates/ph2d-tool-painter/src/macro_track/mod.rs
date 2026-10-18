//! **A trilha do Painter numa macro** (ADR-0186; `docs/Macros/01_plano_onda1.md` §2).
//!
//! - **Gravar** = cada PORTA externa do [`PainterTool`] ([`portas::PORTAS`]) regista o passo que
//!   recebeu ([`passo::Passo`]), e a fita da água ([`crate::tool::paint::fita`]) regista cada resposta
//!   do worker. Só a porta de FORA grava: uma porta chamada por dentro de outra é parte dela.
//! - **Tocar** = um `PainterTool` PRÓPRIO ([`tocar`]) parte do [`inicial`] e recebe os mesmos passos
//!   pelas mesmas portas; a fita responde pelo worker. Nenhuma lei da pintura muda para gravar.
//!
//! Escopo (W6c): os quatro meios. O estado inicial leva o meio e o pincel inteiro de cada um, e trocar
//! de meio a gravar é um passo do painel como outro qualquer.

pub mod em_cena;
pub mod etapas;
pub(crate) mod forca;
pub mod inicial;
pub mod instantaneos;
pub mod marca;
pub mod passo;
pub mod portas;
mod portas_internas;
pub mod roteiro;
pub mod tocar;
pub mod tutorial;

#[cfg(test)]
mod cauda_tests;
#[cfg(test)]
mod composicao_tests;
#[cfg(test)]
mod custo_w5e_tests;
#[cfg(test)]
mod etapas_tests;
#[cfg(test)]
mod fim_tests;
#[cfg(test)]
mod marca_cobertura_tests;
#[cfg(test)]
mod marca_painter_tests;
#[cfg(test)]
mod pressao_tests;
#[cfg(test)]
mod sobrepor_fecho_tests;
#[cfg(test)]
mod sobrepor_tests;
#[cfg(test)]
mod sonda_w12_tests;
#[cfg(test)]
mod sonda_w14_tests;
#[cfg(test)]
mod tests;

use crate::tool::PainterTool;
use crate::tool::paint::fita::{self, Fita, FitaPartilhada};
pub use crate::tool::paint::fita::{Divergencia, MarcaDaAgua, RespostaDoWorker, RitmoDaAgua};
pub use passo::Passo;
use std::sync::{Arc, Mutex};

/// A chave da família na [`ph2d_macro::Track`].
pub const FAMILIA: &str = "painter";

/// Porque armar a gravação não começou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recusa {
    /// Já se está a gravar.
    JaGrava,
    /// Sem tela.
    SemTela,
}

/// A gravação em curso, dentro do tool.
pub struct Gravacao {
    passos: Vec<Passo>,
    /// Profundidade de portas abertas: só a de fora (`1`) grava.
    fundo: u32,
    fita: FitaPartilhada,
    /// O quadro de referência (o tamanho da tela ao armar).
    pub(crate) referencia: (f64, f64),
}

impl PainterTool {
    /// **Arma a gravação**: devolve o estado inicial ([`inicial`]) e o quadro de referência.
    ///
    /// # Errors
    /// [`Recusa`] — o porquê vai ao artista.
    pub fn macro_grava(&mut self) -> Result<(Vec<u8>, (u32, u32)), Recusa> {
        if self.macro_gravacao.is_some() {
            return Err(Recusa::JaGrava);
        }
        let (w, h) = self.source_size();
        if w == 0 || h == 0 {
            return Err(Recusa::SemTela);
        }
        // A água é estado de TELA, não de documento (`tool/paint/wetpaint.rs`): armar encerra a sessão
        // viva pela porta de sempre (o fim da sessão É o bake), como o desfazer e a troca de camada
        // fazem — e o estado inicial é o documento.
        self.wetpaint_end_session();
        // A irmã da aguada: a sessão molhada (o «Wet now», a lavagem ainda viva) seca pela porta de
        // sempre, a mesma do desfazer (`dry_session_now`) — os pixels ficam, a humidade não viaja.
        self.dry_session_now();
        let estado = inicial::captura(self);
        let f: FitaPartilhada = Arc::new(Mutex::new(Fita::Grava(Vec::new())));
        self.wet_instala_fita(Some(Arc::clone(&f)));
        self.macro_gravacao = Some(Box::new(Gravacao {
            passos: Vec::new(),
            fundo: 0,
            fita: f,
            referencia: (f64::from(w), f64::from(h)),
        }));
        Ok((estado, (w, h)))
    }

    /// **Abre a trilha do Painter** em `rec`: arma a gravação ([`Self::macro_grava`]) e abre a `Track`
    /// com o `StableId` da sprite pintada (`alvo`) — irmã do `MotionState::gravacao_abre`.
    ///
    /// # Errors
    /// [`Recusa`].
    pub fn macro_abre(
        &mut self,
        rec: &mut ph2d_macro::MacroRecorder,
        alvo: u64,
    ) -> Result<ph2d_macro::TrackId, Recusa> {
        let (inicial, _) = self.macro_grava()?;
        Ok(rec.open_track(FAMILIA, alvo, inicial))
    }

    /// Está a gravar?
    #[must_use]
    pub fn macro_gravando(&self) -> bool {
        self.macro_gravacao.is_some()
    }

    /// **Os passos desde a última chamada**, codificados — as portas e, depois, as respostas do
    /// worker. Quem chama dá-lhes o tempo do quadro (`MacroRecorder::push`).
    pub fn macro_passos(&mut self) -> Vec<Vec<u8>> {
        let Some(g) = self.macro_gravacao.as_mut() else {
            return Vec::new();
        };
        let mut out: Vec<Vec<u8>> = g.passos.drain(..).map(|p| p.codifica()).collect();
        if let Fita::Grava(v) = &mut *fita::trava(&g.fita) {
            out.extend(v.drain(..).map(|r| Passo::Agua(r).codifica()));
        }
        out
    }

    /// **Pára a gravação** e devolve os últimos passos.
    pub fn macro_para(&mut self) -> Vec<Vec<u8>> {
        if let Some(g) = self.macro_gravacao.as_mut() {
            g.passos.push(Passo::Para);
        }
        self.wet_traz_para_casa();
        let resto = self.macro_passos();
        self.macro_gravacao = None;
        self.wet_instala_fita(None);
        resto
    }

    /// A porta abre-se: grava `passo` se é a de fora. Devolve `true` se abriu um nível (fechar com
    /// [`Self::macro_sai`]).
    pub(crate) fn macro_entra(&mut self, passo: impl FnOnce(&Self) -> Passo) -> bool {
        let Some(fundo) = self.macro_gravacao.as_ref().map(|g| g.fundo) else {
            return false;
        };
        let p = (fundo == 0).then(|| passo(self));
        let g = self.macro_gravacao.as_mut().expect("checado acima");
        g.fundo += 1;
        if let Some(p) = p {
            g.passos.push(p);
        }
        true
    }

    /// A porta fecha-se.
    pub(crate) fn macro_sai(&mut self) {
        if let Some(g) = self.macro_gravacao.as_mut() {
            g.fundo = g.fundo.saturating_sub(1);
        }
    }

    /// Uma porta que a macro não reproduz foi chamada de FORA a gravar: fica na gravação, e o tocador
    /// recusa-a em voz alta ([`tocar::ErroDaMacro::PortaRecusada`]).
    pub(crate) fn macro_recusa(&mut self, porta: &'static str) {
        if let Some(g) = self.macro_gravacao.as_mut()
            && g.fundo == 0
        {
            g.passos.push(Passo::Recusada(porta.to_owned()));
        }
    }

    /// O quadro de referência da gravação em curso (a tela ao armar); sem gravação, a tela de agora.
    pub(crate) fn macro_referencia(&self) -> (f64, f64) {
        self.macro_gravacao.as_ref().map_or_else(
            || {
                let (w, h) = self.source_size();
                (f64::from(w), f64::from(h))
            },
            |g| g.referencia,
        )
    }
}

/// Uma porta GRAVADA: `porta!(self, |t| Passo::…, { corpo })`. O corpo corre numa closure, então um
/// `return` dentro dele é o retorno da porta, e a profundidade fecha em todos os caminhos.
macro_rules! porta {
    ($self:ident, $passo:expr, $corpo:block) => {{
        let aberta = $self.macro_entra($passo);
        #[allow(clippy::redundant_closure_call)]
        let r = (|| $corpo)();
        if aberta {
            $self.macro_sai();
        }
        r
    }};
}
pub(crate) use porta;

/// O painel MACROS arma o Painter por esta porta (W5d) — as três decididas no censo ([`portas::PORTAS`]).
/// O alvo é `0`: a `Macro` fica na sprite pintada.
impl ph2d_macro::Gravadora for PainterTool {
    fn abre(
        &mut self,
        rec: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, &'static str> {
        use ph2d_editor_core::macro_edits as m;
        self.macro_abre(rec, 0).map_err(|r| match r {
            Recusa::JaGrava => m::MACRO_RECUSA_JA_GRAVA,
            Recusa::SemTela => m::MACRO_RECUSA_SEM_TELA,
        })
    }

    fn passos(&mut self) -> Vec<Vec<u8>> {
        self.macro_passos()
    }

    fn para(&mut self) -> Vec<Vec<u8>> {
        self.macro_para()
    }
}
