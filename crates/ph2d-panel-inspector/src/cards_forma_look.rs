//! **O cartão Look da forma de colisão** (GEF02 onda 8, dono 10/11: «a seção Look completa do sprite para o
//! collision shape»): a Opacity e o Pivot de nove pontos com os campos X e Y, como o Look da Sprite. O Self Tint e os
//! cantos não entram (a forma não tem textura nem filhos que herdem), e o Flip da Sprite é da imagem. Escreve o
//! `ShapeLook` pela física (`PhysicsFieldEdit::Look*`, `ph2d_shape_handles::look`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_editor_core::{PhysicsFieldEdit, ShapeLookFacts};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

use crate::cards_sprite_pivot::{NOMES_DO_PIVOT, grade};
use crate::ids;

/// O cartão.
pub(crate) const CARD_SHAPE_LOOK: NodeId = hash_node_id("cards.insp.shape_look");
/// A escala do chip da Opacity: a barra guarda `0..1`, o chip mostra `%`.
const POR_CENTO: f64 = 100.0; // LITERAL-PX-OK: opacity percent scale
/// O passo dos campos X e Y, um centímetro (o mesmo do Offset da física).
const PASSO_M: f64 = 0.01; // LITERAL-PX-OK: meters

thread_local! {
    /// As formas com os campos X e Y abertos à mão, com o pivô num dos nove pontos.
    static ABERTO: std::cell::RefCell<std::collections::BTreeSet<u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

fn aberto(bits: u64, f: &ShapeLookFacts) -> bool {
    f.ponto.is_none() || ABERTO.with(|a| a.borrow().contains(&bits))
}

/// O cartão Look, só quando a seleção é um objeto «Collision Shape».
pub(crate) fn cartao() -> Option<CardDef<'static>> {
    let info = crate::state::current_inspector_physics()?;
    let f = info.look;
    if !f.on {
        return None;
    }
    let bits = info.entity_bits;
    Some(CardDef {
        section: CARD_SHAPE_LOOK,
        title: tr("card.insp.look"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| corpo(c, y, bits, f))),
    })
}

fn corpo(c: &mut CardCtx<'_>, y: f32, bits: u64, f: ShapeLookFacts) -> f32 {
    let y = crate::cards_objeto::com_corpo(c, y, |t, _theme, store, x, w, y| {
        ph2d_editor_core::panel::cards::corpo::bar_field(
            t.scene,
            t.text,
            store,
            t.hit,
            ph2d_editor_core::widget::card_ink::card_body().expect("só num cartão"),
            x,
            w,
            y,
            tr("card.insp.opacity"),
            ids::INSP_SHAPE_OPACITY_CHIP,
            ids::INSP_SHAPE_OPACITY,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: Some(POR_CENTO),
            },
        )
    });
    let fim = c.row(
        &[Cell::new(Widget::Readout { text: "" })
            .label(tr("card.insp.pivot"))
            .key(&ids::INSP_SHAPE_PIVOT_XY)],
        y,
    );
    let campos = aberto(bits, &f);
    let fim = grade(
        c,
        fim - ph2d_card_stack::metrics::FIELD_H,
        &ids::INSP_SHAPE_PIVOT,
        ids::INSP_SHAPE_PIVOT_NOME,
        f.ponto,
        campos,
    );
    if !campos {
        return fim;
    }
    let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
    c.row(
        &[
            Cell::new(Widget::Field {
                id: ids::INSP_SHAPE_PIVOT_XY[0],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gx),
            })
            .label(tr("card.insp.shape_pivot_xy")),
            Cell::new(Widget::Field {
                id: ids::INSP_SHAPE_PIVOT_XY[1],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gy),
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
        store.set_number_range(ids::INSP_SHAPE_OPACITY_CHIP, 0.0, POR_CENTO, 1.0);
        for id in ids::INSP_SHAPE_PIVOT_XY {
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
    crate::cards_sprite::botao(
        store,
        ids::INSP_SHAPE_PIVOT_NOME,
        "card.insp.pivot_nome_tip",
    );
    for id in ids::INSP_SHAPE_PIVOT_XY {
        store.mark_card_field(
            id,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: PASSO_M,
                scrub: PASSO_M,
                default: Some(0.0),
            },
        );
    }
    let Some(info) = crate::state::current_inspector_physics().filter(|i| i.look.on) else {
        return;
    };
    let f = info.look;
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
    for (i, id) in ids::INSP_SHAPE_PIVOT_XY.into_iter().enumerate() {
        if livre(id) {
            crate::unidade_mundo::semeia(host, id, f64::from(f.metros[i]));
        }
    }
}

/// Os cliques e os números do Look da forma. `true` = consumido.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state::current_inspector_physics().filter(|i| i.look.on) else {
        return false;
    };
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_SHAPE_PIVOT_NOME => {
            ABERTO.with(|a| {
                let mut a = a.borrow_mut();
                if !a.remove(&info.entity_bits) {
                    a.insert(info.entity_bits);
                }
            });
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
        WidgetEvent::ValueChanged(id) if id == ids::INSP_SHAPE_OPACITY => {
            let Some((_, v)) = host.store().slider(ids::INSP_SHAPE_OPACITY) else {
                return true;
            };
            PhysicsFieldEdit::LookOpacity(v)
        }
        _ => return false,
    };
    host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
        entity_bits: info.entity_bits,
        edit,
    });
    true
}
