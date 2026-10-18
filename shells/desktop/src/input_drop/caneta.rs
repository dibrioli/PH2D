//! A caneta na shell (ADR-0189): a [`ph2d_pen::porta`] liga-a à janela e drena-a no TOPO de cada
//! quadro — onde os eventos do winit já chegaram — e cada evento convertido entra pelas MESMAS
//! portas do rato (`pointer_moved`/`pointer_button`); os meios leem a `porta::tinta()`.

use ph2d_host::PointerButton as B;
use ph2d_pen::ponteiro::Saida;
use winit::event::MouseButton;

impl crate::App {
    pub(crate) fn pump_pen(&mut self) {
        let Some(w) = self.window.clone() else { return };
        let weak = std::sync::Arc::downgrade(&w);
        let wake = move || {
            if let Some(w) = weak.upgrade() {
                w.request_redraw();
            }
        };
        let now = Self::timestamp_ns();
        let saidas = ph2d_pen::porta::bombeia(w.as_ref(), wake, w.scale_factor(), now);
        self.despacha_caneta(saidas);
    }

    /// Cada saída do conversor pelas portas do RATO.
    pub(crate) fn despacha_caneta(&mut self, saidas: Vec<Saida>) {
        for s in saidas {
            match s {
                Saida::Ponteiro(e) if e.kind == ph2d_host::PointerKind::Move => {
                    self.pointer_moved(e)
                }
                Saida::Ponteiro(e) => self.pointer_button(
                    e,
                    match e.button {
                        B::Primary => MouseButton::Left,
                        B::Secondary => MouseButton::Right,
                        B::Middle => MouseButton::Middle,
                    },
                ),
                Saida::Saiu => ph2d_pen::porta::tinta_do_rato(),
            }
        }
    }
}

#[cfg(test)]
#[path = "caneta_tests.rs"]
mod tests;
