//! ⭐ **O cursor PRESO do campo Figma** (`docs/CartoesUI/02_especificacao.md` §3, D7): enquanto um
//! campo de cartão é arrastado pelo ícone, o cursor some e fica preso, e o número continua a andar
//! para lá da borda do ecrã. O editor-core pede (`WidgetStore::pointer_lock_wanted`); a janela é
//! daqui.
//!
//! | modo | como o movimento chega | onde vale (winit 0.30) |
//! |---|---|---|
//! | `Locked` | `DeviceEvent::MouseMotion` (cru), somado a um ponteiro VIRTUAL | Wayland · macOS · Windows |
//! | `Confined` | `CursorMoved`; perto da borda o cursor volta ao ponto de partida e o salto soma-se (o Blender) | X11 · Wayland · Windows |
//!
//! O modo que pegou vai para o `stderr` (`[cursor-preso] …`) — o 1.º smoke do dono é a metade 2 da
//! sonda `probe_can_the_cursor_be_locked`.

use winit::dpi::PhysicalPosition;
use winit::window::{CursorGrabMode, Window};

/// A distância (px físicos) à borda da janela em que o modo `Confined` devolve o cursor ao centro
/// do gesto — o cursor confinado PARA na borda, então o salto tem de vir antes dela.
const BORDA_PX: f64 = 16.0;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum CursorPreso {
    #[default]
    Livre,
    /// Preso + cru: `casa` é onde o cursor ficou; `virt` o ponteiro que o app vê.
    Locked { casa: (f64, f64), virt: (f64, f64) },
    /// Confinado: `casa` é o ponto a que o cursor volta; `desvio` o que já se andou em saltos.
    Confined {
        casa: (f64, f64),
        desvio: (f64, f64),
    },
    /// O sistema recusou os dois — o arrasto funciona, mas pára na borda. Até o gesto acabar.
    Recusado,
}

impl CursorPreso {
    /// Depois de cada evento do ponteiro e do teclado: prende na subida, solta na descida.
    /// `Some(p)` = o cursor reapareceu em `p`, e o app volta a vê-lo lá.
    pub fn sync(&mut self, win: &Window, quer: bool, ponteiro: (f32, f32)) -> Option<(f32, f32)> {
        match (*self, quer) {
            (Self::Livre, true) => {
                self.prende(win, (f64::from(ponteiro.0), f64::from(ponteiro.1)));
                None
            }
            (Self::Livre, false) | (_, true) => None,
            (_, false) => self.solta(win),
        }
    }

    fn prende(&mut self, win: &Window, casa: (f64, f64)) {
        *self = if win.set_cursor_grab(CursorGrabMode::Locked).is_ok() {
            eprintln!("[cursor-preso] Locked (movimento cru)");
            Self::Locked { casa, virt: casa }
        } else if win.set_cursor_grab(CursorGrabMode::Confined).is_ok() {
            eprintln!("[cursor-preso] Confined (reposicionar na borda)");
            Self::Confined {
                casa,
                desvio: (0.0, 0.0),
            }
        } else {
            eprintln!("[cursor-preso] recusado pelos dois modos — o arrasto pára na borda");
            Self::Recusado
        };
        if !matches!(self, Self::Recusado) {
            win.set_cursor_visible(false);
        }
    }

    fn solta(&mut self, win: &Window) -> Option<(f32, f32)> {
        let estado = std::mem::take(self);
        let casa = match estado {
            Self::Locked { casa, .. } | Self::Confined { casa, .. } => casa,
            Self::Livre | Self::Recusado => return None,
        };
        let _ = win.set_cursor_grab(CursorGrabMode::None);
        if matches!(estado, Self::Confined { .. }) {
            let _ = win.set_cursor_position(PhysicalPosition::new(casa.0, casa.1));
        }
        win.set_cursor_visible(true);
        Some((casa.0 as f32, casa.1 as f32))
    }

    /// O `CursorMoved` com o cursor preso: `None` = engolido (o `Locked` move-se pelo cru); senão a
    /// posição que o app deve ver.
    pub fn moved(
        &mut self,
        win: Option<&Window>,
        pos: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<f64>> {
        match *self {
            Self::Livre | Self::Recusado => Some(pos),
            Self::Locked { .. } => None,
            Self::Confined { casa, desvio } => {
                let virt = (pos.x + desvio.0, pos.y + desvio.1);
                if let Some(win) = win {
                    let size = win.inner_size();
                    if na_borda(
                        (pos.x, pos.y),
                        (f64::from(size.width), f64::from(size.height)),
                    ) && win
                        .set_cursor_position(PhysicalPosition::new(casa.0, casa.1))
                        .is_ok()
                    {
                        *self = Self::Confined {
                            casa,
                            desvio: (virt.0 - casa.0, virt.1 - casa.1),
                        };
                    }
                }
                Some(PhysicalPosition::new(virt.0, virt.1))
            }
        }
    }

    /// O movimento CRU — só conta com o cursor `Locked`; devolve o ponteiro virtual novo.
    pub fn raw(&mut self, (dx, dy): (f64, f64)) -> Option<(f32, f32)> {
        let Self::Locked { casa, virt } = *self else {
            return None;
        };
        let virt = (virt.0 + dx, virt.1 + dy);
        *self = Self::Locked { casa, virt };
        Some((virt.0 as f32, virt.1 as f32))
    }
}

fn na_borda((x, y): (f64, f64), (w, h): (f64, f64)) -> bool {
    x < BORDA_PX || y < BORDA_PX || x > w - BORDA_PX || y > h - BORDA_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_raw_motion_only_counts_while_locked_and_accumulates() {
        let mut c = CursorPreso::default();
        assert_eq!(c.raw((5.0, 5.0)), None);
        c = CursorPreso::Locked {
            casa: (10.0, 20.0),
            virt: (10.0, 20.0),
        };
        assert_eq!(c.raw((5.0, -5.0)), Some((15.0, 15.0)));
        assert_eq!(c.raw((5.0, -5.0)), Some((20.0, 10.0)));
    }

    #[test]
    fn a_locked_move_is_swallowed_and_a_free_one_passes() {
        let p = PhysicalPosition::new(3.0, 4.0);
        assert_eq!(CursorPreso::Livre.moved(None, p), Some(p));
        assert_eq!(CursorPreso::Recusado.moved(None, p), Some(p));
        let mut l = CursorPreso::Locked {
            casa: (0.0, 0.0),
            virt: (0.0, 0.0),
        };
        assert_eq!(l.moved(None, p), None);
    }

    #[test]
    fn a_confined_move_adds_what_the_jumps_already_walked() {
        let mut c = CursorPreso::Confined {
            casa: (100.0, 100.0),
            desvio: (50.0, -10.0),
        };
        let v = c.moved(None, PhysicalPosition::new(120.0, 100.0));
        assert_eq!(v, Some(PhysicalPosition::new(170.0, 90.0)));
    }

    #[test]
    fn the_edge_band_is_on_all_four_sides() {
        let janela = (800.0, 600.0);
        assert!(!na_borda((400.0, 300.0), janela));
        for p in [(1.0, 300.0), (799.0, 300.0), (400.0, 1.0), (400.0, 599.0)] {
            assert!(na_borda(p, janela), "{p:?}");
        }
    }
}
