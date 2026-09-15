//! **O componente do mover de VISTA DE CIMA** (TOP-20 #13) — CONFIG, nunca estado
//! vivo de solver (ADR-0131: *«o undo ordena por bytes»*).
//!
//! A lei dele vive na crate-folha [`ph2d_topdown`]; aqui só está o que o artista
//! autora e o que o ficheiro guarda. Plano:
//! `docs/Components/10_plano_topdown_player.md`.
//!
//! # ⛔ Por que ele NÃO é um modo do [`super::PlatformPlayer`]
//!
//! O Godot resolve com um dropdown (`motion_mode`), e **o argumento dele não
//! transfere**: o `CharacterBody2D` é fino (só colisão) e o nosso
//! `PlatformPlayer` é o controlador inteiro. **Contado no ficheiro:** ele tem
//! **55** campos, e os que significam alguma coisa sem gravidade e sem chão são
//! `speed`, `acceleration`, `brake_scale` e `reaction_push` — *quatro*. Um modo
//! entregaria **51 knobs mortos** na mesma secção de painel.
//!
//! ⭐ O que se reusa é o desenho e o vocabulário: crate de lei pura irmã, ponte no
//! mesmo sítio, e o [`super::PlayerMode`] **partilhado** — ele já é componente
//! separado, e a posse do `Transform` é a mesma pergunta nos dois movers.
//!
//! # ⚠️ Os enums viajam por um valor de FIO explícito
//!
//! A folha `ph2d-topdown` tem **uma** dependência (`libm`, pelo determinismo
//! cross-OS) e não vai ganhar `serde` para isto. ⇒ cada enum declara lá o próprio
//! `to_wire`/`from_wire` com números **literais**, e aqui eles atravessam o
//! `serde` por esses bytes. ⛔ O postcard é posicional: sem números literais, uma
//! reordenação de variantes por gosto trocava o modo de toda cena já gravada, em
//! silêncio.

use bevy_ecs::prelude::Component;
use ph2d_topdown::{
    TopDownLaw,
    direction::{self, DirectionMode},
    rotation::{self, RotationMode},
    viewpoint::{self, Viewpoint},
};
use serde::{Deserialize, Serialize};

/// **Um mover de vista de cima.** Ver o cabeçalho do módulo.
///
/// ⚠️ **Componente registado NOVO ⇒ `PROJECT_SCHEMA` sobe.** Um `ComponentBlob`
/// de `type_id` desconhecido **recusa o load inteiro**, e o degrau é o que
/// transforma isso em *«este ficheiro é de outra versão»* em vez de *«type id
/// desconhecido»* a meio da travessia (a escada em `project_schema.rs`, degraus
/// `123`, `125`, `126`, `127` e `130`).
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TopDownPlayer {
    /// Velocidade máxima, m/s.
    pub speed: f32,
    /// Rampa de arranque, m/s². ⚠️ **Zero é INSTANTÂNEO**, não parado.
    pub acceleration: f32,
    /// Rampa de travagem, m/s². ⚠️ Zero é instantâneo.
    pub deceleration: f32,
    /// Em que direcções ele aceita andar (fio: [`direction::to_wire`]).
    pub direction_mode: u8,
    /// De que ângulo o tabuleiro é visto (fio: [`viewpoint::to_wire`]).
    pub viewpoint: u8,
    /// A elevação do eixo do tabuleiro, graus — só lida por `Custom`.
    pub viewpoint_angle_deg: f32,
    /// Para onde o objecto olha (fio: [`rotation::to_wire`]).
    pub rotation_mode: u8,
    /// Graus por segundo da viragem. ⚠️ Zero é instantâneo.
    pub rotation_speed_deg: f32,
    /// Abaixo deste ângulo de incidência ele **pára** em vez de deslizar.
    /// ⚠️ O default `15.0` é MEDIDO no oráculo.
    pub min_slide_angle_deg: f32,
    /// Quantas vezes o orçamento pode mudar de direcção num tique.
    pub max_slides: u8,
    /// Se a ponte lê as acções nomeadas do Input Map. Desligado, o componente é
    /// **motor puro** e obedece a quem lhe escrever a intenção.
    pub default_controls: bool,
}

impl Default for TopDownPlayer {
    fn default() -> Self {
        Self::from_law(TopDownLaw::default())
    }
}

impl TopDownPlayer {
    /// **A porta ÚNICA** componente ⇒ lei.
    ///
    /// ⚠️ Ela existe para que a tradução dos três enums tenha **um** sítio: a
    /// ponte, o painel e os gates leem daqui, e um `from_wire` solto num deles
    /// seria a segunda resposta à mesma pergunta.
    #[must_use]
    pub fn law(&self) -> TopDownLaw {
        TopDownLaw {
            speed: self.speed,
            acceleration: self.acceleration,
            deceleration: self.deceleration,
            direction: direction::from_wire(self.direction_mode),
            viewpoint: viewpoint::from_wire(self.viewpoint),
            viewpoint_angle_deg: self.viewpoint_angle_deg,
            rotation: rotation::from_wire(self.rotation_mode),
            rotation_speed_deg: self.rotation_speed_deg,
            min_slide_angle_deg: self.min_slide_angle_deg,
            max_slides: self.max_slides,
            default_controls: self.default_controls,
        }
    }

    /// O inverso — usado pelo default e pelo painel ao escrever.
    #[must_use]
    pub fn from_law(l: TopDownLaw) -> Self {
        Self {
            speed: l.speed,
            acceleration: l.acceleration,
            deceleration: l.deceleration,
            direction_mode: direction::to_wire(l.direction),
            viewpoint: viewpoint::to_wire(l.viewpoint),
            viewpoint_angle_deg: l.viewpoint_angle_deg,
            rotation_mode: rotation::to_wire(l.rotation),
            rotation_speed_deg: l.rotation_speed_deg,
            min_slide_angle_deg: l.min_slide_angle_deg,
            max_slides: l.max_slides,
            default_controls: l.default_controls,
        }
    }

    /// Os três enums, já traduzidos — atalho de leitura para o painel.
    #[must_use]
    pub fn modes(&self) -> (DirectionMode, Viewpoint, RotationMode) {
        (
            direction::from_wire(self.direction_mode),
            viewpoint::from_wire(self.viewpoint),
            rotation::from_wire(self.rotation_mode),
        )
    }
}

#[cfg(test)]
#[path = "topdown_tests.rs"]
mod tests;
