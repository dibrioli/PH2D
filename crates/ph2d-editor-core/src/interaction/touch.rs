//! ⭐ **O TOQUE SEM TECLADO** (F5, D10 — `docs/UI_New_and_Simple/vetor_nova_ui/02_estudo_os_apps_amados.md`
//! §3): o dedo que desenha é o rato; os outros dedos são as teclas.
//!
//! - **+1 dedo = Shift, +2 = Shift+Alt** enquanto o primeiro arrasta (Linearity, Graphic, Affinity);
//! - **toque de 2 dedos = desfazer, 3 = refazer, 4 = foco** (universal entre os apps medidos);
//! - **2 dedos que ANDAM = navegar** (F6, R4 #11): o ponto médio arrasta a vista, o afastamento
//!   dá zoom à volta dele — a shell leva-os pelas portas do botão do MEIO e da RODA;
//! - **tocar e segurar = menu de contexto** (o clique direito do dedo);
//! - o **widget** dos modificadores: segurar = segurar, tocar = travar (o Caps Lock).
//!
//! ⚠️ **O primeiro dedo só vira `Down` quando ANDA** além da folga, ou quando levanta (um toque):
//! um segundo dedo que pouse antes disso faz do contacto um GESTO, e nenhum traço nasce para depois
//! ser desfeito. O `Down` sai na ORIGEM — os primeiros pontos do traço não se perdem.
//!
//! ⚠️ **O toque de 2 dedos e a navegação partilham o início; a fronteira é a FOLGA.** Um contacto
//! de dois dedos que não andou além dela e levantou depressa desfaz; um que andou navega e nunca
//! desfaz. A navegação arranca SEM salto: o primeiro movimento além da folga semeia a referência e
//! não move nada — o que se move é o que vier depois (um 3.º dedo, ou um dedo a sair, semeia outra
//! vez pelo mesmo motivo).
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
    /// Dois dedos começaram a NAVEGAR, com o ponto médio aqui: o botão do meio pousa.
    NavBegin([f32; 2]),
    /// O ponto médio andou isto (pt): o arrasto do botão do meio.
    Pan([f32; 2]),
    /// Os dedos afastaram-se: `factor` multiplica o `height_world` (< 1 aproxima), à volta de `about`.
    Zoom {
        factor: f32,
        about: [f32; 2],
    },
    /// A navegação acabou (ficou menos de dois dedos): o botão do meio levanta.
    NavEnd,
    /// ⭐ O **beliscão RÁPIDO** (F8, o *Quick-Pinch* do Procreate): uma navegação que acabou antes de
    /// um toque-e-segurar e cujo afastamento mudou mais do que a folga e mais do que o ponto médio
    /// andou. `closing` = os dedos juntaram-se (*enquadrar tudo*); afastaram-se = a vista anterior.
    QuickPinch {
        closing: bool,
    },
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    origin: [f32; 2],
    at: [f32; 2],
}

/// A referência da navegação: o ponto médio e o afastamento (média das distâncias a ele).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Nav {
    mid: [f32; 2],
    spread: f32,
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
    nav: Option<Nav>,
    navigating: bool,
    /// O afastamento quando a navegação arrancou e o último com dois dedos ou mais no vidro — os
    /// dois extremos do beliscão rápido.
    pinch: Option<(Nav, Nav)>,
    /// Um dedo andou desde o último quadro: a navegação publica-se no `tick`.
    nav_dirty: bool,
    /// O que os contactos SINTÉTICOS (uma cena de smoke) pediram e o `tick` ainda não entregou.
    pending: Vec<TouchOut>,
    /// px por pt, anotado pela shell: os contactos sintéticos chegam em px (o rect da área) e o
    /// reconhecedor mede em pt. `None` = 1.
    scale: Option<f32>,
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
                // Um dedo a mais muda o ponto médio sem ninguém ter andado: semeia-se outra vez.
                self.nav = None;
                Vec::new()
            }
            TouchPhase::Move => self.moved_to(id, at),
            TouchPhase::End | TouchPhase::Cancel => self.lifted(id, phase, now),
        }
    }

    /// Um contacto SINTÉTICO (a cena de smoke), em px da janela: o que ele pede fica na fila até
    /// ao próximo `tick`, que é por onde a shell o aplica — a MESMA porta que um dedo real.
    pub fn feed_synthetic(&mut self, id: u64, phase: TouchPhase, at_px: [f32; 2], now: u128) {
        let s = self.scale.unwrap_or(1.0);
        let outs = self.feed(id, phase, [at_px[0] / s, at_px[1] / s], now, None);
        self.pending.extend(outs);
    }

    /// A shell anota os px por pt da janela (em todo quadro): a conversão dos contactos sintéticos.
    pub fn note_scale(&mut self, px_per_pt: f32) {
        if px_per_pt.is_finite() && px_per_pt > 0.0 {
            self.scale = Some(px_per_pt);
        }
    }

    fn cloud(&self) -> Nav {
        #[allow(clippy::cast_precision_loss)]
        let n = self.fingers.len() as f32;
        let mid = self
            .fingers
            .values()
            .fold([0.0, 0.0], |m, f| [m[0] + f.at[0] / n, m[1] + f.at[1] / n]);
        let spread = self
            .fingers
            .values()
            .map(|f| (f.at[0] - mid[0]).hypot(f.at[1] - mid[1]) / n)
            .sum();
        Nav { mid, spread }
    }

    /// Dois (ou mais) dedos a andar: o ponto médio arrasta, o afastamento dá zoom.
    ///
    /// ⚠️ **Uma vez por QUADRO (do `tick`), nunca por evento.** Cada dedo chega num evento próprio;
    /// publicar a cada um compõe um zoom à volta do ponto médio de UM dedo movido com outro à volta
    /// do ponto médio dos dois — e a diferença é um arrasto espúrio de `δ²/4s` por quadro (medido na
    /// foto da cena 11: a âncora fugia ~20 px). Com todos os dedos já no sítio, não há dois pontos.
    fn navigate(&mut self) -> Vec<TouchOut> {
        if !(self.nav_dirty && self.gesture && !self.committed && self.fingers.len() >= 2) {
            return Vec::new();
        }
        self.nav_dirty = false;
        let cur = self.cloud();
        let Some(prev) = self.nav.replace(cur) else {
            if self.navigating {
                return Vec::new();
            }
            self.navigating = true;
            self.pinch = Some((cur, cur));
            return vec![TouchOut::NavBegin(cur.mid)];
        };
        let mut out = Vec::new();
        let d = [cur.mid[0] - prev.mid[0], cur.mid[1] - prev.mid[1]];
        if d != [0.0, 0.0] {
            out.push(TouchOut::Pan(d));
        }
        if prev.spread > 0.0 && cur.spread > 0.0 {
            let factor = prev.spread / cur.spread;
            if factor.is_finite() && factor != 1.0 {
                out.push(TouchOut::Zoom {
                    factor,
                    about: cur.mid,
                });
            }
        }
        out
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
        if self.gesture && !self.committed && self.moved && self.fingers.len() >= 2 {
            self.nav_dirty = true;
            return Vec::new();
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
        // O último afastamento do beliscão é o dos dedos ONDE LEVANTAM (o `tick` pode não ter
        // corrido desde o último movimento; gravá-lo também no `tick` era redundante — mutação F8).
        if self.navigating && self.fingers.len() >= 2 && self.fingers.contains_key(&id) {
            let cur = self.cloud();
            if let Some((_, last)) = self.pinch.as_mut() {
                *last = cur;
            }
        }
        let Some(f) = self.fingers.remove(&id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        // Um dedo a menos muda o ponto médio sem ninguém ter andado: semeia-se outra vez.
        self.nav = None;
        if self.navigating && self.fingers.len() < 2 {
            self.navigating = false;
            out.push(TouchOut::NavEnd);
        }
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
            if let Some((a, b)) = self.pinch
                && quick
                && phase == TouchPhase::End
            {
                let ds = b.spread - a.spread;
                let dm = (b.mid[0] - a.mid[0]).hypot(b.mid[1] - a.mid[1]);
                if ds.abs() > SLOP_PT && ds.abs() > dm {
                    out.push(TouchOut::QuickPinch { closing: ds < 0.0 });
                }
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
        self.nav = None;
        self.navigating = false;
        self.nav_dirty = false;
        self.pinch = None;
    }

    /// O relógio, em todo quadro: entrega o que os contactos sintéticos pediram, e um dedo parado
    /// além de [`LONG_PRESS_NS`] abre o menu de contexto sem esperar por outro evento (um dedo
    /// parado pode não mandar nenhum).
    pub fn tick(&mut self, now: u128) -> Vec<TouchOut> {
        let mut out = std::mem::take(&mut self.pending);
        out.extend(self.navigate());
        if let Some(f) = self.primary.and_then(|p| self.fingers.get(&p)) {
            // (Um dedo único que andou além da folga já é traço: o `committed` cobre o «andou».)
            let still = !self.committed && !self.gesture && !self.long_fired;
            if still && self.fingers.len() == 1 && now.saturating_sub(self.t0) >= LONG_PRESS_NS {
                self.long_fired = true;
                out.push(TouchOut::ContextMenu(f.origin));
            }
        }
        out
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
