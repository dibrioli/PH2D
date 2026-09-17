//! ⭐⭐⭐ **O EMISSOR DE PARTÍCULAS de um objecto** — o item **#18** do TOP-20
//! (`docs/Components/14_plano_particle_emitter.md`).
//!
//! Isto é **só CONFIG** — o que o artista escolhe, gravado. A simulação é a do Motion
//! (`motion.emitter` + o laço de forças), e quem a monta e corre é a crate `ph2d-particles`: um
//! componente não carrega um segundo motor de partículas.
//!
//! # O modelo é o do oráculo (Godot 4.7.2, MIT)
//!
//! `amount` partículas por **ciclo**, e um ciclo dura uma **vida**: contínuo = `amount / life` por
//! segundo; rajada única (`one_shot`) = `amount` de uma vez, espalhadas por `(1 − explosiveness)`
//! do ciclo. `amount` é também o **tecto** de vivas ao mesmo tempo (o `max` do nó). O relógio foi
//! medido sem interface (`docs/Components/ferramentas/godot_particles_probe.gd`) e é a lei do
//! `ph2d-particles`.
//!
//! ⚠️ **Os tectos são os do NÓ, não daqui** (§0.0): o `amount` até ao `MAX_ALIVE` do emissor, e as
//! faixas do painel derivadas do `params_ui` dele.
//!
//! # ⛔ O que anda NÃO está aqui
//!
//! As partículas vivas, o relógio local, os segmentos ligado/desligado — tudo isso é da corrida e
//! vive na ponte, fora do mundo. Um estado vivo registado faria cada tique virar um passo de
//! `Ctrl+Z` (a lei do `TimerRuntime`).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **De onde nascem** — as quatro formas do `motion.emitter`, **na ordem do índice dele**
/// (`shape_mode`). ⚠️ A ordem é o contrato com o nó; acrescentar é no fim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmissionShape {
    /// Um ponto — a origem do objecto.
    #[default]
    Point,
    /// Um disco cheio de raio `shape_size`.
    Disc,
    /// Um anel de raio `shape_size`.
    Ring,
    /// Um rectângulo de meias-dimensões `shape_size`.
    Rect,
}

impl EmissionShape {
    /// Todas, na ordem do índice do nó.
    pub const ALL: [Self; 4] = [Self::Point, Self::Disc, Self::Ring, Self::Rect];

    /// O índice que o `shape_mode` do nó lê.
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Self::Point => 0,
            Self::Disc => 1,
            Self::Ring => 2,
            Self::Rect => 3,
        }
    }
}

/// **Onde as partículas vivem** — a pergunta *«se o objecto anda, o penacho vai com ele?»*.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleSpace {
    /// **Ficam onde nasceram** — mover o objecto deixa um rasto (o default do oráculo:
    /// `local_coords = false`). A partícula guarda a pose do objecto no **instante do nascimento**.
    #[default]
    World,
    /// **Andam com o objecto** — o penacho inteiro é transformado pela pose de agora (a chama
    /// presa à tocha).
    Local,
}

/// **O emissor de partículas de um objecto** — o componente registado, e **CONFIG**.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParticleEmitter {
    /// **Começa a emitir quando a corrida começa?** (o `emitting` do oráculo no nascimento.)
    pub emitting: bool,
    /// **Uma rajada só**, e depois pára (L1).
    pub one_shot: bool,
    /// Partículas por ciclo — e o tecto de vivas ao mesmo tempo.
    pub amount: u32,
    /// Segundos de vida de cada partícula — e a duração de um ciclo.
    pub life: f32,
    /// Quanto a vida pode ENCURTAR, em fracção de `life` (`0..1`).
    pub life_random: f32,
    /// Quanto do ciclo a emissão se aperta: `0` = espalhada pelo ciclo, `1` = tudo de uma vez.
    pub explosiveness: f32,
    /// Segundos de simulação corridos ANTES do primeiro quadro (L3).
    pub prewarm: f32,
    /// A velocidade do relógio das partículas (L4). `1` = o tempo do jogo.
    pub time_scale: f32,
    /// A semente — o mesmo número dá as mesmas partículas.
    pub seed: u32,
    /// De onde nascem.
    pub shape: EmissionShape,
    /// O tamanho da forma: raio (`Disc`/`Ring`, só o `x`) ou meias-dimensões (`Rect`), em metros.
    pub shape_size: [f32; 2],
    /// Velocidade de saída, em metros por segundo.
    pub speed: f32,
    /// Quanto a velocidade varia por partícula, em fracção de `speed`.
    pub speed_random: f32,
    /// A direcção de saída em graus (`90` = para cima), relativa ao objecto.
    pub angle: f32,
    /// A abertura do cone, em graus.
    pub spread: f32,
    /// A gravidade, em metros por segundo² (Y para cima; `[0, -9.8]` puxa para baixo).
    pub gravity: [f32; 2],
    /// O amortecimento — quanto a velocidade se perde por segundo (`0` = nada).
    pub damping: f32,
    /// O lado de cada partícula, em metros.
    pub size: f32,
    /// Quanto o tamanho varia por partícula, em fracção de `size`.
    pub size_random: f32,
    /// O tamanho ao MORRER, em fracção do tamanho ao nascer (`1` = não muda).
    pub size_end: f32,
    /// A cor ao nascer (RGBA).
    pub color: [f32; 4],
    /// A cor ao morrer (RGBA) — a do nascimento dá uma cor única.
    pub color_end: [f32; 4],
    /// Onde as partículas vivem.
    pub space: ParticleSpace,
    /// O sinal que LIGA a emissão (vazio = nenhum).
    pub start_on: String,
    /// O sinal que DESLIGA a emissão — as vivas acabam a vida (L5).
    pub stop_on: String,
    /// O sinal que RECOMEÇA do zero — as vivas somem (L7).
    pub restart_on: String,
    /// O sinal que o emissor GRITA quando a emissão acabou e a última partícula morreu (L2).
    pub finished_signal: String,
}

impl Default for ParticleEmitter {
    /// Um jacto pequeno e visível: 16 partículas por segundo, para cima, a cair de volta.
    fn default() -> Self {
        Self {
            emitting: true,
            one_shot: false,
            amount: 16,
            life: 1.0,
            life_random: 0.0,
            explosiveness: 0.0,
            prewarm: 0.0,
            time_scale: 1.0,
            seed: 0,
            shape: EmissionShape::Point,
            shape_size: [0.5, 0.5],
            speed: 4.0,
            speed_random: 0.0,
            angle: 90.0,
            spread: 30.0,
            gravity: [0.0, -9.8],
            damping: 0.0,
            size: 0.15,
            size_random: 0.0,
            size_end: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            color_end: [1.0, 1.0, 1.0, 1.0],
            space: ParticleSpace::World,
            start_on: String::new(),
            stop_on: String::new(),
            restart_on: String::new(),
            finished_signal: String::new(),
        }
    }
}
