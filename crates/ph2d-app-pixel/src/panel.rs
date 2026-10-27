//! **O painel PIXEL do lado da família** (ADR-0192): os pedidos dele chegam pelo barramento
//! ([`PixelApp::pede`]) e aplicam-se aqui; o instantâneo que ele pinta sai daqui; e ele ABRE ao
//! entrar em Pixel Sprite ▸ Draw e FECHA ao sair (no flanco — entre os dois, o *Window → Pixel*
//! manda). A coluna da esquerda ([`publish_bars`]) e as amostras de cor dela também passam aqui.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{ToolBars, WidgetStore};
use ph2d_editor_core::pixel_panel::{NewSpriteMode, PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{Toast, ToastQueue};

use crate::{PixelApp, Tool};

/// O `Panel::ID` da `ph2d-panel-pixel-tools` (a família não vê o painel).
pub const PANEL_ID: &str = ph2d_editor_core::screens::hero::PANEL_PIXEL;
/// O `Panel::ID` da `ph2d-panel-pixel-timeline` (abre e fecha com o PIXEL; o `Tab` alterna-o).
pub const TIMELINE_ID: &str = ph2d_editor_core::screens::hero::PANEL_PIXEL_TIMELINE;

impl PixelApp {
    /// Um gesto do painel PIXEL (o dreno do barramento); aplica-se no [`frame`] deste quadro.
    pub fn pede(&mut self, r: PixelRequest) {
        self.pedidos.push(r);
    }

    /// Aplica um gesto do painel (os da paleta e da cor vão ao documento em Draw).
    pub fn aplica(&mut self, r: PixelRequest) {
        if crate::edicao::assenta_antes(&r) {
            self.assenta();
        }
        if self.aplica_selecao(&r)
            || self.aplica_edicao(&r)
            || self.aplica_camada(&r)
            || self.aplica_quadro(&r)
            || self.aplica_paleta(r.clone())
        {
            return;
        }
        match r {
            PixelRequest::Tool(t) => {
                crate::input::set_tool(self, t);
                if t == Tool::Transform
                    && let Some(key) = self.chave
                    && !self.levanta(key)
                {
                    self.recusa("pixel.select.refuse.nothing");
                }
            }
            // A última usada na mão; o mesmo clique abre (ou fecha) o flyout.
            PixelRequest::Group(g) => {
                let aberto = self.groups.open == Some(g);
                crate::input::set_tool(self, self.groups.face(g));
                self.groups.open = (!aberto).then_some(g);
            }
            PixelRequest::Options(o) => self.options = *o,
            PixelRequest::Fg(c) => self.fg = c,
            PixelRequest::Bg(c) => self.bg = c,
            PixelRequest::Swap => std::mem::swap(&mut self.fg, &mut self.bg),
            _ => {}
        }
    }
}

/// O nome de um modo de cor (as chaves do New Sprite).
#[must_use]
pub fn mode_label(m: NewSpriteMode) -> &'static str {
    let key = ph2d_editor_core::ids::NEW_SPRITE_MODES
        .iter()
        .find(|(n, ..)| *n == m)
        .map_or("pixel.new_sprite.mode.rgba", |(.., k)| *k);
    ph2d_i18n::tr(key)
}

/// **Uma vez por quadro, depois do modo**: aplica os pedidos (e o OK do *Color Mode*), abre/fecha o
/// painel no flanco do Draw, passa os avisos e devolve o instantâneo que a shell publica.
pub fn frame(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
) -> PixelPanelInfo {
    let editing = crate::mode::editing(world, hero);
    app.chave = editing.and_then(|b| crate::input::key_of(world, b));
    for r in std::mem::take(&mut app.pedidos) {
        app.aplica(r);
    }
    if let Some(c) = hero.store.take_color_mode_request()
        && let Some(key) = app.chave
    {
        app.converte(key, c);
    }
    if let Some(c) = app.abre_color_mode
        && let Some(doc) = app.chave.and_then(|k| app.docs.get(&k))
    {
        app.abre_color_mode = None;
        let s = &mut hero.store;
        s.open_color_mode_dialog(crate::paleta::modo(doc.sprite.mode()));
        s.set_color_mode_target(c.target);
        s.set_color_mode_dither(c.dither);
        s.set_color_mode_gray(c.gray);
    }
    if let Some(c) = hero.store.take_pixel_edit_request() {
        app.aplica_dialogo(c);
    }
    if let Some(kind) = app.abre_dialogo.take()
        && let Some(doc) = app.chave.and_then(|k| app.docs.get(&k))
    {
        let size = (doc.sprite.width(), doc.sprite.height());
        hero.store.open_pixel_edit_dialog(kind, size);
    }
    app.sincroniza_edicao(&mut hero.store, app.chave);
    for (erro, texto) in app.take_avisos() {
        let _ = toasts.push(if erro {
            Toast::error(texto)
        } else {
            Toast::info(texto)
        });
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
        hero.panel_visibility
            .insert(TIMELINE_ID, app.painel_em_edicao);
        // A banda de baixo é UMA: em Draw a linha do tempo é a do Pixel; a da casa volta ao sair.
        let casa = ph2d_editor_core::screens::hero::PANEL_TIMELINE;
        if app.painel_em_edicao {
            app.timeline_casa = hero.panel_visibility.get(casa).copied();
            hero.panel_visibility.insert(casa, false);
        } else if let Some(v) = app.timeline_casa.take() {
            hero.panel_visibility.insert(casa, v);
        }
    }
    if std::mem::take(&mut app.alterna_timeline) {
        let on = hero
            .panel_visibility
            .get(TIMELINE_ID)
            .copied()
            .unwrap_or(false);
        hero.panel_visibility.insert(TIMELINE_ID, !on);
    }
    let doc = app.chave.and_then(|k| app.doc(k));
    let palette = doc.map(|d| d.sprite.palette().to_vec());
    let mode = doc.map_or(NewSpriteMode::Rgba, |d| {
        crate::paleta::modo(d.sprite.mode())
    });
    let n = palette.as_ref().map_or(0, Vec::len);
    let layers = doc
        .map(|d| crate::camadas::rows(&d.sprite))
        .unwrap_or_default();
    let active_layer = doc.map(|d| d.layer.0);
    let (frames, cels, tags) = doc
        .map(|d| crate::quadros::timeline(&d.sprite))
        .unwrap_or_default();
    PixelPanelInfo {
        frames,
        active_frame: doc.map(|d| d.frame.0),
        cels,
        tags,
        playing: doc.is_some_and(|d| d.play.is_some()),
        onion: app.onion_on,
        onion_tint: app.onion.tint,
        editing: palette.is_some(),
        tool: app.tool,
        fg: app.fg,
        bg: app.bg,
        palette: palette.unwrap_or_default(),
        options: app.options,
        mode,
        selected: app.sel.iter().copied().filter(|&i| i < n).collect(),
        anchor: app.ultima.filter(|&i| i < n),
        sort_key: app.sort_key,
        layers,
        active_layer,
        has_clip: app.clip.is_some(),
        has_selection: doc.is_some_and(|d| d.sprite.mask().is_some()),
        size: doc.map_or((0, 0), |d| (d.sprite.width(), d.sprite.height())),
        transform: app
            .solta
            .as_ref()
            .filter(|s| Some(s.key) == app.chave)
            .map(crate::solta::Solta::info),
        can_reselect: doc.is_some_and(|d| d.sprite.last_mask().is_some()),
        presets: app.predefinicoes().to_vec(),
        groups: app.groups,
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
pub fn publish_bars(app: &PixelApp, hero: &mut HeroScreen, bars: ToolBars) {
    if app.painel_em_edicao && !bars.column.is_empty() {
        let readout = app
            .zoom
            .map(|n| ph2d_i18n::tr_with("pixel.zoom", &[("n", &n.to_string())]))
            .into_iter()
            .collect();
        hero.store.set_tool_bars(Some(ToolBars {
            readout,
            pixel_rail: true,
            ..bars
        }));
    } else if hero.store.tool_bars().is_some_and(|b| b.pixel_rail) {
        hero.store.set_tool_bars(None);
    }
}
