//! ⭐ **O CLIQUE CURTO das ferramentas do vetor escolhe o objecto de outro tipo** (dono 24/10: *«as
//! outras tools de vector ainda não selecionam outros objetos com o click curto»*). O press que cai
//! sobre um objecto que não é vetor fica SEGURO: soltar sem arrastar selecciona-o; arrastar além do
//! [`DRAG_THRESHOLD_PX`] devolve o press à ferramenta, no ponto onde começou, e ela desenha como
//! sempre. `docs/GameEngineFuncional/01_W1_modos_selecao_inspector.md`.

use ph2d_editor_core::interaction::drag::DRAG_THRESHOLD_PX;
use ph2d_tool_vector::DrawMode;

/// Um press seguro: o objecto de cima (bits) e o ponto de ecrã onde começou.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeldPress {
    /// O objecto de outro tipo que o clique curto escolhe.
    pub bits: u64,
    /// Onde o press começou (ecrã) — o ponto que a ferramenta recebe se virar arrasto.
    pub at: (f32, f32),
}

/// ⭐ **A ferramenta deixa segurar o press?** ⛔ Não onde o clique sobre outro objecto já É trabalho:
/// o conta-gotas recolhe a cor da imagem; a caneta com um caminho a meio põe o próximo nó (contornar
/// uma referência); o texto em edição reposiciona o cursor. O Select nem chega aqui (o pick é dele).
#[must_use]
pub fn may_hold(mode: DrawMode, pen_drawing: bool, text_editing: bool) -> bool {
    match mode {
        DrawMode::Select | DrawMode::Eyedropper => false,
        DrawMode::Pen => !pen_drawing,
        DrawMode::Text => !text_editing,
        _ => true,
    }
}

/// O ponteiro andou o bastante para o press seguro ser um ARRASTO.
#[must_use]
pub fn became_a_drag(held: HeldPress, now: (f32, f32)) -> bool {
    let (dx, dy) = (now.0 - held.at.0, now.1 - held.at.1);
    dx.hypot(dy) > DRAG_THRESHOLD_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ GATE — o conta-gotas, a caneta a meio e o texto em edição não seguram; as outras seguram.
    #[test]
    fn only_the_tools_whose_click_is_not_work_hold_the_press() {
        assert!(may_hold(DrawMode::Shape, false, false));
        assert!(may_hold(DrawMode::Node, false, false));
        assert!(
            may_hold(DrawMode::Pen, false, false),
            "a caneta sem caminho"
        );
        assert!(
            !may_hold(DrawMode::Pen, true, false),
            "a caneta a meio do caminho"
        );
        assert!(may_hold(DrawMode::Text, false, false), "o texto sem cursor");
        assert!(!may_hold(DrawMode::Text, false, true), "o texto em edição");
        assert!(!may_hold(DrawMode::Eyedropper, false, false));
        assert!(!may_hold(DrawMode::Select, false, false));
    }

    /// ⭐ GATE — o limiar é o da casa (4 px): dentro é clique, fora é arrasto.
    #[test]
    fn a_held_press_becomes_a_drag_past_the_house_threshold() {
        let h = HeldPress {
            bits: 1,
            at: (10.0, 10.0),
        };
        assert!(!became_a_drag(h, (13.0, 10.0)));
        assert!(!became_a_drag(h, (12.0, 13.0)), "3,6 px");
        assert!(became_a_drag(h, (15.0, 10.0)));
    }
}
