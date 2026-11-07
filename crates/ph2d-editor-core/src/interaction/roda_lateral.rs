//! A roda de lado sobre uma grelha que rola as colunas: Shift+roda ou a roda horizontal andam a barra
//! dela. A grelha publica a região ao pintar; o painel limpa-a antes de pintar e ao fechar.

use crate::zones::Rect;
use ph2d_a11y::NodeId;
use std::cell::RefCell;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Regiao {
    /// A barra (`Slider`, valor em `0..=1`) que a roda anda.
    pub barra: NodeId,
    /// O painel que tem de estar por cima neste ponto.
    pub painel: NodeId,
    pub rect: Rect,
    /// Quantos píxeis de roda levam a barra de ponta a ponta.
    pub curso_px: f32,
}

thread_local! {
    static REGIOES: RefCell<Vec<Regiao>> = const { RefCell::new(Vec::new()) };
}

pub fn publica(r: Regiao) {
    limpa(r.barra);
    REGIOES.with(|v| v.borrow_mut().push(r));
}

pub fn limpa(barra: NodeId) {
    REGIOES.with(|v| v.borrow_mut().retain(|r| r.barra != barra));
}

/// A região sob este ponto, com `por_cima` o painel que manda nele.
#[must_use]
pub fn em(x: f32, y: f32, por_cima: Option<NodeId>) -> Option<Regiao> {
    REGIOES.with(|v| {
        v.borrow()
            .iter()
            .find(|r| r.rect.contains(x, y) && por_cima == Some(r.painel))
            .copied()
    })
}

/// A roda lateral deste evento: Shift vira a vertical; sem Shift, só o eixo x quando manda no gesto.
#[must_use]
pub fn delta_lateral(ev: &ph2d_host::WheelEvent) -> f32 {
    if ev.modifiers.shift {
        ev.delta_y + ev.delta_x
    } else if ev.delta_x.abs() > ev.delta_y.abs() {
        ev.delta_x
    } else {
        0.0
    }
}
