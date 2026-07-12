//! **O rótulo** — o texto que PERTENCE a uma forma (ou a um conector) e a segue.
//!
//! É o que falta para um diagrama ser um diagrama. Sem isto o texto é um objeto solto: o
//! usuário o posiciona à mão, move a caixa, e a legenda fica para trás.
//!
//! # Um rótulo NÃO é um tipo novo
//!
//! É o mesmo objeto de TEXTO de sempre (`VecShape::Text`) mais um **vínculo** — este
//! componente. Consequência de graça (a mesma do [`crate::VecConnector`]): a pose do rótulo é
//! uma **função pura** do hospedeiro, re-derivada a cada frame pela shell, então undo e save o
//! cobrem sem uma linha a mais — os dois capturam o mundo ECS, e o vínculo está nele.
//!
//! Inventar um `LabelShape` daria um segundo caminho de fonte, de layout, de estilo e de
//! edição, todos para reimplementar o que o texto já faz.
//!
//! # O alvo é um `VecPathId`, nunca bits de entidade
//!
//! Pela MESMA razão do conector, e ela é dura: `Entity::to_bits()` é um id de **alocação**, e o
//! undo restaura o mundo **respawnando** as entidades — com bits novos. Guardar o hospedeiro
//! por bits significaria: o usuário desfaz qualquer coisa, e **todo rótulo se solta**. O
//! `VecPathId` do documento vetorial sobrevive ao clone (undo) e ao postcard (save).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O rótulo.** A entidade que o carrega é um objeto de texto normal (`VecShape::Text` +
/// `VecPathRef`); este componente diz apenas *de quem* ele é, e *onde* em relação a esse alguém.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLabel {
    /// O objeto a que o rótulo pertence — o `VecPathId` cru (o ECS não conhece a cena vetorial,
    /// e não precisa: para ele é um `u64` opaco e estável). Uma FORMA (âncora = o centro da bbox
    /// de mundo dela) ou um CONECTOR (âncora = o meio da rota, por comprimento de arco).
    pub host: u64,
    /// O deslocamento em relação à âncora, em MUNDO. Nasce em zero (o rótulo cai no centro) e
    /// **absorve** o arrasto do usuário: mover o rótulo com o gizmo não o desprende do
    /// hospedeiro, apenas muda onde ele fica em relação a ele.
    ///
    /// Em mundo, e não na régua da forma (ao contrário do [`crate::Anchor::Port`] do conector):
    /// o rótulo é um objeto de texto, e ele não deve girar nem esticar quando a caixa gira ou
    /// estica — é o que todo editor de diagrama faz, porque texto ilegível não é rótulo.
    pub offset: [f64; 2],
}

impl SimComponent for VecLabel {}

impl VecLabel {
    /// Um rótulo novo, centrado no hospedeiro (offset zero).
    #[must_use]
    pub fn on(host: u64) -> Self {
        Self {
            host,
            offset: [0.0, 0.0],
        }
    }

    /// **Onde o centro do rótulo tem de cair**: a âncora do hospedeiro mais o offset.
    #[must_use]
    pub fn target(&self, anchor: [f64; 2]) -> [f64; 2] {
        [anchor[0] + self.offset[0], anchor[1] + self.offset[1]]
    }

    /// **Absorve** um arrasto: o usuário levou o centro do rótulo para `center`, e a partir de
    /// agora é ali (em relação à âncora) que ele mora.
    ///
    /// É o inverso exato de [`Self::target`], e essa identidade é o que impede os dois bugs do
    /// desenho: ou o rótulo salta de volta ao centro quando o usuário o solta (se o passe SEMPRE
    /// dirige), ou ele para de seguir a forma (se SEMPRE absorve — o offset engoliria o
    /// movimento do hospedeiro).
    pub fn absorb(&mut self, center: [f64; 2], anchor: [f64; 2]) {
        self.offset = [center[0] - anchor[0], center[1] - anchor[1]];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um rótulo novo nasce CENTRADO no hospedeiro.
    #[test]
    fn a_fresh_label_sits_at_the_centre_of_its_host() {
        let l = VecLabel::on(7);
        assert_eq!(l.host, 7);
        assert_eq!(l.offset, [0.0, 0.0]);
        assert_eq!(
            l.target([3.0, -4.0]),
            [3.0, -4.0],
            "sem offset, o alvo E a ancora"
        );
    }

    /// **`absorb` é o inverso EXATO de `target`** — e é essa identidade que faz o arrasto
    /// funcionar. Absorvido um deslocamento, dirigir a pose a partir da mesma âncora devolve o
    /// ponto onde o usuário largou o rótulo: nada salta de volta.
    ///
    /// E o vínculo SOBREVIVE: com a âncora nova (o hospedeiro se moveu), o alvo anda o mesmo
    /// tanto — o offset não engoliu o movimento da forma.
    #[test]
    fn absorbing_a_drag_is_the_exact_inverse_of_driving_the_pose() {
        let mut l = VecLabel::on(1);
        let anchor = [10.0, 10.0];
        // O usuário arrastou o rótulo para (13, 14).
        l.absorb([13.0, 14.0], anchor);
        assert_eq!(l.offset, [3.0, 4.0]);
        assert_eq!(
            l.target(anchor),
            [13.0, 14.0],
            "dirigir devolve onde ele foi largado"
        );
        // O hospedeiro anda (+5, −2): o rótulo anda JUNTO, mantendo o deslocamento.
        assert_eq!(l.target([15.0, 8.0]), [18.0, 12.0]);
    }
}
