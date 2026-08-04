//! **O componente do PLAYER DE PLATAFORMA** (W2) — a config autorada.
//!
//! Módulo irmão dos outros de `components/` pelo motivo de sempre (LOC e
//! isolamento), e a fronteira dele é a regra mais importante desta wave:
//!
//! # ⚠️ CONFIG, nunca estado vivo
//!
//! Um controlador de plataforma carrega estado por-tick — `is_grounded`, o
//! coyote timer, o jump buffer. **Nada disso mora aqui.** O `canonicalize` do
//! undo ordena as entidades pelos BYTES dos componentes, então um campo que
//! muda por tick faria **cada frame virar um passo de undo** (o ADR-0131 escreve
//! essa lei, e o `PhysicsJoint` já a honra).
//!
//! O estado vivo mora na PONTE, ao lado do `grab` — e a partir da W7 ele deixa
//! de ser guardado e passa a ser **derivado da fita de entrada**, que é o que o
//! torna reproduzível num scrub.

use bevy_ecs::component::Component;
use ph2d_platformer::{JumpConfig, PlayerConfig, ReactionConfig, RideConfig, WalkConfig};
use serde::{Deserialize, Serialize};

/// **Este corpo é um player de plataforma.**
///
/// Presença = o comportamento existe; os campos são os ganhos da perna
/// ([`RideConfig`], a lei pura). Ausente é o mundo de antes desta wave, byte a
/// byte — nenhum corpo sem o componente muda de trajetória.
///
/// ⚠️ **Só faz sentido num corpo `Dynamic`**, e por FÍSICA, não por gosto: a
/// mola é um impulso, e um impulso não move um corpo estático nem um kinematic
/// (massa infinita — o fato que a W-BakeJoint mediu e que faz a MÃO recusar os
/// dois). A row do Inspector será oferecida para Dynamic apenas, e a ponte
/// recusa em silêncio o resto.
///
/// ⚠️ **Componente NOVO ⇒ blob-key própria ⇒ `PROJECT_SCHEMA` NÃO bumpa** (o
/// precedente do `PhysicsJoint`/W3): um arquivo antigo simplesmente não o tem, e
/// o load o deixa ausente. O que bumpa é apendar campo a um componente que já
/// existe, porque o postcard é posicional.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlatformPlayer {
    /// A que altura o personagem paira, medida do CENTRO do corpo para baixo.
    pub float_height: f32,
    /// Quanto acima da altura de repouso a mola ainda age (ver [`RideConfig`]).
    pub cling_distance: f32,
    /// Rigidez da perna, em aceleração-por-metro.
    pub spring_strength: f32,
    /// Amortecimento, em fração da velocidade relativa removida por tick.
    ///
    /// ⚠️ Tem TETO medido ([`RideConfig::MAX_DAMPING`]) — acima dele o boost
    /// inverte a velocidade em vez de matá-la, e o personagem pipoca.
    pub spring_damping: f32,

    /// Velocidade de cruzeiro, m/s — **relativa ao chão** (W3).
    pub speed: f32,
    /// Aceleração no chão, m/s².
    pub acceleration: f32,
    /// Aceleração no ar — o controle aéreo. `0` conserva o arco do salto.
    pub air_acceleration: f32,
    /// A inclinação máxima em que o personagem fica de pé, em GRAUS.
    ///
    /// ⚠️ Graus na fronteira, cosseno no motor — a mesma política do ângulo de
    /// joint (graus na UI, radianos no componente): o artista autora o número
    /// que ele entende, e a conversão acontece **uma vez**, na porta única
    /// [`WalkConfig::max_slope_cos`].
    pub max_slope_deg: f32,

    /// A altura de um pulo COMPLETO, metros — **com gravidade neutra** (W4).
    pub jump_height: f32,
    /// Multiplicador de gravidade na saída, acima de [`Self::takeoff_speed`].
    pub takeoff_gravity: f32,
    /// A velocidade acima da qual a gravidade de saída age, m/s.
    pub takeoff_speed: f32,
    /// Multiplicador perto do ápice — ⚠️ **abaixo de 1 ALONGA** (a decisão do
    /// módulo: o *forgiveness* do Celeste, não o *snappiness* do tnua).
    pub peak_gravity: f32,
    /// A janela do ápice, m/s.
    pub peak_speed: f32,
    /// Multiplicador na queda — acima de 1 desce mais rápido do que sobe.
    pub fall_gravity: f32,
    /// Multiplicador enquanto sobe com o botão SOLTO — a altura variável.
    pub cut_gravity: f32,

    /// **COYOTE TIME** (W8) — segundos de perdão depois de sair do chão.
    pub coyote_time: f32,
    /// **JUMP BUFFER** (W8) — segundos que um aperto cedo demais sobrevive.
    pub jump_buffer: f32,

    /// **Quanto do PESO volta para o chão** (W6). `1` transmite inteiro.
    ///
    /// ⚠️ Em `0` o personagem é um FANTASMA: ele paira sobre uma jangada sem a
    /// afundar e uma balança não o pesa. O default é `1` porque a física é essa;
    /// o número existe para desligar, não para ligar.
    pub reaction_support: f32,
    /// **Quanto da CAMINHADA volta para o chão** (W6). Nasce em `0`.
    ///
    /// ⚠️ O default oposto ao irmão é de PRODUTO, não de física: com ele em `1`
    /// a plataforma escorrega para trás como um tapete quando o personagem anda
    /// em cima dela — atrito honesto e péssimo de jogar.
    pub reaction_movement: f32,
}

impl PlatformPlayer {
    /// A config da lei que este componente descreve.
    ///
    /// Existe para que a ponte **não** remonte o `PlayerConfig` campo a campo:
    /// duas cópias da mesma tradução divergem no dia em que um campo novo entra
    /// só numa delas.
    #[must_use]
    pub fn config(&self) -> PlayerConfig {
        PlayerConfig {
            ride: RideConfig {
                float_height: self.float_height,
                cling_distance: self.cling_distance,
                spring_strength: self.spring_strength,
                spring_damping: self.spring_damping,
            },
            walk: WalkConfig {
                speed: self.speed,
                acceleration: self.acceleration,
                air_acceleration: self.air_acceleration,
                max_slope_deg: self.max_slope_deg,
            },
            jump: JumpConfig {
                jump_height: self.jump_height,
                takeoff_gravity: self.takeoff_gravity,
                takeoff_speed: self.takeoff_speed,
                peak_gravity: self.peak_gravity,
                peak_speed: self.peak_speed,
                fall_gravity: self.fall_gravity,
                cut_gravity: self.cut_gravity,
                coyote_time: self.coyote_time,
                jump_buffer: self.jump_buffer,
            },
            react: ReactionConfig {
                support: self.reaction_support,
                movement: self.reaction_movement,
            },
        }
    }
}

impl Default for PlatformPlayer {
    /// ⚠️ **Ponto de partida, não default de produto** — os números que shipam
    /// saem da varredura da wave, com a tabela ao lado (CLAUDE.md §0).
    fn default() -> Self {
        let c = PlayerConfig::STARTING_POINT;
        Self {
            float_height: c.ride.float_height,
            cling_distance: c.ride.cling_distance,
            spring_strength: c.ride.spring_strength,
            spring_damping: c.ride.spring_damping,
            speed: c.walk.speed,
            acceleration: c.walk.acceleration,
            air_acceleration: c.walk.air_acceleration,
            max_slope_deg: c.walk.max_slope_deg,
            jump_height: c.jump.jump_height,
            takeoff_gravity: c.jump.takeoff_gravity,
            takeoff_speed: c.jump.takeoff_speed,
            peak_gravity: c.jump.peak_gravity,
            peak_speed: c.jump.peak_speed,
            fall_gravity: c.jump.fall_gravity,
            cut_gravity: c.jump.cut_gravity,
            coyote_time: c.jump.coyote_time,
            jump_buffer: c.jump.jump_buffer,
            reaction_support: c.react.support,
            reaction_movement: c.react.movement,
        }
    }
}
