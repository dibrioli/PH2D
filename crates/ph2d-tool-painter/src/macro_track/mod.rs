//! **A trilha do Painter numa macro** (ADR-0186; `docs/Macros/01_plano_onda1.md` §2).
//!
//! - **Gravar** = cada PORTA externa do [`PainterTool`] ([`portas::PORTAS`]) regista o passo que
//!   recebeu ([`passo::Passo`]), e a fita da água ([`crate::tool::paint::fita`]) regista cada resposta
//!   do worker. Só a porta de FORA grava: uma porta chamada por dentro de outra é parte dela.
//! - **Tocar** = um `PainterTool` PRÓPRIO ([`tocar`]) parte do [`inicial`] e recebe os mesmos passos
//!   pelas mesmas portas; a fita responde pelo worker. Nenhuma lei da pintura muda para gravar.
//!
//! Escopo (W2): o meio **Wet Paint**. Armar noutro meio recusa-se ([`Recusa::Meio`]); o estado inicial
//! dos outros meios não viaja ainda.

pub mod em_cena;
pub mod etapas;
pub mod inicial;
pub mod instantaneos;
pub mod passo;
pub mod portas;
mod portas_internas;
pub mod roteiro;
pub mod tocar;

#[cfg(test)]
mod etapas_tests;
#[cfg(test)]
mod tests;

use crate::tool::PainterTool;
use crate::tool::paint::fita::{self, Fita, FitaPartilhada};
pub use crate::tool::paint::fita::{Divergencia, MarcaDaAgua, RespostaDoWorker};
pub use passo::Passo;
use std::sync::{Arc, Mutex};

/// A chave da família na [`ph2d_macro::Track`].
pub const FAMILIA: &str = "painter";

/// Porque armar a gravação não começou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Recusa {
    /// Só o Wet Paint grava nesta versão.
    Meio,
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
    /// O meio já saiu do Wet Paint (a recusa ficou gravada).
    saiu_do_meio: bool,
}

/// O nome da recusa quando o meio sai do Wet Paint a gravar.
pub const PORTA_DO_MEIO: &str = "o meio saiu do Wet Paint";

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
        if !self.wetpaint_armed() {
            return Err(Recusa::Meio);
        }
        // A água é estado de TELA, não de documento (`tool/paint/wetpaint.rs`): armar encerra a sessão
        // viva pela porta de sempre (o fim da sessão É o bake), como o desfazer e a troca de camada
        // fazem — e o estado inicial é o documento.
        self.wetpaint_end_session();
        let estado = inicial::captura(self);
        let f: FitaPartilhada = Arc::new(Mutex::new(Fita::Grava(Vec::new())));
        self.wet_instala_fita(Some(Arc::clone(&f)));
        self.macro_gravacao = Some(Box::new(Gravacao {
            passos: Vec::new(),
            fundo: 0,
            fita: f,
            referencia: (f64::from(w), f64::from(h)),
            saiu_do_meio: false,
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

    /// A porta fecha-se. Ao fechar a de FORA, se o meio deixou de ser o Wet Paint a gravação marca a
    /// recusa (uma vez): o pincel dos outros meios não está no estado inicial, e o tocador pintaria outra
    /// coisa EM SILÊNCIO (medido: ~1 750 texels a 128²).
    pub(crate) fn macro_sai(&mut self) {
        let armado = self.wetpaint_armed();
        if let Some(g) = self.macro_gravacao.as_mut() {
            g.fundo = g.fundo.saturating_sub(1);
            if g.fundo == 0 && !armado && !g.saiu_do_meio {
                g.saiu_do_meio = true;
                g.passos.push(Passo::Recusada(PORTA_DO_MEIO.to_owned()));
            }
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
