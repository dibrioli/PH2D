//! A costura do painel PIXEL — o registo dos widgets e o pedido que cada gesto faz.
//!
//! ⚠️ Os botões registam-se INCONDICIONALMENTE (quem decide se o clique é possível é a PINTURA:
//! sem hit-rect não há `Click`). As células da paleta registam-se por quadro, por quem as pinta.

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, PanelRowFamily, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelRequest, PixelSelect};
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_i18n::tr;

pub(crate) fn populate(store: &mut WidgetStore) {
    // O cabeçalho de cada cartão dobra: é uma secção dobrável, o cânone do Physics e do Inspector.
    store.mark_collapsible_section(ids::PIXEL_SEC_OPTIONS);
    store.mark_collapsible_section(ids::PIXEL_SEC_BRUSHES);
    store.mark_collapsible_section(ids::PIXEL_SEC_TRANSFORM);
    store.mark_collapsible_section(ids::PIXEL_SEC_COLORS);
    store.mark_collapsible_section(ids::PIXEL_SEC_LAYERS);
    store.mark_collapsible_section(ids::PIXEL_SEC_PALETTE);
    store.mark_collapsible_section(ids::PIXEL_SEC_PICKER);
    store.mark_collapsible_section(ids::PIXEL_SEC_SYMMETRY);
    store.mark_collapsible_section(ids::PIXEL_SEC_SELECT);
    store.mark_collapsible_section(ids::PIXEL_SEC_EDIT);
    store.mark_collapsible_section(ids::PIXEL_SEC_SPRITE);
    let botoes = ids::TOOLS.iter().map(|(_, id)| *id);
    let grupos = ids::GROUPS.iter().map(|(_, id)| *id);
    for id in botoes
        .chain(grupos)
        .chain([ids::PIXEL_CLOSE, ids::PIXEL_SWAP])
    {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    store.set_tooltip(ids::PIXEL_SWAP, tr("panel.pixel.swap_tip"));
    store.set_tooltip(ids::TEXT_FONT_DD, tr("panel.pixel.text.font_tip"));
    store.set_tooltip(ids::TEXT_SIZE, tr("panel.pixel.text.size_tip"));
    // As amostras da coluna abrem o seletor partilhado num Down — marcadas AQUI, porque a coluna as
    // pinta com o painel fechado.
    store.register_picker_swatch(ids::PIXEL_FG);
    store.register_picker_swatch(ids::PIXEL_BG);
    // As opções: segmentos são botões, caixas são caixas, números são campos com faixa.
    let segmentos = ids::SHAPES.iter().map(|(_, id)| *id).chain(ids::SEGMENTS);
    let segmentos = segmentos.chain(crate::ids_pintura::DITHER_MATRICES.map(|(_, id, _)| id));
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
    for id in ids::CHECKS.into_iter().chain([ids::LAYER_NORMAL]) {
        let state = CheckboxState::Normal;
        let value = CheckboxValue::Unchecked;
        store.register(id, InteractiveState::Checkbox { state, value });
    }
    let omissao = PixelOptions::default();
    for (id, (min, max, step)) in ids::NUMBERS.into_iter().chain(crate::ids_pintura::NUMBERS) {
        let value = crate::seam_opcoes::numero(&omissao, id);
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
    crate::seam_selecao::populate(store);
    crate::seam_formas::populate(store);
    crate::seam_pincel::populate(store);
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
            value: 255.0, // LITERAL-PX-OK: byte de cor/opacidade 0..=255 (normalização sRGB)
            buffer: "255".to_owned(),
            caret: 0,
            last_committed: 255.0, // LITERAL-PX-OK: byte de cor/opacidade 0..=255 (normalização sRGB)
            selection_anchor: None,
        },
    );
    store.set_number_range(ids::LAYER_OPACITY, 0.0, 255.0, 1.0); // LITERAL-PX-OK: byte de cor/opacidade 0..=255 (normalização sRGB)
    for id in [
        ids::LAYER_BLEND,
        ids::PALETTE_MENU,
        ids::PRESETS_MENU,
        ids::TEXT_FONT_DD,
    ] {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: ph2d_editor_core::widget::DropdownState::Normal,
                open: false,
                selected_index: Some(0),
            },
        );
    }
    store.set_tooltip(ids::PALETTE_MENU, tr("panel.pixel.palette_menu_tip"));
    store.set_tooltip(ids::PRESETS_MENU, tr("panel.pixel.presets_menu_tip"));
    for i in 0..ph2d_editor_core::pixel_panel::PixelBlend::ALL.len() {
        let state = ButtonState::Normal;
        store.register(ids::blend_option_id(i), InteractiveState::Button { state });
    }
}

pub(crate) use crate::seam_camadas::{desliga_o_que_a_normal_recusa, register_layers, seed_rename};
use crate::seam_camadas::{pedido_camada, pedido_normal};

/// O maior tamanho de paleta (um índice é um byte).
const MAX_COLORS: f64 = ph2d_editor_core::pixel_panel::PIXEL_MAX_COLORS as f64;

/// O registo POR QUADRO das células da paleta (os ids derivam do índice) — e elas são as células
/// arrastáveis da grelha (a família `PixelPalette` do despacho); sem paleta, nenhuma.
pub(crate) fn register_palette(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        store.register_if_absent(ids::palette_id(i), InteractiveState::Plain);
    }
    store.set_panel_row_ids(
        PanelRowFamily::PixelPalette,
        (0..n).map(ids::palette_id).collect(),
    );
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

/// Uma amostra largada: a selecção muda-se para onde a pré-visualização a mostrava (com Ctrl,
/// copia-se). A queda é a que se pintou no último quadro ([`crate::paint::Queda`]), nunca a célula
/// sob o rato: assim o que se larga é o que se viu, também nos intervalos entre as células.
fn larga(
    st: &mut crate::PixelToolsPanelState,
    dragged: NodeId,
    copy: bool,
) -> Option<PixelRequest> {
    let q = st.paleta.queda.take()?;
    with_current(|info| {
        let from = celula(dragged, info.palette.len())?;
        let to = if copy { q.copiar } else { q.mover };
        info.editing
            .then_some(PixelRequest::DragSwatch { from, to, copy })
    })
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
        if let Some((g, _)) = ids::GROUPS.iter().find(|(_, b)| *b == id) {
            return Some(PixelRequest::Group(*g));
        }
        if id == ids::PIXEL_SWAP {
            return Some(PixelRequest::Swap);
        }
        if let Some(r) = crate::seam_pincel::pedido(info, id) {
            return Some(r);
        }
        if let Some(o) = crate::seam_opcoes::com_segmento(info.options, id) {
            return Some(PixelRequest::Options(Box::new(o)));
        }
        if let Some((k, _)) = ids::SORT_KEYS.iter().find(|(_, b)| *b == id) {
            return Some(PixelRequest::SortKey(*k));
        }
        if let Some((_, r, _)) = ids::ACTIONS
            .iter()
            .chain(&ids::PRESET_ACTIONS)
            .chain(&ids::SELECT_ACTIONS)
            .chain(&ids::EDIT_ACTIONS)
            .chain(&ids::SPRITE_ACTIONS)
            .chain(&ids::GUIDE_ACTIONS)
            .find(|(b, ..)| *b == id)
        {
            return Some(r.clone());
        }
        if let Some((sel, _)) = ids::SELECTORS.iter().find(|(_, b)| *b == id) {
            let o = PixelOptions {
                selector: *sel,
                ..info.options
            };
            return Some(PixelRequest::Options(Box::new(o)));
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

/// Uma linha de um menu da barra da paleta fecha-o (os dois: só um abre de cada vez).
fn fecha_menus_da_paleta(host: &mut dyn PanelHostInternal) {
    for m in [ids::PALETTE_MENU, ids::PRESETS_MENU] {
        if let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(m) {
            *open = false;
        }
    }
}

pub(crate) fn apply_event(
    st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let Some(o) = crate::seam_user::evento(host, &ev) {
        return o;
    }
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
            let pedido = v.map(|v| PixelRequest::LayerOpacity(v.round().clamp(0.0, 255.0) as u8)); // LITERAL-PX-OK: byte de cor/opacidade 0..=255 (normalização sRGB)
            let pedido = pedido.filter(|_| with_current(|i| i.editing));
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id)
            if id == ids::LAYER_BLEND
                || id == ids::PALETTE_MENU
                || id == ids::PRESETS_MENU
                || id == ids::TEXT_FONT_DD
                || ids::THRESHOLDS.iter().any(|(_, t, _)| *t == id) =>
        {
            return EventOutcome::Consumed;
        }
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
            ..
        } => {
            let copy = host.store().cmd_held();
            return empurra(host, larga(st, dragged, copy));
        }
        WidgetEvent::Click(id)
            if ids::SELECTOR_AREAS
                .iter()
                .any(|(a, b)| *a == id || *b == id) =>
        {
            let pedido = crate::seam_seletor::escolhe(st, host, id);
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id) if id == ids::DYNAMICS => {
            st.pincel.dinamica = !st.pincel.dinamica;
            return EventOutcome::Consumed;
        }
        WidgetEvent::Toggled(id) if id == ids::DYN_STABILIZER => {
            let pedido = with_current(|i| crate::seam_pincel::alterna(i, id).filter(|_| i.editing));
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if ids::DYN_NUMBERS.iter().any(|(n, _)| *n == id) => {
            let v = host.store().number_value(id);
            let pedido = with_current(|i| crate::seam_pincel::campo(i, id, v));
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id) if crate::paint::fonte::linha(id).is_some() => {
            let pedido = crate::paint::fonte::escolhe(host, id);
            return empurra(host, pedido);
        }
        WidgetEvent::Click(id) if pedido_camada(id).is_some() => {
            return empurra(host, pedido_camada(id));
        }
        WidgetEvent::Click(id) => {
            if crate::paint::paleta_menu::e_linha(id) {
                fecha_menus_da_paleta(host);
            }
            id
        }
        WidgetEvent::Toggled(id) if id == ids::LAYER_NORMAL => {
            return empurra(host, pedido_normal());
        }
        WidgetEvent::Toggled(id) if crate::seam_camadas::pedido_efeito(id).is_some() => {
            return empurra(host, crate::seam_camadas::pedido_efeito(id));
        }
        WidgetEvent::Toggled(id) => {
            // O valor vem do MODELO (o instantâneo), nunca do visual da caixa.
            let pedido = with_current(|info| {
                let mut o = info.options;
                let b = crate::seam_opcoes::caixa(&mut o, id)?;
                *b = !*b;
                info.editing.then_some(PixelRequest::Options(Box::new(o)))
            });
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if crate::ids_pintura::e_numero(id) => {
            let v = host.store().number_value(id);
            let pedido = with_current(|info| {
                let v = v?;
                info.editing.then(|| {
                    PixelRequest::Options(Box::new(crate::seam_opcoes::com_numero(
                        info.options,
                        id,
                        v,
                    )))
                })
            });
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if crate::seam_formas::is_field(id) => {
            let v = host.store().number_value(id);
            let pedido = with_current(|info| {
                let o = crate::seam_formas::com_numero(info.options, id, v?)?;
                info.editing.then_some(PixelRequest::Options(Box::new(o)))
            });
            return empurra(host, pedido);
        }
        WidgetEvent::ValueChanged(id) if ids::XFORM_FIELDS.iter().any(|(_, n, _)| *n == id) => {
            let v = host.store().number_value(id);
            return empurra(host, crate::seam_selecao::campo_da_moldura(id, v));
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
                let n = v.round().clamp(1.0, MAX_COLORS) as u16; // CLAMP-OK: limites constantes e v finito (filtrado acima)
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

pub(crate) fn empurra(
    host: &mut dyn PanelHostInternal,
    pedido: Option<PixelRequest>,
) -> EventOutcome {
    match pedido {
        Some(r) => {
            host.bus_mut().push(EditorAction::Pixel(r));
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
