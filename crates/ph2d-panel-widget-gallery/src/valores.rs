//! Os controlos do vocabulário que AGEM sobre os valores (DIRETIVA_IMPLEMENTACAO §2: zero no-op
//! silencioso — a galeria é o modelo que todo painel copia): a amostra do Fill abre o seletor de
//! cor do app, o «−» tira o Fill e o «+» do cartão vazio devolve-o (à Figma), o Apply grava os
//! valores de PARTIDA e o Cancel e o ↺ voltam a eles.

use crate::ids as g;
use crate::state::{Choices, Values, WidgetGalleryState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_tokens::{ColorToken, Theme};

/// O retrato dos valores vivos: as escolhas e os números do store.
pub(crate) fn current(choices: Choices, store: &WidgetStore) -> Values {
    Values {
        choices,
        numbers: g::FIELDS.map(|f| store.number_value(f.0).unwrap_or(f.1)),
    }
}

/// A cor que a amostra mostra: a escolhida, ou o `Accent` da tinta `ink`.
pub(crate) fn fill_rgba(ch: Choices, ink: CardInk) -> [u8; 4] {
    ch.fill_rgba
        .unwrap_or_else(|| ink.color(ColorToken::Accent).to_rgba8().to_u8_array())
}

/// ⭐ Os dois regimes da amostra (a lei das amostras do Inspector): com o seletor apontado para
/// ELA, a cor que ele escreve no `widget_color` é a do Fill; senão, nada. Corre por quadro, antes
/// de pintar.
pub(crate) fn sync_picked(state: &mut WidgetGalleryState, store: &WidgetStore, theme: Theme) {
    if store.picker_target() != Some(g::FILL_SWATCH) {
        return;
    }
    if let Some(c) = store.widget_color(g::FILL_SWATCH)
        && c != fill_rgba(state.choices, CardInk::of_list(theme))
    {
        state.choices.fill_rgba = Some(c);
    }
}

/// Trata o clique `id` se é um destes controlos; devolve se tratou.
pub(crate) fn on_click(
    state: &mut WidgetGalleryState,
    host: &mut dyn PanelHostInternal,
    id: NodeId,
) -> bool {
    if id == g::FILL_SWATCH {
        toggle_picker(state.choices, host);
    } else if id == g::FILL_REMOVE {
        state.choices.fill = false;
        close_picker(host.store_mut());
    } else if id == g::FILL_ADD {
        state.choices.fill = true;
    } else if id == g::PRIMARY {
        state.applied = current(state.choices, host.store());
    } else if id == g::SECONDARY || id == g::RESET {
        let store = host.store_mut();
        close_picker(store);
        state.choices = state.applied.choices;
        for ((field, ..), n) in g::FIELDS.iter().zip(state.applied.numbers) {
            store.set_number_value(*field, n);
        }
    } else {
        return false;
    }
    true
}

/// Abre o seletor de cor do app apontado para a amostra, semeado com a cor que ela mostra — ou
/// fecha-o, se já era dela (o padrão das amostras do Painter).
fn toggle_picker(ch: Choices, host: &mut dyn PanelHostInternal) {
    if host.store().picker_target() == Some(g::FILL_SWATCH) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let [r, gr, b, a] = fill_rgba(ch, CardInk::of_list(host.theme()));
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(r, gr, b, a),
    );
    store.set_widget_color(g::FILL_SWATCH, [r, gr, b, a]);
    store.set_picker_target(Some(g::FILL_SWATCH));
}

/// O seletor aberto para uma amostra que some (ou cuja cor volta atrás) fecha-se: senão o quadro
/// seguinte reescreveria a cor escolhida por cima da reposta.
fn close_picker(store: &mut WidgetStore) {
    if store.picker_target() == Some(g::FILL_SWATCH) {
        store.set_picker_target(None);
    }
}
