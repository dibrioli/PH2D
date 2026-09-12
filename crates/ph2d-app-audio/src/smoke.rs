//! **AS CENAS DE SMOKE DO ÁUDIO** — o tom de teste (`PH2D_AUDIO_SMOKE`), o ficheiro em ciclo
//! (`PH2D_AUDIO_FILE`) e as seis encenações do Audio Editor, todas lidas AQUI ([`AudioSystem::stage_armed_smokes`]).
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
    pub fn play_test_tone(&mut self) {
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
    pub fn play_file(&mut self, path: &std::path::Path) {
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

/// ⭐ **Os roteadores de smoke da família** — declarados ao registo pelo `crate::FAMILY`.
///
/// ⚠️ **São BOOLEANOS** (armados pela PRESENÇA da variável), então o `max_level` é `1`: não há
/// `match` de níveis a contar. O que o registo ganha é o gate de colisão — duas famílias a reclamar a
/// mesma variável passariam mudas. O `PH2D_AUDIO_FILE` fica de fora de propósito: é um CAMINHO, não
/// uma cena.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_LOOP_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_MULTIBAND_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_VOICE_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_DELIVERY_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_KNOB_SMOKE",
        max_level: 1,
    },
    ph2d_app_host::SmokeRouter {
        env: "PH2D_AUDIO_ML_SMOKE",
        max_level: 1,
    },
];

impl AudioSystem {
    /// **Encena o que o dono armou por variável de ambiente** — morava no arranque da shell
    /// (`main.rs`), e a família declarava roteadores que não lia. ⚠️ As seis encenações do Audio Editor
    /// só existem com a feature do painel, e o `cfg` diz isso aqui em vez de a shell as chamar sempre.
    pub fn stage_armed_smokes(&mut self) {
        if std::env::var_os("PH2D_AUDIO_SMOKE").is_some() {
            self.play_test_tone();
        }
        if let Some(path) = std::env::var_os("PH2D_AUDIO_FILE") {
            self.play_file(std::path::Path::new(&path));
        }
        #[cfg(feature = "panel-audio-editor")]
        self.stage_editor_smokes();
    }

    /// As seis encenações do Audio Editor — ver o comentário de cada uma no `main.rs` de antes.
    #[cfg(feature = "panel-audio-editor")]
    fn stage_editor_smokes(&mut self) {
        // O loop pronto a auditar (W6): abra o pill do Audio Editor para o ver.
        if std::env::var_os("PH2D_AUDIO_LOOP_SMOKE").is_some() {
            self.editor_loop_smoke();
        }
        // O A/B do Multiband: um bombo sobre agudos constantes + Multiband contra Compress no mesmo Ratio.
        if std::env::var_os("PH2D_AUDIO_MULTIBAND_SMOKE").is_some() {
            self.editor_multiband_smoke();
        }
        // A família da VOZ (W4): fala sintetizada + o Vocoder nas duas pontas do Breath + o Granular.
        if std::env::var_os("PH2D_AUDIO_VOICE_SMOKE").is_some() {
            self.editor_voice_smoke();
        }
        // Os destinos de entrega (W6): um brilho a 15 kHz que uma variante a 24 kHz não pode levar.
        if std::env::var_os("PH2D_AUDIO_DELIVERY_SMOKE").is_some() {
            self.editor_delivery_smoke();
        }
        // O arrasto de knob do ADR-0120: clip de 3 min, selecção, rack de UM estágio.
        // `PH2D_AUDIO_SLOW_PREVIEW=1` volta ao caminho antigo — a feature é byte-idêntica.
        if std::env::var_os("PH2D_AUDIO_KNOB_SMOKE").is_some() {
            self.editor_knob_smoke();
        }
        // O AI Denoise (W7): tom vozeado sob chiado a ~0 dB SNR (precisa de `--features audio-ml`).
        if std::env::var_os("PH2D_AUDIO_ML_SMOKE").is_some() {
            self.editor_ml_smoke();
        }
    }
}
