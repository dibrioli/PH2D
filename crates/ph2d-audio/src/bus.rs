//! [`BusId`] — mixer routing targets. Sub-buses sum into the Master bus.
//!
//! A minimal, game-idiomatic bus tree (Unity's AudioMixer groups / Godot's
//! audio buses): every voice routes to a **sub-bus** (or straight to Master),
//! each sub-bus has its own fader + meter, and all sub-buses sum into the
//! single Master bus that carries the master gain + low-pass filter.
//!
//! The set is fixed and small so the audio thread indexes a `[_; SUB_BUS_COUNT]`
//! array with no allocation and no hashing. Adding a bus is a one-line enum edit
//! plus its slot in [`BusId::SUB_BUSES`]; the mixer, meter, and scratch size off
//! [`SUB_BUS_COUNT`] automatically.

/// Where a voice's output is summed. [`BusId::Master`] renders straight into the
/// master mix; the sub-buses pass through their own fader first.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum BusId {
    /// The final mix bus (master gain + filter). Voices routed here bypass any
    /// sub-bus fader.
    #[default]
    Master,
    /// Music sub-bus.
    Music,
    /// Sound-effects sub-bus.
    Sfx,
    /// UI-sound sub-bus (clicks, notifications).
    Ui,
    /// Voice / dialogue sub-bus.
    Voice,
}

/// Number of sub-buses (everything except [`BusId::Master`]). The mixer's bus
/// strips, the meter's per-bus peaks, and the UI's strip count all size off this.
pub const SUB_BUS_COUNT: usize = 4;

impl BusId {
    /// The sub-buses in canonical order — the index each maps to in the mixer's
    /// strip array, the meter's per-bus peaks, and the UI's strips. The shell's
    /// mixer bridge iterates this so the panel's per-bus channels stay aligned.
    pub const SUB_BUSES: [BusId; SUB_BUS_COUNT] =
        [BusId::Music, BusId::Sfx, BusId::Ui, BusId::Voice];

    /// This bus's index into the sub-bus arrays, or `None` for [`BusId::Master`]
    /// (which has no sub-bus strip — it *is* the master mix).
    pub(crate) fn sub_index(self) -> Option<usize> {
        match self {
            BusId::Master => None,
            BusId::Music => Some(0),
            BusId::Sfx => Some(1),
            BusId::Ui => Some(2),
            BusId::Voice => Some(3),
        }
    }

    // ⛔⛔ **Aqui viveu um `label()` que prometia por escrito uma amarra INEXISTENTE** (apagado em
    //    2026-09-19). Ele dizia-se *«human-readable label, so the UI names strips from the same
    //    source of truth as the routing (no drift between the enum and the panel text)»* — e a
    //    varredura da workspace inteira não achou **um único chamador**. Quem nomeia as strips é o
    //    `SUB_BUS_LABELS` da `ph2d-panel-audio-mixer`, um array de CHAVES escrito à mão, e o painel
    //    declara no doc dele que **não depende desta crate** de propósito.
    //
    //    ⚠️ *A deriva que o doc dizia impedir era exactamente a que existia:* as duas listas nunca
    //    se tocaram. ⇒ a cura é apagar o órfão (a terceira espécie do `CLAUDE.md` §5.0: um órfão
    //    lê-se igual a um morto, e tratá-lo como morto levaria alguém a ligar o painel a esta
    //    crate para curar um defeito que não existe).
    //
    //    ⛔ E o que **NÃO** se faz aqui é traduzi-lo: publicar um rótulo que ninguém pinta poria uma
    //    segunda palavra por barramento na tabela, ao lado das `panel.audio_mixer.bus.*` que já lá
    //    estão. A ordem e a contagem das duas listas continuam atadas pelo `SUB_BUS_COUNT`.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sub_buses_map_to_dense_indices() {
        // Every sub-bus has a distinct index in 0..SUB_BUS_COUNT, and Master has
        // none — the invariant the mixer's `[_; SUB_BUS_COUNT]` array relies on.
        assert_eq!(BusId::Master.sub_index(), None);
        for (i, &bus) in BusId::SUB_BUSES.iter().enumerate() {
            assert_eq!(bus.sub_index(), Some(i), "{bus:?} must map to index {i}");
        }
    }

    #[test]
    fn default_is_master() {
        assert_eq!(BusId::default(), BusId::Master);
    }
}
