//! **A VOZ DO SOM DE UI** (estudo de UI viva, D1) — irmão do [`super`] pelo teto de 600 LOC, e o
//! corte é por ASSUNTO: ali fica o motor (o dispositivo, os buses, o mixer, o editor); aqui, o
//! único disparo que o CHROME faz.
//!
//! ⚠️ **Ele é a única voz deste app que não vem de um documento.** Todas as outras tocam o que o
//! artista autorou; esta toca o que a mão dele acabou de fazer.

use super::{AudioSystem, BusId, PlayParams, signals};

impl AudioSystem {
    /// **TOCA UM SOM DE UI** (estudo de UI viva, D1) — um disparo curto, sintetizado, no bus SFX.
    ///
    /// ⚠️ **Sem `looping`, e sem guardar a voz.** Um som de UI é um evento, não uma fonte: guardar
    /// o `VoiceId` obrigaria a decidir quando o parar, e a resposta seria sempre *"ele já parou"*.
    /// O `poll` por quadro já larga o `Arc` na thread principal (HR-3).
    ///
    /// ⚠️ **No bus SFX**, e não no Master: é lá que o mixer do artista o pode baixar sem levar a
    /// música dele junto — a mesma razão de o sinal de teste já lá tocar.
    pub(crate) fn play_ui(&mut self, what: crate::ui_sound::UiSound) {
        let (hz, secs, gain) = what.voice();
        let data = signals::blip_loop(self.format, hz, secs, secs, gain);
        let params = PlayParams {
            bus: BusId::Sfx,
            ..PlayParams::default()
        };
        if let Err(e) = self.engine.play(data, params) {
            eprintln!("audio: ui sound dropped ({e})");
        }
    }
}
