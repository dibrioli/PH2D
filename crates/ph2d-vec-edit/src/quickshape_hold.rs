//! **O GATILHO do QuickShape** — desenhar e SEGURAR (o QuickShape do Procreate, R4 L165–169).
//!
//! O [`crate::Pencil`] guarda as amostras; a shell, a cada quadro com o traço aberto, pergunta
//! [`crate::Pencil::hold_tick`] com o relógio dela. Parado [`HOLD_NS`] ⇒ [`crate::quickshape::recognise`];
//! se houver forma, o caminho vivo passa a mostrá-la e o arrasto sem soltar ESCALA e RODA à volta
//! do [`QuickShape::pivot`] (com `regular`, a rotação anda em degraus de 15° e a forma regulariza).

use crate::quickshape::{self, QuickShape, REGULAR_STEP_DEG};

/// Quanto o ponteiro fica parado para a forma nascer (ns). É o `LONG_PRESS_NS` do toque
/// (`ph2d_editor_core::interaction::touch`); esta crate não vê a fundação, e um gate na
/// `ph2d-app-vec` prova que os dois são o MESMO número.
pub const HOLD_NS: u128 = 500_000_000;

/// O que um quadro fez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldTick {
    /// Nada mudou.
    Idle,
    /// A forma NASCEU agora (a shell toca o clique).
    Recognised,
    /// A forma mudou (o `regular` virou a meio).
    Reshaped,
}

#[derive(Default)]
pub(crate) struct Hold {
    live: Option<Live>,
    /// `Esc` devolveu o traço livre: este gesto não volta a reconhecer.
    dismissed: bool,
    /// O carimbo da última amostra já tentada — uma pausa sem forma não se re-tenta a cada quadro.
    tried_at: Option<u128>,
}

struct Live {
    base: QuickShape,
    regular: bool,
    /// Onde a mão estava quando a forma nasceu, e onde está agora.
    anchor: [f64; 2],
    pointer: [f64; 2],
}

impl Hold {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn is_live(&self) -> bool {
        self.live.is_some()
    }

    pub(crate) fn tick(
        &mut self,
        samples: &[[f64; 2]],
        last_t: u128,
        now: u128,
        px_to_world: f64,
        regular: bool,
        enabled: bool,
    ) -> HoldTick {
        if let Some(l) = &mut self.live {
            if l.regular == regular {
                return HoldTick::Idle;
            }
            if let Some(b) = quickshape::recognise(samples, px_to_world, regular) {
                l.base = b;
            }
            l.regular = regular;
            return HoldTick::Reshaped;
        }
        if !enabled
            || self.dismissed
            || now.saturating_sub(last_t) < HOLD_NS
            || self.tried_at == Some(last_t)
        {
            return HoldTick::Idle;
        }
        self.tried_at = Some(last_t);
        let Some(base) = quickshape::recognise(samples, px_to_world, regular) else {
            return HoldTick::Idle;
        };
        let end = samples[samples.len() - 1];
        self.live = Some(Live {
            base,
            regular,
            anchor: end,
            pointer: end,
        });
        HoldTick::Recognised
    }

    pub(crate) fn drag(&mut self, p: [f64; 2]) -> bool {
        self.live.as_mut().map(|l| l.pointer = p).is_some()
    }

    /// Devolve o traço livre; `true` se havia forma.
    pub(crate) fn dismiss(&mut self) -> bool {
        self.dismissed = true;
        self.live.take().is_some()
    }

    /// A forma como a mão a pôs agora.
    pub(crate) fn shape(&self) -> Option<QuickShape> {
        let l = self.live.as_ref()?;
        let pivot = l.base.pivot();
        let (v0, v1) = (sub(l.anchor, pivot), sub(l.pointer, pivot));
        let n0 = v0[0].hypot(v0[1]);
        if n0 <= f64::MIN_POSITIVE {
            return Some(l.base.clone());
        }
        let mut rot = v1[1].atan2(v1[0]) - v0[1].atan2(v0[0]);
        if l.regular {
            rot = quickshape::snap(rot, REGULAR_STEP_DEG.to_radians());
        }
        Some(l.base.transformed(pivot, v1[0].hypot(v1[1]) / n0, rot))
    }
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
