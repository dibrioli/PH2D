//! **O que se ouve.** A família só fala com um `Escuta`; a shell entrega o `AudioEngine`, e o teste um falso.

use ph2d_audio::{AudioEngine, AudioFormat, ChannelLayout, PlayParams, SampleData};
use ph2d_sfx_recipe::SAMPLE_RATE;

pub trait Escuta {
    fn toca(&mut self, som: SampleData);
    fn para(&mut self);
}

impl Escuta for AudioEngine {
    fn toca(&mut self, som: SampleData) {
        if let Err(e) = self.play_preview(som, PlayParams::default()) {
            tracing::warn!("sfx lab: preview did not start: {e}");
        }
    }

    fn para(&mut self) {
        if let Err(e) = self.stop_preview() {
            tracing::warn!("sfx lab: preview did not stop: {e}");
        }
    }
}

/// O buffer da escuta a partir do cursor.
pub(crate) fn som(amostras: &[f32], cursor_s: f32) -> SampleData {
    let de = (ph2d_sfx_recipe::frames_of(cursor_s.max(0.0))).min(amostras.len());
    SampleData::from_interleaved(
        amostras[de..].to_vec(),
        AudioFormat {
            sample_rate: SAMPLE_RATE,
            channels: ChannelLayout::Mono,
        },
    )
}
