//! **`render(receita, semente, versão_do_motor) -> PCM`, a porta única** (SFX Lab, ADR-0198). A prévia e o arquivo
//! saem daqui, então não têm como divergir. As camadas se somam: o [`RenderCache`] refaz só a que mudou, e a
//! função livre [`render`] é ele sobre um cache vazio.

mod blep;
mod body;
mod envelope;
mod limitador;
mod noise;
mod recording;
mod rng;
mod tone;

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_sfx_recipe::{ENGINE_V1, Layer, LayerId, Recipe, SAMPLE_RATE, Source, Takes, frames_of};

pub use limitador::{TETO_DA_ESCUTA, limitar_escuta};

/// O silêncio do motor: 2⁻²⁴, um passo de quantização de um `f32` perto de 1,0. Um ressonador abaixo disto pára,
/// o que também impede números subnormais, lentos e dependentes do modo da thread.
pub(crate) const FLOOR: f32 = 1.0 / (1u32 << 24) as f32;

#[derive(Clone, Debug, PartialEq)]
pub struct Pcm {
    pub rate: u32,
    /// Mono.
    pub samples: Vec<f32>,
    /// As camadas de gravação cujo material não está no projeto: elas saíram em silêncio.
    pub missing: Vec<LayerId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// A receita pede um motor que esta árvore não toca: nunca se troca o som em silêncio.
    UnknownEngine(u32),
    /// O cache pediria mais memória do que o motor admite.
    TooLarge { bytes: u64 },
    /// Quem pediu desistiu: uma edição mais nova já vale. O que ficou pronto continua no cache.
    Cancelled,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownEngine(v) => write!(f, "sound engine {v} is not available"),
            Self::TooLarge { bytes } => write!(f, "this recipe needs {bytes} bytes of audio"),
            Self::Cancelled => f.write_str("the sound was edited again"),
        }
    }
}

/// O maior áudio que um render guarda, entre o resultado e o cache. O recurso é a RAM da menor máquina do projeto, 8 GiB
/// (`hw-profile`); oito camadas no teto de duração pedem 184 MB (`03_W2_medicao.md`).
pub const MAX_BYTES: u64 = 1 << 30;

/// A semente de uma camada: da receita e do id estável, nunca da posição.
#[must_use]
pub fn layer_seed(recipe_seed: u64, id: LayerId) -> u64 {
    rng::Rng::new(recipe_seed ^ id.0.wrapping_mul(0xD6E8_FEB8_6659_FD93)).next_u64()
}

/// FNV-1a sobre os bits de cada amostra: um bit diferente muda o resultado.
#[must_use]
pub fn pcm_hash(pcm: &[f32]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325_u64;
    for s in pcm {
        for b in s.to_bits().to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// A porta única, sem memória.
///
/// # Errors
/// `engine` não é um motor desta árvore.
pub fn render(recipe: &Recipe, seed: u64, engine: u32, takes: &Takes) -> Result<Pcm, RenderError> {
    RenderCache::default().render(recipe, seed, engine, takes)
}

struct Cached {
    key: Vec<u8>,
    pcm: Arc<[f32]>,
    missing: bool,
}

/// O que cada camada renderizou por último, antes do ganho e do início. Mudar só esses dois não refaz nada.
#[derive(Default)]
pub struct RenderCache {
    layers: BTreeMap<LayerId, Cached>,
    /// Quantas camadas o último `render` teve de calcular.
    pub last_rendered: usize,
}

impl RenderCache {
    /// Dá os mesmos bytes de [`render`], cheio ou vazio: a soma é sempre sobre os buffers, em ordem de id.
    ///
    /// # Errors
    /// `engine` não é um motor desta árvore.
    pub fn render(
        &mut self,
        recipe: &Recipe,
        seed: u64,
        engine: u32,
        takes: &Takes,
    ) -> Result<Pcm, RenderError> {
        self.render_cancelavel(recipe, seed, engine, takes, &|| false)
    }

    /// A porta única: [`Self::render`] é ela sem desistência. `cancelado` nunca muda um byte: ou o som sai inteiro,
    /// ou sai [`RenderError::Cancelled`]. Pergunta-se entre camadas, entre modos do corpo e a cada
    /// [`recording::PERGUNTA_A_CADA`] amostras de gravação; tom e ruído levam no máximo ~140 ms em 120 s (`03_W2_medicao.md`).
    ///
    /// # Errors
    /// `engine` não é um motor desta árvore, ou `cancelado` respondeu `true`.
    pub fn render_cancelavel(
        &mut self,
        recipe: &Recipe,
        seed: u64,
        engine: u32,
        takes: &Takes,
        cancelado: &dyn Fn() -> bool,
    ) -> Result<Pcm, RenderError> {
        if engine != ENGINE_V1 {
            return Err(RenderError::UnknownEngine(engine));
        }
        let mut r = recipe.clone();
        r.sanitize();
        // A ordem de soma é a do id: reordenar a pilha na tela não muda um bit.
        r.layers.sort_by_key(|l| l.id);

        self.last_rendered = 0;
        self.layers
            .retain(|id, _| r.layers.iter().any(|l| l.id == *id));

        let total = r.frames();
        let bytes = (r.layers.len() as u64 + 1).saturating_mul(total as u64 * 4);
        if bytes > MAX_BYTES {
            return Err(RenderError::TooLarge { bytes });
        }
        let mut mix = vec![0.0f32; total];
        let mut missing = Vec::new();
        for layer in &r.layers {
            if layer.muted {
                continue;
            }
            if cancelado() {
                return Err(RenderError::Cancelled);
            }
            let start = frames_of(layer.start).min(total);
            let len = total - start;
            let semente = layer_seed(seed, layer.id);
            let key = key_of(layer, semente, engine, len, takes);
            let hit = self.layers.get(&layer.id).is_some_and(|c| c.key == key);
            if !hit {
                let Some((pcm, falta)) = render_layer(layer, semente, len, takes, cancelado) else {
                    return Err(RenderError::Cancelled);
                };
                self.layers.insert(
                    layer.id,
                    Cached {
                        key,
                        pcm: pcm.into(),
                        missing: falta,
                    },
                );
                self.last_rendered += 1;
            }
            let cached = &self.layers[&layer.id];
            if cached.missing {
                missing.push(layer.id);
            }
            for (m, s) in mix[start..].iter_mut().zip(cached.pcm.iter()) {
                *m += *s * layer.gain;
            }
        }
        Ok(Pcm {
            rate: SAMPLE_RATE,
            samples: mix,
            missing,
        })
    }
}

/// Tudo o que decide os bytes da camada. Ganho e início ficam de fora: eles só agem na soma.
fn key_of(layer: &Layer, semente: u64, engine: u32, len: usize, takes: &Takes) -> Vec<u8> {
    let presente = match &layer.source {
        Source::Recording(r) => takes.get(&r.take).is_some(),
        _ => true,
    };
    postcard::to_allocvec(&(
        &layer.source,
        &layer.envelope,
        semente,
        engine,
        len as u64,
        presente,
    ))
    .unwrap_or_default()
}

/// O buffer de uma camada, sem ganho. `true` = o material dela não está no projeto. `None` = desistiu no meio, e o
/// pedaço calculado não serve a ninguém.
fn render_layer(
    layer: &Layer,
    semente: u64,
    len: usize,
    takes: &Takes,
    cancelado: &dyn Fn() -> bool,
) -> Option<(Vec<f32>, bool)> {
    let mut buf = vec![0.0f32; len];
    // Depois do silêncio garantido do envelope, o kernel nem corre.
    let audivel = layer.envelope.silent_from().map_or(len, |f| f.min(len));
    let mut falta = false;
    let corpo = &mut buf[..audivel];
    match &layer.source {
        Source::Noise(n) => noise::render(corpo, semente, n),
        Source::Tone(t) => tone::render(corpo, t),
        Source::Body(b) => body::render(corpo, semente, b, cancelado)?,
        Source::Recording(r) => match takes.get(&r.take) {
            Some(take) => recording::render(corpo, take, r, cancelado)?,
            None => falta = true,
        },
    }
    envelope::apply(&layer.envelope, corpo);
    Some((buf, falta))
}
