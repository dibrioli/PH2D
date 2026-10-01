//! **A NAVEGAÇÃO** (plano 30, W3) — a REGIÃO onde se anda e o AGENTE que acha o caminho sozinho.
//!
//! Os dois são CONFIG (ADR-0131: *«o undo ordena por bytes»*): a malha andável é DERIVADA dos
//! colisores estáticos e nunca gravada, e o caminho de um agente é estado VIVO que mora na ponte e
//! entra no anel de checkpoints. A lei vive em duas folhas — [`ph2d_nav`] (o caminho mais curto e a
//! condução) e `ph2d-navmesh` (a malha a partir dos obstáculos).
//!
//! # ⚠️ O agente não ANDA — ele PEDE
//!
//! Quem move o corpo é o [`super::TopDownPlayer`] da mesma entidade, que acelera, trava e desliza
//! na parede (plano 30 §2.1). O agente escreve a INTENÇÃO dele pelo mesmo canal que o teclado usa
//! (`PhysicsBridge::set_player_input`), e por isso o mover tem de estar com `default_controls`
//! desligado — senão os dois falam ao mesmo tempo. ⛔ Um mover próprio da navegação seria uma
//! segunda lei de aceleração e de deslize, e a do #13 já foi medida contra o oráculo.
//!
//! # ⛔ Ainda NÃO registados (e a ausência é a lei da casa)
//!
//! Um componente registado sem quem o escreva no Inspector é um campo que o artista não alcança
//! (gate `every_registered_physics_component_has_a_ui_writer`). Eles entram no registo na W4, no
//! mesmo commit que as secções do painel; até lá só as cenas os constroem.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **A região onde os agentes andam** — um rectângulo centrado no `Transform` desta entidade.
///
/// ⚠️ **Os obstáculos não se autoram aqui**: são os corpos ESTÁTICOS, não sensores, cuja camada
/// esteja em [`obstacle_layers`](Self::obstacle_layers). Uma parede é uma parede porque colide, e
/// uma segunda lista de «o que bloqueia» divergiria da física no primeiro dia.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavRegion {
    /// Meia-largura e meia-altura da região, em metros.
    pub half_extents: [f32; 2],
    /// As camadas de colisão cujos corpos estáticos bloqueiam (bit `i` = camada `i`).
    pub obstacle_layers: u8,
}

impl Default for NavRegion {
    fn default() -> Self {
        Self {
            half_extents: [10.0, 10.0],
            obstacle_layers: u8::MAX,
        }
    }
}

impl NavRegion {
    /// O rectângulo da região, em mundo, a partir do centro dela.
    #[must_use]
    pub fn rect(&self, center: [f32; 2]) -> [[f32; 2]; 2] {
        let [hx, hy] = [self.half_extents[0].abs(), self.half_extents[1].abs()];
        [
            [center[0] - hx, center[1] - hy],
            [center[0] + hx, center[1] + hy],
        ]
    }
}

/// **Para onde um agente vai.**
///
/// ⚠️ **Append-only**: o postcard guarda a variante pela POSIÇÃO.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum NavTarget {
    /// Sem alvo — o agente fica parado.
    #[default]
    None,
    /// Quem tem este `stable_name_id`. ⚠️⚠️ **O NOME e nunca os bits da entidade** — o undo
    /// respawna o mundo com bits novos (a lei do `ProjectileMotion::homing_target`).
    Named(u64),
    /// Um ponto fixo do mundo, em metros.
    Point([f32; 2]),
}

/// **Um agente que acha o caminho sozinho.** Ver o cabeçalho do módulo.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavAgent {
    /// Para onde ele vai.
    pub target: NavTarget,
    /// O raio do corpo para efeitos de caminho, em metros. ⚠️ **`0` é DERIVADO do colisor**
    /// (o círculo que o envolve) — é o caso comum, e escrever o número à mão é para um corpo cuja
    /// forma não é a que deve passar nas portas.
    pub radius: f32,
    /// A esta distância do alvo ele pára e conta como chegado.
    pub arrive_distance: f32,
    /// O alvo tem de andar mais do que isto para valer um recálculo.
    pub repath_distance: f32,
    /// Sem progresso durante isto (segundos), ele está PRESO e recalcula. `0` desliga.
    pub stuck_after_s: f32,
    /// Desligado, o agente fica parado e esquece o caminho.
    pub active: bool,
    /// O sinal ao CHEGAR (vazio = calado).
    pub on_arrived: String,
    /// O sinal quando não há caminho — ou só um PARCIAL, até ao ponto mais perto (vazio = calado).
    pub on_no_path: String,
    /// O sinal ao ficar PRESO (vazio = calado).
    pub on_stuck: String,
}

impl Default for NavAgent {
    fn default() -> Self {
        Self {
            target: NavTarget::None,
            radius: 0.0,
            arrive_distance: 0.1,
            repath_distance: 0.5,
            stuck_after_s: 1.0,
            active: true,
            on_arrived: String::new(),
            on_no_path: String::new(),
            on_stuck: String::new(),
        }
    }
}
