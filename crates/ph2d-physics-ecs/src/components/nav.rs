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
//! # ⭐ Registados desde a W4, no MESMO commit que as secções do Inspector
//!
//! A lição da vida (plano 28, W3): a cópia de um molde leva **só o que está REGISTADO**, logo um
//! componente por registar não é «ainda não gravável» — é invisível à fábrica, ao `Ctrl+Z` e ao
//! ficheiro de uma vez. E registá-lo sem painel deixaria números no ficheiro que nenhuma linha deixa
//! mexer (gate `every_registered_physics_component_has_a_ui_writer`). ⇒ os dois juntos.
//!
//! ⚠️ O [`NavNow`] **não** se regista, e a ausência é a lei: ele é DERIVADO da memória da ponte.

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
    /// ⭐ **O mais perto que pertence a esta tag** (o `TagId`, com a subárvore) — plano 30, W6.
    /// Em linha recta; empate pela ordem da identidade; o próprio agente não conta.
    NearestTagged(u64),
    /// ⭐ **A PATRULHA pela forma com este `stable_name_id`** — plano 30, W6. Visita os pontos da
    /// forma por ordem: fechada dá voltas, aberta vai e volta. ⚠️ O NOME da forma, nunca os bits.
    Patrol(u64),
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
    /// ⭐ **Desvia dos outros corpos que andam** (plano 30, W5): os outros agentes, o herói, os corpos
    /// dinâmicos — sem sair da área andável. Desligado, ele vai a direito pelo caminho e os OUTROS
    /// desviam-se dele por inteiro. ⚠️ Append-only (o postcard é posicional).
    pub avoidance: bool,
    /// ⭐ (W7) **Evita as zonas que o FEREM** (decisão do dono, plano 30 §11.1): um `Damage` parado
    /// que o `Health` deste agente sente (a resistência ao tipo do dano não o anula) é um FURO no
    /// caminho dele. Desligado — ou imune ao fogo — ele atravessa a lava. ⚠️ Append-only: é o
    /// último campo.
    pub avoid_harm: bool,
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
            avoidance: true,
            avoid_harm: true,
        }
    }
}

/// ⭐ (W7) **Uma ÁREA DE CUSTO** — a lama que atrasa, a zona que nenhum agente pisa. A forma é o
/// colisor desta entidade (sensor ou não), recuada pelo raio de cada agente como um obstáculo.
///
/// ⚠️ **Proibida não é um custo infinito, é um FURO** na malha: as ilhas, o ponto alcançável mais
/// perto e as paredes do desvio leem a malha, e com um custo infinito todos eles mentiriam.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavCostArea {
    /// Quanto custa atravessar, em múltiplos do chão (`1` = o chão; `3` = cada metro aqui vale três,
    /// e o agente dá a volta se a volta custar menos). Abaixo de `1`, um caminho que ele PREFERE.
    pub cost: f32,
    /// Proibida: nenhum agente a pisa.
    pub forbidden: bool,
}

impl Default for NavCostArea {
    fn default() -> Self {
        Self {
            cost: 3.0,
            forbidden: false,
        }
    }
}

/// ⭐ (W7) **Um ATALHO** desta entidade (a entrada) até à que tem o nome [`to`](Self::to) (a saída):
/// um teleporte, ou uma porta por onde só se passa num sentido.
///
/// ⚠️ A porta de um sentido é DOIS componentes: uma [`NavCostArea`] proibida no vão (ninguém o usa
/// para voltar) e um atalho a andar (`teleport = false`) através dele.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavLink {
    /// A saída: quem tem este `stable_name_id`. ⚠️ O NOME e nunca os bits (o undo respawna o mundo).
    pub to: u64,
    /// Também da saída para a entrada.
    pub two_way: bool,
    /// O corpo SALTA para a saída. Desligado, o agente ANDA a direito até ela.
    pub teleport: bool,
    /// O custo a mais de o atravessar, em metros de chão (`0` = só o que ele é).
    pub cost: f32,
    /// O sinal quando um agente o atravessa (vazio = calado).
    pub on_crossed: String,
}

impl Default for NavLink {
    fn default() -> Self {
        Self {
            to: 0,
            two_way: false,
            teleport: true,
            cost: 0.0,
            on_crossed: String::new(),
        }
    }
}

/// ⭐⭐ **Os pontos da PATRULHA de um agente, em mundo** (plano 30, W6) — DERIVADOS da forma que o
/// [`NavTarget::Patrol`] nomeia, pela família (que é quem vê a geometria vectorial: o ECS não a tem).
///
/// ⛔ **NÃO registado**, pela lei do [`NavNow`]: a fonte é a forma desenhada; registado, ele entraria
/// no ficheiro como uma segunda resposta a *«por onde passa a ronda?»*.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct NavRoute {
    /// Os pontos, por ordem, em metros.
    pub points: Vec<[f32; 2]>,
    /// A forma é fechada (a ronda dá voltas) ou aberta (vai e volta).
    pub closed: bool,
}

/// ⭐⭐ **O agente AGORA** — o que a ponte publica no mundo no fim de cada `dispatch`, para quem não
/// alcança a ponte: o Inspector (*«Moving · 3,2 m to go»*).
///
/// ⛔⛔ **DERIVADO e NÃO registado**, pelo precedente do `HealthNow`: a fonte é a memória da ponte,
/// que vai no anel de checkpoints; registado, ele entraria no `.ph2dproj` e no `Ctrl+Z` como uma
/// segunda resposta a *«onde está o agente no caminho?»*, e um undo devolveria o número de um
/// instante com a ponte noutro.
///
/// ⚠️ **Ausente** antes do 1.º tique, e num agente que a ponte SALTA (sem corpo, sem mover, mover a
/// ler o teclado) — *um número de outra corrida lido como o de agora é pior do que nenhum*, e é o
/// Inspector que diz porquê.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct NavNow {
    /// O estado da condução.
    pub status: ph2d_nav::Status,
    /// O que falta andar pelo caminho, em metros.
    pub remaining: f32,
    /// O raio com que ele procura o caminho — o autorado, ou o DERIVADO do colisor quando é `0`.
    pub radius: f32,
    /// ⭐ A ordem de um verbo nesta corrida — `None` = nenhum falou (vale o *Active* autorado);
    /// `Some(true)` = um `Start`; `Some(false)` = um `Stop`. Sem ela o Inspector dizia *«Switched
    /// off»* de um agente desligado que um `Start` pôs a andar.
    pub ordem: Option<bool>,
    /// O alvo que um `Start` com nome lhe deu (`stable_name_id`; `0` = o autorado).
    pub alvo_da_ordem: u64,
}
