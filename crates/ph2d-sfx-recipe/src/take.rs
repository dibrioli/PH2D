//! O material gravado: PCM guardado como chegou, com a taxa dele. `Take` vem decodificado de propósito: passar
//! pelo decodificador a cada render faria os bytes dependerem dele.

use std::collections::BTreeMap;
use std::sync::Arc;

/// O maior módulo de uma amostra guardada. Muito além de qualquer áudio: com [`crate::MAX_GAIN`], a soma fica finita.
pub const MAX_SAMPLE: f32 = 1.0e6;

/// O endereço do material pelo conteúdo: taxa, canais e os bits das amostras.
pub type TakeId = [u8; 32];

#[derive(Clone, Debug, PartialEq)]
pub struct Take {
    pub id: TakeId,
    pub rate: u32,
    pub channels: u16,
    /// Intercaladas e sempre finitas.
    pub samples: Arc<[f32]>,
}

impl Take {
    /// A porta de entrada do material no motor: o que não é número vira silêncio, e o resto cabe em [`MAX_SAMPLE`].
    #[must_use]
    pub fn new(rate: u32, channels: u16, mut samples: Vec<f32>) -> Self {
        for s in &mut samples {
            *s = if s.is_finite() {
                s.clamp(-MAX_SAMPLE, MAX_SAMPLE)
            } else {
                0.0
            };
        }
        let (rate, channels) = (rate.max(1), channels.max(1));
        Self {
            id: id_of(rate, channels, &samples),
            rate,
            channels,
            samples: samples.into(),
        }
    }

    /// Quantos instantes o material tem, em qualquer número de canais.
    #[must_use]
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }

    /// A média dos canais no instante `i`; fora do material, silêncio.
    #[must_use]
    pub fn mono(&self, i: usize) -> f32 {
        let n = usize::from(self.channels);
        let Some(canais) = self
            .samples
            .get(i.saturating_mul(n)..(i + 1).saturating_mul(n))
        else {
            return 0.0;
        };
        canais.iter().sum::<f32>() / f32::from(self.channels)
    }
}

fn id_of(rate: u32, channels: u16, samples: &[f32]) -> TakeId {
    let mut h = blake3::Hasher::new();
    h.update(&rate.to_le_bytes());
    h.update(&channels.to_le_bytes());
    for s in samples {
        h.update(&s.to_bits().to_le_bytes());
    }
    *h.finalize().as_bytes()
}

/// O material de um projeto, por conteúdo: importar o mesmo áudio duas vezes guarda uma cópia.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Takes {
    by_id: BTreeMap<TakeId, Take>,
}

impl Takes {
    pub fn add(&mut self, take: Take) -> TakeId {
        let id = take.id;
        self.by_id.insert(id, take);
        id
    }

    #[must_use]
    pub fn get(&self, id: &TakeId) -> Option<&Take> {
        self.by_id.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Take> {
        self.by_id.values()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}
