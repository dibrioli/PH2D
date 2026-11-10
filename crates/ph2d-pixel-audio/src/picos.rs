//! A onda da linha da faixa: mínimo e máximo por janela de [`PICO_MS`], calculados uma vez por asset.

use ph2d_audio::SampleData;

/// A janela de um pico, em ms da fonte (a linha desenha um traço por pico).
pub const PICO_MS: u32 = 10;

/// O mínimo e o máximo (dos dois canais) de uma janela.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pico {
    pub min: f32,
    pub max: f32,
}

/// Os picos de `d`, um por [`PICO_MS`] da fonte (o último pode ser mais curto).
#[must_use]
pub fn picos(d: &SampleData) -> Vec<Pico> {
    let rate = d.format().sample_rate.max(1) as usize;
    let janela = (rate * PICO_MS as usize / 1000).max(1);
    let n = d.frame_count();
    (0..n.div_ceil(janela))
        .map(|i| {
            let mut p = Pico { min: 0.0, max: 0.0 };
            for q in i * janela..((i + 1) * janela).min(n) {
                for v in d.frame_stereo(q) {
                    p.min = p.min.min(v);
                    p.max = p.max.max(v);
                }
            }
            p
        })
        .collect()
}
