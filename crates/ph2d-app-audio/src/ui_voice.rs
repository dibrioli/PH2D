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
    pub fn play_ui(&mut self, what: crate::ui_sound::UiSound, diag: bool) {
        let (hz, secs, gain) = what.voice();
        let data = signals::blip_loop(self.format, hz, secs, secs, gain);
        let params = PlayParams {
            bus: BusId::Sfx,
            ..PlayParams::default()
        };
        match self.engine.play(data, params) {
            Ok(v) => {
                if diag {
                    eprintln!(
                        "[ui-sound]   -> voz {v:?} · {hz} Hz · {secs}s · ganho {gain} · bus SFX"
                    );
                }
            }
            Err(e) => eprintln!("[ui-sound]   -> RECUSADO pelo motor: {e}"),
        }
    }

    /// ⭐ **A CAMPAINHA do cronómetro do quadro** (MiroClone W7, `docs/MiroClone/05_facilitacao.md` F6:
    /// o Miro avisa *«visually and with a chime»*). ⚠️ Não é um som de UI: é um ALARME que o facilitador
    /// armou e cuja vez ninguém tem à mão — por isso não passa pela preferência do som de UI (*«só o
    /// que a mão fez»*); desliga-se no altifalante do próprio cartão. Duas notas em quinta, dedilhadas
    /// e a morrer devagar: lê-se como «acabou», não como um clique.
    pub fn play_chime(&mut self) {
        for (hz, gain) in CHIME {
            let data = signals::pluck_loop(self.format, hz, CHIME_SECS, gain);
            let params = PlayParams {
                bus: BusId::Sfx,
                ..PlayParams::default()
            };
            if let Err(e) = self.engine.play(data, params) {
                eprintln!("[chime] recusado pelo motor: {e}");
            }
        }
    }
}

/// As notas da campainha `(Hz, ganho)`: um dó e a quinta dele, duas oitavas acima do lá.
const CHIME: [(f32, f32); 2] = [(1046.5, 0.25), (1568.0, 0.18)];
/// A duração da campainha (s): mais longa que um som de UI (que cabe em 90 ms).
const CHIME_SECS: f32 = 1.2;
