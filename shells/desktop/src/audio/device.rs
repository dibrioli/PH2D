//! **A ESCOLHA E A ABERTURA DO DISPOSITIVO DE SAÍDA** — irmão de [`super`] pelo teto de
//! 600 LOC do shell (HR-18), e o corte é por RESPONSABILIDADE: ali mora o `AudioSystem`, que
//! é a superfície de CONTROLO do mixer; aqui mora a pergunta *«que formato o dispositivo
//! aceita, e como escrevemos nele»*.
//!
//! ⚠️ As três funções são uma resposta só e têm de concordar entre si: [`supported_by_us`]
//! diz o que sabemos escrever, [`pick_writable_config`] procura-o na lista do dispositivo, e
//! [`build_stream`] escreve. Um formato aceite pelas duas primeiras e ausente do `match` da
//! terceira cairia no braço `_ => u16` — que escreveria **lixo**, não silêncio.

use cpal::traits::DeviceTrait;
use cpal::{FromSample, SizedSample};
use ph2d_audio::AudioRenderer;

/// Os três formatos de amostra que [`build_stream`] sabe escrever.
///
/// ⚠️ **Esta lista e o `match` do `build_stream` são a MESMA resposta e têm de concordar.**
/// Se um dia entrar um quarto formato, ele entra nos dois — um formato aceite aqui e ausente
/// lá cai no braço `_ => u16`, que escreveria lixo em vez de silêncio.
pub(super) fn supported_by_us(f: cpal::SampleFormat) -> bool {
    matches!(
        f,
        cpal::SampleFormat::F32 | cpal::SampleFormat::I16 | cpal::SampleFormat::U16
    )
}

/// O melhor formato **escrevível** que o dispositivo oferece, na taxa mais próxima de `rate`.
///
/// Preferência `F32` > `I16` > `U16`: é a ordem em que se perde menos ao converter o mix `f32`.
/// ⚠️ A taxa preferida é a que o dispositivo escolheria por omissão — trocar de formato não é
/// razão para também trocar de taxa, e `with_max_sample_rate` poria um DAC de 192 kHz a correr
/// no topo por causa de uma pergunta que era sobre outra coisa.
pub(super) fn pick_writable_config(
    ranges: impl Iterator<Item = cpal::SupportedStreamConfigRange>,
    rate: cpal::SampleRate,
) -> Option<cpal::SupportedStreamConfig> {
    let rank = |f: cpal::SampleFormat| match f {
        cpal::SampleFormat::F32 => 3,
        cpal::SampleFormat::I16 => 2,
        cpal::SampleFormat::U16 => 1,
        _ => 0,
    };
    let best = ranges
        .filter(|r| supported_by_us(r.sample_format()))
        .max_by_key(|r| (rank(r.sample_format()), r.channels()))?;
    // `SupportedStreamConfigRange` é `Copy`, então o `best` sobrevive à primeira tentativa.
    best.try_with_sample_rate(rate)
        .or_else(|| Some(best.with_max_sample_rate()))
}

/// Build the output stream for device sample type `T`. The mixer renders into a
/// reused `f32` scratch (mono/stereo per `our_channels`), which is then
/// converted + scattered into the device's `dev_channels` layout. Mirrors the
/// cpal `beep.rs` reference (DIRETIVA §1) for the `T::from_sample` conversion.
pub(super) fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut renderer: AudioRenderer,
    dev_channels: usize,
    our_channels: usize,
    // ⚠️ cpal 0.18: os erros de construção unificaram-se num `cpal::Error` só
    // (`BuildStreamError` deixou de existir). O `err_fn` da callback continua a
    // inferir sozinho.
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample + FromSample<f32>,
{
    let err_fn = |e| eprintln!("audio: stream error: {e}");
    // Owned by the callback; sized once (when the block size stabilizes), then
    // reused — no allocation in the warm hot path (HR-3).
    let mut scratch: Vec<f32> = Vec::new();
    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let frames = data.len() / dev_channels.max(1);
            let needed = frames * our_channels;
            if scratch.len() != needed {
                scratch.resize(needed, 0.0);
            }
            renderer.render(&mut scratch, frames);
            for f in 0..frames {
                for c in 0..dev_channels {
                    let s = if our_channels == 1 {
                        scratch[f]
                    } else if c < 2 {
                        scratch[f * 2 + c]
                    } else {
                        0.0
                    };
                    data[f * dev_channels + c] = T::from_sample(s);
                }
            }
        },
        err_fn,
        None,
    )
}
