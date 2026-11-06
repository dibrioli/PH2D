//! **A lei dos valores:** nada que saia daqui faz o motor produzir um não-número. Vale para receita aberta de
//! arquivo e para receita que a tela vier a escrever. As faixas dizem o que o motor aguenta; a faixa de um
//! controle de tela é outra conversa, em W3.

use crate::{
    Body, EnvPoint, Envelope, Layer, ModeLaw, Noise, Recipe, Recording, SAMPLE_RATE, Source, Tone,
    Wave, frames_of,
};

/// A duração máxima de uma receita. O recurso é a RAM do cache: 4 bytes × 48 000 × segundos, por camada
/// (120 s = 23 MB; medição em `docs/SfxLab/03_W2_medicao.md`).
pub const MAX_SECONDS: f32 = 120.0;
/// A altura máxima de um kernel: 0,45 da taxa, a banda que o antialias entrega (`03_W2_medicao.md`).
pub const MAX_HZ: f32 = SAMPLE_RATE as f32 * 0.45;
/// A altura mínima: abaixo disto a fase de um segundo de som quase não anda.
pub const MIN_HZ: f32 = 1.0;
/// Com [`crate::MAX_SAMPLE`], mantém a soma dentro do `f32`. Não é faixa de tela.
pub const MAX_GAIN: f32 = 1.0e6;
/// A base mínima de um corpo, o começo do audível. Junto com [`MAX_HZ`], é ela que limita o número de modos: 1 080 num
/// corpo harmônico (`03_W2_medicao.md`).
pub const MIN_BODY_HZ: f32 = 20.0;
/// O corte da leitura de material: cada oitava acima dobra os toques do filtro (`03_W2_medicao.md`).
pub const MAX_SEMITONES: f32 = 48.0;

const MIN_T60: f32 = 0.001;
const MIN_CONTACT_MS: f32 = 1000.0 / SAMPLE_RATE as f32;
const MAX_CONTACT_MS: f32 = 1000.0;

/// O valor na faixa; um não-número vira `padrao`.
#[must_use]
fn faixa(x: f32, lo: f32, hi: f32, padrao: f32) -> f32 {
    if x.is_nan() { padrao } else { x.clamp(lo, hi) }
}

fn hz(x: f32, padrao: f32) -> f32 {
    faixa(x, MIN_HZ, MAX_HZ, padrao)
}

impl Recipe {
    /// Põe a receita dentro da lei. Idempotente.
    pub fn sanitize(&mut self) {
        self.seconds = faixa(self.seconds, 0.0, MAX_SECONDS, 1.0);
        for l in &mut self.layers {
            l.sanitize();
        }
        let maior = self.layers.iter().map(|l| l.id.0).max().unwrap_or(0);
        self.next_layer = self.next_layer.max(maior.saturating_add(1));
    }
}

impl Layer {
    fn sanitize(&mut self) {
        self.gain = faixa(self.gain, 0.0, MAX_GAIN, 1.0);
        self.start = faixa(self.start, 0.0, MAX_SECONDS, 0.0);
        self.envelope.sanitize();
        match &mut self.source {
            Source::Noise(n) => n.sanitize(),
            Source::Tone(t) => t.sanitize(),
            Source::Body(b) => b.sanitize(),
            Source::Recording(r) => r.sanitize(),
        }
    }
}

impl Noise {
    fn sanitize(&mut self) {
        self.hz = hz(self.hz, 1_000.0);
        self.hz_end = hz(self.hz_end, self.hz);
        self.q = faixa(self.q, 0.05, 1_000.0, 1.0);
    }
}

impl Tone {
    fn sanitize(&mut self) {
        self.hz = hz(self.hz, 440.0);
        self.hz_end = hz(self.hz_end, self.hz);
        self.glide = faixa(self.glide, 0.0, MAX_SECONDS, 0.0);
        if let Wave::Fm {
            ratio,
            index,
            index_end,
        } = &mut self.wave
        {
            *ratio = faixa(*ratio, 0.0, 1_000.0, 1.0);
            *index = faixa(*index, 0.0, 1_000.0, 0.0);
            *index_end = faixa(*index_end, 0.0, 1_000.0, *index);
        }
    }
}

impl Body {
    fn sanitize(&mut self) {
        if let ModeLaw::Stretched { stretch } = &mut self.law {
            // De corda ideal a além da barra: abaixo de 1 os modos se amontoam sem fim abaixo da banda.
            *stretch = faixa(*stretch, 1.0, 3.0, 1.0);
        }
        self.base_hz = faixa(self.base_hz, MIN_BODY_HZ, MAX_HZ, 220.0);
        self.t60 = faixa(self.t60, MIN_T60, MAX_SECONDS, 1.0);
        self.damping = faixa(self.damping, 0.0, 1_000.0, 0.0);
        self.rolloff = faixa(self.rolloff, 0.0, 8.0, 1.0);
        self.contact_ms = faixa(self.contact_ms, MIN_CONTACT_MS, MAX_CONTACT_MS, 1.0);
        self.noise = faixa(self.noise, 0.0, 1.0, 0.0);
    }
}

impl Recording {
    fn sanitize(&mut self) {
        self.semitones = faixa(self.semitones, -MAX_SEMITONES, MAX_SEMITONES, 0.0);
        self.offset = faixa(self.offset, 0.0, f32::MAX, 0.0);
    }
}

impl Envelope {
    fn sanitize(&mut self) {
        self.points.retain(|p| !p.t.is_nan());
        for p in &mut self.points {
            p.t = p.t.clamp(0.0, MAX_SECONDS);
            p.v = faixa(p.v, 0.0, 1.0, 0.0);
        }
        self.points.sort_by(|a, b| a.t.total_cmp(&b.t));
    }

    /// Os limites em amostras do trecho que esta curva pode tocar: `None` = não há silêncio garantido no fim.
    #[must_use]
    pub fn silent_from(&self) -> Option<usize> {
        let EnvPoint { t, v, .. } = *self.points.last()?;
        (v == 0.0).then(|| frames_of(t))
    }
}
