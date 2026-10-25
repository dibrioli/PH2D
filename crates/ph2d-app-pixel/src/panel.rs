//! **O painel PIXEL do lado da família** (ADR-0192): os pedidos dele chegam pelo barramento
//! ([`PixelApp::pede`]) e aplicam-se aqui; o instantâneo que ele pinta sai daqui; e ele ABRE ao
//! entrar em Pixel Sprite ▸ Draw e FECHA ao sair (no flanco — entre os dois, o *Window → Pixel*
//! manda). A coluna da esquerda ([`publish_bars`]) e as amostras de cor dela também passam aqui.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{BarButton, ToolBars, WidgetStore};
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::HeroScreen;

use crate::PixelApp;

/// O `Panel::ID` da `ph2d-panel-pixel-tools` (a família não vê o painel).
pub const PANEL_ID: &str = ph2d_editor_core::screens::hero::PANEL_PIXEL;

impl PixelApp {
    /// Um gesto do painel PIXEL (o dreno do barramento); aplica-se no [`frame`] deste quadro.
    pub fn pede(&mut self, r: PixelRequest) {
        self.pedidos.push(r);
    }

    /// Aplica um gesto do painel.
    pub fn aplica(&mut self, r: PixelRequest) {
        match r {
            PixelRequest::Tool(t) => crate::input::set_tool(self, t),
            PixelRequest::Options(o) => self.options = o,
            PixelRequest::Fg(c) => self.fg = c,
            PixelRequest::Bg(c) => self.bg = c,
            PixelRequest::Swap => std::mem::swap(&mut self.fg, &mut self.bg),
        }
    }
}

/// **Uma vez por quadro, depois do modo**: aplica os pedidos, abre/fecha o painel no flanco do Draw e
/// devolve o instantâneo que a shell publica.
pub fn frame(app: &mut PixelApp, world: &ph2d_ecs::World, hero: &mut HeroScreen) -> PixelPanelInfo {
    for r in std::mem::take(&mut app.pedidos) {
        app.aplica(r);
    }
    let [fg, bg] = &mut app.semeadas;
    sync_swatch(
        &mut hero.store,
        ph2d_editor_core::ids::PIXEL_FG,
        &mut app.fg,
        fg,
    );
    sync_swatch(
        &mut hero.store,
        ph2d_editor_core::ids::PIXEL_BG,
        &mut app.bg,
        bg,
    );
    let editing = crate::mode::editing(world, hero);
    let selecionado = hero.gizmo.mode.active().is_none()
        && hero
            .gizmo
            .selection
            .is_some_and(|b| crate::input::key_of(world, b).is_some());
    hero.store
        .set_family_owns_canvas(editing.is_some() || selecionado);
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
        options: app.options,
    }
}

/// A amostra `id` e a cor dela: o que o seletor partilhado escreveu desde o último quadro entra na
/// família; depois a amostra volta a mostrar a cor da família (o conta-gotas, o `X`, a paleta).
fn sync_swatch(store: &mut WidgetStore, id: NodeId, cor: &mut [u8; 4], semeada: &mut [u8; 4]) {
    if let Some(w) = store.widget_color(id)
        && w != *semeada
    {
        *cor = w;
    }
    store.set_widget_color(id, *cor);
    *semeada = *cor;
}

/// ⭐ **A coluna da esquerda em Draw** (dono 25/10: as ferramentas à esquerda, como no Painter):
/// publica as barras do Pixel com os grupos que o painel compõe e a dica dele, e o zoom no rodapé;
/// fora do Draw limpa só as SUAS. A marca `pixel_rail` guarda o seletor de modo na fila de cima.
pub fn publish_bars(
    app: &PixelApp,
    hero: &mut HeroScreen,
    column: Vec<Vec<BarButton>>,
    hint: String,
) {
    if app.painel_em_edicao && !column.is_empty() {
        let readout = app
            .zoom
            .map(|n| ph2d_i18n::tr_with("pixel.zoom", &[("n", &n.to_string())]))
            .into_iter()
            .collect();
        hero.store.set_tool_bars(Some(ToolBars {
            column,
            hint,
            readout,
            pixel_rail: true,
            ..ToolBars::default()
        }));
    } else if hero.store.tool_bars().is_some_and(|b| b.pixel_rail) {
        hero.store.set_tool_bars(None);
    }
}
