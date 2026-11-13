//! A lei da janela sem a barra do sistema (faixa única, line/BarraDeMenus onda 4) que o estado guarda:
//! o pedido dos botões, a borda sob o ponteiro e o Down que arrasta ou maximiza. A geometria e o pintor
//! dos botões estão em `screens::hero::window_buttons`, que reexporta isto.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_tokens::Spacing;

/// O que a faixa pede à janela (a shell lê-o com [`WidgetStore::take_window_request`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowRequest {
    Minimize,
    ToggleMaximize,
    Close,
}

/// Uma borda ou canto da janela, para redimensionar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// A espessura da borda que redimensiona (dentro da janela) e o lado do canto, em px lógicos.
#[must_use]
pub fn edge_band() -> (f32, f32) {
    (Spacing::Sm.px(), Spacing::Xl.px())
}

/// ⭐ **Que borda está sob o ponteiro?** `(x, y)` em px lógicos numa janela `w × h`. A janela
/// maximizada não tem bordas (`None`). Os cantos ganham a um lado maior que a borda: o alvo
/// diagonal é o que a mão procura e o que o fino deixaria escapar.
#[must_use]
pub fn edge_at(x: f32, y: f32, w: f32, h: f32, maximized: bool) -> Option<Edge> {
    if maximized || x < 0.0 || y < 0.0 || x >= w || y >= h {
        return None;
    }
    let (band, corner) = edge_band();
    let (n, s) = (y < band, y >= h - band);
    let (wst, e) = (x < band, x >= w - band);
    let (n_c, s_c) = (y < corner, y >= h - corner);
    let (w_c, e_c) = (x < corner, x >= w - corner);
    Some(if (n && w_c) || (wst && n_c) {
        Edge::NorthWest
    } else if (n && e_c) || (e && n_c) {
        Edge::NorthEast
    } else if (s && w_c) || (wst && s_c) {
        Edge::SouthWest
    } else if (s && e_c) || (e && s_c) {
        Edge::SouthEast
    } else if n {
        Edge::North
    } else if s {
        Edge::South
    } else if wst {
        Edge::West
    } else if e {
        Edge::East
    } else {
        return None;
    })
}

/// O que um Down primário pede à janela antes de chegar ao editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PressAction {
    Resize(Edge),
    Move,
    ToggleMaximize,
}

/// ⭐ **A decisão do Down** (a shell só a executa): a borda redimensiona; o fundo da faixa arrasta,
/// e o segundo Down nele dentro da janela do duplo clique maximiza/restaura.
#[derive(Clone, Debug, Default)]
pub struct WindowPress {
    last_backdrop_ns: Option<u128>,
}

impl WindowPress {
    /// O Down seguinte no fundo não faz par de duplo clique com o anterior.
    pub fn forget_backdrop(&mut self) {
        self.last_backdrop_ns = None;
    }

    /// `hit` é o alvo do editor sob o ponteiro; `t_ns` o relógio da shell.
    pub fn press(
        &mut self,
        x: f32,
        y: f32,
        size: (f32, f32),
        maximized: bool,
        hit: Option<NodeId>,
        t_ns: u128,
    ) -> Option<PressAction> {
        if let Some(e) = edge_at(x, y, size.0, size.1, maximized) {
            self.last_backdrop_ns = None;
            return Some(PressAction::Resize(e));
        }
        if hit != Some(ids::MENUBAR_BACKDROP) {
            self.last_backdrop_ns = None;
            return None;
        }
        let duplo = self.last_backdrop_ns.is_some_and(|t0| {
            t_ns.saturating_sub(t0) <= crate::interaction::DOUBLE_CLICK_WINDOW_NS
        });
        if duplo {
            self.last_backdrop_ns = None;
            Some(PressAction::ToggleMaximize)
        } else {
            self.last_backdrop_ns = Some(t_ns);
            Some(PressAction::Move)
        }
    }
}
