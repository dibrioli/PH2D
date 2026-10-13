//! O gravador: o ÚNICO escritor de uma gravação. O tempo vem do chamador.

use crate::doc::{Event, MacroDoc, MacroTime, RefRect, Track};

/// Índice de uma trilha aberta neste gravador.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TrackId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// O instante é anterior ao início da gravação ou ao último passo da trilha.
    TimeWentBackwards {
        track: usize,
    },
    UnknownTrack {
        track: usize,
    },
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for RecordError {}

pub struct MacroRecorder {
    doc: MacroDoc,
    armed_at_us: u64,
}

impl MacroRecorder {
    /// `armed_at_us` é o instante (relógio do chamador) que vira o tempo zero da macro.
    #[must_use]
    pub fn new(reference: RefRect, armed_at_us: u64) -> Self {
        MacroRecorder {
            doc: MacroDoc::new(reference),
            armed_at_us,
        }
    }

    pub fn open_track(&mut self, family: &str, target: u64, initial: Vec<u8>) -> TrackId {
        self.doc.tracks.push(Track {
            family: family.to_owned(),
            target,
            initial,
            events: Vec::new(),
        });
        TrackId(self.doc.tracks.len() - 1)
    }

    /// Grava um passo em `now_us` (o mesmo relógio de `armed_at_us`).
    pub fn push(
        &mut self,
        track: TrackId,
        now_us: u64,
        payload: Vec<u8>,
    ) -> Result<(), RecordError> {
        let tr = self
            .doc
            .tracks
            .get_mut(track.0)
            .ok_or(RecordError::UnknownTrack { track: track.0 })?;
        let t = now_us
            .checked_sub(self.armed_at_us)
            .map(MacroTime)
            .ok_or(RecordError::TimeWentBackwards { track: track.0 })?;
        if tr.events.last().is_some_and(|last| t < last.t) {
            return Err(RecordError::TimeWentBackwards { track: track.0 });
        }
        tr.events.push(Event { t, payload });
        Ok(())
    }

    #[must_use]
    pub fn doc(&self) -> &MacroDoc {
        &self.doc
    }

    #[must_use]
    pub fn finish(self) -> MacroDoc {
        self.doc
    }
}
