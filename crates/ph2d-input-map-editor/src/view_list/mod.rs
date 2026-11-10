//! **A · List** (plano `docs/InputMap/01_plano.md` §3, maquete A): o nome novo e o Add, a busca (por nome ou
//! por tecla), uma linha por acção com as teclas em chips de nome inteiro, a escuta no próprio lugar, os dois
//! números só nas linhas com manche ou gatilho, e o pé que explica.

mod footer;
mod input;
mod rows;
pub mod state;
mod text;

pub use footer::binding_label;
pub use input::tip_of;
pub(crate) use input::{drag, key, press, raw_key, release, wheel};

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding, Key};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;
use crate::ids;

/// As acções que a busca deixa ver, pela ordem do mapa.
#[must_use]
pub fn shown(doc: &InputMapDoc) -> Vec<ActionId> {
    let q = doc
        .store
        .text(ids::SEARCH)
        .unwrap_or("")
        .trim()
        .to_lowercase();
    let letter = (q.chars().count() == 1)
        .then(|| q.chars().next())
        .flatten()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| Key(u32::from(c.to_ascii_uppercase())));
    doc.snap
        .map
        .actions()
        .iter()
        .filter(|a| match doc.list.search_key {
            Some(k) => a.bindings.contains(&Binding::Key(k)),
            None => {
                q.is_empty()
                    || a.name.to_lowercase().contains(&q)
                    || letter.is_some_and(|k| a.bindings.contains(&Binding::Key(k)))
            }
        })
        .map(|a| a.id)
        .collect()
}

/// Pinta a vista dentro do cartão `body` (o cartão já está pintado).
pub fn paint(
    doc: &mut InputMapDoc,
    body: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let ink = CardInk::of_list(theme);
    let x = body.x + m::CARD_PAD_X;
    let w = (body.w - m::CARD_PAD_X * 2.0).max(0.0);
    let mut y = body.y + m::CARD_PAD_TOP;
    let (new_name, add) = (tr("input_map_list.new_name"), tr("input_map_list.add"));
    {
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, x, w);
        y = c.row(
            &[
                Cell::new(Widget::Text {
                    id: ids::NEW_NAME,
                    placeholder: new_name,
                }),
                Cell::new(Widget::Button {
                    id: ids::ADD,
                    text: add,
                    primary: false,
                }),
            ],
            y,
        );
    }
    if let Some(why) = doc.list.refused {
        let r = Rect::new(x, y + m::FIELD_ROW_GAP, w, m::FIELD_H);
        text::line(
            &mut doc.census,
            ts,
            scene,
            tr(why),
            r,
            m::VALUE_PX,
            ink.color(ColorToken::Warn),
        );
        y = r.y + r.h;
    }
    y += m::FIELD_ROW_GAP;
    y = paint_search(doc, Rect::new(x, y, w, m::FIELD_H), scene, ts, ink) + m::FIELD_ROW_GAP;
    let bottom = body.y + body.h - m::CARD_PAD_BOTTOM;
    rows::paint_rows(
        doc,
        Rect::new(x, y, w, (bottom - y).max(0.0)),
        scene,
        ts,
        ink,
    );
}

/// A busca: o campo de texto do cartão, ou — com um filtro por tecla — o chip da tecla com o ×.
fn paint_search(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let Some(k) = doc.list.search_key else {
        let ph = tr("input_map_list.search");
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, r.x, r.w);
        return c.full(
            None,
            Widget::Search {
                id: ids::SEARCH,
                placeholder: ph,
            },
            r.y,
        );
    };
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
    doc.hits.register(ids::SEARCH, r);
    let label = k.label();
    let chip_w = ts.prefix_width(&label, m::VALUE_PX) + m::SEG_INSET * 2.0;
    let chip = Rect::new(
        r.x + m::SEG_INSET,
        r.y + m::SEG_PAD * 2.0,
        chip_w,
        r.h - m::SEG_PAD * 4.0,
    );
    fill_rounded_rect(
        scene,
        chip,
        m::SEG_RADIUS,
        ink.color(ColorToken::CardFieldHover),
    );
    let inner = Rect::new(
        chip.x + m::SEG_INSET,
        chip.y,
        chip.w - m::SEG_INSET * 2.0,
        chip.h,
    );
    text::line(
        &mut doc.census,
        ts,
        scene,
        &label,
        inner,
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    let g = m::KEY_MARK_BOX;
    let x_box = Rect::new(r.x + r.w - g - m::SEG_INSET, r.y + (r.h - g) * 0.5, g, g);
    paint_icon(
        scene,
        IconId::Close,
        x_box,
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    doc.hits.register(ids::SEARCH_CLEAR, x_box);
    r.y + r.h
}

/// A moldura de um alvo focado ou à escuta (o anel do foco).
pub(crate) fn ring(scene: &mut VectorScene, r: Rect, ink: CardInk) {
    stroke_rounded_rect(
        scene,
        r,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        ink.color(ColorToken::FocusRing),
    );
}
