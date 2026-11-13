//! A faixa única (line/BarraDeMenus, onda 4): a lei mora em
//! `ph2d_editor_core::screens::hero::window_buttons`; aqui só o `winit`. `PH2D_FAIXA_UNICA=1` liga-a; por
//! omissão a janela tem a barra do sistema (smoke do dono 12/11: a caneta não arrasta). Prova no KWin: `docs/BarraDeMenus/ferramentas/kwin_faixa_unica.sh`.

use crate::App;
use ph2d_editor_core::screens::hero::window_buttons::{
    self as wb, Edge, PressAction, WindowRequest,
};
use winit::event_loop::ActiveEventLoop;
use winit::window::{CursorIcon, ResizeDirection, Window};

/// A janela nasce sem a barra do sistema? Só com `PH2D_FAIXA_UNICA=1` até a caneta arrastar.
pub(crate) fn ligada() -> bool {
    std::env::var("PH2D_FAIXA_UNICA").is_ok_and(|v| v == "1")
}

fn tamanho(w: &Window) -> (f32, f32) {
    let s = w.inner_size();
    (s.width as f32, s.height as f32)
}

fn direcao(e: Edge) -> ResizeDirection {
    match e {
        Edge::North => ResizeDirection::North,
        Edge::South => ResizeDirection::South,
        Edge::East => ResizeDirection::East,
        Edge::West => ResizeDirection::West,
        Edge::NorthEast => ResizeDirection::NorthEast,
        Edge::NorthWest => ResizeDirection::NorthWest,
        Edge::SouthEast => ResizeDirection::SouthEast,
        Edge::SouthWest => ResizeDirection::SouthWest,
    }
}

impl App {
    /// O Down primário em `(x, y)`: `true` = a janela tomou-o (arrasto, borda, maximizar).
    ///
    /// A caneta (`caneta`) chega fora do evento do winit: move e redimensiona pelo `down` dela
    /// (`ph2d_pen::porta::arrasta_janela`); sem caneta Wayland o Down segue para o editor.
    pub(crate) fn faixa_unica_down(&mut self, x: f32, y: f32, caneta: bool) -> bool {
        let (Some(w), Some(h)) = (
            self.window.as_ref(),
            self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()),
        ) else {
            return false;
        };
        let acao = wb::press(h, x, y, tamanho(w), Self::timestamp_ns());
        if std::env::var_os("PH2D_FAIXA_LOG").is_some() {
            eprintln!(
                "[faixa única] down ({x:.0}, {y:.0}) alvo {:?} → {acao:?}",
                h.hit_index.hit(x, y)
            );
        }
        let r = match acao {
            None => return false,
            Some(PressAction::Resize(e)) if caneta => {
                return ph2d_pen::porta::arrasta_janela(w, Some(direcao(e)));
            }
            Some(PressAction::Move) if caneta => return ph2d_pen::porta::arrasta_janela(w, None),
            Some(PressAction::Resize(e)) => w.drag_resize_window(direcao(e)),
            Some(PressAction::Move) => w.drag_window(),
            Some(PressAction::ToggleMaximize) => {
                w.set_maximized(!w.is_maximized());
                Ok(())
            }
        };
        if let Err(e) = r {
            eprintln!("[faixa única] o sistema recusou: {e}");
        }
        true
    }

    /// O cursor de redimensionar sobre a borda (antes de qualquer outro do editor).
    pub(crate) fn faixa_unica_cursor(
        &self,
        h: &ph2d_editor_core::screens::hero::HeroScreen,
    ) -> Option<CursorIcon> {
        let w = self.window.as_ref()?;
        let (x, y) = self.last_pointer;
        Some(match wb::hover_edge(h, x, y, tamanho(w))? {
            Edge::North | Edge::South => CursorIcon::NsResize,
            Edge::East | Edge::West => CursorIcon::EwResize,
            Edge::NorthWest | Edge::SouthEast => CursorIcon::NwseResize,
            Edge::NorthEast | Edge::SouthWest => CursorIcon::NeswResize,
        })
    }

    /// A cada quadro: diz ao editor se a janela é do app e cumpre o pedido dos três botões.
    pub(crate) fn faixa_unica_tick(&mut self, event_loop: &ActiveEventLoop) {
        let Some(w) = self.window.as_ref() else {
            return;
        };
        let frame = (!w.is_decorated()).then(|| w.is_maximized());
        let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        h.store.set_window_frame(frame);
        let pedido = h.store.take_window_request();
        if std::env::var_os("PH2D_FAIXA_LOG").is_some() {
            let largura = tamanho(w).0;
            let livre = wb::free_backdrop_x(h, largura).unwrap_or(-1.0);
            let linha = format!("fundo livre x={livre:.0} largura={largura:.0} pedido={pedido:?}");
            static ULTIMA: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
            if let Ok(mut u) = ULTIMA.lock()
                && *u != linha
            {
                eprintln!("[faixa única] {linha}");
                *u = linha;
            }
        }
        match pedido {
            Some(WindowRequest::Minimize) => w.set_minimized(true),
            Some(WindowRequest::ToggleMaximize) => w.set_maximized(!w.is_maximized()),
            // O mesmo caminho do fechar do sistema (o jogo no palco pára primeiro).
            Some(WindowRequest::Close) if self.undo.janela_do_jogo.is_some() || self.no_palco() => {
                self.jogo_para()
            }
            Some(WindowRequest::Close) => self.on_close_request(event_loop),
            None => {}
        }
    }
}
