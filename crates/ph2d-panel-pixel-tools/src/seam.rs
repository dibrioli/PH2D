//! A costura do painel PIXEL — o registo dos widgets e o pedido que cada gesto faz.
//!
//! ⚠️ Os botões registam-se INCONDICIONALMENTE (quem decide se o clique é possível é a PINTURA:
//! sem hit-rect não há `Click`). As células da paleta registam-se por quadro, por quem as pinta.

use crate::ids;
use crate::state::with_current;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelOptions, PixelRequest};
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
    for id in segmentos {
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
}

/// O valor do campo `id` nas opções.
pub(crate) fn numero(o: &PixelOptions, id: NodeId) -> f64 {
    match id {
        ids::SIZE => f64::from(o.size),
        ids::ANGLE => f64::from(o.angle),
        ids::CORNER => f64::from(o.corner_radius),
        ids::TOLERANCE => f64::from(o.tolerance),
        ids::SPRAY_WIDTH => f64::from(o.spray_width),
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
    match id {
        ids::CONNECT_4 => o.eight = false,
        ids::CONNECT_8 => o.eight = true,
        ids::SAMPLE_LAYER => o.sample_visible = false,
        ids::SAMPLE_VISIBLE => o.sample_visible = true,
        _ => return None,
    }
    Some(o)
}

/// O registo POR QUADRO das células da paleta (os ids derivam do índice).
pub(crate) fn register_palette(store: &mut WidgetStore, n: usize) {
    for i in 0..n {
        store.register_if_absent(ids::palette_id(i), InteractiveState::Plain);
    }
}

/// O pedido de um clique, se o id é deste painel e há sprite em edição.
fn pedido(id: ph2d_a11y::NodeId) -> Option<PixelRequest> {
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
        info.palette.iter().enumerate().find_map(|(i, c)| {
            if ids::palette_id(i) == id {
                Some(PixelRequest::Fg(*c))
            } else if ids::palette_bg_id(i) == id {
                Some(PixelRequest::Bg(*c))
            } else {
                None
            }
        })
    })
}

pub(crate) fn apply_event(
    _st: &mut crate::PixelToolsPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let id = match ev {
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
        _ => return EventOutcome::Ignored,
    };
    if id == ids::PIXEL_CLOSE {
        host.set_panel_visible(crate::PixelToolsPanel::ID, false);
        return EventOutcome::Consumed;
    }
    empurra(host, pedido(id))
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
