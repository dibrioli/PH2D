#![forbid(unsafe_code)]
//! **A LEI de um player de VISTA DE CIMA** — pura, sem rapier, sem ECS, sem shell.
//!
//! Dado *(config, a entrada, a velocidade de agora, dt)*, esta crate responde **que
//! velocidade o corpo passa a ter**, **para onde ele olha**, e **em que direcções
//! pedir deslocamento ao mundo**. Quem traduz isso em chamadas de solver é a ponte
//! (`ph2d-physics-ecs::bridge::topdown`); quem desenha e autora é a shell.
//! Plano: [`docs/Components/10_plano_topdown_player.md`].
//!
//! # ⭐⭐⭐ A lei que separa esta crate da irmã `ph2d-platformer`
//!
//! Não é a gravidade — é o **ORÇAMENTO DE MOVIMENTO**:
//!
//! ```text
//! plataforma      deslize = PROJECÇÃO   ⇒ anda |v|·dt·sin θ  (rasteja na parede)
//! vista de cima   deslize = ORÇAMENTO   ⇒ anda |v|·dt,  na tangente
//! ```
//!
//! Medido no oráculo (Godot 4.7.2 MIT, `motion_mode = FLOATING`, corrido sem
//! interface): a 45° de incidência isso é **`1,414×`**, a 20° é **`2,92×`**. E a
//! mesma sonda mediu que o **nosso** `PhysicsWorld::move_character` fazia a
//! projecção em todos os ângulos — *a casa não tinha esta lei*.
//!
//! ⚠️ **Ela não cabe numa função:** o orçamento gasta-se contra o mundo, e a lei
//! não tem mundo. Por isso a saída é um **plano de passos re-entrante**
//! ([`slide::first_step`] / [`slide::next_step`]) que a ponte alimenta com
//! *«quanto coube»*. Uma segunda função «com deslize» seria a porta pela qual as
//! duas leis divergiam.
//!
//! # ⚠️ A ORDEM das quatro perguntas é load-bearing
//!
//! ```text
//! entrada crua ──► quantizar ──► reprojectar ──► direcção de MUNDO ──► velocidade
//!                  (4/8/livre)   (viewpoint)              │
//!                       ▲                                 └──► a ROTAÇÃO lê AQUI
//!            no espaço da INTENÇÃO
//! ```
//!
//! ⚠️⚠️ **Quantizar depois de reprojectar é o defeito que faz a isometria não
//! servir para nada:** num tabuleiro 2:1, *«4 direcções»* tem de encaixar nas
//! diagonais do tabuleiro, e um `snap` feito no ecrã encaixa nos eixos do ecrã.
//! As duas ordens compilam, e só uma é um jogo isométrico.
//!
//! ⚠️ E a **rotação lê o fim** — o desenho está no ecrã, não no tabuleiro.
//!
//! # A unidade
//!
//! Metros e segundos, como o resto da casa (o `Transform` já é metros). O corpus
//! do oráculo está em **pixels**, e a comparação é feita em fracções do orçamento,
//! que é adimensional — é isso que a torna legítima.

pub mod direction;
pub mod intent;
pub mod rotation;
pub mod slide;
pub mod viewpoint;

/// **Um vector 2D** — um par cru, como na irmã `ph2d-platformer`.
///
/// ⚠️ Deliberadamente **não** é um tipo de uma biblioteca de matemática: esta
/// crate é uma folha, e uma folha que empurra `glam` para os consumidores deixa
/// de ser barata. `+ - * /` e `sqrt` são exactos no IEEE-754.
pub type Vec2 = [f32; 2];

/// O comprimento de um vector.
#[must_use]
#[inline]
pub fn len(v: Vec2) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

/// O produto escalar.
#[must_use]
#[inline]
pub fn dot(a: Vec2, b: Vec2) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// **Normaliza, ou devolve `None`** se o vector for curto demais para ter direcção.
///
/// ⚠️ O piso é `1e-6`, e a decisão de devolver `Option` em vez de um zero é a lei
/// que a `ph2d-arclen` desta casa pagou por escrito: *velocidade zero não é
/// direcção ausente lida como zero — é direcção ausente, e quem a recebe tem de
/// decidir o que fazer*. Aqui quem decide é a rotação (fica onde estava) e o
/// deslize (não há passo nenhum).
#[must_use]
pub fn normalize(v: Vec2) -> Option<Vec2> {
    let n = len(v);
    if n > 1.0e-6 {
        Some([v[0] / n, v[1] / n])
    } else {
        None
    }
}

/// **A configuração inteira de um player de vista de cima** — o que o componente
/// registado guarda, sem nada de ECS.
///
/// ⚠️ **São ONZE campos, e o número é o argumento de desenho.** O
/// `PlatformPlayer` desta casa tem **55**, e os que significam alguma coisa sem
/// gravidade e sem chão são quatro (`speed`, `acceleration`, `brake_scale`,
/// `reaction_push`) — um *modo* daquele componente entregaria **51 knobs mortos**
/// na mesma secção de painel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TopDownLaw {
    /// Velocidade máxima, m/s.
    pub speed: f32,
    /// Rampa de arranque, m/s². ⚠️ **Zero é INSTANTÂNEO** (ver [`intent::advance`]).
    pub acceleration: f32,
    /// Rampa de travagem, m/s². ⚠️ Zero é instantâneo.
    pub deceleration: f32,
    /// Em que direcções ele aceita andar.
    pub direction: direction::DirectionMode,
    /// Como a intenção é reprojectada para o ecrã.
    pub viewpoint: viewpoint::Viewpoint,
    /// A elevação do eixo do tabuleiro, em graus — ⚠️ **só lida por
    /// [`viewpoint::Viewpoint::Custom`]**.
    pub viewpoint_angle_deg: f32,
    /// Para onde o objecto olha.
    pub rotation: rotation::RotationMode,
    /// Graus por segundo da viragem. ⚠️ Zero é instantâneo.
    pub rotation_speed_deg: f32,
    /// Abaixo deste ângulo de incidência ele **pára** em vez de deslizar.
    /// ⚠️ O default `15.0` é o do oráculo, e é MEDIDO — ver [`slide`].
    pub min_slide_angle_deg: f32,
    /// Quantas vezes o orçamento pode mudar de direcção num tique.
    pub max_slides: u8,
    /// Se `true`, a ponte lê as acções nomeadas do Input Map. Se `false`, o
    /// componente é **motor puro** e obedece a quem lhe escrever a intenção
    /// (sinal, timeline, script) — a lei transversal 3 da síntese.
    pub default_controls: bool,
}

impl Default for TopDownLaw {
    /// ⚠️ **Os defaults são de PRODUTO, e dois deles são medidos:** o
    /// `min_slide_angle_deg` e o `max_slides` são os do oráculo (`15°`, `4`).
    /// As rampas nascem a zero porque *instantâneo* é o arranque arcade — quem
    /// quer peso escreve o número.
    fn default() -> Self {
        Self {
            speed: 4.0,
            acceleration: 0.0,
            deceleration: 0.0,
            direction: direction::DirectionMode::EightWay,
            viewpoint: viewpoint::Viewpoint::TopDown,
            viewpoint_angle_deg: 26.565_052,
            rotation: rotation::RotationMode::None,
            rotation_speed_deg: 720.0,
            min_slide_angle_deg: 15.0,
            max_slides: 4,
            default_controls: true,
        }
    }
}

/// **A porta ÚNICA da intenção**: entrada crua ⇒ direcção de MUNDO.
///
/// ⚠️ Ela existe para que a ORDEM (quantizar → reprojectar) tenha **um** sítio.
/// Duas chamadas soltas na ponte seriam duas respostas à mesma pergunta, e a
/// que envelhece é a que o artista vê.
///
/// ⚠️⚠️ **A MEMÓRIA entra na ASSINATURA, e isso é a decisão** (ordem do dono, 2026-09-15: em 4
/// direcções a última seta manda). Uma função-irmã *«sem memória»* seria a segunda porta pela qual
/// a lei se perderia — exactamente o defeito que a entrega do dedo pagou no mesmo dia. ⇒ quem
/// chama tem de ter onde guardar a dominância, e o sítio é o [`TopDownState`], que já viaja no anel
/// de checkpoints.
#[must_use]
pub fn world_direction(raw: Vec2, law: &TopDownLaw, dominancia: &mut direction::Dominance) -> Vec2 {
    let quantizado = direction::quantize(raw, law.direction, dominancia.observe(raw));
    viewpoint::reproject(quantizado, law.viewpoint, law.viewpoint_angle_deg)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "dominance_tests.rs"]
mod dominance_tests;

/// **A MEMÓRIA de um mover de vista de cima entre tiques** — a velocidade que as
/// rampas acumulam.
///
/// ⚠️ **Ela não é componente, e não é opcional que não seja:** um campo que muda
/// por tique dentro de um componente registado faria o undo desta casa ver **cada
/// quadro como um passo** (a lei do módulo de física, e a auditoria da §11 do
/// Sprite mediu-a). Ela vive na ponte, dentro do `ControllerMemory` que entra no
/// anel de checkpoints — é isso que a faz sobreviver a um scrub.
///
/// ⚠️ **Só a velocidade.** O ângulo com que o corpo olha é escrito no `Transform`,
/// que é onde a pose de um corpo cinemático já vive — guardá-lo aqui também seria
/// a segunda resposta à mesma pergunta.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TopDownState {
    /// A velocidade de agora, m/s.
    pub velocity: Vec2,
    /// ⭐ **Quem mandou por último**, e só o 4-direcções a lê
    /// ([`direction::Dominance`]). ⚠️ Ela tem de viver AQUI e não num mapa ao
    /// lado: é este struct que entra no anel de checkpoints, e é isso que faz um
    /// scrub devolver o mundo **e** o comando do mesmo tique.
    pub dominance: direction::Dominance,
}
