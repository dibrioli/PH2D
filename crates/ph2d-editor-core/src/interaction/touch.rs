//! ⭐ **O TOQUE SEM TECLADO** (F5, D10 — `docs/UI_New_and_Simple/vetor_nova_ui/02_estudo_os_apps_amados.md`
//! §3): o dedo que desenha é o rato; os outros dedos são as teclas.
//!
//! - **+1 dedo = Shift, +2 = Shift+Alt** enquanto o primeiro arrasta (Linearity, Graphic, Affinity);
//! - **toque de 2 dedos = desfazer, 3 = refazer, 4 = foco** (universal entre os apps medidos);
//! - **tocar e segurar = menu de contexto** (o clique direito do dedo);
//! - o **widget** dos modificadores: segurar = segurar, tocar = travar (o Caps Lock).
//!
//! ⚠️ **O primeiro dedo só vira `Down` quando ANDA** além da folga, ou quando levanta (um toque):
//! um segundo dedo que pouse antes disso faz do contacto um GESTO, e nenhum traço nasce para depois
//! ser desfeito. O `Down` sai na ORIGEM — os primeiros pontos do traço não se perdem.
//!
//! ⚠️ **Os números são os do UIKit** (o iPad é o alvo): `UILongPressGestureRecognizer` tem
//! `minimumPressDuration = 0.5 s` e `allowableMovement = 10 pt` por omissão.

use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

/// Tocar e segurar (ns) — o `minimumPressDuration` do UIKit.
pub const LONG_PRESS_NS: u128 = 500_000_000;
/// A folga de um dedo parado (pt lógicos) — o `allowableMovement` do UIKit.
pub const SLOP_PT: f32 = 10.0; // LITERAL-PX-OK: UIKit allowableMovement, não um tamanho de UI

/// A fase de um contacto — a do `winit::event::TouchPhase`, sem depender dele.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchPhase {
    Start,
    Move,
    End,
    Cancel,
}

/// Os modificadores que o toque segura.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TouchMods {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl TouchMods {
    #[must_use]
    pub fn or(self, o: Self) -> Self {
        Self {
            shift: self.shift || o.shift,
            alt: self.alt || o.alt,
            ctrl: self.ctrl || o.ctrl,
        }
    }

    #[must_use]
    pub fn any(self) -> bool {
        self.shift || self.alt || self.ctrl
    }

    fn toggled(self, chip: NodeId) -> Self {
        let mut m = self;
        match chip_slot(chip) {
            Some(0) => m.shift = !m.shift,
            Some(1) => m.alt = !m.alt,
            Some(2) => m.ctrl = !m.ctrl,
            _ => {}
        }
        m
    }
}

/// Os três botões do widget, na ordem em que se pintam.
pub const CHIPS: [NodeId; 3] = [
    crate::ids::TOUCH_MOD_SHIFT,
    crate::ids::TOUCH_MOD_ALT,
    crate::ids::TOUCH_MOD_CTRL,
];

fn chip_slot(id: NodeId) -> Option<usize> {
    CHIPS.iter().position(|&c| c == id)
}

/// O que um contacto pede à shell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TouchOut {
    /// O dedo que desenha, como o botão principal do rato.
    Press([f32; 2]),
    Drag([f32; 2]),
    Release([f32; 2]),
    Undo,
    Redo,
    Focus,
    /// Tocar e segurar: o clique direito no ponto.
    ContextMenu([f32; 2]),
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    origin: [f32; 2],
    at: [f32; 2],
}

/// ⭐ **O estado do toque** — os dedos no vidro e o widget dos modificadores.
#[derive(Clone, Debug, Default)]
pub struct TouchGestures {
    fingers: BTreeMap<u64, Finger>,
    primary: Option<u64>,
    t0: u128,
    committed: bool,
    gesture: bool,
    most: usize,
    moved: bool,
    long_fired: bool,
    chips: BTreeMap<u64, (NodeId, u128)>,
    locked: TouchMods,
    seen: bool,
}

fn far(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).hypot(a[1] - b[1]) > SLOP_PT
}

impl TouchGestures {
    /// Um contacto. `chip` = o botão do widget sob um dedo que POUSA (só no `Start`).
    pub fn feed(
        &mut self,
        id: u64,
        phase: TouchPhase,
        at: [f32; 2],
        now: u128,
        chip: Option<NodeId>,
    ) -> Vec<TouchOut> {
        self.seen = true;
        match phase {
            TouchPhase::Start => {
                if let Some(c) = chip.filter(|c| chip_slot(*c).is_some()) {
                    self.chips.insert(id, (c, now));
                    return Vec::new();
                }
                if self.fingers.is_empty() {
                    self.reset();
                    self.primary = Some(id);
                    self.t0 = now;
                } else if !self.committed {
                    self.gesture = true;
                }
                self.fingers.insert(id, Finger { origin: at, at });
                self.most = self.most.max(self.fingers.len());
                Vec::new()
            }
            TouchPhase::Move => self.moved_to(id, at),
            TouchPhase::End | TouchPhase::Cancel => self.lifted(id, phase, now),
        }
    }

    fn moved_to(&mut self, id: u64, at: [f32; 2]) -> Vec<TouchOut> {
        let Some(f) = self.fingers.get_mut(&id) else {
            return Vec::new();
        };
        f.at = at;
        let origin = f.origin;
        if far(origin, at) {
            self.moved = true;
        }
        if self.primary != Some(id) || self.gesture || self.long_fired {
            return Vec::new();
        }
        if self.committed {
            vec![TouchOut::Drag(at)]
        } else if far(origin, at) {
            self.committed = true;
            vec![TouchOut::Press(origin), TouchOut::Drag(at)]
        } else {
            Vec::new()
        }
    }

    fn lifted(&mut self, id: u64, phase: TouchPhase, now: u128) -> Vec<TouchOut> {
        if let Some((c, t)) = self.chips.remove(&id) {
            if phase == TouchPhase::End && now.saturating_sub(t) < LONG_PRESS_NS {
                self.locked = self.locked.toggled(c);
            }
            return Vec::new();
        }
        let Some(f) = self.fingers.remove(&id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        if self.primary == Some(id) {
            if self.committed {
                out.push(TouchOut::Release(f.at));
                self.committed = false;
            } else if !self.gesture && !self.long_fired && phase == TouchPhase::End {
                out.extend([TouchOut::Press(f.origin), TouchOut::Release(f.origin)]);
            }
            self.primary = None;
        }
        if self.fingers.is_empty() {
            let quick = now.saturating_sub(self.t0) < LONG_PRESS_NS;
            if self.gesture && !self.moved && quick && phase == TouchPhase::End {
                out.extend(match self.most {
                    2 => Some(TouchOut::Undo),
                    3 => Some(TouchOut::Redo),
                    4 => Some(TouchOut::Focus),
                    _ => None,
                });
            }
            self.reset();
        }
        out
    }

    fn reset(&mut self) {
        self.primary = None;
        self.committed = false;
        self.gesture = false;
        self.most = 0;
        self.moved = false;
        self.long_fired = false;
    }

    /// O relógio: um dedo parado além de [`LONG_PRESS_NS`] abre o menu de contexto, sem esperar
    /// por outro evento (um dedo parado pode não mandar nenhum).
    pub fn tick(&mut self, now: u128) -> Option<TouchOut> {
        let f = self.fingers.get(&self.primary?)?;
        let still = !self.committed && !self.gesture && !self.long_fired && !self.moved;
        if still && self.fingers.len() == 1 && now.saturating_sub(self.t0) >= LONG_PRESS_NS {
            self.long_fired = true;
            return Some(TouchOut::ContextMenu(f.origin));
        }
        None
    }

    /// Os modificadores que o toque segura AGORA: os dedos extra durante um traço, os botões do
    /// widget premidos, e os travados.
    #[must_use]
    pub fn modifiers(&self) -> TouchMods {
        let extra = if self.committed {
            self.fingers.len().saturating_sub(1)
        } else {
            0
        };
        let fingers = TouchMods {
            shift: extra >= 1,
            alt: extra >= 2,
            ctrl: false,
        };
        let held = self
            .chips
            .values()
            .fold(TouchMods::default(), |m, &(c, _)| {
                m.or(TouchMods::default().toggled(c))
            });
        fingers.or(held).or(self.locked)
    }

    /// Os travados — o widget mostra-os SEMPRE, mesmo sem toque.
    #[must_use]
    pub fn locked(&self) -> TouchMods {
        self.locked
    }

    /// Um clique (rato ou dedo já resolvido) num botão do widget: trava ou destrava.
    pub fn toggle_lock(&mut self, chip: NodeId) {
        self.locked = self.locked.toggled(chip);
    }

    /// Houve toque nesta sessão (o widget aparece) ou há um modificador travado.
    #[must_use]
    pub fn widget_visible(&self) -> bool {
        self.seen || self.locked.any()
    }

    /// O botão do widget premido por um dedo agora?
    #[must_use]
    pub fn chip_held(&self, chip: NodeId) -> bool {
        self.chips.values().any(|&(c, _)| c == chip)
    }
}

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
