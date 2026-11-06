//! **A receita de um som** (SFX Lab, W2): camadas, fontes, envelopes, semente e versão do motor. O som sai de
//! `ph2d_sfx_dsp::render`; aqui mora só o documento e a lei dos valores que ele aceita.
//! Plano: `docs/SfxLab/01_plano.md`. Motor: ADR-0198.

mod envelope;
pub mod factory;
mod library;
mod sanitize;
mod take;

pub use envelope::{EnvPoint, Envelope};
#[cfg(feature = "component")]
pub use library::project;
pub use library::{LibraryBlobError, SfxLibrary};
pub use sanitize::{MAX_GAIN, MAX_HZ, MAX_SECONDS, MAX_SEMITONES, MIN_BODY_HZ, MIN_HZ};
pub use take::{MAX_SAMPLE, Take, TakeId, Takes};

use serde::{Deserialize, Serialize};

/// A taxa do render. Fixa: o documento mede tempo em segundos e os bytes têm de ser os mesmos em qualquer máquina.
pub const SAMPLE_RATE: u32 = 48_000;
/// O motor que esta árvore sabe tocar. Muda quando um kernel muda de bytes.
pub const ENGINE_V1: u32 = 1;

/// A identidade de uma camada: dá a semente dela e sobrevive a mover, apagar e inserir vizinhos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub seed: u64,
    /// O motor com que a receita foi feita.
    pub engine: u32,
    pub seconds: f32,
    pub layers: Vec<Layer>,
    /// O próximo id livre: um id apagado não volta, senão a camada nova herdaria a semente da velha.
    pub next_layer: u64,
    /// O som que o artista aprovou. Sobrevive a qualquer edição e a qualquer motor.
    pub approved: Option<Approved>,
}

/// O áudio aprovado: um som do projeto, a impressão digital dele e o motor que o fez.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Approved {
    pub sound: String,
    pub blake3: [u8; 32],
    pub engine: u32,
}

impl Recipe {
    #[must_use]
    pub fn new(seed: u64, seconds: f32) -> Self {
        Self {
            seed,
            engine: ENGINE_V1,
            seconds,
            layers: Vec::new(),
            next_layer: 1,
            approved: None,
        }
    }

    /// Esta receita ouvindo só a camada `id`. Serve à escuta; o documento continua como está.
    #[must_use]
    pub fn solo(&self, id: LayerId) -> Self {
        let mut r = self.clone();
        for l in &mut r.layers {
            l.muted = l.id != id;
        }
        r
    }

    /// Acrescenta uma camada com um id que nunca foi usado nesta receita.
    pub fn add(&mut self, gain: f32, source: Source, envelope: Envelope) -> LayerId {
        let id = LayerId(self.next_layer);
        self.next_layer = self.next_layer.saturating_add(1);
        self.layers.push(Layer {
            id,
            gain,
            start: 0.0,
            muted: false,
            source,
            envelope,
        });
        id
    }

    #[must_use]
    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// O total de amostras: arredonda em `f64`, onde todo `f32` de duração é exato.
    #[must_use]
    pub fn frames(&self) -> usize {
        frames_of(self.seconds)
    }
}

/// Segundos em amostras a [`SAMPLE_RATE`].
#[must_use]
pub fn frames_of(seconds: f32) -> usize {
    (f64::from(seconds) * f64::from(SAMPLE_RATE)).round() as usize
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    /// Linear.
    pub gain: f32,
    /// Onde a camada começa, em segundos.
    pub start: f32,
    /// Fora da soma. A semente e o cache das outras camadas não sabem disto.
    pub muted: bool,
    pub source: Source,
    pub envelope: Envelope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    Noise(Noise),
    Tone(Tone),
    Body(Body),
    Recording(Recording),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseFilter {
    /// Sem filtro: `hz` e `q` não mudam o som.
    White,
    Low,
    Band,
    High,
}

/// Ruído semeado por um filtro de estado variável, com o corte indo de `hz` a `hz_end` na duração da camada.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Noise {
    pub filter: NoiseFilter,
    pub hz: f32,
    pub hz_end: f32,
    pub q: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Wave {
    Sine,
    Triangle,
    Saw,
    Square,
    /// Seno modulado em fase: o modulador é `ratio × hz`, e o índice anda de `index` a `index_end` no `glide`.
    Fm {
        ratio: f32,
        index: f32,
        index_end: f32,
    },
}

/// Um oscilador cuja altura anda de `hz` a `hz_end` em `glide` segundos, em curva exponencial.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tone {
    pub wave: Wave,
    pub hz: f32,
    pub hz_end: f32,
    pub glide: f32,
}

/// Como as frequências dos modos se distribuem sobre a base.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModeLaw {
    /// `base × k^stretch`: 1 é corda ideal, perto de 2 é barra rígida.
    Stretched { stretch: f32 },
    /// Barra livre nas duas pontas (Euler-Bernoulli): 1, 2,756, 5,404, 8,933…
    Bar,
    /// Membrana circular (zeros de Bessel). A tabela tem 16 modos.
    Membrane,
}

/// **Corpo vibrante:** um golpe curto excita um banco paralelo de ressonadores amortecidos.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Body {
    pub law: ModeLaw,
    pub base_hz: f32,
    /// Quantos modos, no máximo; os que passam da banda audível da taxa caem fora.
    pub modes: u32,
    /// Quanto o modo da base leva para cair 60 dB, em segundos.
    pub t60: f32,
    /// Quanto os modos agudos morrem antes: 0 é metal, valores altos são madeira.
    pub damping: f32,
    /// A queda do ganho por modo, `1 / k^rolloff`.
    pub rolloff: f32,
    /// A duração do contato, em milissegundos: curto excita os agudos, longo só os graves.
    pub contact_ms: f32,
    /// 0 é só o pulso liso do contato; 1 é só o ruído semeado.
    pub noise: f32,
}

/// Um material gravado, tocado com outra altura.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recording {
    pub take: TakeId,
    pub semitones: f32,
    /// De onde a leitura começa, em segundos do material.
    pub offset: f32,
}
