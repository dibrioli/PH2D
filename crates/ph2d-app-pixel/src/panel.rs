//! **O painel PIXEL do lado da família** (ADR-0192): os pedidos dele chegam pelo barramento
//! ([`PixelApp::pede`]) e aplicam-se aqui; o instantâneo que ele pinta sai daqui; e ele ABRE ao
//! entrar em Pixel Sprite ▸ Edit e FECHA ao sair (no flanco — entre os dois, o *Window → Pixel*
//! manda).

use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::HeroScreen;

use crate::PixelApp;

/// O `Panel::ID` da `ph2d-panel-pixel-tools` (a família não vê o painel).
pub const PANEL_ID: &str = "pixel";

impl PixelApp {
    /// Um gesto do painel PIXEL (o dreno do barramento); aplica-se no [`frame`] deste quadro.
    pub fn pede(&mut self, r: PixelRequest) {
        self.pedidos.push(r);
    }

    /// Aplica um gesto do painel.
    pub fn aplica(&mut self, r: PixelRequest) {
        match r {
            PixelRequest::Tool(t) => self.tool = t,
            PixelRequest::Fg(c) => self.fg = c,
            PixelRequest::Bg(c) => self.bg = c,
            PixelRequest::Swap => std::mem::swap(&mut self.fg, &mut self.bg),
        }
    }
}

/// **Uma vez por quadro, depois do modo**: aplica os pedidos, abre/fecha o painel no flanco do Edit e
/// devolve o instantâneo que a shell publica.
pub fn frame(app: &mut PixelApp, world: &ph2d_ecs::World, hero: &mut HeroScreen) -> PixelPanelInfo {
    for r in std::mem::take(&mut app.pedidos) {
        app.aplica(r);
    }
    let editing = crate::mode::editing(world, hero);
    if editing.is_some() != app.painel_em_edicao {
        app.painel_em_edicao = editing.is_some();
        hero.panel_visibility.insert(PANEL_ID, app.painel_em_edicao);
    }
    let palette = editing
        .and_then(|b| crate::input::key_of(world, b))
        .and_then(|k| app.doc(k))
        .map(|d| d.sprite.palette().to_vec());
    PixelPanelInfo {
        editing: palette.is_some(),
        tool: app.tool,
        fg: app.fg,
        bg: app.bg,
        palette: palette.unwrap_or_default(),
    }
}
