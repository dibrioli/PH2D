//! ⭐ **A linha Key do cartão do Trigger** (onda 4 do F3, dono 09/11): a tecla da acção, escolhida
//! ali mesmo. O Input Map continua a ser a fonte — o botão só arma a escuta dele.
//!
//! | estado | o botão diz | o clique |
//! |---|---|---|
//! | campo Action vazio | `Set key`, desligado | nada |
//! | acção que o mapa não conhece | `Create and set key` | cria a acção e espera a tecla |
//! | acção sem tecla | `Set key` | espera a tecla |
//! | acção com tecla(s) | `Space` (as teclas) | espera a tecla nova, que TROCA as de teclado |
//! | à escuta | `Press a key… (Esc cancels)` | cancela |
//!
//! Tecla repetida noutra acção é aceite (dono 09/11) e a linha di-lo em cinzento.

use super::*;
use ph2d_editor_core::action_trigger_edits::{InspectorTriggerRow, NoMapa};
use ph2d_i18n::{tr, tr_with};

const ROW_H: f32 = ph2d_tokens::ROW_H_PX;

/// **Esta fileira está à espera da tecla?** A escuta guarda o id da acção; a fileira sabe o dela.
#[must_use]
pub(crate) fn a_escuta(store: &WidgetStore, row: &InspectorTriggerRow) -> bool {
    row.tecla.id.is_some() && store.input_map_listening() == row.tecla.id
}

/// O rótulo do botão, pela tabela do cabeçalho.
#[must_use]
pub fn rotulo_do_botao(store: &WidgetStore, row: &InspectorTriggerRow) -> String {
    if a_escuta(store, row) {
        tr("panel.inspector.trigger.press_a_key").to_owned()
    } else if row.action.trim().is_empty() {
        tr("panel.inspector.trigger.set_key").to_owned()
    } else if row.no_mapa == NoMapa::Desconhecida {
        tr("panel.inspector.trigger.create_and_set_key").to_owned()
    } else if row.tecla.teclas.is_empty() {
        tr("panel.inspector.trigger.set_key").to_owned()
    } else {
        row.tecla.teclas.join(", ")
    }
}

/// A linha Key. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn linha_da_tecla(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTriggerRow,
) -> f32 {
    let rotulo = tr("panel.inspector.trigger.key_label");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H,
        rotulo,
        seccao,
    );
    let id = crate::ids::INSP_TRIGGER_SET_KEY;
    let escuta = a_escuta(store, row);
    let mut botao = Button::new(id, rotulo_do_botao(store, row)).visual(store.button_visual(id));
    if row.action.trim().is_empty() {
        botao = botao.state(ButtonState::Disabled);
    } else {
        hit_index.register(id, linha.control);
    }
    if escuta {
        botao = botao.accent();
    }
    paint_button(&botao, linha.control, scene, text_system, theme);
    let mut cur_y = linha.fim(ph2d_tokens::row_pitch_px());
    if !escuta && let Some((tecla, outra)) = &row.tecla.tambem {
        let font = TypeToken::Sm.px();
        let msg = tr_with(
            "panel.inspector.trigger.key_also_used_by",
            &[("key", tecla), ("action", outra)],
        );
        paint_text(
            text_system,
            scene,
            &msg,
            x,
            cur_y,
            font,
            w,
            resolve(ColorToken::Text3, theme),
        );
        cur_y += font + ph2d_tokens::control_gap_px();
    }
    cur_y
}
