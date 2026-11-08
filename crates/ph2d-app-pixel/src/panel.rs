//! **O painel PIXEL do lado da família** (ADR-0196): os pedidos dele chegam pelo barramento
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
/// O `Panel::ID` da `ph2d-panel-pixel-timeline` (abre e fecha com o PIXEL; o `F4` alterna-o).
pub const TIMELINE_ID: &str = ph2d_editor_core::screens::hero::PANEL_PIXEL_TIMELINE;

impl PixelApp {
    /// Um gesto do painel PIXEL (o dreno do barramento); aplica-se no [`frame`] deste quadro.
    pub fn pede(&mut self, r: PixelRequest) {
        self.pedidos.push(r);
    }

    /// Os pedidos à espera do quadro (para os gates).
    #[doc(hidden)]
    #[must_use]
    pub fn pending(&self) -> &[PixelRequest] {
        &self.pedidos
    }

    /// Aplica um gesto do painel (os da paleta e da cor vão ao documento em Draw).
    pub fn aplica(&mut self, r: PixelRequest) {
        if crate::edicao::assenta_antes(&r) {
            self.assenta();
            crate::viva::assenta(self);
        }
        if let (PixelRequest::UserData(o, e), Some(k)) = (&r, self.chave) {
            let op = self
                .docs
                .get(&k)
                .and_then(|d| ph2d_pixel_ponte::user::op(&d.sprite, *o, e));
            self.passo_nomeado(k, op, "pixel.history.user_data");
            return;
        }
        if crate::tiles::aplica_pedido(self, &r)
            || self.aplica_selecao(&r)
            || self.aplica_edicao(&r)
            || self.aplica_camada(&r)
            || self.aplica_intervalo(&r)
            || self.aplica_quadro(&r)
            || self.aplica_pincel(&r)
            || self.aplica_ficheiro(&r)
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
            // P6d: um campo da forma (ou o Fill) re-cozinha a forma viva na hora.
            PixelRequest::Options(o) => {
                let antes = self.options;
                crate::viva::opcoes(self, *o);
                crate::texto::normaliza(&antes, &mut self.options);
                crate::texto::refaz(self);
            }
            PixelRequest::Fg(c) => self.escolhe_fg(c),
            PixelRequest::Bg(c) => self.bg = c,
            PixelRequest::Swap => {
                let bg = self.bg;
                self.bg = self.fg;
                self.escolhe_fg(bg);
            }
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
    crate::cartao::publica(app, world, hero, editing);
    crate::smoke::conduz_o_cartao(world, hero);
    crate::smoke_intervalo::conduz(hero);
    crate::smoke_efeitos::conduz(app, hero);
    crate::smoke_exporta::conduz(app);
    hero.store.set_pixel_edit_avoid(app.na_tela);
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
    if let Some(a) = hero.store.take_pixel_file_request() {
        app.pede(PixelRequest::File(a));
        for r in std::mem::take(&mut app.pedidos) {
            app.aplica(r);
        }
    }
    crate::exportar::frame(app, hero);
    crate::tiles::frame(app, hero);
    crate::slices::frame(app, hero);
    historia(app);
    if let Some(c) = hero.store.take_pixel_edit_request() {
        app.aplica_dialogo(c);
    }
    if let Some(kind) = app.abre_dialogo.take()
        && let Some(doc) = app.chave.and_then(|k| app.docs.get(&k))
    {
        let size = (doc.sprite.width(), doc.sprite.height());
        hero.store
            .set_pixel_filter_kernels(crate::filtros::nomes_dos_kernels());
        hero.store
            .set_pixel_filter_kernel_channels(crate::filtros::canais_dos_kernels());
        hero.store.open_pixel_edit_dialog(kind, size);
    }
    let curva = hero
        .hit_index
        .rect_for(ph2d_editor_core::ids::PIXEL_FILTER_CURVE);
    hero.store.drive_pixel_curve(curva);
    crate::filtros::acompanha(app, &hero.store);
    app.sincroniza_edicao(&mut hero.store, app.chave);
    if let Some(key) = app.chave {
        app.acompanha_espaco(key);
    }
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
    let sujeito = (editing.is_some() || selecionado)
        .then_some(hero.gizmo.mode.locked_entity().or(hero.gizmo.selection))
        .flatten()
        .map(|b| (b, editing.is_some(), hero.store.active_layout()));
    // No 1.º quadro a ordem dos painéis ainda está vazia: os que se pintam entram por cima do pedido.
    if sujeito != app.sujeito_do_painel && !hero.store.panel_z_order().is_empty() {
        app.sujeito_do_painel = sujeito;
        painel_do_sujeito(app, hero, sujeito);
    }
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
    if std::mem::take(&mut app.mostra_timeline) {
        hero.panel_visibility.insert(TIMELINE_ID, true);
    }
    let pv = ph2d_editor_core::screens::hero::PANEL_PIXEL_PREVIEW;
    let aberta = hero.panel_visibility.get(pv).copied().unwrap_or(false);
    let aberta = (aberta != std::mem::take(&mut app.preview.alternar))
        || std::mem::take(&mut app.preview.abrir);
    hero.panel_visibility.insert(pv, aberta);
    let doc = app.chave.and_then(|k| Some((k, app.docs.get(&k)?)));
    app.preview
        .publish(aberta, doc.map(|(k, d)| (k, &d.sprite, d.stamp, d.frame)));
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
    let palette_ok = palette.is_some();
    let user = doc.map_or_else(Default::default, |d| {
        ph2d_pixel_ponte::user::info(&d.sprite, d.layer, d.frame)
    });
    PixelPanelInfo {
        user,
        frames,
        active_frame: doc.map(|d| d.frame.0),
        cels,
        tags,
        range: doc.map(crate::state::Doc::range_info).unwrap_or_default(),
        playing: doc.is_some_and(|d| d.play.is_some()),
        playback: ph2d_editor_core::pixel_panel::PixelPlayback {
            speed: doc.map_or_else(Default::default, |d| d.speed),
            ..app.tocar
        },
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
        remap_pending: app.remap.as_ref().is_some_and(|r| Some(r.0) == app.chave),
        layers,
        active_layer,
        slice_keys: app
            .chave
            .map_or_else(Vec::new, |k| crate::slices::marcas_de_quadro(app, k)),
        active_tilemap: doc.is_some_and(|d| {
            d.sprite
                .layer(d.layer)
                .is_some_and(|l| l.tileset().is_some())
        }),
        has_clip: app.clip.is_some(),
        has_selection: doc.is_some_and(|d| d.sprite.mask().is_some()),
        text_font_name: if app.tool == Tool::Text {
            crate::texto::nome_da_fonte(app.options.text_font)
        } else {
            String::new()
        },
        text_fonts: if app.tool == Tool::Text {
            crate::texto::publica_previas();
            crate::texto::fontes()
        } else {
            Vec::new()
        },
        size: doc.map_or((0, 0), |d| (d.sprite.width(), d.sprite.height())),
        transform: app
            .solta
            .as_ref()
            .filter(|s| Some(s.key) == app.chave)
            .map(crate::solta::Solta::info)
            .or_else(|| {
                let g = app.gesture.as_ref().filter(|g| Some(g.key) == app.chave)?;
                g.viva.as_ref().map(crate::viva::Viva::info)
            }),
        can_reselect: doc.is_some_and(|d| d.sprite.last_mask().is_some()),
        presets: app.predefinicoes().to_vec(),
        groups: app.groups,
        live_shape: app.live_shape().is_some(),
        pattern: app.pincel.as_ref().map(|p| p.size()),
        brushes: if palette_ok {
            app.espacos_info()
        } else {
            Vec::new()
        },
        on_axis: app.sobre_eixo
            || app.gesture.as_ref().is_some_and(|g| {
                Some(g.key) == app.chave && matches!(g.kind, crate::gesture::Kind::Axis { .. })
            }),
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

/// O Pixel Sprite que manda nos painéis, se está em Draw, e a área de trabalho.
pub(crate) type Sujeito = (
    u64,
    bool,
    ph2d_editor_core::screens::task_layout::TaskLayout,
);

/// Dono, 05/11: **no flanco** de um Pixel Sprite passar a ser o sujeito (ou de a área de trabalho
/// mudar com ele), o painel é o dele: o Inspector em Object, o Pixel em Draw. O Bones gravado aberto
/// na arrumação sai da frente enquanto durar, e volta depois; reabri-lo à mão no meio vale.
fn painel_do_sujeito(app: &mut PixelApp, hero: &mut HeroScreen, sujeito: Option<Sujeito>) {
    use ph2d_editor_core::ids::{INSP_PANEL, PIXEL_PANEL};
    let Some((_, em_draw, _)) = sujeito else {
        if std::mem::take(&mut app.ossos_abertos) {
            hero.panel_visibility.insert("skeleton", true);
        }
        return;
    };
    app.ossos_abertos |= hero.is_panel_visible("skeleton");
    hero.panel_visibility.insert("skeleton", false);
    // Trocar de área de trabalho fecha tudo e abre a lista dela: o painel do sujeito reabre aqui.
    hero.panel_visibility
        .insert(if em_draw { PANEL_ID } else { "inspector" }, true);
    hero.store
        .bump_panel_z(if em_draw { PIXEL_PANEL } else { INSP_PANEL });
}

/// P12a: os cliques da janela UNDO HISTORY e a vista da história do documento em Draw.
fn historia(app: &mut PixelApp) {
    use ph2d_panel_pixel_history::canal::{self, HistoryRequest, HistoryView};
    for r in canal::take_requests() {
        let HistoryRequest::GoTo(n) = r;
        if let Some(key) = app.chave {
            app.go_to(key, n);
        }
    }
    let doc = app.chave.and_then(|k| app.docs.get(&k));
    canal::publish_view(doc.map_or_else(HistoryView::default, |d| HistoryView::of(&d.history)));
}
