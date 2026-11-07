//! ⭐ **O scrub do campo Figma para quem não vive no `WidgetStore`** (as abas `DocumentSurface`; o editor de
//! animações, dono 07/11). A lei é a do campo de cartão (`dispatch::card_field` + `pointer_move`): nasce na
//! pega, só vira scrub depois do limiar, só a horizontal conta, um passo por pixel, Shift ×10, Alt ×0,1,
//! incremental e preso ao intervalo. Abaixo do limiar é um clique: quem usa abre a escrita.

use super::drag::NUMBER_INPUT_DRAG_THRESHOLD_PX;
use ph2d_card_stack::metrics::{SCRUB_ALT_MUL, SCRUB_SHIFT_MUL};
use ph2d_host::Modifiers;

/// Um scrub em curso, desde o Down na pega.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldScrub {
    /// O valor no Down (o Esc volta a ele).
    pub start: f64,
    /// O valor contínuo: o documento pode arredondar sem o scrub travar.
    pub value: f64,
    x0: f32,
    last_x: f32,
    step: f64,
    bounds: (f64, f64),
    moved: bool,
}

impl FieldScrub {
    /// `step` = valor por pixel; `bounds` = o intervalo do modelo (em qualquer ordem).
    #[must_use]
    pub fn new(x: f32, start: f64, step: f64, bounds: (f64, f64)) -> Self {
        Self {
            start,
            value: start,
            x0: x,
            last_x: x,
            step,
            bounds: (bounds.0.min(bounds.1), bounds.0.max(bounds.1)),
            moved: false,
        }
    }

    /// Já passou o limiar (senão o gesto é um clique).
    #[must_use]
    pub fn moved(&self) -> bool {
        self.moved
    }

    /// O rato em `x`: o valor novo, ou `None` enquanto ainda é um clique.
    pub fn motion(&mut self, x: f32, mods: Modifiers) -> Option<f64> {
        if !self.moved {
            if (x - self.x0).abs() < NUMBER_INPUT_DRAG_THRESHOLD_PX {
                return None;
            }
            // Ancora aqui: o caminho até ao limiar não vira um salto.
            self.moved = true;
            self.last_x = x;
            return Some(self.value);
        }
        let delta = f64::from(x - self.last_x) * self.step * step_mul(mods);
        self.last_x = x;
        let (lo, hi) = self.bounds;
        self.value = (self.value + delta).clamp(lo, hi); // CLAMP-OK: normalizado em `new`
        Some(self.value)
    }
}

/// O multiplicador do passo pelas teclas (o de `WidgetStore::card_step_mul`).
#[must_use]
pub fn step_mul(mods: Modifiers) -> f64 {
    if mods.shift {
        SCRUB_SHIFT_MUL
    } else if mods.alt {
        SCRUB_ALT_MUL
    } else {
        1.0
    }
}
