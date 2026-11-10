//! **A · List** (plano `docs/InputMap/01_plano.md` §3, maquete A): o nome novo e o Add, a busca (por nome ou
//! por tecla), uma linha por acção com as teclas em chips de nome inteiro, a escuta no próprio lugar, os dois
//! números só nas linhas com manche ou gatilho, e o pé que explica.

mod foot;
pub(crate) mod footer;
mod input;
mod looks;
mod rows;
pub mod state;
pub(crate) mod text;

pub use footer::binding_label;
pub use input::tip_of;
pub(crate) use input::{
    act, commit_typing, drag, drag_text, drag_thumb, is_typing, key, open_number, press,
    press_text, raw_key, release, wheel,
};

/// Antes de o cartão pintar um campo de texto: a fase do piscar do caret (só o focado pisca e pede quadro).
fn blink(doc: &mut InputMapDoc, id: ph2d_a11y::NodeId) {
    let on = doc.store.focus_id() != Some(id) || doc.caret_on(id);
    doc.store.set_caret_visible(id, on);
}

/// Depois de o cartão pintar o campo `id`: onde cai cada fronteira de byte, pela lei do pintor que o desenhou
/// (o focado: `text_input_text_origin_x` a `TypeToken::Base`, menos o quanto a linha deslizou para o caret caber;
/// o em repouso: `FIELD_PAD_R` a `VALUE_PX`).
fn record_stops(doc: &mut InputMapDoc, ts: &mut TextSystem, id: ph2d_a11y::NodeId, theme: Theme) {
    let Some(r) = doc.hits.rect_for(id) else {
        return;
    };
    let text = doc.store.text(id).unwrap_or("").to_owned();
    let (x0, px) = if doc.store.focus_id() == Some(id) {
        let px = ph2d_tokens::TypeToken::Base.px();
        let x0 = ph2d_editor_core::widget::text_input_text_origin_x(r);
        (x0 - text_scroll(doc, ts, id, r, &text, px, theme), px)
    } else {
        (r.x + m::FIELD_PAD_R, m::VALUE_PX)
    };
    let stops = text
        .char_indices()
        .map(|(i, _)| i)
        .chain([text.len()])
        .map(|i| (i, x0 + ts.prefix_width(&text[..i], px)))
        .collect();
    doc.list.text_stops.retain(|(o, _)| *o != id);
    doc.list.text_stops.push((id, stops));
}

/// Quanto a linha do campo focado deslizou para a esquerda (a lei do `caret_scroll` do `TextInput`): o caret
/// passa da largura de dentro (menos a reserva do «×» e do sufixo) ⟹ desliza o que passa mais o próprio caret.
fn text_scroll(
    doc: &InputMapDoc,
    ts: &mut TextSystem,
    id: ph2d_a11y::NodeId,
    r: Rect,
    text: &str,
    px: f32,
    theme: Theme,
) -> f32 {
    use ph2d_editor_core::widget::field_suffix;
    let caret = match doc.store.get(id) {
        Some(ph2d_editor_core::interaction::InteractiveState::TextInput { caret, .. }) => *caret,
        _ => 0,
    };
    let border = ph2d_tokens::visuals::Chrome::of(theme).field_focus.width;
    let mut reserve = if id == ids::SEARCH && !text.is_empty() {
        ph2d_editor_core::panel::cards::busca::reserva(r, border)
    } else {
        0.0
    };
    reserve += field_suffix::de(id, text).map_or(0.0, |s| field_suffix::largura(ts, &s));
    let inner_w = (r.w - ph2d_editor_core::widget::field_pad_x() * 2.0 - reserve).max(0.0);
    let caret_w = ts.prefix_width(&text[..caret.min(text.len())], px);
    (caret_w - (inner_w - StrokeToken::Thin.px())).max(0.0)
}

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{ActionId, Binding, Key};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::VectorScene;

use crate::doc::InputMapDoc;
use crate::ids;
use crate::interact::{Control, ControlRole};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::ButtonState;

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
    // O Add é um botão pequeno à direita do nome (maquete A1), nunca meia linha.
    let add_w = (text::title_width(ts, add, m::VALUE_PX)
        + (m::BUTTON_TEXT_PAD + m::FIELD_PAD_R) * 2.0)
        .max(m::FIELD_VALUE_MIN_W)
        .min(w);
    let name_w = (w - add_w - m::FIELD_GAP).max(0.0);
    let typed = doc
        .store
        .text(ids::NEW_NAME)
        .unwrap_or("")
        .trim()
        .to_owned();
    let add_off = if typed.is_empty() {
        Some("input_map_list.add_off_empty")
    } else if doc.snap.map.id(&typed).is_some() {
        Some("input_map_list.add_off_taken")
    } else {
        None
    };
    if let Some(InteractiveState::Button { state }) = doc.store.get_mut(ids::ADD) {
        *state = if add_off.is_some() {
            ButtonState::Disabled
        } else {
            ButtonState::Normal
        };
    }
    blink(doc, ids::NEW_NAME);
    blink(doc, ids::SEARCH);
    {
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, x, name_w);
        let text = Widget::Text {
            id: ids::NEW_NAME,
            placeholder: new_name,
        };
        let below = c.full(None, text, y);
        let mut c = CardCtx::flutuante(
            scene,
            ts,
            &doc.store,
            &mut doc.hits,
            ink,
            x + w - add_w,
            add_w,
        );
        let button = Widget::Button {
            id: ids::ADD,
            text: add,
            primary: false,
        };
        y = below.max(c.full(None, button, y));
    }
    record_stops(doc, ts, ids::NEW_NAME, theme);
    if let Some(r) = doc.hits.rect_for(ids::NEW_NAME) {
        doc.control(Control::new(
            ids::NEW_NAME,
            r,
            ControlRole::TextField,
            new_name,
        ));
    }
    if let Some(r) = doc.hits.rect_for(ids::ADD) {
        if doc.is_pressed(ids::ADD) {
            ring(scene, r, ink);
        }
        // A aresta lê a caixa como botão: neutra desligado, no acento só quando o Add pode agir.
        let edge = if add_off.is_some() {
            ColorToken::BorderStrong
        } else {
            ColorToken::BorderEmph
        };
        stroke_rounded_rect(scene, r, m::FIELD_RADIUS, m::FIELD_BORDER, ink.color(edge));
        doc.census.filled.push((r, edge));
        doc.control(Control::new(ids::ADD, r, ControlRole::Button, add).disabled(add_off));
    }
    if let Some(why) = doc.list.refused {
        let r = Rect::new(x, y + m::FIELD_ROW_GAP, w, m::FIELD_H);
        text::body(
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
    // A lupa à cabeça do campo, e o texto (ou o chip da tecla) recuado depois dela (maquete A1).
    let g = m::KEY_MARK_BOX;
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(ColorToken::CardField));
    let lupa = Rect::new(r.x + m::SEG_INSET, r.y + (r.h - g) * 0.5, g, g);
    paint_icon(
        scene,
        IconId::Search,
        lupa,
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    let indent = m::SEG_INSET + g;
    let Some(k) = doc.list.search_key else {
        let ph = tr("input_map_list.search");
        let mut c = CardCtx::flutuante(
            scene,
            ts,
            &doc.store,
            &mut doc.hits,
            ink,
            r.x + indent,
            r.w - indent,
        );
        let below = c.full(
            None,
            Widget::Search {
                id: ids::SEARCH,
                placeholder: ph,
            },
            r.y,
        );
        record_stops(doc, ts, ids::SEARCH, ink.theme);
        let field = doc.hits.rect_for(ids::SEARCH).unwrap_or(r);
        doc.control(Control::new(ids::SEARCH, field, ControlRole::TextField, ph));
        // O × só com texto: limpa e o foco fica na busca.
        if !doc.store.text(ids::SEARCH).unwrap_or("").is_empty() {
            clear_button(doc, scene, clear_box(r), ink);
        }
        return below;
    };
    let ph = tr("input_map_list.search");
    doc.control(Control::new(ids::SEARCH, r, ControlRole::TextField, ph));
    // A tecla procurada é o texto do campo, regular (maquete A3): o amarelo fica nas teclas das linhas.
    let label = k.label();
    let x_box = clear_box(r);
    let inner = Rect::new(
        r.x + indent + m::SEG_INSET,
        r.y,
        (x_box.x - r.x - indent - m::SEG_INSET * 2.0).max(0.0),
        r.h,
    );
    text::body(
        &mut doc.census,
        ts,
        scene,
        &label,
        inner,
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    // O × só com o campo focado ou sob o ponteiro; o alvo fica registado sempre.
    clear_button(doc, scene, x_box, ink);
    r.y + r.h
}

/// O alvo do × da busca: a caixa de ícone do cartão (`HEAD_ICON_BOX`, ≥ 24 px) encostada à direita do campo `r`.
pub(crate) fn clear_box(r: Rect) -> Rect {
    let s = m::HEAD_ICON_BOX.min(r.h);
    Rect::new(r.x + r.w - s - m::SEG_PAD, r.y + (r.h - s) * 0.5, s, s)
}

/// O × da busca: um botão do registo (hover e premido à vista) com o glifo de ícone do cartão ao meio do alvo.
fn clear_button(doc: &mut InputMapDoc, scene: &mut VectorScene, x_box: Rect, ink: CardInk) {
    let id = ids::SEARCH_CLEAR;
    if doc.is_pressed(id) || doc.is_hovered(id) {
        let bg = if doc.is_pressed(id) {
            ColorToken::AccentSoft
        } else {
            ColorToken::CardFieldHover
        };
        fill_rounded_rect(scene, x_box, m::SEG_RADIUS, ink.color(bg));
        doc.census.filled.push((x_box, bg));
    }
    let gl = m::HEAD_ICON_GLYPH.min(x_box.w);
    let glyph = Rect::new(
        x_box.x + (x_box.w - gl) * 0.5,
        x_box.y + (x_box.h - gl) * 0.5,
        gl,
        gl,
    );
    paint_icon(
        scene,
        IconId::Close,
        glyph,
        ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
    let label = tr("input_map_list.clear_search");
    doc.control(Control::new(id, x_box, ControlRole::Button, label));
}

/// A tinta que se lê sobre o amarelo do destaque (escura no tema escuro, a do tema no claro).
pub(crate) fn on_highlight(ink: CardInk) -> ph2d_vector::Color {
    let under = ColorToken::InputMapHighlight.resolve(ink.theme);
    ph2d_editor_core::paint::token_to_vello(ph2d_tokens::skin_body::readable_on(under, ink.theme))
}

/// A caixa à escuta: o anel do foco TRACEJADO (maquete A2), contado no censo.
pub(crate) fn listening_ring(
    census: &mut crate::census::Census,
    scene: &mut VectorScene,
    r: Rect,
    ink: CardInk,
) {
    ph2d_editor_core::paint::stroke_rounded_rect_dashed(
        scene,
        r,
        m::SEG_RADIUS,
        m::FIELD_BORDER,
        m::LOOSE_DASH,
        ink.color(ColorToken::FocusRing),
    );
    census.dashed.push(r);
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
