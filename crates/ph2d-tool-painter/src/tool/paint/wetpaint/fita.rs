//! **A FITA do worker** (filho de [`super`]) — o que torna a água de uma macro idêntica à gravada.
//!
//! A única entrada NÃO determinística da água é o worker ([`super::offthread`]): quantos estágios ele
//! correu quando o motor volta, se um pedido sem espera o encontra, o que o contador de passos diz.
//! Todo o resto (os dabs, os knobs, o composite) é função do que a thread do quadro faz. A fita grava
//! cada resposta dele, pela ORDEM em que a thread do quadro as pede; o tocador responde pela fita e
//! anda o motor local até a [`MarcaDaAgua`] gravada. Nenhuma lei da pintura muda para gravar.
//!
//! Desenho e recusas: ADR-0186 §Painter; `docs/Macros/01_plano_onda1.md` §2.

use ph2d_wet_paint::painter::Engine;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// O ESTADO da água entre dois estágios: o passo (`Sim::frame`) e o próximo estágio do passo em voo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarcaDaAgua {
    pub frame: u64,
    pub estagio: Option<u8>,
}

impl MarcaDaAgua {
    #[must_use]
    pub fn de(e: &Engine) -> Self {
        Self {
            frame: e.sim.frame,
            estagio: e.sim.stage_in_flight(),
        }
    }

    /// A ordem dos estados: o passo `f` em voo vem depois do `f − 1` fechado e antes do `f` fechado.
    fn ordem(self) -> (u64, u16) {
        (self.frame, self.estagio.map_or(u16::MAX, u16::from))
    }
}

/// Uma resposta do worker, como a thread do quadro a recebeu.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RespostaDoWorker {
    /// O motor voltou para casa neste estado.
    Voltou(MarcaDaAgua),
    /// O pedido sem espera (o tique, o ponteiro com o traço aberto) não o encontrou.
    NaoVoltou,
    /// O contador de passos do worker, lido pelo tique.
    Passos(u64),
}

/// A pergunta que a thread do quadro fez — o tocador confere-a com a resposta gravada.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Pergunta {
    Volta,
    TentaVoltar,
    Passos,
}

#[derive(Clone, Debug)]
pub enum Fita {
    /// A gravar: as respostas por ordem.
    Grava(Vec<RespostaDoWorker>),
    /// A tocar: as respostas que faltam, e a primeira divergência.
    Toca {
        fila: VecDeque<RespostaDoWorker>,
        divergiu: Option<Divergencia>,
    },
}

/// A reprodução deixou de ser a gravação — nunca em silêncio: o tocador lê-a depois de cada passo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Divergencia {
    /// A thread do quadro perguntou algo que a gravação não respondeu ali.
    PerguntaInesperada,
    /// A fita acabou antes das perguntas.
    FitaAcabou,
    /// O motor local não chega à marca gravada (passou dela ou parou antes).
    MarcaInalcancavel(MarcaDaAgua),
}

pub type FitaPartilhada = Arc<Mutex<Fita>>;

/// A fita para ler ou escrever. Um pânico com ela travada não a estraga: as respostas são valores.
pub(crate) fn trava(f: &FitaPartilhada) -> MutexGuard<'_, Fita> {
    f.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Fita {
    pub(crate) fn grava(&mut self, r: RespostaDoWorker) {
        if let Fita::Grava(v) = self {
            v.push(r);
        }
    }

    /// A próxima resposta gravada para `p`. `None` = divergiu (registado).
    pub(crate) fn responde(&mut self, p: Pergunta) -> Option<RespostaDoWorker> {
        let Fita::Toca { fila, divergiu } = self else {
            return None;
        };
        if divergiu.is_some() {
            return None;
        }
        let Some(r) = fila.pop_front() else {
            *divergiu = Some(Divergencia::FitaAcabou);
            return None;
        };
        let cabe = matches!(
            (p, r),
            (Pergunta::Volta, RespostaDoWorker::Voltou(_))
                | (
                    Pergunta::TentaVoltar,
                    RespostaDoWorker::Voltou(_) | RespostaDoWorker::NaoVoltou
                )
                | (Pergunta::Passos, RespostaDoWorker::Passos(_))
        );
        if cabe {
            Some(r)
        } else {
            *divergiu = Some(Divergencia::PerguntaInesperada);
            None
        }
    }

    pub(crate) fn diverge(&mut self, d: Divergencia) {
        if let Fita::Toca { divergiu, .. } = self
            && divergiu.is_none()
        {
            *divergiu = Some(d);
        }
    }

    pub(crate) fn tocando(&self) -> bool {
        matches!(self, Fita::Toca { .. })
    }
}

/// O trabalho do worker, refeito na thread de quem toca: estágios até a marca. `false` = inalcançável
/// (o motor já passou dela, ou um estágio deixou de avançar — a água secou antes da marca).
pub(crate) fn anda_ate(e: &mut Engine, alvo: MarcaDaAgua) -> bool {
    loop {
        let agora = MarcaDaAgua::de(e);
        if agora == alvo {
            return true;
        }
        if agora.ordem() > alvo.ordem() {
            return false;
        }
        e.step_stage();
        if MarcaDaAgua::de(e) == agora {
            return false;
        }
    }
}
