//! As duas linhas de um item do cartão CONSTRAINTS que dependem do modo: o alvo com «Pick» e a força.
//! Irmão do `constraint.rs` (tecto de LOC).

use super::*;
use ph2d_editor_core::constraint_edits::InspectorConstraintRow;
use ph2d_editor_core::property_row::{Seccao, abaixo_do_botao, caixa_do_botao, paint_label_row};
use ph2d_editor_core::widget::SliderOrientation;
use ph2d_i18n::{TextKey, tr};

/// A linha do ALVO (ou do objecto do peso): o nome (ou «None») e, por baixo, «Pick» — premido
/// enquanto escolhe.
#[allow(clippy::too_many_arguments)]
pub(super) fn alvo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    (rotulo, nome, nada, escolhendo): (&str, Option<&str>, &str, bool),
    id: NodeId,
    seccao: Seccao,
) -> f32 {
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr(rotulo),
        seccao,
    );
    let font = TypeToken::Sm.px();
    let (nome, cor) = match nome {
        Some(n) => (n, ColorToken::Text1),
        None => (tr(nada), ColorToken::Text3),
    };
    let rotulo_pick = tr(if escolhendo {
        "panel.inspector.constraint.picking"
    } else {
        "panel.inspector.constraint.pick"
    });
    // Em cartão, o nome numa caixa e o verbo ao lado, na mesma linha (lei da compactação).
    if let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() {
        use ph2d_card_stack::metrics as m;
        let c = linha.control;
        let pick_w = text_system
            .prefix_width_weighted(
                rotulo_pick,
                ph2d_editor_core::widget::Button::label_font_px(),
                ph2d_text::FontWeight::MEDIUM,
            )
            .ceil()
            + 2.0 * m::BUTTON_TEXT_PAD;
        let caixa = Rect::new(c.x, c.y, (c.w - pick_w - m::FIELD_GAP).max(0.0), c.h);
        let pick = Rect::new(caixa.x + caixa.w + m::FIELD_GAP, c.y, pick_w, c.h);
        ph2d_editor_core::paint::fill_rounded_rect(
            scene,
            caixa,
            m::FIELD_RADIUS,
            ink.color(ColorToken::CardField),
        );
        ph2d_editor_core::paint::paint_text_elided(
            text_system,
            scene,
            nome,
            caixa.x + m::BUTTON_TEXT_PAD,
            caixa.y + ph2d_editor_core::paint::folga_do_texto(caixa.h, font),
            font,
            (caixa.w - 2.0 * m::BUTTON_TEXT_PAD).max(0.0),
            ink.color(if cor == ColorToken::Text1 {
                ColorToken::KanbanCardFg
            } else {
                ColorToken::CardLabel
            }),
        );
        hit_index.register(id, pick);
        let botao = Button::new(id, rotulo_pick).kind(ButtonKind::Default);
        let botao = if escolhendo {
            botao.state(ButtonState::Pressed)
        } else {
            botao.visual(store.button_visual(id))
        };
        paint_button(&botao, pick, scene, text_system, theme);
        return linha.fim(ph2d_tokens::row_pitch_px());
    }
    paint_text(
        text_system,
        scene,
        nome,
        linha.control.x,
        linha.control.y + (linha.control.h - font) * 0.5,
        font,
        linha.control.w,
        resolve(cor, theme),
    );
    let cur_y = linha.fim(ph2d_tokens::row_pitch_px());
    let caixa = caixa_do_botao(text_system, x, w, cur_y, rotulo_pick);
    hit_index.register(id, caixa);
    let botao = Button::new(id, rotulo_pick).kind(ButtonKind::Default);
    let botao = if escolhendo {
        botao.state(ButtonState::Pressed)
    } else {
        botao.visual(store.button_visual(id))
    };
    paint_button(&botao, caixa, scene, text_system, theme);
    abaixo_do_botao(caixa)
}

/// A barra da FORÇA (`0..1`) com o losango do Auto Key na coluna do decorator.
#[allow(clippy::too_many_arguments)]
pub(super) fn forca(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    (x, w, y): (f32, f32, f32),
    row: &InspectorConstraintRow,
    k: usize,
    seccao: Seccao,
) -> f32 {
    let id = ids::INSP_CONSTRAINT_FORCA[k];
    let chave = TextKey::new("panel.inspector.constraint.influence");
    // Em cartão é o campo de 0 a 100 %, como a Opacity: a barra não se pinta e continua a avisar.
    if ph2d_editor_core::widget::card_ink::card_body().is_some() {
        return ph2d_editor_core::property_row::paint_fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            y,
            chave,
            &[ids::INSP_CONSTRAINT_FORCA_CHIP[k]],
            1.0,
            Some(ph2d_editor_core::widget::Unit::Percent),
            seccao,
        );
    }
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        chave.tr(),
        seccao,
    );
    let visual = store.slider_visual(id);
    let valor = match store.slider(id) {
        Some((SliderState::Dragging, v)) => v,
        _ => row.forca,
    };
    hit_index.register(id, linha.control);
    ph2d_editor_core::widget::paint_slider_track(
        linha.control,
        valor,
        SliderOrientation::Horizontal,
        visual,
        scene,
        theme,
    );
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        &ph2d_editor_core::widget::zero_sem_sinal(&format!("{valor:.2}")),
        linha.control.x + Spacing::Sm.px(),
        linha.control.y + (linha.control.h - font) * 0.5,
        font,
        linha.control.w,
        resolve(ColorToken::Text1, theme),
    );
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        linha.dot,
        &[ids::INSP_CONSTRAINT_FORCA[k]],
        chave.into(),
    ) {
        hit_index.register(alvo, linha.dot);
    }
    linha.fim(ph2d_tokens::row_pitch_px())
}
