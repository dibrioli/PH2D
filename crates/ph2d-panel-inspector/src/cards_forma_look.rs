//! **O cartão Look da forma de colisão** (GEF02 onda 8, dono 10/11: «a seção Look completa do sprite para o
//! collision shape»): a Color, a Opacity, o traço do contorno (largura e Dashed) e o Pivot de nove pontos com os
//! campos X e Y, como o Look da Sprite. O Self Tint e os
//! cantos não entram (a forma não tem textura nem filhos que herdem), e o Flip da Sprite é da imagem. Escreve o
//! `ShapeLook` pela física (`PhysicsFieldEdit::Look*`, `ph2d_shape_handles::look`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::{
    CheckboxState, CheckboxValue, SliderOrientation, SliderState, TextInputState,
};
use ph2d_editor_core::{PhysicsFieldEdit, ShapeLookFacts};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

use crate::cards_sprite_pivot::{NOMES_DO_PIVOT, grade_de_lado};
use crate::ids;

/// O cartão.
pub(crate) const CARD_SHAPE_LOOK: NodeId = hash_node_id("cards.insp.shape_look");
/// A escala do chip da Opacity: a barra guarda `0..1`, o chip mostra `%`.
const POR_CENTO: f64 = 100.0; // LITERAL-PX-OK: opacity percent scale
/// O passo dos campos X e Y, um centímetro (o mesmo do Offset da física).
const PASSO_M: f64 = 0.01; // LITERAL-PX-OK: meters
/// O passo dos campos do traço, meio píxel de tela (o piso `TRACO_MINIMO`).
const PASSO_PX: f64 = 0.5; // LITERAL-PX-OK: screen px of the outline stroke
/// Os campos do traço (px de tela) e a dica de cada um.
const CAMPOS_DO_TRACO: [(NodeId, &str); 3] = [
    (ids::INSP_SHAPE_STROKE_W, "card.insp.shape_stroke_tip"),
    (ids::INSP_SHAPE_DASH_GAP[0], "card.insp.shape_dash_tip"),
    (ids::INSP_SHAPE_DASH_GAP[1], "card.insp.shape_gap_tip"),
];

/// O cartão Look, só quando a seleção é um objeto «Collision Shape».
pub(crate) fn cartao() -> Option<CardDef<'static>> {
    let info = crate::state::current_inspector_physics()?;
    let f = info.look;
    if !f.on {
        return None;
    }
    Some(CardDef {
        section: CARD_SHAPE_LOOK,
        title: tr("card.insp.look"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| corpo(c, y, f))),
    })
}

fn corpo(c: &mut CardCtx<'_>, y: f32, f: ShapeLookFacts) -> f32 {
    let y = crate::cards_objeto::com_corpo(c, y, |t, theme, store, x, w, y| {
        crate::sections::look::paint_cor_da_forma(t.scene, t.text, theme, t.hit, store, x, w, y)
    });
    let y = traco(c, y, f);
    // O Pivot (dono 10/11): a grade à esquerda, da altura dos campos X e Y juntos, e o «Pivot from centre» sempre à
    // vista à direita, X por cima de Y.
    use ph2d_card_stack::metrics::{FIELD_H as m, FIELD_ROW_GAP};
    let lado = 2.0 * m + FIELD_ROW_GAP;
    let (x0, w0) = (c.x, c.w);
    c.w = lado;
    // O rótulo como o da Sprite: uma linha com «Pivot», e a grade pinta-se por cima da caixa dela.
    let fim_rotulo = c.row(
        &[Cell::new(Widget::Readout { text: "" }).label(tr("card.insp.pivot"))],
        y,
    );
    let pontos = &ids::INSP_SHAPE_PIVOT;
    let fim_grade = grade_de_lado(c, fim_rotulo - m, lado, pontos, None, f.ponto);
    let folga = ph2d_tokens::Spacing::Md.px();
    (c.x, c.w) = (x0 + lado + folga, w0 - lado - folga);
    let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
    let fim_x = c.row(
        &[Cell::new(Widget::Field {
            id: ids::INSP_SHAPE_PIVOT_XY[0],
            glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gx),
        })
        .label(tr("card.insp.shape_pivot_xy"))
        .key(&ids::INSP_SHAPE_PIVOT_XY)],
        y,
    );
    let fim_y = c.row(
        &[Cell::new(Widget::Field {
            id: ids::INSP_SHAPE_PIVOT_XY[1],
            glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gy),
        })],
        fim_x,
    );
    (c.x, c.w) = (x0, w0);
    fim_grade.max(fim_y)
}

/// ⭐ O traço do contorno (dono 10/11): a largura com a caixa «Dashed» ao lado; tracejado, o traço e o vão por baixo.
fn traco(c: &mut CardCtx<'_>, y: f32, f: ShapeLookFacts) -> f32 {
    let value = if f.tracejado {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let fim = c.row(
        &[
            Cell::new(Widget::Field {
                id: ids::INSP_SHAPE_STROKE_W,
                glyph: Glyph::Text(tr("card.vector.glyph.w")),
            })
            .label(tr("card.insp.shape_stroke"))
            .key(&[ids::INSP_SHAPE_STROKE_W]),
            Cell::new(Widget::Check {
                id: ids::INSP_SHAPE_DASHED,
                label: tr("card.insp.shape_dashed").into(),
                value,
                off: false,
            }),
        ],
        y,
    );
    if !f.tracejado {
        return fim;
    }
    let [d, g] = ids::INSP_SHAPE_DASH_GAP;
    c.row(
        &[
            Cell::new(Widget::Field {
                id: d,
                glyph: Glyph::Text(tr("card.vector.glyph.dash")),
            })
            .label(tr("card.insp.shape_dash_gap"))
            .key(&ids::INSP_SHAPE_DASH_GAP),
            Cell::new(Widget::Field {
                id: g,
                glyph: Glyph::Text(tr("card.vector.glyph.gap")),
            }),
        ],
        fim,
    )
}

/// Uma vez por quadro: os widgets, as dicas e os números vivos (o pivô também anda pelo canvas e pela curva).
pub(crate) fn prepara(host: &mut dyn PanelHostInternal) {
    let store = host.store_mut();
    if store.get(ids::INSP_SHAPE_OPACITY).is_none() {
        store.register(
            ids::INSP_SHAPE_OPACITY,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 1.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            ids::INSP_SHAPE_OPACITY_CHIP,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: POR_CENTO,
                buffer: format_number(POR_CENTO),
                caret: 0,
                last_committed: POR_CENTO,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped_integer(
            ids::INSP_SHAPE_OPACITY,
            ids::INSP_SHAPE_OPACITY_CHIP,
            POR_CENTO as f32,
            0.0,
        );
        let minima = f64::from(ShapeLookFacts::OPACIDADE_MINIMA) * POR_CENTO;
        store.set_number_range(ids::INSP_SHAPE_OPACITY_CHIP, minima, POR_CENTO, 1.0);
        // `Plain`: focável, o Down arma e o Up dá `Click` (a cor vive em `widget_colors`).
        store.register(ids::INSP_SHAPE_TINT_SWATCH, InteractiveState::Plain);
        store.register(
            ids::INSP_SHAPE_DASHED,
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Unchecked,
            },
        );
        store.set_tooltip(ids::INSP_SHAPE_DASHED, tr("card.insp.shape_dashed_tip"));
        for (id, dica) in CAMPOS_DO_TRACO {
            store.set_tooltip(id, tr(dica));
        }
        for id in ids::INSP_SHAPE_PIVOT_XY
            .into_iter()
            .chain(CAMPOS_DO_TRACO.map(|(id, _)| id))
        {
            store.register(
                id,
                InteractiveState::NumberInput {
                    state: TextInputState::Normal,
                    value: 0.0,
                    buffer: format_number(0.0),
                    caret: 0,
                    last_committed: 0.0,
                    selection_anchor: None,
                },
            );
        }
    }
    for (id, nome) in ids::INSP_SHAPE_PIVOT.into_iter().zip(NOMES_DO_PIVOT) {
        crate::cards_sprite::botao(store, id, nome);
    }
    // O X/Y do pivô é número de mundo: o passo passa pela porta px/m (e o censo vê o campo pintado).
    let (_, passo) = crate::unidade_mundo::no_campo(
        &ids::INSP_SHAPE_PIVOT_XY,
        Some(ph2d_editor_core::widget::Unit::Meters),
        PASSO_M,
    );
    for id in ids::INSP_SHAPE_PIVOT_XY {
        store.mark_card_field(
            id,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: passo,
                scrub: passo,
                default: Some(0.0),
            },
        );
    }
    for (id, _) in CAMPOS_DO_TRACO {
        store.mark_card_field(
            id,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: PASSO_PX,
                scrub: PASSO_PX,
                default: None,
            },
        );
    }
    let Some(info) = crate::state::current_inspector_physics().filter(|i| i.look.on) else {
        return;
    };
    let f = info.look;
    if let Some(InteractiveState::Checkbox { value, .. }) = store.get_mut(ids::INSP_SHAPE_DASHED) {
        *value = if f.tracejado {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
    }
    let (foco, mao) = (store.focus_id(), store.number_input_drag().map(|d| d.id));
    let arrasta_barra = matches!(
        store.slider(ids::INSP_SHAPE_OPACITY),
        Some((SliderState::Dragging, _))
    );
    let livre = |id: NodeId| foco != Some(id) && mao != Some(id);
    if !arrasta_barra && livre(ids::INSP_SHAPE_OPACITY_CHIP) {
        if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(ids::INSP_SHAPE_OPACITY)
        {
            *value = f.opacity;
        }
        store.set_number_value(
            ids::INSP_SHAPE_OPACITY_CHIP,
            (f64::from(f.opacity) * POR_CENTO).round(),
        );
    }
    let tracos = [f.largura, f.traco[0], f.traco[1]];
    for ((id, _), v) in CAMPOS_DO_TRACO.into_iter().zip(tracos) {
        if livre(id) {
            store.set_number_value(id, f64::from(v));
        }
    }
    for (i, id) in ids::INSP_SHAPE_PIVOT_XY.into_iter().enumerate() {
        if livre(id) {
            crate::unidade_mundo::semeia(host, id, f64::from(f.metros[i]));
        }
    }
    // A Color: com o seletor nela, a cor escolhida vira edição; senão a amostra espelha a gravada.
    // Sem cor própria a amostra mostra a da camada, a que o canvas pinta (a do tema).
    let tema = host.theme();
    let gravada = if f.cor_propria {
        crate::state_tint::tint_f32_to_u8(f.tint)
    } else {
        let c = ph2d_editor_core::paint::resolve(
            ph2d_tokens::ColorToken::layer(usize::from(f.camada)),
            tema,
        );
        crate::state_tint::tint_f32_to_u8(c.components)
    };
    let do_seletor = host.store().picker_target() == Some(ids::INSP_SHAPE_TINT_SWATCH)
        && host
            .store()
            .picker_subject()
            .is_none_or(|p| p == info.entity_bits);
    match host.store().widget_color(ids::INSP_SHAPE_TINT_SWATCH) {
        Some(escolhida) if do_seletor && escolhida != gravada => {
            host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
                entity_bits: info.entity_bits,
                edit: PhysicsFieldEdit::LookTint(crate::state_tint::tint_u8_to_f32(escolhida)),
            });
        }
        _ if !do_seletor => host
            .store_mut()
            .set_widget_color(ids::INSP_SHAPE_TINT_SWATCH, gravada),
        _ => {}
    }
}

/// Os cliques e os números do Look da forma. `true` = consumido.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state::current_inspector_physics().filter(|i| i.look.on) else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_SHAPE_TINT_SWATCH => {
            // A semente é a cor que a amostra já mostra (a própria, ou a da camada): `prepara` espelha-a.
            host.store_mut().open_picker_on(id);
            return true;
        }
        WidgetEvent::Click(id) => {
            let Some(i) = ids::INSP_SHAPE_PIVOT.iter().position(|o| *o == id) else {
                return false;
            };
            PhysicsFieldEdit::LookPivotPoint([(i % 3) as u8, (i / 3) as u8])
        }
        WidgetEvent::ValueChanged(id) if ids::INSP_SHAPE_PIVOT_XY.contains(&id) => {
            let Some(v) = crate::unidade_mundo::le(host, id) else {
                return true;
            };
            if id == ids::INSP_SHAPE_PIVOT_XY[0] {
                PhysicsFieldEdit::LookPivotX(v as f32)
            } else {
                PhysicsFieldEdit::LookPivotY(v as f32)
            }
        }
        WidgetEvent::Toggled(id) if id == ids::INSP_SHAPE_DASHED => {
            let f = info.look;
            PhysicsFieldEdit::LookStroke {
                width: f.largura,
                dashed: !f.tracejado,
                dash: f.traco,
            }
        }
        WidgetEvent::ValueChanged(id) if CAMPOS_DO_TRACO.iter().any(|(c, _)| *c == id) => {
            let Some(v) = host.store().number_value(id) else {
                return true;
            };
            let v = (v as f32).max(ShapeLookFacts::TRACO_MINIMO);
            let f = info.look;
            let (mut width, mut dash) = (f.largura, f.traco);
            match id {
                ids::INSP_SHAPE_STROKE_W => width = v,
                _ if id == ids::INSP_SHAPE_DASH_GAP[0] => dash[0] = v,
                _ => dash[1] = v,
            }
            PhysicsFieldEdit::LookStroke {
                width,
                dashed: f.tracejado,
                dash,
            }
        }
        WidgetEvent::ValueChanged(id) if id == ids::INSP_SHAPE_OPACITY => {
            let Some((_, v)) = host.store().slider(ids::INSP_SHAPE_OPACITY) else {
                return true;
            };
            PhysicsFieldEdit::LookOpacity(v.max(ShapeLookFacts::OPACIDADE_MINIMA))
        }
        _ => return false,
    };
    host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
        entity_bits: info.entity_bits,
        edit,
    });
    true
}
