//! A caneta de UMA janela, no fio dela: o receptor (Linux), o [`Conversor`](crate::ponteiro::Conversor) e a [`Tinta`] do último
//! evento de ponteiro — o estado que a shell não tem onde guardar (a `App` está no tecto de campos).
//!
//! A shell chama [`bombeia`] no topo de cada quadro, despacha cada [`Saida`] pelas portas do rato
//! (que chamam [`poe_tinta`]) e larga tudo com [`larga`] antes de o laço do winit acabar; os meios
//! leem a pressão, o tilt e a borracha por [`tinta`].

use std::cell::Cell;
#[cfg(target_os = "linux")]
use std::cell::RefCell;

use ph2d_host::PointerEvent;

use crate::ponteiro::{Saida, Tinta};

thread_local! {
    static TINTA: Cell<Tinta> = const { Cell::new(Tinta::RATO) };
    static EVENTO_NS: Cell<u128> = const { Cell::new(0) };
    static DENTRO: Cell<bool> = const { Cell::new(true) };
}

// `None` = ainda não ligado; `Some(None)` = não há (X11, ou o compositor sem tablet-v2).
#[cfg(target_os = "linux")]
thread_local! {
    static RECEPTOR: RefCell<Option<Option<crate::PenReceiver>>> = const { RefCell::new(None) };
    static CONVERSOR: RefCell<crate::ponteiro::Conversor> = RefCell::new(Default::default());
}

/// A tinta do último evento de ponteiro (o rato: `1.0`, `Mouse`).
#[must_use]
pub fn tinta() -> Tinta {
    TINTA.with(Cell::get)
}

/// O `timestamp_ns` do último evento de ponteiro: a caneta chega em lote, o relógio de agora não serve.
#[must_use]
pub fn evento_ns() -> u128 {
    EVENTO_NS.with(Cell::get)
}

/// Cada evento de ponteiro despachado, rato ou caneta.
pub fn poe_tinta(evt: &PointerEvent) {
    TINTA.with(|t| t.set(Tinta::de(evt)));
    EVENTO_NS.with(|t| t.set(evt.timestamp_ns));
}

/// O ponteiro está sobre a janela: `false` depois de `CursorLeft` / foco perdido, `true` desde o
/// movimento seguinte (o anel do pincel some fora da janela, AU-AU13-7).
#[must_use]
pub fn dentro() -> bool {
    DENTRO.with(Cell::get)
}

/// O ponteiro entrou (`true`, a porta do movimento) ou saiu (`false`, a mão perdida) da janela.
pub fn poe_dentro(dentro: bool) {
    DENTRO.with(|d| d.set(dentro));
}

/// A caneta saiu ([`Saida::Saiu`]): o rato volta a mandar.
pub fn tinta_do_rato() {
    TINTA.with(|t| t.set(Tinta::RATO));
}

/// O cursor da caneta acompanha o do rato.
pub fn cursor(icon: cursor_icon::CursorIcon) {
    #[cfg(target_os = "linux")]
    RECEPTOR.with(|c| {
        if let Some(Some(r)) = c.borrow().as_ref() {
            r.set_cursor(icon);
        }
    });
    #[cfg(not(target_os = "linux"))]
    let _ = icon;
}

/// A caneta pousada na faixa move a janela (`borda = None`) ou redimensiona-a pela borda, como o rato
/// pelo `drag_window`. `false` = não há caneta Wayland nesta janela (a faixa deixa o Down ao editor).
/// O compositor fica com o contacto: o conversor larga a ponta sem `Up`.
#[cfg(all(target_os = "linux", feature = "winit"))]
pub fn arrasta_janela(
    janela: &winit::window::Window,
    borda: Option<winit::window::ResizeDirection>,
) -> bool {
    use wayland_protocols::xdg::shell::client::xdg_toplevel::ResizeEdge as E;
    use winit::platform::wayland::WindowExtWayland as _;
    use winit::window::ResizeDirection as D;
    let Some(toplevel) = janela.xdg_toplevel() else {
        return false;
    };
    let arrasto = match borda {
        None => crate::wayland::Arrasto::Mover,
        Some(d) => crate::wayland::Arrasto::Borda(match d {
            D::North => E::Top,
            D::South => E::Bottom,
            D::East => E::Right,
            D::West => E::Left,
            D::NorthEast => E::TopRight,
            D::NorthWest => E::TopLeft,
            D::SouthEast => E::BottomRight,
            D::SouthWest => E::BottomLeft,
        }),
    };
    let pediu = RECEPTOR.with(|r| match r.borrow().as_ref() {
        Some(Some(rx)) => {
            rx.arrasta(toplevel, arrasto);
            true
        }
        _ => false,
    });
    if pediu {
        CONVERSOR.with(|c| c.borrow_mut().solta());
    }
    pediu
}

/// Fora do Linux a janela não segue a caneta (ainda — ADR-0189 §2).
#[cfg(all(not(target_os = "linux"), feature = "winit"))]
pub fn arrasta_janela(
    _: &winit::window::Window,
    _: Option<winit::window::ResizeDirection>,
) -> bool {
    false
}

/// Larga o receptor ANTES de a ligação do winit fechar.
pub fn larga() {
    #[cfg(target_os = "linux")]
    RECEPTOR.with(|r| drop(r.borrow_mut().take()));
}

/// Liga a caneta à janela na 1.ª chamada (`wake` acorda o quadro a cada lote), e devolve o que
/// chegou desde a última, convertido. `escala` = o factor de escala; `agora_ns` = o relógio da shell.
#[cfg(target_os = "linux")]
pub fn bombeia(
    janela: &(impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle),
    wake: impl Fn() + Send + 'static,
    escala: f64,
    agora_ns: u128,
) -> Vec<Saida> {
    let lote: Vec<_> = RECEPTOR.with(|r| {
        let mut r = r.borrow_mut();
        let r = r.get_or_insert_with(|| {
            crate::PenReceiver::attach(janela, wake).unwrap_or_else(|e| {
                eprintln!("pen on Wayland unavailable: {e}");
                None
            })
        });
        r.as_ref().map(|d| d.drain().collect()).unwrap_or_default()
    });
    let mut saidas = Vec::new();
    if !lote.is_empty() {
        CONVERSOR.with(|c| {
            c.borrow_mut()
                .converte(&lote, escala, agora_ns, &mut saidas)
        });
    }
    saidas
}

/// Sem caneta fora do Linux (ainda — ADR-0189 §2).
#[cfg(not(target_os = "linux"))]
pub fn bombeia<J>(_: &J, _: impl Fn() + Send + 'static, _: f64, _: u128) -> Vec<Saida> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_host::{PenInfo, PointerButton, PointerKind, PointerSource};

    #[test]
    fn the_ink_is_the_last_events_and_the_mouse_is_the_default() {
        assert_eq!((tinta(), evento_ns()), (Tinta::RATO, 0));
        let mut e =
            crate::ponteiro::do_rato(1.0, 2.0, PointerKind::Move, PointerButton::Primary, 7);
        e.pressure = 0.3;
        e.source = PointerSource::Pen(PenInfo::default());
        poe_tinta(&e);
        assert_eq!((tinta(), evento_ns()), (Tinta::de(&e), 7));
        tinta_do_rato();
        assert_eq!(tinta(), Tinta::RATO);
    }
}
