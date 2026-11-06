//! **As medidas do som:** o que o arquivo tem, sem limitador. Corre na thread do render; a pintura é de `ph2d-panel-sfxlab`.

use ph2d_audio::dsp::integrated_lufs;
use ph2d_audio_spectral::stft::{Stft, magnitude};

/// Bandas do espectro, em escala logarítmica. A coluna tem ~100 px: 32 bandas são ~3 px cada.
pub const BANDAS: usize = 32;
const HZ_MIN: f32 = 20.0;
const HZ_MAX: f32 = 20_000.0;
/// O piso do desenho do espectro e do pico.
pub const DB_PISO: f32 = -60.0;
/// A janela da FFT e o salto: resolução de 23 Hz, e o salto cobre o clique curto.
const JANELA: usize = 2048;
const SALTO: usize = 512;
const SEM_SINAL: f32 = 1.0e-9;

#[derive(Clone, Debug, PartialEq)]
pub struct Medida {
    /// Em dB, sem limitador. Passa de 0 quando o arquivo estoura.
    pub pico_db: f32,
    pub lufs: f32,
    /// dB de cada banda, com retenção do máximo no tempo.
    pub espectro: [f32; BANDAS],
}

impl Default for Medida {
    fn default() -> Self {
        Self {
            pico_db: f32::NEG_INFINITY,
            lufs: f32::NEG_INFINITY,
            espectro: [DB_PISO; BANDAS],
        }
    }
}

impl Medida {
    /// CLIP acende se uma amostra do arquivo passou de 1,0.
    #[must_use]
    pub fn clip(&self) -> bool {
        self.pico_db > 0.0
    }
}

#[must_use]
pub fn pico_db(amostras: &[f32]) -> f32 {
    let p = amostras.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
    if p > 0.0 {
        20.0 * p.log10()
    } else {
        f32::NEG_INFINITY
    }
}

fn banda_de(hz: f32) -> Option<usize> {
    if !(HZ_MIN..HZ_MAX).contains(&hz) {
        return None;
    }
    let t = (hz / HZ_MIN).ln() / (HZ_MAX / HZ_MIN).ln();
    Some(((t * BANDAS as f32) as usize).min(BANDAS - 1))
}

/// O espectro do som inteiro, por retenção do máximo: um clique curto não some na média.
#[must_use]
pub fn espectro(amostras: &[f32], taxa: u32) -> [f32; BANDAS] {
    let mut out = [DB_PISO; BANDAS];
    if amostras.is_empty() {
        return out;
    }
    let mut stft = Stft::with(JANELA, SALTO);
    let bins = stft.bins();
    let hz_por_bin = taxa as f32 / JANELA as f32;
    // Mesma referência do espectrograma: um seno em escala cheia lê 0 dB.
    let referencia = JANELA as f32 * 0.25;
    let mapa: Vec<Option<usize>> = (0..bins).map(|b| banda_de(b as f32 * hz_por_bin)).collect();
    let mut linear = [0.0_f32; BANDAS];
    stft.analyze(amostras, |_, spec| {
        for (c, banda) in spec.iter().zip(&mapa) {
            if let Some(i) = banda {
                linear[*i] = linear[*i].max(magnitude(*c));
            }
        }
    });
    for (o, v) in out.iter_mut().zip(linear) {
        *o = (20.0 * (v / referencia).max(SEM_SINAL).log10()).max(DB_PISO);
    }
    out
}

/// O que a thread do render calcula.
#[must_use]
pub fn mede(amostras: &[f32], taxa: u32) -> Medida {
    Medida {
        pico_db: pico_db(amostras),
        lufs: integrated_lufs(amostras, 1, taxa),
        espectro: espectro(amostras, taxa),
    }
}
