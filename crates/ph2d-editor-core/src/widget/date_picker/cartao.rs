//! ⭐ **O [`super::DatePicker`] DENTRO de um cartão** (dono 03/11, «Dates fora do padrão»; `docs/CartoesUI/05`
//! §4): rótulo em cima, caixas `FIELD_H`, a data do tamanho que ela mede (nunca cortada), a hora com o
//! resto ou na fila de baixo, marcas `CHECK_BOX`, o chip do cartão, Save e Remove lado a lado, e a
//! superfície no tom aceso do cartão. Fora de um cartão nada disto corre: o Kanban fica como aprovado.

use super::paint::{DatePickerHit, DatePickerLayout, reminder_menu};
use super::{DateField, DatePicker, DateSlot};
use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_title, stroke_rounded_rect,
};
use crate::widget::calendar::{CalendarLayout, paint_calendar};
use crate::widget::card_ink::CardInk;
use crate::widget::checkbox::{card_check_rect, paint_card_check};
use crate::widget::{Button, ButtonKind, card_field, paint_button, paint_dropdown_chip};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

/// A largura que `texto` pinta numa caixa do cartão, com as folgas dela.
fn caixa_de(ts: &mut TextSystem, texto: &str) -> f32 {
    let w = ts.prefix_width_weighted(texto, m::VALUE_PX, FontWeight::MEDIUM);
    (w.ceil() + m::FIELD_PAD_R * 2.0).max(m::FIELD_VALUE_MIN_W)
}

/// A caixa que um botão pede para a palavra dele.
fn botao_de(ts: &mut TextSystem, b: &Button) -> f32 {
    let w = ts.prefix_width_weighted(&b.label, b.font_size(), FontWeight::MEDIUM);
    crate::paint::rect_for_label(w)
}

fn salvar(p: &DatePicker) -> Button {
    Button::new(NodeId(p.id.0.wrapping_add(4)), p.labels.save.clone()).kind(ButtonKind::Accent)
}

fn remover(p: &DatePicker) -> Button {
    Button::new(NodeId(p.id.0.wrapping_add(5)), p.labels.remove.clone())
}

impl DatePickerLayout {
    /// A disposição dentro de um cartão: mede os textos em `ts`. Mesma régua para pintar e clicar.
    #[must_use]
    pub fn of_card(p: &DatePicker, rect: Rect, ts: &mut TextSystem) -> Self {
        let g = m::FIELD_GAP;
        let x = rect.x + m::CARD_PAD_X;
        let w = (rect.w - m::CARD_PAD_X * 2.0).max(0.0);
        let mut y = rect.y + m::CARD_PAD_TOP;
        let head = m::HEAD_ICON_BOX;
        let close = Rect::new(x + w - head, y, head, head);
        let title = Rect::new(x, y, (w - head).max(0.0), head);
        y += head + m::CARD_HEAD_BOTTOM;
        let calendar = Rect::new(x, y, w, CalendarLayout::preferred_height());
        y += calendar.h;

        let none = |y: f32| Rect::new(x, y, 0.0, 0.0);
        let rotulo = |y: &mut f32, x: f32, w: f32| {
            *y += m::LABEL_TOP;
            let r = Rect::new(x, *y, w, m::LABEL_PX);
            *y += m::LABEL_PX + m::LABEL_BOTTOM;
            r
        };
        let marca = if p.due_only {
            0.0
        } else {
            m::CHECK_BOX + m::CHECK_GAP
        };
        let fx = x + marca;
        let resto = (w - marca).max(0.0);
        // ⭐ A data mede-se: as duas caixas com o maior dos dois textos, nunca menos.
        let data = caixa_de(ts, &p.start_text.text)
            .max(caixa_de(ts, &p.due_text.text))
            .min(resto);
        let hora = caixa_de(ts, &p.time_text.text);
        let lado_a_lado = data + g + hora <= resto;

        let (start_label, start_check, start_field) = if p.due_only {
            (none(y), none(y), none(y))
        } else {
            let label = rotulo(&mut y, x, w);
            let r = Rect::new(fx, y, data, m::FIELD_H);
            let c = card_check_rect(x, y, m::FIELD_H);
            y += m::FIELD_H;
            (label, c, r)
        };
        let mut topo_hora = y;
        let label_x = if lado_a_lado { fx + data + g } else { fx };
        let label_w = (x + w - label_x).max(0.0);
        let due_label = rotulo(&mut y, x, if lado_a_lado { label_x - x - g } else { w });
        let due_check = if p.due_only {
            none(y)
        } else {
            card_check_rect(x, y, m::FIELD_H)
        };
        let due_w = if lado_a_lado { data } else { resto };
        let due_field = Rect::new(fx, y, due_w, m::FIELD_H);
        y += m::FIELD_H;
        let time_label = if lado_a_lado {
            rotulo(&mut topo_hora, label_x, label_w)
        } else {
            let l = rotulo(&mut y, label_x, label_w);
            topo_hora = y;
            y += m::FIELD_H;
            l
        };
        let time_field = Rect::new(label_x, topo_hora, label_w, m::FIELD_H);

        let (reminder_label, reminder) = if p.due_only {
            (none(y), none(y))
        } else {
            let label = rotulo(&mut y, x, w);
            let r = Rect::new(x, y, w, m::FIELD_H);
            y += m::FIELD_H;
            (label, r)
        };
        y += m::CARD_GAP;
        // Save e Remove: lado a lado, como Save/Cancel de Actions; só a palavra que não cabe os separa.
        let meia = ((w - g) * 0.5).max(0.0);
        let (save, remove) = if botao_de(ts, &salvar(p)).max(botao_de(ts, &remover(p))) <= meia {
            (
                Rect::new(x, y, meia, m::FIELD_H),
                Rect::new(x + meia + g, y, meia, m::FIELD_H),
            )
        } else {
            (
                Rect::new(x, y, w, m::FIELD_H),
                Rect::new(x, y + m::FIELD_H + g, w, m::FIELD_H),
            )
        };
        Self {
            title,
            close,
            calendar,
            start_label,
            start_check,
            start_field,
            due_label,
            due_check,
            due_field,
            time_label,
            time_field,
            reminder_label,
            reminder,
            save,
            remove,
        }
    }

    /// A altura que o painel pede no cartão, na largura `w`.
    #[must_use]
    pub fn preferred_height_in_card(p: &DatePicker, w: f32, ts: &mut TextSystem) -> f32 {
        let l = Self::of_card(p, Rect::new(0.0, 0.0, w, 0.0), ts);
        l.remove.y + l.remove.h + m::CARD_PAD_BOTTOM
    }
}

/// Uma caixa de data ou de hora, a caixa do campo do cartão.
#[allow(clippy::too_many_arguments)]
fn campo(
    p: &DatePicker,
    f: DateField,
    on: bool,
    r: Rect,
    ink: CardInk,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let editing = p.editing == Some(f);
    let aceso = editing || p.hover == Some(DatePickerHit::Field(f));
    let fundo = if aceso {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(fundo));
    let azul = ink.color(ColorToken::KanbanBlueBold);
    if editing {
        // FRAME-RAW-OK: o anel de FOCO do campo de data em edição
        stroke_rounded_rect(scene, r, m::FIELD_RADIUS, m::FIELD_BORDER, azul);
    }
    let field = p.field(f);
    let tx = r.x + m::FIELD_PAD_R;
    let ty = r.y + (r.h - m::VALUE_PX) * 0.5;
    let tinta = ink.color(if on {
        ColorToken::KanbanCardFg
    } else {
        ColorToken::CardLabel
    });
    let area = (r.w - m::FIELD_PAD_R).max(0.0);
    paint_text(ts, scene, &field.text, tx, ty, m::VALUE_PX, area, tinta);
    if editing {
        let antes = field.text.get(..field.caret).unwrap_or_default();
        let cx = tx + ts.prefix_width(antes, m::VALUE_PX);
        let barra = Rect::new(cx, ty, StrokeToken::Default.px(), m::VALUE_PX);
        fill_rounded_rect(scene, barra, 0.0, azul);
    }
}

pub(super) fn paint(
    p: &DatePicker,
    rect: Rect,
    ink: CardInk,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let l = DatePickerLayout::of_card(p, rect, ts);
    fill_rounded_rect(
        scene,
        rect,
        m::CARD_RADIUS,
        ink.color(ColorToken::KanbanCardRaised),
    );
    let lb = &p.labels;
    paint_text_title(
        ts,
        scene,
        &lb.title,
        l.title.x,
        l.title.y + (l.title.h - m::CARD_TITLE_PX) * 0.5,
        m::CARD_TITLE_PX,
        l.title.w,
        ink.color(ColorToken::KanbanCardFg),
    );
    if p.hover == Some(DatePickerHit::Close) {
        let fundo = ink.color(ColorToken::CardFieldHover);
        fill_rounded_rect(scene, l.close, m::HEAD_ICON_RADIUS, fundo);
    }
    let gi = m::HEAD_ICON_GLYPH;
    paint_icon(
        scene,
        IconId::Close,
        Rect::new(
            l.close.x + (l.close.w - gi) * 0.5,
            l.close.y + (l.close.h - gi) * 0.5,
            gi,
            gi,
        ),
        ink.color(ColorToken::CardLabel),
        StrokeToken::Default.px(),
    );
    paint_calendar(&p.calendar, l.calendar, scene, ts, theme);

    let rotulo = |ts: &mut TextSystem, scene: &mut VectorScene, r: Rect, texto: &str| {
        card_field::paint_label(scene, ts, ink, r.x, r.y, r.w, texto);
    };
    let marca = |scene: &mut VectorScene, r: Rect, on: bool, slot: DateSlot| {
        let hover = p.hover == Some(DatePickerHit::Check(slot));
        paint_card_check(scene, ink, r, on, hover);
    };
    if !p.due_only {
        rotulo(ts, scene, l.start_label, &lb.start);
        marca(scene, l.start_check, p.start_on, DateSlot::Start);
        campo(
            p,
            DateField::Start,
            p.start_on,
            l.start_field,
            ink,
            scene,
            ts,
        );
        marca(scene, l.due_check, p.due_on, DateSlot::Due);
    }
    rotulo(ts, scene, l.due_label, &lb.due);
    campo(p, DateField::Due, p.due_on, l.due_field, ink, scene, ts);
    rotulo(ts, scene, l.time_label, &lb.time);
    campo(p, DateField::Time, p.due_on, l.time_field, ink, scene, ts);

    let menu = reminder_menu(Some(p));
    if !p.due_only {
        rotulo(ts, scene, l.reminder_label, &lb.reminder);
        paint_dropdown_chip(&menu, l.reminder, scene, ts, theme);
    }
    paint_button(&salvar(p), l.save, scene, ts, theme);
    paint_button(&remover(p), l.remove, scene, ts, theme);
    if menu.open {
        crate::widget::paint_dropdown_popover(&menu, l.reminder, scene, ts, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::card_ink::{CardInk, com};
    use crate::widget::showcase::card_back::demo_date_picker;

    fn medidas(w: f32) -> (DatePickerLayout, f32) {
        let p = demo_date_picker(NodeId(1));
        let mut ts = TextSystem::new();
        let l = com(CardInk::of_list(Theme::Dark), || {
            DatePickerLayout::of_card(&p, Rect::new(0.0, 0.0, w, 0.0), &mut ts)
        });
        (l, caixa_de(&mut ts, &p.due_text.text))
    }

    /// ⭐ A foto do dono (03/11): «10/20/2026» cortada. Na coluna de um cartão as duas caixas de data
    /// têm o que o texto mede; a hora fica com o resto. *Mutação: voltar à metade.*
    #[test]
    fn a_date_in_a_card_keeps_the_width_it_measures() {
        let (l, data) = medidas(ph2d_tokens::PANEL_MIN_W_PX);
        assert!(data > m::FIELD_PAD_R * 2.0, "o texto mediu-se: {data}");
        assert_eq!(
            l.due_field.w, data,
            "o que o texto mede, nem menos nem a metade"
        );
        assert_eq!(l.start_field.w, l.due_field.w);
        assert_eq!(l.due_field.y, l.time_field.y, "cabe na mesma fila");
        assert_eq!(l.time_label.x, l.time_field.x, "o rótulo por cima da caixa");
        let fim = l.calendar.x + l.calendar.w;
        assert!((l.time_field.x + l.time_field.w - fim).abs() < 0.5);
    }

    /// ⭐ Numa coluna que não dá para as duas, a hora desce: nada se corta nem transborda.
    /// *Mutação: tirar o ramo da fila de baixo.*
    #[test]
    fn a_column_too_narrow_for_both_puts_the_time_on_its_own_row() {
        let (l, data) = medidas(m::CHECK_BOX + m::CHECK_GAP + m::CARD_PAD_X * 2.0 + data_e_folga());
        assert!(l.due_field.w >= data, "{} < {data}", l.due_field.w);
        assert!(l.time_field.y >= l.due_field.y + l.due_field.h);
        assert!(l.reminder_label.y >= l.time_field.y + l.time_field.h);
        let fim = l.calendar.x + l.calendar.w;
        assert!(l.time_field.x + l.time_field.w <= fim + 0.5);
    }

    fn data_e_folga() -> f32 {
        medidas(ph2d_tokens::PANEL_MIN_W_PX).1 + m::FIELD_GAP
    }

    /// ⭐ As medidas são as do cartão, e Save e Remove ficam lado a lado.
    #[test]
    fn the_boxes_in_a_card_are_the_cards_own() {
        let (l, _) = medidas(ph2d_tokens::PANEL_MIN_W_PX);
        for r in [
            l.start_field,
            l.due_field,
            l.time_field,
            l.reminder,
            l.save,
            l.remove,
        ] {
            assert_eq!(r.h, m::FIELD_H);
        }
        for c in [l.start_check, l.due_check] {
            assert_eq!((c.w, c.h), (m::CHECK_BOX, m::CHECK_BOX));
        }
        assert_eq!(l.save.y, l.remove.y);
        assert_eq!(l.save.w, l.remove.w);
        assert_eq!(l.remove.x, l.save.x + l.save.w + m::FIELD_GAP);
    }

    /// O CONTROLO: fora de um cartão a régua do Kanban não mudou.
    #[test]
    fn outside_a_card_the_kanban_ruler_is_untouched() {
        let l = DatePickerLayout::of(Rect::new(0.0, 0.0, 300.0, 0.0)); // LITERAL-PX-OK: teste
        assert_eq!(l.due_field.w, l.time_field.w, "as duas metades do Kanban");
        assert!(l.remove.y > l.save.y, "um por baixo do outro");
        assert_eq!(l.time_label.w, 0.0);
    }
}
