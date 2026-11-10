//! **O som do sprite** (P13 T): tocar as faixas do [`Sprite::audio`] em sincronia com o Play.
//!
//! - [`Sons`]: os assets decodificados, uma vez cada, com o cache de [`picos`] para a onda.
//! - [`Tocador`]: põe as faixas não mudas no mixer a partir de `t` (com `inicio_ms` e `ganho`),
//!   alimenta os streams ([`Tocador::bombear`], uma vez por quadro do host) e pára.
//! - [`Relogio`]: o playhead do mixer em ms; [`Relogio::dt`] é o `dt` do Play quando há som.
//!
//! O mixer é o `ph2d_audio::AudioEngine` do host: quem tem o dispositivo é a shell. Os gates
//! dirigem o `AudioRenderer` à mão, sem dispositivo (nenhum som sai).
//!
//! [`Sprite::audio`]: ph2d_pixel_core::Sprite::audio

pub mod faixas;
mod picos;
mod relogio;
mod sons;

pub use picos::{PICO_MS, Pico, picos};
pub use relogio::{Relogio, dt_do_play};
pub use sons::{Som, Sons};

use ph2d_audio::{
    AudioEngine, AudioError, PlayParams, STREAM_CHUNK_FRAMES, SampleData, StreamFeeder, VoiceId,
};
use ph2d_pixel_core::FaixaDeAudio;

/// Uma faixa a tocar: o produtor do stream e onde ele vai na fonte.
struct Corrente {
    feeder: StreamFeeder,
    dados: SampleData,
    /// O próximo quadro da fonte a dar; negativo = silêncio até o som começar.
    pos: i64,
    /// Onde parar (quadro da fonte, exclusivo): o fim do som ou o fim do pedaço do scrub.
    ate: i64,
    voz: VoiceId,
}

/// As vozes de um Play (ou de um pedaço de scrub) com som.
#[derive(Default)]
pub struct Tocador {
    correntes: Vec<Corrente>,
    relogio: Option<Relogio>,
}

/// Quadros da fonte em `ms` (arredonda para baixo; negativo fica negativo).
fn quadros(ms: i64, rate: u32) -> i64 {
    (ms * i64::from(rate)).div_euclid(1000)
}

impl Tocador {
    /// Toca as faixas não mudas de `faixas` a partir de `t_ms` (o instante do quadro onde o Play
    /// começa). Faixa cujo asset não está em `sons` fica calada. Pára o que estivesse a tocar.
    pub fn tocar(
        &mut self,
        engine: &mut AudioEngine,
        faixas: &[FaixaDeAudio],
        sons: &Sons,
        t_ms: u64,
    ) -> Result<(), AudioError> {
        self.inicia(engine, faixas, sons, t_ms, None)
    }

    /// O scrub: um pedaço curto de `dur_ms` a partir de `t_ms`, que acaba sozinho. Sem relógio.
    pub fn pedaco(
        &mut self,
        engine: &mut AudioEngine,
        faixas: &[FaixaDeAudio],
        sons: &Sons,
        t_ms: u64,
        dur_ms: u32,
    ) -> Result<(), AudioError> {
        self.inicia(engine, faixas, sons, t_ms, Some(dur_ms))?;
        self.relogio = None;
        Ok(())
    }

    fn inicia(
        &mut self,
        engine: &mut AudioEngine,
        faixas: &[FaixaDeAudio],
        sons: &Sons,
        t_ms: u64,
        dur_ms: Option<u32>,
    ) -> Result<(), AudioError> {
        self.parar(engine);
        let t = i64::try_from(t_ms).unwrap_or(i64::MAX / 2_000_000);
        for f in faixas.iter().filter(|f| !f.mudo) {
            let Some(som) = sons.som(&f.asset) else {
                continue;
            };
            let rate = som.dados.format().sample_rate.max(1);
            let pos = quadros(t - i64::from(f.inicio_ms), rate);
            let fim = som.dados.frame_count() as i64;
            let ate = dur_ms.map_or(fim, |d| fim.min(pos + quadros(i64::from(d), rate)));
            if pos >= ate {
                continue;
            }
            let (feeder, handle) = ph2d_audio::stream(rate);
            let playhead = handle.playhead();
            let mut c = Corrente {
                feeder,
                dados: som.dados.clone(),
                pos,
                ate,
                voz: VoiceId::NONE,
            };
            c.bombear();
            let params = PlayParams {
                gain: f.ganho,
                ..PlayParams::default()
            };
            c.voz = engine.play_stream(handle, params)?;
            // O relógio segue a faixa que mais dura: é a que fica viva até ao fim.
            let resta_ms = (ate - pos) * 1000 / i64::from(rate);
            if self
                .relogio
                .as_ref()
                .is_none_or(|r| r.resta_ms() < resta_ms)
            {
                self.relogio = Some(Relogio::novo(t_ms, playhead, resta_ms));
            }
            self.correntes.push(c);
        }
        Ok(())
    }

    /// Mantém os streams cheios. Chamar uma vez por quadro do host (não bloqueia, não espera).
    pub fn bombear(&mut self) {
        for c in &mut self.correntes {
            c.bombear();
        }
    }

    /// Pára todas as vozes (imediato) e esquece o relógio.
    pub fn parar(&mut self, engine: &AudioEngine) {
        for c in self.correntes.drain(..) {
            let _ = engine.stop(c.voz);
        }
        self.relogio = None;
    }

    /// O relógio do som, quando há faixa a tocar; `None` = o Play anda no relógio do host.
    pub fn relogio(&mut self) -> Option<&mut Relogio> {
        self.relogio.as_mut()
    }

    /// Quantas faixas estão no mixer.
    #[must_use]
    pub fn faixas_a_tocar(&self) -> usize {
        self.correntes.len()
    }
}

impl Corrente {
    fn bombear(&mut self) {
        while self.pos < self.ate {
            let Some(mut chunk) = self.feeder.take_empty() else {
                return;
            };
            let n =
                usize::try_from((self.ate - self.pos).min(STREAM_CHUNK_FRAMES as i64)).unwrap_or(0);
            let buf = chunk.buffer_mut();
            for k in 0..n {
                let q = self.pos + k as i64;
                let [l, r] = usize::try_from(q).map_or([0.0, 0.0], |q| self.dados.frame_stereo(q));
                buf[2 * k] = l;
                buf[2 * k + 1] = r;
            }
            chunk.set_frames(n);
            if self.feeder.submit(chunk).is_err() {
                return;
            }
            self.pos += n as i64;
        }
        if !self.feeder.is_finished() {
            self.feeder.finish();
        }
    }
}
