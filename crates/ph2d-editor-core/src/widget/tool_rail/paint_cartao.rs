//! A fila de chips em cartões (`docs/CartoesUI/15_CHROME_CENSO.md`, M5 e M6): chips planos, campos do
//! cartão, legenda em caixa normal e a marca de animação na linha da legenda. A geometria e os alvos são
//! os de sempre ([`super::entry_rects`]); `PH2D_CARDS=0` não passa por aqui.

use super::caption::{paint_field_caption, paint_sub_label_card};
use super::paint::chip_with_key_mark;
use super::*;
use crate::widget::cromo_barra::Barra;
use ph2d_card_stack::metrics as m;

fn aceso(state: ButtonState) -> bool {
    matches!(
        state,
        ButtonState::Hovered | ButtonState::Focused | ButtonState::Pressed
    )
}

/// O chip de opção cuja face termina no «▾» do produtor: o texto e a seta do cartão, no lugar do glifo.
fn face_de_opcao(
    barra: Barra,
    face: &str,
    chip: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    let font = super::compound_face_font_px();
    let clip = rect_to_vello(chip);
    scene.push_clip(&clip);
    if let Some(nome) = face.strip_suffix('▾') {
        let seta = m::CARD_CHEVRON;
        let texto = Rect::new(chip.x, chip.y, chip.w - seta, chip.h);
        // O orçamento é a caixa toda: a seta ocupa o lugar do glifo que a fila já mediu (`size_row_pulldowns`).
        let orcamento = texto.w;
        crate::paint::paint_text_centered_com_orcamento(
            text_system,
            scene,
            nome.trim_end(),
            texto,
            orcamento,
            font,
            barra.texto(),
        );
        let glifo = Rect::new(
            chip.x + chip.w - seta - m::SEG_PAD,
            chip.y + (chip.h - seta) * 0.5,
            seta,
            seta,
        );
        let traco = StrokeToken::Default.px();
        paint_icon(
            scene,
            crate::icons::IconId::ChevronDown,
            glifo,
            barra.rotulo(),
            traco,
        );
    } else {
        paint_text_centered(text_system, scene, face, chip, font, barra.texto());
    }
    scene.pop_layer();
}

/// Pinta a fila. Os alvos regista-os quem chama, como no caminho de antes.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_em_cartoes(
    barra: Barra,
    rail: &ToolRail,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    store: &WidgetStore,
    axis: RailAxis,
) {
    let slots = super::entry_rects(rail, rect, store.rail_button_size(), axis);
    for (slot, entry) in slots.iter().zip(rail.entries.iter()) {
        let r = slot.rect;
        match entry {
            ToolRailEntry::Icon {
                id,
                icon,
                active,
                sub,
                ..
            } => {
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let on = *active || state == ButtonState::Pressed;
                let fg = barra.chip(scene, r, on, aceso(state));
                paint_icon(scene, *icon, r, fg, StrokeToken::Default.px());
                paint_sub_label_card(barra, text_system, scene, sub, rect.x, r, axis);
            }
            ToolRailEntry::Glyph {
                id,
                path,
                active,
                sub,
                ..
            } => {
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let on = *active || state == ButtonState::Pressed;
                let _ = barra.chip(scene, r, on, aceso(state));
                crate::widget::paint_icon_button(
                    r,
                    crate::widget::IconGlyph::Path(path),
                    crate::widget::IconButtonStyle::Plain,
                    (state, crate::motion::SETTLED),
                    scene,
                    theme,
                );
                paint_sub_label_card(barra, text_system, scene, sub, rect.x, r, axis);
            }
            ToolRailEntry::Compound {
                id,
                face,
                sub,
                active,
                ..
            } => {
                let chip = chip_with_key_mark(scene, theme, *id, r);
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                let on = *active || state == ButtonState::Pressed;
                barra.campo(scene, chip, on, aceso(state));
                face_de_opcao(barra, face, chip, scene, text_system);
                paint_field_caption(barra, text_system, scene, sub, *id, r, axis, rect.x);
            }
            ToolRailEntry::Swatch {
                id,
                color,
                active,
                sub,
                mixed,
                ..
            } => {
                let state = store.button_state(*id).unwrap_or(ButtonState::Normal);
                if *mixed {
                    crate::widget::paint_mixed_swatch(r, scene, theme);
                } else {
                    let [cr, cg, cb, ca] = *color;
                    let fill = ph2d_vector::Color::from_rgba8(cr, cg, cb, ca); // LITERAL-COLOR-OK: user paint colour
                    fill_rounded_rect(scene, r, m::FIELD_RADIUS, fill);
                }
                let on = *active || state == ButtonState::Pressed;
                if on || aceso(state) {
                    crate::paint::stroke_frame(
                        scene,
                        r,
                        m::FIELD_RADIUS,
                        theme,
                        ph2d_tokens::visuals::Feel::Focused,
                        StrokeToken::Default.px(),
                        barra.acento(),
                    );
                }
                paint_sub_label_card(barra, text_system, scene, sub, rect.x, r, axis);
            }
            ToolRailEntry::Number {
                id, sub, display, ..
            } => {
                let (state, buffer, caret, anchor) = store.number_chip_view(*id);
                let chip = chip_with_key_mark(scene, theme, *id, r);
                let focado = state == crate::widget::TextInputState::Focused;
                let sob = store.hot_id() == Some(*id);
                barra.campo(scene, chip, false, sob);
                if focado {
                    crate::paint::stroke_frame(
                        scene,
                        chip,
                        m::FIELD_RADIUS,
                        theme,
                        ph2d_tokens::visuals::Feel::Focused,
                        StrokeToken::Default.px(),
                        barra.acento(),
                    );
                }
                let _veste = crate::widget::card_ink::publica(barra.tinta());
                crate::widget::slider_with_chip::paint_number_chip_do_cartao(
                    chip,
                    state,
                    store.number_value(*id).unwrap_or(0.0),
                    display.as_deref(),
                    buffer,
                    store.number_input_mixed(*id),
                    caret,
                    anchor,
                    scene,
                    text_system,
                    theme,
                    sob || focado,
                );
                paint_field_caption(barra, text_system, scene, sub, *id, r, axis, rect.x);
            }
            ToolRailEntry::Text { text, .. } => {
                let font = super::compound_face_font_px();
                paint_text_centered(text_system, scene, text, r, font, barra.texto());
            }
            ToolRailEntry::Divider => {
                scene.fill_rect(rect_to_vello(r), barra.divisoria());
            }
        }
    }
}
