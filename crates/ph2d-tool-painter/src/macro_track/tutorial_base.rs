//! **A base de um critério** do tutorial do Painter (W6d, `docs/Macros/03_plano_W6.md` §6.3): a
//! trilha que toca SEM a pincelada, medida no instante da medida da gravada.

use super::{EnsinoDoPainter, toca};
use crate::macro_track::etapas::{self, LeitorDoPainter};
use crate::macro_track::tocar::{ErroDaMacro, TocadorDoPainter, respostas_de};
use ph2d_macro::{LeitorDaFamilia, Track};
use std::sync::atomic::{AtomicBool, Ordering};

/// **A trilha que toca sem a pincelada `k`**: apagada pela edição do produto, e o tempo dela devolvido
/// à etapa viva seguinte — o toque seguinte cai no mesmo instante que na trilha que toca.
fn sem_a_pincelada(track: &Track, k: usize) -> Result<Track, ErroDaMacro> {
    let erro = |e| match e {
        etapas::ErroDeEdicaoDoPainter::Macro(x) => x,
        _ => ErroDaMacro::PortaRecusada(format!("etapa {k}")),
    };
    // O estado inicial não muda: não se copia.
    let mut t = Track {
        family: track.family.clone(),
        target: track.target,
        initial: Vec::new(),
        events: track.events.clone(),
        edicoes: track.edicoes.clone(),
    };
    let gravadas = LeitorDoPainter.etapas(&track.events);
    let e = gravadas[k];
    let dur = track.events[e.fim].t.0 - track.events[e.inicio].t.0;
    etapas::apaga(&mut t, k).map_err(erro)?;
    let viva = |g: &ph2d_macro::Etapa| {
        !t.edicoes
            .iter()
            .any(|ed| ed.ancora == g.inicio && ed.apagada)
    };
    if let Some(s) = (k + 1..gravadas.len()).find(|&s| viva(&gravadas[s])) {
        let ja = t
            .edicoes
            .iter()
            .find(|ed| ed.ancora == gravadas[s].inicio)
            .map_or(0, |ed| ed.espera_us);
        let us = ja.saturating_add(i64::try_from(dur).unwrap_or(i64::MAX));
        etapas::espera(&mut t, s, us).map_err(erro)?;
    }
    t.tocavel(&LeitorDoPainter).map_err(ErroDaMacro::Edicao)
}

/// A base de um critério por medir: os eventos da trilha sem a pincelada desde o 1.º que difere da
/// trilha que toca até ao instante da medida.
pub(super) struct SemAPincelada {
    k: usize,
    pub(super) c: usize,
    /// O 1.º evento em que as duas trilhas diferem (o mesmo índice nas duas).
    pub(super) desde: usize,
    /// Os eventos da trilha sem a pincelada `desde..` o instante da medida.
    eventos: Vec<Vec<u8>>,
    /// As respostas do worker da trilha sem a pincelada (a fita do instantâneo passa a ser esta).
    respostas: std::sync::Arc<[crate::tool::paint::fita::RespostaDoWorker]>,
}

impl SemAPincelada {
    pub(super) fn de(
        track: &Track,
        tocavel: &Track,
        k: usize,
        c: usize,
        fronteira: usize,
    ) -> Result<Self, ErroDaMacro> {
        let sem = sem_a_pincelada(track, k)?;
        let desde = tocavel
            .events
            .iter()
            .zip(&sem.events)
            .position(|(a, b)| a != b)
            .unwrap_or(tocavel.events.len().min(sem.events.len()))
            .min(fronteira);
        // O toque seguinte na trilha sem ela: as etapas são as mesmas menos a dela, antes dele.
        let ate = if fronteira < tocavel.events.len() {
            let st = LeitorDoPainter.etapas(&tocavel.events);
            let ss = LeitorDoPainter.etapas(&sem.events);
            st.iter()
                .position(|e| e.inicio == fronteira)
                .filter(|&s| s > 0 && ss.len() + 1 == st.len())
                .map(|s| ss[s - 1].inicio)
                .ok_or_else(|| ErroDaMacro::PortaRecusada(format!("etapa {k}")))?
        } else {
            sem.events.len()
        }
        .max(desde);
        let respostas = respostas_de(sem.events.iter().map(|e| e.payload.as_slice()))?;
        Ok(Self {
            k,
            c,
            desde,
            eventos: sem.events[desde..ate]
                .iter()
                .map(|e| e.payload.clone())
                .collect(),
            respostas: respostas.into(),
        })
    }

    /// A fracção dos `novos` que a trilha sem a pincelada pinta, a partir de `toc` (a reprodução da
    /// trilha que toca parada em [`Self::desde`]). `None` = `parar`.
    pub(super) fn mede(
        &self,
        e: &EnsinoDoPainter,
        track: &Track,
        toc: &TocadorDoPainter,
        parar: &AtomicBool,
    ) -> Result<Option<f32>, ErroDaMacro> {
        let mut b = match toc.recomeca(&self.respostas) {
            Some(b) => b,
            // O instantâneo não é um estado da trilha sem ela (a fita já consumiu uma resposta da
            // pincelada): do estado inicial.
            None => {
                let sem = sem_a_pincelada(track, self.k)?;
                let mut b = TocadorDoPainter::novo(
                    &track.initial,
                    sem.events.iter().map(|e| e.payload.as_slice()),
                )?;
                for ev in &sem.events[..self.desde] {
                    if parar.load(Ordering::Relaxed) {
                        return Ok(None);
                    }
                    toca(&mut b, &ev.payload)?;
                }
                b
            }
        };
        for ev in &self.eventos {
            if parar.load(Ordering::Relaxed) {
                return Ok(None);
            }
            toca(&mut b, ev)?;
        }
        Ok(Some(e.fracao_pintada(b.tool(), &e.criterios[self.c])))
    }
}
