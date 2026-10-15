//! O tocador: despacha os passos de TODAS as trilhas por ordem de tempo.
//!
//! O tempo pedido vem do chamador (o playhead × a velocidade); a ordem entre trilhas é
//! global e determinística — empate no mesmo instante: a trilha de índice menor primeiro.
//! Uma falha do despacho PÁRA no passo que falhou (o cursor não o salta).

use crate::doc::{Event, MacroDoc, MacroTime, Track};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MacroPlayer {
    /// Próximo evento por trilha.
    cursors: Vec<usize>,
    time: MacroTime,
}

impl MacroPlayer {
    #[must_use]
    pub fn new(doc: &MacroDoc) -> Self {
        MacroPlayer {
            cursors: vec![0; doc.tracks.len()],
            time: MacroTime::ZERO,
        }
    }

    /// O tempo até onde já se despachou.
    #[must_use]
    pub fn time(&self) -> MacroTime {
        self.time
    }

    /// Quantos passos já se despacharam, somando as trilhas.
    #[must_use]
    pub fn dispatched(&self) -> usize {
        self.cursors.iter().sum()
    }

    #[must_use]
    pub fn is_finished(&self, doc: &MacroDoc) -> bool {
        self.cursors
            .iter()
            .zip(&doc.tracks)
            .all(|(&c, tr)| c >= tr.events.len())
    }

    /// Despacha, por ordem global, todo passo com `t ≤ to` ainda não despachado.
    /// Devolve quantos foram despachados nesta chamada.
    pub fn advance_to<E>(
        &mut self,
        doc: &MacroDoc,
        to: MacroTime,
        mut apply: impl FnMut(usize, &Track, &Event) -> Result<(), E>,
    ) -> Result<usize, E> {
        self.cursors.resize(doc.tracks.len(), 0);
        let mut n = 0;
        while let Some(k) = self.next_due(doc, to) {
            let tr = &doc.tracks[k];
            apply(k, tr, &tr.events[self.cursors[k]])?;
            self.cursors[k] += 1;
            n += 1;
        }
        self.time = self.time.max(to);
        Ok(n)
    }

    /// Como [`Self::advance_to`], mas pára quando [`Self::dispatched`] chega a `limite` — o instante fica no
    /// último passo despachado (≤ `to`), e um `advance_to` seguinte continua dali. É onde a família tira um
    /// instantâneo, entre dois passos de um mesmo quadro se for preciso.
    pub fn advance_to_limit<E>(
        &mut self,
        doc: &MacroDoc,
        to: MacroTime,
        limite: usize,
        mut apply: impl FnMut(usize, &Track, &Event) -> Result<(), E>,
    ) -> Result<usize, E> {
        self.cursors.resize(doc.tracks.len(), 0);
        let mut n = 0;
        while self.dispatched() < limite
            && let Some(k) = self.next_due(doc, to)
        {
            let tr = &doc.tracks[k];
            let ev = &tr.events[self.cursors[k]];
            apply(k, tr, ev)?;
            self.cursors[k] += 1;
            self.time = self.time.max(ev.t);
            n += 1;
        }
        if self.dispatched() < limite {
            self.time = self.time.max(to);
        }
        Ok(n)
    }

    /// Volta ao início (`reset` restaura o estado inicial de cada trilha) e avança até `to`.
    /// É o scrub para trás; os instantâneos que o encurtam são da família.
    pub fn seek<E>(
        &mut self,
        doc: &MacroDoc,
        to: MacroTime,
        mut reset: impl FnMut(usize, &Track) -> Result<(), E>,
        apply: impl FnMut(usize, &Track, &Event) -> Result<(), E>,
    ) -> Result<usize, E> {
        for (k, tr) in doc.tracks.iter().enumerate() {
            reset(k, tr)?;
        }
        *self = MacroPlayer::new(doc);
        self.advance_to(doc, to, apply)
    }

    fn next_due(&self, doc: &MacroDoc, to: MacroTime) -> Option<usize> {
        let mut best: Option<(MacroTime, usize)> = None;
        for (k, tr) in doc.tracks.iter().enumerate() {
            let Some(ev) = tr.events.get(self.cursors[k]) else {
                continue;
            };
            if ev.t > to {
                continue;
            }
            if best.is_none_or(|(t, _)| ev.t < t) {
                best = Some((ev.t, k));
            }
        }
        best.map(|(_, k)| k)
    }
}
