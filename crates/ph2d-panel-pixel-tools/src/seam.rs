//! A costura do painel PIXEL — o registo dos widgets e o pedido que cada gesto faz.
//!
//! ⚠️ Os botões registam-se INCONDICIONALMENTE (quem decide se o clique é possível é a PINTURA:
//! sem hit-rect não há `Click`). As células da paleta registam-se por quadro, por quem as pinta.

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{
    InteractiveState, PanelRowDrop, PanelRowFamily, WidgetEvent, WidgetStore,
};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelRequest, PixelSelect};
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_i18n::{tr, tr_with};

pub(crate) fn populate(store: &mut WidgetStore) {
    let botoes = ids::TOOLS.iter().map(|(_, id)| *id);
    for id in botoes.chain([ids::PIXEL_CLOSE, ids::PIXEL_SWAP]) {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    // O tooltip diz o atalho: «Pencil (B)».
    for (t, id) in ids::TOOLS {
        let dica = tr_with(
            "pixel.tool_switched",
            &[("tool", &tr(t.label_key())), ("key", &t.shortcut_label())],
        );
        store.set_tooltip(id, dica);
    }
    store.set_tooltip(ids::PIXEL_SWAP, tr("panel.pixel.swap_tip"));
    // As amostras da coluna abrem o seletor partilhado num Down — marcadas AQUI, porque a coluna as
    // pinta com o painel fechado.
    store.register_picker_swatch(ids::PIXEL_FG);
    store.register_picker_swatch(ids::PIXEL_BG);
    // As opções: segmentos são botões, caixas são caixas, números são campos com faixa.
    let segmentos = ids::SHAPES.iter().map(|(_, id)| *id).chain(ids::SEGMENTS);
    let inks = ids::INKS.iter().map(|(_, id)| *id);
    let sorts = ids::SORT_KEYS.iter().map(|(_, id)| *id);
    let acoes = ids::ACTIONS.iter().map(|(id, ..)| *id);
    let presets = ids::PRESET_ACTIONS.iter().map(|(id, ..)| *id);
    let seletores = ids::SELECTORS.iter().map(|(_, id)| *id);
    let mais = inks
        .chain(sorts)
        .chain(acoes)
        .chain(presets)
        .chain(seletores)
        .chain([ids::COLOR_MODE]);
    for id in segmentos.chain(mais) {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    for id in ids::CHECKS {
        let state = CheckboxState::Normal;
        let value = CheckboxValue::Unchecked;
        store.register(id, InteractiveState::Checkbox { state, value });
    }
    let omissao = PixelOptions::default();
    for (id, (min, max, step)) in ids::NUMBERS {
        let value = numero(&omissao, id);
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: value.to_string(),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, min, max, step);
    }
    let n = MAX_COLORS;
    store.register(
        ids::PALETTE_SIZE,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: n,
            buffer: n.to_string(),
            caret: 0,
            last_committed: n,
            selection_anchor: None,
        },
    );
    store.set_number_range(ids::PALETTE_SIZE, 1.0, n, 1.0);
    // As áreas do seletor de cor: o clique lê a posição do ponteiro contra o rect pintado.
    for (id, _) in ids::SELECTOR_AREAS {
        store.register(id, InteractiveState::Plain);
    }
    // O *Edit color*: um Down abre o seletor partilhado (a família lê-o de volta).
    store.register_picker_swatch(ids::PIXEL_PAL_EDIT);
    store.register(ids::PIXEL_PAL_EDIT, InteractiveState::Plain);
    // As camadas: as acções são botões, a opacidade um campo 0..255, o modo um menu.
    for (id, ..) in ids::LAYER_ACTIONS {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    store.register(
        ids::LAYER_OPACITY,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 255.0,
            buffer: "255".to_owned(),
            caret: 0,
            last_committed: 255.0,
            selection_anchor: None,
        },
    );
    store.set_number_range(ids::LAYER_OPACITY, 0.0, 255.0, 1.0);
    store.register(
        ids::LAYER_BLEND,
        InteractiveState::Dropdown {
            state: ph2d_editor_core::widget::DropdownState::Normal,
            open: false,
            selected_index: Some(0),
        },
    );
    for i in 0..ph2d_editor_core::pixel_panel::PixelBlend::ALL.len() {
        let state = ButtonState::Normal;
        store.register(ids::blend_option_id(i), InteractiveState::Button { state });
    }
}

/// O registo POR QUADRO das partes das linhas de camada (os ids derivam do id da camada), com a
/// dica de cada ícone.
pub(crate) fn register_layers(
    store: &mut WidgetStore,
    rows: &[ph2d_editor_core::pixel_panel::PixelLayerRow],
) {
    let dicas = [
        (ids::LayerWidget::Eye, "panel.pixel.layer.eye"),
        (ids::LayerWidget::Lock, "panel.pixel.layer.lock"),
        (ids::LayerWidget::Solo, "panel.pixel.layer.solo"),
        (ids::LayerWidget::Collapse, "panel.pixel.layer.collapse"),
    ];
    for r in rows {
        for w in ids::LayerWidget::ALL {
            let id = ids::layer_widget_id(r.id, w);
            let state = ButtonState::Normal;
            store.register_if_absent(id, InteractiveState::Button { state });
            if let Some((_, k)) = dicas.iter().find(|(x, _)| *x == w) {
                store.set_tooltip(id, tr(k));
            }
        }
    }
}

/// Semeia e foca o campo do renomear UMA vez quando ele abre (semear a cada quadro apagaria o que
/// o artista escreve); se a camada sumiu, o renomear fecha.
pub(crate) fn seed_rename(st: &mut crate::PixelToolsPanelState, store: &mut WidgetStore) {
    let Some((id, false)) = st.rename else {
        return;
    };
    let Some(name) = with_current(|i| i.layers.iter().find(|r| r.id == id).map(|r| r.name.clone()))
    else {
        st.rename = None;
        return;
    };
    let caret = name.len();
    store.register(
        ids::LAYER_RENAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: name,
            caret,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::LAYER_RENAME));
    store.mark_cancel_on_escape(ids::LAYER_RENAME);
    st.rename = Some((id, true));
}

/// O pedido de um clique numa parte de linha de camada.
fn pedido_camada(id: NodeId) -> Option<PixelRequest> {
    use ids::LayerWidget as W;
    with_current(|info| {
        if !info.editing {
            return None;
        }
        if let Some((_, a, _)) = ids::LAYER_ACTIONS.iter().find(|(b, ..)| *b == id) {
            return Some(PixelRequest::LayerAction(*a));
        }
        info.layers.iter().find_map(|r| {
            W::ALL.iter().find_map(|&w| {
                (ids::layer_widget_id(r.id, w) == id).then_some(match w {
                    W::Row => PixelRequest::LayerSelect(r.id),
                    W::Eye => PixelRequest::LayerVisible(r.id),
                    W::Lock => PixelRequest::LayerLocked(r.id),
                    W::Solo => PixelRequest::LayerSolo(r.id),
                    W::Collapse => PixelRequest::LayerCollapsed(r.id),
                })
            })
        })
    })
}

/// O maior tamanho de paleta (um índice é um byte).
const MAX_COLORS: f64 = ph2d_editor_core::pixel_panel::PIXEL_MAX_COLORS as f64;

/// O valor do campo `id` nas opções.
pub(crate) fn numero(o: &PixelOptions, id: NodeId) -> f64 {
    match id {
        ids::SIZE => f64::from(o.size),
        ids::ANGLE => f64::from(o.angle),
        ids::CORNER => f64::from(o.corner_radius),
        ids::TOLERANCE => f64::from(o.tolerance),
        ids::SPRAY_WIDTH => f64::from(o.spray_width),
        ids::REPLACE_TOLERANCE => f64::from(o.replace_tolerance),
        _ => f64::from(o.spray_speed),
    }
}

/// As opções com o campo `id` em `v` (preso à faixa dele, inteiro).
fn com_numero(mut o: PixelOptions, id: NodeId, v: f64) -> PixelOptions {
    let (min, max) = ids::NUMBERS
        .iter()
        .find(|(n, _)| *n == id)
        .map_or((0.0, 0.0), |(_, (a, b, _))| (*a, *b));
    if !v.is_finite() {
        return o;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "presa à faixa do campo antes da conversão"
    )]
    let (u, i) = {
        let r = v.round().clamp(min, max);
        (r as u32, r as i32)
    };
    match id {
        ids::SIZE => o.size = u,
        ids::ANGLE => o.angle = i,
        ids::CORNER => o.corner_radius = u,
        ids::TOLERANCE => o.tolerance = u8::try_from(u).unwrap_or(u8::MAX),
        ids::SPRAY_WIDTH => o.spray_width = u,
        ids::SPRAY_SPEED => o.spray_speed = u,
        ids::REPLACE_TOLERANCE => o.replace_tolerance = u8::try_from(u).unwrap_or(u8::MAX),
        _ => {}
    }
    o
}

/// A caixa `id` nas opções (`&mut` do campo dela).
pub(crate) fn caixa(o: &mut PixelOptions, id: NodeId) -> Option<&mut bool> {
    Some(match id {
        ids::PIXEL_PERFECT => &mut o.pixel_perfect,
        ids::CONTIGUOUS => &mut o.contiguous,
        ids::SYM_H => &mut o.sym_h,
        ids::SYM_V => &mut o.sym_v,
        ids::SYM_D1 => &mut o.sym_d1,
        ids::SYM_D2 => &mut o.sym_d2,
        ids::TILE_X => &mut o.tile_x,
        ids::TILE_Y => &mut o.tile_y,
        _ => return None,
    })
}

/// As opções com o segmento `id` escolhido, se é um.
fn com_segmento(mut o: PixelOptions, id: NodeId) -> Option<PixelOptions> {
    if let Some((shape, _)) = ids::SHAPES.iter().find(|(_, s)| *s == id) {
        o.shape = *shape;
        return Some(o);
    }
    if let Some((ink, _)) = ids::INKS.iter().find(|(_, s)| *s == id) {
        o.ink = *ink;
        return Some(o);
    }
    match id {
        ids::CONNECT_4 => o.eight = false,
        ids::CONNECT_8 => o.eight = true,
        ids::SAMPLE_LAYER => o.sample_visible = false,
        ids::SAMPLE_VISIBLE => o.sample_visible = true,
        _ => return None,
    }
    Some(o)
}

/// O registo POR QUADRO das células da paleta (os ids derivam do índice) — e elas são as células
/// arrastáveis da grelha (a família `PixelPalette` do despacho); sem paleta, nenhuma.
pub(crate) fn register_palette(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        store.register_if_absent(ids::palette_id(i), InteractiveState::Plain);
    }
    store.set_panel_row_ids(PanelRowFamily::PixelPalette, (0..n).map(ids::palette_id).collect());
}

/// O registo POR QUADRO dos botões das predefinições gravadas.
pub(crate) fn register_presets(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        let state = ButtonState::Normal;
        store.register_if_absent(ids::preset_id(i), InteractiveState::Button { state });
    }
}

/// A célula da paleta deste id.
fn celula(id: NodeId, n: usize) -> Option<usize> {
    (0..n).find(|&i| ids::palette_id(i) == id)
}

/// Uma amostra largada sobre outra célula: a selecção muda-se para lá.
fn larga(dragged: NodeId, drop: PanelRowDrop) -> Option<PixelRequest> {
    let PanelRowDrop::Inside(alvo) = drop else {
        return None;
    };
    with_current(|info| {
        let n = info.palette.len();
        let (from, to) = (celula(dragged, n)?, celula(alvo, n)?);
        (info.editing && from != to).then_some(PixelRequest::DragSwatch { from, to })
    })
}

/// **Um clique numa área do seletor**: a posição do ponteiro no rect pintado dá a cor (esquerdo =
/// frente, direito = fundo, que guarda o alfa dele).
/// **O arrasto no seletor** (smoke do dono 25/10: «os sliders da roda não arrastam, apenas
/// clicam»): com o botão premido sobre o quadro ou uma barra, a cada quadro em que o ponteiro se
/// moveu a cor de frente segue-o — a mesma conta do clique, fora do rect presa à borda.
pub(crate) fn arrasta_seletor(st: &mut crate::PixelToolsPanelState, host: &mut dyn PanelHostInternal) {
    let activo = host.store().active_id();
    let Some(id) = activo.filter(|a| ids::SELECTOR_AREAS.iter().any(|(p, _)| p == a)) else {
        st.seletor.arrasto = None;
        return;
    };
    let ponto = host.store().last_pointer();
    if ponto.is_none() || ponto == st.seletor.arrasto {
        return;
    }
    st.seletor.arrasto = ponto;
    if let Some(r) = escolhe(st, host, id) {
        host.bus_mut().push(EditorAction::Pixel(r));
    }
}

/// Quantas cores mandadas o seletor reconhece como suas (o atraso do instantâneo é de um quadro;
/// a folga cobre um arrasto rápido com vários quadros entre o pedido e o instantâneo).
const MANDADAS: usize = 8;

fn escolhe(
    st: &mut crate::PixelToolsPanelState,
    host: &dyn PanelHostInternal,
    id: NodeId,
) -> Option<PixelRequest> {
    use crate::seletor;
    let (k, fundo) = ids::SELECTOR_AREAS.iter().enumerate().find_map(|(k, (a, b))| {
        if *a == id {
            Some((k, false))
        } else if *b == id {
            Some((k, true))
        } else {
            None
        }
    })?;
    let r = st.seletor.rects[k]?;
    let [px, py] = host.store().last_pointer()?;
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let u = f64::from((px - r.x) / r.w).clamp(0.0, 1.0);
    let v = f64::from((py - r.y) / r.h).clamp(0.0, 1.0);
    let (modo, fg, bg, editing) = with_current(|i| (i.options.selector, i.fg, i.bg, i.editing));
    if !editing {
        return None;
    }
    let p = st.seletor.params;
    let [r0, g0, b0, a0] = fg;
    let (rgb, alfa, p) = match k {
        0 => {
            let p = seletor::no_quadro(modo, p, u, v);
            (seletor::cor(modo, p), a0, p)
        }
        1 => {
            let p = seletor::na_barra(modo, p, u);
            (seletor::cor(modo, p), a0, p)
        }
        _ => ([r0, g0, b0], seletor::alfa(u), p),
    };
    let [r, g, b] = rgb;
    if fundo {
        let a = if k == 2 { alfa } else { bg[3] };
        return Some(PixelRequest::Bg([r, g, b, a]));
    }
    let c = [r, g, b, alfa];
    st.seletor.params = p;
    st.seletor.de = Some((modo, c));
    let m = &mut st.seletor.mandadas;
    m.push(c);
    let excesso = m.len().saturating_sub(MANDADAS);
    m.drain(..excesso);
    Some(PixelRequest::Fg(c))
}

/// O pedido de um clique, se o id é deste painel e há sprite em edição.
fn pedido(id: ph2d_a11y::NodeId, how: PixelSelect) -> Option<PixelRequest> {
    with_current(|info| {
        if !info.editing {
            return None;
        }
        if let Some((t, _)) = ids::TOOLS.iter().find(|(_, b)| *b == id) {
            return Some(PixelRequest::Tool(*t));
        }
        if id == ids::PIXEL_SWAP {
            return Some(PixelRequest::Swap);
        }
        if let Some(o) = com_segmento(info.options, id) {
            return Some(PixelRequest::Options(o));
        }
        if let Some((k, _)) = ids::SORT_KEYS.iter().find(|(_, b)| *b == id) {
            return Some(PixelRequest::SortKey(*k));
        }
        if let Some((_, r, _)) = ids::ACTIONS
            .iter()
            .chain(&ids::PRESET_ACTIONS)
            .find(|(b, ..)| *b == id)
        {
            return Some(r.clone());
        }
        if let Some((sel, _)) = ids::SELECTORS.iter().find(|(_, b)| *b == id) {
            let o = PixelOptions {
                selector: *sel,
                ..info.options
            };
            return Some(PixelRequest::Options(o));
        }
        if let Some(nome) = (0..info.presets.len())
            .find(|&i| ids::preset_id(i) == id)
            .map(|i| info.presets[i].clone())
        {
            return Some(PixelRequest::LoadPreset(nome));
        }
        info.palette.iter().enumerate().find_map(|(i, c)| {
            if ids::palette_id(i) == id {
                Some(PixelRequest::Swatch { index: i, how })
            } else if ids::palette_bg_id(i) == id {
                Some(PixelRequest::Bg(*c))
            } else {
                None
            }
        })
    })
}

pub(crate) fn apply_event(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let id = match ev {
        WidgetEvent::DoubleClick(id) => {
            let alvo = with_current(|i| {
                i.layers
                    .iter()
                    .find(|r| ids::layer_widget_id(r.id, ids::LayerWidget::Row) == id)
                    .map(|r| r.id)
            });
            return match alvo {
                Some(layer) => {
                    st.rename = Some((layer, false));
                    EventOutcome::Consumed
                }
                None => EventOutcome::Ignored,
            };
        }
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) if id == ids::LAYER_RENAME => {
            let Some((layer, _)) = st.rename.take() else {
                return EventOutcome::Consumed;
            };
            let name = match host.store().get(ids::LAYER_RENAME) {
                Some(InteractiveState::TextInput { text, .. }) => text.trim().to_owned(),
                _ => String::new(),
            };
            if host.store().focus_id() == Some(ids::LAYER_RENAME) {
                host.store_mut().set_focus(None);
            }
            return empurra(host, Some(PixelRequest::LayerRename { id: layer, name }));
        }
        WidgetEvent::Cancel(id) if id == ids::LAYER_RENAME => {
            st.rename = None;
            if host.store().focus_id() == Some(ids::LAYER_RENAME) {
                host.store_mut().set_focus(None);
            }
            return EventOutcome::Consumed;
        }
        WidgetEvent::ValueChanged(id) if id == ids::LAYER_OPACITY => {
            let v = host.store().number_value(id).filter(|v| v.is_finite());
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "presa a 0..=255 antes da conversão"
            )]
            let pedido = v.map(|v| PixelRequest::LayerOpacity(v.round().clamp(0.0, 255.0) as u8));
            let pedido = pedido.filter(|_| with_current(|i| i.editing));
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id) if id == ids::LAYER_BLEND => return EventOutcome::Consumed,
        WidgetEvent::Click(id)
            if (0..ph2d_editor_core::pixel_panel::PixelBlend::ALL.len())
                .any(|i| ids::blend_option_id(i) == id) =>
        {
            let i = (0..ph2d_editor_core::pixel_panel::PixelBlend::ALL.len())
                .find(|&i| ids::blend_option_id(i) == id)
                .unwrap_or(0);
            if let Some(InteractiveState::Dropdown {
                open,
                selected_index,
                ..
            }) = host.store_mut().get_mut(ids::LAYER_BLEND)
            {
                *open = false;
                *selected_index = Some(i);
            }
            let b = ph2d_editor_core::pixel_panel::PixelBlend::ALL[i];
            let pedido = with_current(|info| info.editing.then_some(PixelRequest::LayerBlend(b)));
            return empurra(host, pedido);
        }
        WidgetEvent::PanelRowReparent {
            family: PanelRowFamily::PixelPalette,
            dragged,
            drop,
        } => return empurra(host, larga(dragged, drop)),
        WidgetEvent::Click(id)
            if ids::SELECTOR_AREAS
                .iter()
                .any(|(a, b)| *a == id || *b == id) =>
        {
            let pedido = escolhe(st, host, id);
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id) if pedido_camada(id).is_some() => {
            return empurra(host, pedido_camada(id));
        }
        WidgetEvent::Click(id) => id,
        WidgetEvent::Toggled(id) => {
            // O valor vem do MODELO (o instantâneo), nunca do visual da caixa.
            let pedido = with_current(|info| {
                let mut o = info.options;
                let b = caixa(&mut o, id)?;
                *b = !*b;
                info.editing.then_some(PixelRequest::Options(o))
            });
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if ids::NUMBERS.iter().any(|(n, _)| *n == id) => {
            let v = host.store().number_value(id);
            let pedido = with_current(|info| {
                let v = v?;
                info.editing
                    .then(|| PixelRequest::Options(com_numero(info.options, id, v)))
            });
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if id == ids::PALETTE_SIZE => {
            let v = host.store().number_value(id);
            let pedido = with_current(|info| {
                let v = v.filter(|v| v.is_finite())?;
                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    reason = "presa a 1..=256 antes da conversão"
                )]
                let n = v.round().clamp(1.0, MAX_COLORS) as u16;
                info.editing.then_some(PixelRequest::PaletteSize(n))
            });
            return empurra(host, pedido);
        }
        _ => return EventOutcome::Ignored,
    };
    if id == ids::PIXEL_CLOSE {
        host.set_panel_visible(crate::PixelToolsPanel::ID, false);
        return EventOutcome::Consumed;
    }
    if id == ids::COLOR_MODE {
        let Some(mode) = with_current(|i| i.editing.then_some(i.mode)) else {
            return EventOutcome::Ignored;
        };
        host.store_mut().open_color_mode_dialog(mode);
        return EventOutcome::Consumed;
    }
    // `Shift` = da âncora até aqui, `Ctrl` = liga/desliga (a paleta do Aseprite).
    let how = if host.store().shift_held() {
        PixelSelect::Range
    } else if host.store().cmd_held() {
        PixelSelect::Toggle
    } else {
        PixelSelect::Only
    };
    empurra(host, pedido(id, how))
}

fn empurra(host: &mut dyn PanelHostInternal, pedido: Option<PixelRequest>) -> EventOutcome {
    match pedido {
        Some(r) => {
            host.bus_mut().push(EditorAction::Pixel(r));
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
