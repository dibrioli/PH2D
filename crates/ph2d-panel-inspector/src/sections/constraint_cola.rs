//! **As colas no cartão CONSTRAINTS** (`docs/BlenderConstraints/05_W5_diferenciais.md` §2.5): uma linha por
//! cola que toca o objecto — o tipo e o OUTRO objecto, a frase do estado do grupo e «Unglue» — e, com a cola
//! armada, a frase que diz o que o próximo clique no palco faz. Irmão do `constraint.rs` (tecto de LOC).

use super::*;
use ph2d_editor_core::constraint_edits::{InspectorColaRow, InspectorConstraintsInfo};
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use ph2d_i18n::{tr, tr_with};

/// Uma cola, na posição `k`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    id: NodeId,
    cola: &InspectorColaRow,
) -> f32 {
    let font = TypeToken::Md.px();
    let chave = if cola.pino {
        "panel.inspector.constraint.glue.pin_with"
    } else {
        "panel.inspector.constraint.glue.rigid_with"
    };
    paint_text(
        text_system,
        scene,
        &tr_with(chave, &[("name", &cola.parceiro)]),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text1, theme),
    );
    let mut cur_y = y + ph2d_tokens::row_pitch_px();
    if let Some(frase) = cola.estado.frase() {
        let cor = if cola.estado.erro() {
            ColorToken::Warn
        } else {
            ColorToken::Text3
        };
        cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, tr(frase), cor);
    }
    let rotulo = tr("panel.inspector.constraint.glue.unglue");
    let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo);
    hit_index.register(id, caixa);
    paint_button(
        &Button::new(id, rotulo)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id)),
        caixa,
        scene,
        text_system,
        theme,
    );
    abaixo_do_botao(caixa) + ph2d_tokens::control_gap_px()
}

/// As colas do objecto e a frase da cola armada. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn colas(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    i: &InspectorConstraintsInfo,
) -> f32 {
    let mut cur_y = y;
    // `zip` com a tabela de ids: o excedente não se pinta (nem fica um «Unglue» mudo).
    for (cola, &id) in i.colas.iter().zip(ids::INSP_COLA_DESCOLAR.iter()) {
        cur_y = linha(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            (x, w, cur_y),
            id,
            cola,
        );
    }
    if let Some(pino) = i.cola_armada {
        let frase = if pino {
            "panel.inspector.constraint.glue.pick_pin"
        } else {
            "panel.inspector.constraint.glue.pick_rigid"
        };
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(frase),
            ColorToken::Text2,
        );
    }
    cur_y
}
