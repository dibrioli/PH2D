//! **O conjunto de sons de um objeto e o sorteio de qual toca agora.** Folha sem FFT e sem áudio: guarda
//! NOMES de sons do projeto, nunca caminhos, e a lei do sorteio é uma só para a cena e para o Audio Editor.
//!
//! Só `SceneAudio::play` sorteia na cena. O estado do sorteio ([`Picker`]) vive fora do componente: ele anda
//! a cada disparo, e um componente registrado que anda vira passo de undo.

#[cfg(feature = "component")]
use bevy_ecs::component::Component;
use ph2d_det_math::DetMath;
use serde::{Deserialize, Serialize};

/// Como o próximo som é escolhido.
///
/// ⛔ A ORDEM é o formato gravado: o postcard guarda a posição da variante. Só acrescente no fim.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PickStrategy {
    /// Sorteio por peso a cada disparo.
    Random,
    /// Um depois do outro, na ordem da lista; o peso não conta.
    Sequence,
    /// Baralho: todos saem uma vez, por peso, antes de algum repetir.
    #[default]
    Shuffle,
}

impl PickStrategy {
    /// As estratégias na ordem do seletor.
    pub const ALL: [PickStrategy; 3] = [
        PickStrategy::Random,
        PickStrategy::Sequence,
        PickStrategy::Shuffle,
    ];

    /// ⛔ O nome do manifesto do Audio Editor, lido de volta de arquivo: não é o rótulo, e não se traduz.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PickStrategy::Random => "Random",
            PickStrategy::Sequence => "Sequence",
            PickStrategy::Shuffle => "Shuffle",
        }
    }

    /// A chave da palavra que o artista lê, resolvida por quem pinta.
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            PickStrategy::Random => "audio.pick.random",
            PickStrategy::Sequence => "audio.pick.sequence",
            PickStrategy::Shuffle => "audio.pick.shuffle",
        }
    }

    /// Lê um nome de manifesto (desconhecido → `None`).
    #[must_use]
    pub fn from_name(s: &str) -> Option<PickStrategy> {
        PickStrategy::ALL.into_iter().find(|k| k.name() == s)
    }

    /// A posição desta estratégia em [`PickStrategy::ALL`]: a etiqueta do seletor.
    #[must_use]
    pub fn index(self) -> u8 {
        PickStrategy::ALL
            .iter()
            .position(|o| *o == self)
            .and_then(|i| u8::try_from(i).ok())
            .unwrap_or(0)
    }

    /// A estratégia a `delta` passos desta em [`PickStrategy::ALL`], dando a volta.
    #[must_use]
    pub fn cycled(self, delta: i32) -> PickStrategy {
        let n = PickStrategy::ALL.len() as i32;
        let i = PickStrategy::ALL
            .iter()
            .position(|k| *k == self)
            .unwrap_or(0) as i32;
        PickStrategy::ALL[(i + delta).rem_euclid(n) as usize]
    }

    /// Um manifesto antigo do Audio Editor não guarda «sem repetir»: `Shuffle` nunca repete o último.
    #[must_use]
    pub fn no_repeat_of_the_editor(self) -> bool {
        self == PickStrategy::Shuffle
    }
}

/// Faixa do peso. Abaixo do piso conta como o piso, então um sorteio nunca divide por zero.
pub const WEIGHT_RANGE: (f32, f32) = (0.01, 16.0);
/// Variação máxima por disparo, em semitons e em decibéis.
pub const MAX_JITTER: f32 = 24.0;

/// Um som do conjunto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundEntry {
    /// O NOME na `ProjectSounds`. Nunca um caminho.
    pub name: String,
    pub weight: f32,
    /// Um som desligado fica na lista e não sai no sorteio.
    pub enabled: bool,
}

impl SoundEntry {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            weight: 1.0,
            enabled: true,
        }
    }
}

/// **Os sons de um objeto.** Com ao menos um som ligado, ele manda em vez do `sound` do `AudioSource2D`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "component", derive(Component))]
pub struct SoundSet {
    pub sounds: Vec<SoundEntry>,
    pub strategy: PickStrategy,
    /// Nunca o mesmo som duas vezes seguidas, quando há dois ou mais ligados.
    pub no_repeat: bool,
    /// Variação de altura por disparo, `± semitons`.
    pub pitch_jitter_semitones: f32,
    /// Variação de volume por disparo, `± dB`.
    pub gain_jitter_db: f32,
}

impl Default for SoundSet {
    fn default() -> Self {
        Self {
            sounds: Vec::new(),
            strategy: PickStrategy::default(),
            no_repeat: true,
            pitch_jitter_semitones: 0.0,
            gain_jitter_db: 0.0,
        }
    }
}

impl SoundSet {
    /// O que o sorteio olha de cada som.
    #[must_use]
    pub fn candidates(&self) -> Vec<Candidate> {
        self.sounds
            .iter()
            .map(|s| Candidate {
                weight: s.weight,
                enabled: s.enabled && !s.name.trim().is_empty(),
            })
            .collect()
    }

    /// Tem algum som que possa sair?
    #[must_use]
    pub fn has_sounds(&self) -> bool {
        self.candidates().iter().any(|c| c.enabled)
    }

    /// Põe `name` na lista. Numa lista vazia o `antigo` (o `sound` de sempre) entra primeiro, para o objeto
    /// não perder o som que já tinha. `false` = o nome é vazio ou já está na lista.
    pub fn add_keeping(&mut self, antigo: &str, name: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.sounds.iter().any(|e| e.name == name) {
            return false;
        }
        let antigo = antigo.trim();
        if self.sounds.is_empty() && !antigo.is_empty() && antigo != name {
            self.sounds.push(SoundEntry::new(antigo));
        }
        self.sounds.push(SoundEntry::new(name));
        true
    }

    /// Tira o som `i`. `false` = índice fora.
    pub fn remove_at(&mut self, i: usize) -> bool {
        let dentro = i < self.sounds.len();
        if dentro {
            self.sounds.remove(i);
        }
        dentro
    }

    /// Liga ou desliga o som `i`. `false` = índice fora.
    pub fn set_enabled(&mut self, i: usize, on: bool) -> bool {
        self.sounds.get_mut(i).map(|e| e.enabled = on).is_some()
    }

    /// O peso do som `i`. `false` = índice fora.
    pub fn set_weight(&mut self, i: usize, v: f32) -> bool {
        self.sounds.get_mut(i).map(|e| e.weight = v).is_some()
    }

    /// A estratégia na posição `tag` de [`PickStrategy::ALL`]. `false` = fora da tabela.
    pub fn set_strategy_at(&mut self, tag: usize) -> bool {
        PickStrategy::ALL
            .get(tag)
            .map(|k| self.strategy = *k)
            .is_some()
    }

    /// Ao lado das outras edições, que dizem se mudaram algo. `true` sempre.
    pub fn set_no_repeat(&mut self, on: bool) -> bool {
        self.no_repeat = on;
        true
    }

    /// A variação de altura. `true` sempre.
    pub fn set_pitch_jitter(&mut self, semitones: f32) -> bool {
        self.pitch_jitter_semitones = semitones;
        true
    }

    /// A variação de volume. `true` sempre.
    pub fn set_gain_jitter(&mut self, db: f32) -> bool {
        self.gain_jitter_db = db;
        true
    }

    /// A lei dos números: a mesma para o cartão, para o script e para arquivo adulterado.
    pub fn sanitize(&mut self) {
        let livre = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, MAX_JITTER) // CLAMP-OK: a faixa autorada da variação
            } else {
                0.0
            }
        };
        self.pitch_jitter_semitones = livre(self.pitch_jitter_semitones);
        self.gain_jitter_db = livre(self.gain_jitter_db);
        for s in &mut self.sounds {
            s.name = s.name.trim().to_owned();
            s.weight = if s.weight.is_finite() {
                s.weight.clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1) // CLAMP-OK: a faixa autorada do peso
            } else {
                1.0
            };
        }
    }
}

/// O que o sorteio precisa saber de um som.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub weight: f32,
    pub enabled: bool,
}

/// O desvio de um disparo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jitter {
    /// Multiplicador da velocidade de leitura (`1` = altura original).
    pub pitch: f32,
    /// Ganho linear (`1` = sem mudança).
    pub gain: f32,
}

impl Jitter {
    pub const NONE: Jitter = Jitter {
        pitch: 1.0,
        gain: 1.0,
    };
}

/// **O estado do sorteio.** Determinístico: a mesma semente dá a mesma sequência.
#[derive(Debug, Clone)]
pub struct Picker {
    rng: u64,
    /// O cursor de `Sequence`, dentro dos sons ligados.
    seq: usize,
    last: Option<usize>,
    /// O que ainda não saiu nesta volta de `Shuffle`.
    bag: Vec<usize>,
}

impl Default for Picker {
    fn default() -> Self {
        Picker::with_seed(0x9E37_79B9_7F4A_7C15)
    }
}

impl Picker {
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: seed,
            seq: 0,
            last: None,
            bag: Vec::new(),
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn next_unit(&mut self) -> f32 {
        const ESCALA: f32 = 1.0 / (1u32 << 24) as f32;
        ((self.next_u64() >> 40) as u32) as f32 * ESCALA
    }

    /// **O índice do próximo som**, ou `None` se nenhum está ligado.
    ///
    /// Com `no_repeat` o último som sai da urna antes do sorteio: a garantia não depende do peso, nem de
    /// repetir o sorteio um número de vezes.
    pub fn pick(
        &mut self,
        sons: &[Candidate],
        strategy: PickStrategy,
        no_repeat: bool,
    ) -> Option<usize> {
        let ligados: Vec<usize> = (0..sons.len()).filter(|&i| sons[i].enabled).collect();
        let Some(&primeiro) = ligados.first() else {
            self.last = None;
            self.bag.clear();
            return None;
        };
        if ligados.len() == 1 {
            self.last = Some(primeiro);
            self.seq = 0;
            self.bag.clear();
            return Some(primeiro);
        }
        let barrar = if no_repeat { self.last } else { None };
        let escolhido = match strategy {
            PickStrategy::Sequence => {
                let i = self.seq % ligados.len();
                self.seq = (i + 1) % ligados.len();
                ligados[i]
            }
            PickStrategy::Random => {
                let urna: Vec<usize> = ligados
                    .iter()
                    .copied()
                    .filter(|&i| Some(i) != barrar)
                    .collect();
                self.por_peso(sons, &urna)
            }
            PickStrategy::Shuffle => {
                // Um som que foi desligado ou removido sai do baralho.
                self.bag.retain(|i| ligados.contains(i));
                if self.bag.is_empty() {
                    self.bag.clone_from(&ligados);
                }
                let urna: Vec<usize> = self
                    .bag
                    .iter()
                    .copied()
                    .filter(|&i| Some(i) != barrar)
                    .collect();
                // O baralho só tem o último ⇒ ele fica para depois, e sai um da volta seguinte.
                let urna = if urna.is_empty() {
                    let resto: Vec<usize> = ligados
                        .iter()
                        .copied()
                        .filter(|&i| Some(i) != barrar)
                        .collect();
                    self.bag.clone_from(&ligados);
                    resto
                } else {
                    urna
                };
                let e = self.por_peso(sons, &urna);
                self.bag.retain(|&i| i != e);
                e
            }
        };
        self.last = Some(escolhido);
        Some(escolhido)
    }

    fn por_peso(&mut self, sons: &[Candidate], urna: &[usize]) -> usize {
        let peso = |i: usize| sons[i].weight.clamp(WEIGHT_RANGE.0, WEIGHT_RANGE.1); // CLAMP-OK: peso de arquivo
        let total: f32 = urna.iter().map(|&i| peso(i)).sum();
        let mut r = self.next_unit() * total;
        for &i in urna {
            let w = peso(i);
            if r < w {
                return i;
            }
            r -= w;
        }
        urna[urna.len() - 1] // arredondamento
    }

    /// **O desvio de um disparo**: dois sorteios independentes, `2^(±st/12)` e `10^(±dB/20)`.
    pub fn jitter(&mut self, pitch_semitones: f32, gain_db: f32) -> Jitter {
        let bipolar = |p: &mut Self| p.next_unit() * 2.0 - 1.0;
        let st = pitch_semitones.clamp(0.0, MAX_JITTER) * bipolar(self); // CLAMP-OK: faixa autorada
        let db = gain_db.clamp(0.0, MAX_JITTER) * bipolar(self); // CLAMP-OK: faixa autorada
        Jitter {
            pitch: 2.0_f32.det_powf(st / 12.0),
            gain: 10.0_f32.det_powf(db / 20.0),
        }
    }
}

#[cfg(test)]
mod tests;
