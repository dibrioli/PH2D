//! **O componente do PROJÉCTIL** (TOP-20 #14) — CONFIG, nunca estado vivo de solver
//! (ADR-0131: *«o undo ordena por bytes»*).
//!
//! A lei dele vive na crate-folha [`ph2d_projectile`]; aqui só está o que o artista autora e o que
//! o ficheiro guarda. Plano: `docs/Components/11_plano_projectile_motion.md`.
//!
//! # ⛔ Por que ele não é um corpo DINÂMICO com restituição
//!
//! **MEDIDO** antes de uma linha ser escrita (`tests/it/mede_o_que_a_composicao_ja_da.rs`): um
//! corpo dinâmico com `restitution = 1` dos dois lados **já ricocheteia exactamente** — razão
//! `1,000` e o `vx` do espelho ao terceiro decimal em todos os ângulos. ⇒ *o ricochete não é a
//! razão de este componente existir.*
//!
//! ⭐ A razão é a linha seguinte da mesma tabela: o **mesmo** tiro contra uma **caixa leve** sai
//! com `10,252` em vez de `12,001` e por outro caminho. Um projéctil dinâmico é um **participante**
//! da física — ele empurra o que toca e paga por isso. Uma bala de arcade não tem massa.
//!
//! ⇒ ele é **CINEMÁTICO**, como os dois controladores irmãos, e ganha de graça: massa irrelevante,
//! determinismo cross-OS, e anti-túnel por construção (o `move_character_from` é um *shape-cast*).
//!
//! # ⚠️ O que ele guarda, e o que ele NÃO guarda
//!
//! A velocidade, os metros percorridos e os saltos gastos **mudam por tique** ⇒ vivem no
//! `ProjectileState`, dentro do `ControllerMemory` que entra no anel de checkpoints. ⛔ Aqui
//! estariam a fazer o undo desta casa ver **cada quadro como um passo**.

use bevy_ecs::prelude::Component;
use ph2d_projectile::ProjectileLaw;
use serde::{Deserialize, Serialize};

/// **Um projéctil de arcade.** Ver o cabeçalho do módulo.
///
/// ⚠️ **Componente registado NOVO ⇒ `PROJECT_SCHEMA` sobe.** Um `ComponentBlob` de `type_id`
/// desconhecido **recusa o load inteiro**, e o degrau é o que transforma isso em *«este ficheiro é
/// de outra versão»* em vez de *«type id desconhecido»* a meio da travessia.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectileMotion {
    /// A rapidez com que ele nasce, m/s, na direcção para que o corpo está virado.
    pub initial_speed: f32,
    /// Aceleração ao longo da direcção de voo, m/s². Negativa trava.
    pub acceleration: f32,
    /// Tecto de rapidez, m/s. ⚠️ **`0` é SEM TECTO**, não «parado».
    pub max_speed: f32,
    /// A gravidade que faz o arco, m/s² para baixo. `0` = tiro recto.
    pub gravity: f32,
    /// A fracção da rapidez que sobrevive a um ricochete.
    pub bounciness: f32,
    /// Quantos ricochetes o voo aguenta. ⚠️ **`0` = acaba no primeiro toque.**
    pub max_bounces: u8,
    /// Metros **percorridos** até o voo acabar. ⚠️ **`0` é SEM LIMITE.**
    pub range: f32,
    /// A flecha aponta para onde voa.
    pub face_velocity: bool,
    /// Aceleração de perseguição, m/s². `0` = não persegue.
    pub homing_accel: f32,
    /// **Quem ele persegue** — o `stable_name_id` do alvo, `0` = ninguém.
    ///
    /// ⚠️⚠️ **É o NOME e nunca os bits da entidade** (lei 4 da síntese, e a lei do repo): o undo
    /// respawna o mundo inteiro com bits novos, e um alvo guardado em bits apontaria para outra
    /// coisa — ou para nada — no primeiro `Ctrl+Z`.
    pub homing_target: u64,
}

impl Default for ProjectileMotion {
    fn default() -> Self {
        Self::from_law(ProjectileLaw::default(), 0)
    }
}

impl ProjectileMotion {
    /// **A porta ÚNICA** componente ⇒ lei.
    ///
    /// ⚠️ Ela existe para que a tradução tenha **um** sítio: a ponte, o painel e os gates leem
    /// daqui, e uma cópia solta num deles seria a segunda resposta à mesma pergunta.
    #[must_use]
    pub fn law(&self) -> ProjectileLaw {
        ProjectileLaw {
            initial_speed: self.initial_speed,
            acceleration: self.acceleration,
            max_speed: self.max_speed,
            gravity: self.gravity,
            bounciness: self.bounciness,
            max_bounces: self.max_bounces,
            range: self.range,
            face_velocity: self.face_velocity,
            homing_accel: self.homing_accel,
        }
    }

    /// O inverso — usado pelo default e pelo painel ao escrever.
    #[must_use]
    pub fn from_law(l: ProjectileLaw, homing_target: u64) -> Self {
        Self {
            initial_speed: l.initial_speed,
            acceleration: l.acceleration,
            max_speed: l.max_speed,
            gravity: l.gravity,
            bounciness: l.bounciness,
            max_bounces: l.max_bounces,
            range: l.range,
            face_velocity: l.face_velocity,
            homing_accel: l.homing_accel,
            homing_target,
        }
    }
}

#[cfg(test)]
#[path = "projectile_tests.rs"]
mod tests;
