//! **AS DUAS CENAS DE SMOKE DO DISPOSITIVO** — o tom de teste (`PH2D_AUDIO_SMOKE`) e o ficheiro em
//! ciclo (`PH2D_AUDIO_FILE`).
//!
//! ⚠️ **Irmão de [`super`] por CAP de FICHEIRO** (HR-18, 600): o livro das vozes da cena (TOP-20 #4)
//! levou-o a 616, e a regra da casa é cortar para o irmão, nunca declarar uma excepção.
//!
//! ⚠️ **E o corte é por RESPONSABILIDADE, não por tamanho:** o resto do módulo é o dispositivo e o
//! mixer; estas duas são cenas de diagnóstico, armadas por uma variável de ambiente. *Nenhuma delas
//! é o sistema de áudio — elas são clientes dele.*

use ph2d_audio::{BusId, PlayParams};

use super::AudioSystem;
use super::signals::sine_tone;

impl AudioSystem {
    /// Queue a short 440 Hz test tone — the `PH2D_AUDIO_SMOKE` beep that proves
    /// the control → audio → device path end to end.
    pub(crate) fn play_test_tone(&mut self) {
        let tone = sine_tone(self.format, 440.0, 0.6, 0.4);
        let params = PlayParams {
            bus: BusId::Sfx,
            ..PlayParams::default()
        };
        match self.engine.play(tone, params) {
            Ok(_) => println!("audio: playing 440 Hz test tone on the SFX bus (PH2D_AUDIO_SMOKE)"),
            Err(e) => eprintln!("audio: test tone dropped ({e})"),
        }
    }

    /// Decode and loop-play an audio file (the `PH2D_AUDIO_FILE` smoke). The
    /// clip's own sample rate is resampled to the device rate by the voice.
    pub(crate) fn play_file(&mut self, path: &std::path::Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("audio: cannot read {}: {e}", path.display());
                return;
            }
        };
        let data = match ph2d_audio_decode::decode(&bytes) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("audio: decode failed for {}: {e}", path.display());
                return;
            }
        };
        let fmt = data.format();
        let secs = fmt.frames_to_secs(data.frame_count() as u64);
        let params = PlayParams {
            looping: true,
            bus: BusId::Music,
            ..PlayParams::default()
        };
        match self.engine.play(data, params) {
            Ok(_) => println!(
                "audio: looping {} on the Music bus ({secs:.1}s, {} Hz, {:?})",
                path.display(),
                fmt.sample_rate,
                fmt.channels
            ),
            Err(e) => eprintln!("audio: play failed ({e})"),
        }
    }
}
