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

/// Quanto a mão pode tremer e continuar PARADA (px de tela): a folga do toque — `SLOP_PT`, o
/// `allowableMovement` do UIKit, os mesmos 10 da fronteira toque × arrasto do F6 (gate na
/// `ph2d-app-vec`). Sem ela, o tremor de uma mão em repouso (mais que o passo de 1,5 px) reiniciava
/// o meio segundo — a «pouca sensibilidade» do smoke do dono (09/10).
pub const HOLD_SLOP_PX: f64 = 10.0;

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

/// ⭐ O gatilho inteiro (o lápis do vetor e a caneta do quadro, W23 B13, usam ESTE).
#[derive(Default)]
pub struct Hold {
    live: Option<Live>,
    /// `Esc` devolveu o traço livre: este gesto não volta a reconhecer.
    dismissed: bool,
    /// Quantas amostras havia na última tentativa — uma pausa sem forma só se re-tenta quando a mão
    /// deixa uma amostra nova.
    tried_at: Option<usize>,
}

struct Live {
    base: QuickShape,
    regular: bool,
    /// Onde a mão estava quando a forma nasceu, e onde está agora.
    anchor: [f64; 2],
    pointer: [f64; 2],
}

impl Hold {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        self.live.is_some()
    }

    /// Um quadro: `rest_t` = quando a mão parou ([`rest_index`]), `now` o relógio dela (ns).
    pub fn tick(
        &mut self,
        samples: &[[f64; 2]],
        rest_t: u128,
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
            || now.saturating_sub(rest_t) < HOLD_NS
            || self.tried_at == Some(samples.len())
        {
            return HoldTick::Idle;
        }
        self.tried_at = Some(samples.len());
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

    /// Com a forma viva, o arrasto escala e roda; `false` = não há forma.
    pub fn drag(&mut self, p: [f64; 2]) -> bool {
        self.live.as_mut().map(|l| l.pointer = p).is_some()
    }

    /// Devolve o traço livre; `true` se havia forma.
    pub fn dismiss(&mut self) -> bool {
        self.dismissed = true;
        self.live.take().is_some()
    }

    /// A forma como a mão a pôs agora.
    #[must_use]
    pub fn shape(&self) -> Option<QuickShape> {
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

/// ⭐ Desde que amostra a mão está PARADA: a primeira de onde até à última nada saiu de `slop`
/// (mundo; [`HOLD_SLOP_PX`] × px). O tremor de uma mão em repouso passa o passo mínimo, e cada
/// amostra reiniciaria o meio segundo.
#[must_use]
pub fn rest_index(samples: &[[f64; 2]], slop: f64) -> usize {
    let Some(&end) = samples.last() else {
        return 0;
    };
    let mut j = samples.len() - 1;
    while j > 0 && (samples[j - 1][0] - end[0]).hypot(samples[j - 1][1] - end[1]) <= slop {
        j -= 1;
    }
    j
}

fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
