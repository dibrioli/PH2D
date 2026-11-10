//! O cartão de baixo: «jump · 2 keys» e quem a lê, as ligações em ladrilhos (× no hover), o ladrilho tracejado
//! que escuta, a frase da tecla repetida, o aviso de quem ninguém lê e — com manche — a barra do curso.

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, stroke_rounded_rect_dashed};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::readers::shared_bindings;
use ph2d_input::{Binding, InputAction};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::state::{KbHit, Sub};
use super::{geometry as g, target};
use crate::doc::InputMapDoc;
use crate::view_list::footer::{and_list, binding_label, reader_names, readers_of};
use crate::view_list::text::{line, title as semibold, title_width, wrap_after};

const TITLE_H: f32 = m::FIELD_H;
const TILE_H: f32 = m::FIELD_H * 2.0;
const CAP: f32 = m::FIELD_H - m::SEG_INSET;

fn title(a: &InputAction) -> String {
    let n = a.bindings.len();
    let keys_only = a.bindings.iter().all(|b| matches!(b, Binding::Key(_)));
    let k = match n {
        0 => "input_map_keyboard.title.none",
        1 if keys_only => "input_map_keyboard.title.keys_one",
        _ if keys_only => "input_map_keyboard.title.keys_many",
        _ => "input_map_keyboard.title.inputs_many",
    };
    tr(k)
        .replace("{action}", &a.name)
        .replace("{n}", &n.to_string())
}

/// A frase grande e a legenda de um ladrilho: «↑» + «Up Arrow», «Z» + «Keyboard», «LS ←» + «Gamepad».
fn face(b: Binding) -> (String, Option<ph2d_editor_core::icons::IconId>, String) {
    match b {
        Binding::Key(k) => match g::arrow_icon(k) {
            Some(i) => (String::new(), Some(i), k.label()),
            None => (
                k.label(),
                None,
                tr("input_map_keyboard.device.keyboard").to_owned(),
            ),
        },
        other => {
            let short = g::pad()
                .iter()
                .find(|p| super::same_input(p.binding, other))
                .map_or_else(|| binding_label(other), |p| tr(p.short_key).to_owned());
            (
                short,
                g::half_icon(other),
                tr("input_map_keyboard.device.gamepad").to_owned(),
            )
        }
    }
}

/// A altura de uma linha de frase partida.
const PHRASE_LINE_H: f32 = m::VALUE_PX + m::SEG_INSET;

/// Uma frase de aviso (fundo e faixa do aviso) ou de explicação, PARTIDA dentro do cartão: devolve a altura.
#[allow(clippy::too_many_arguments)]
fn phrase(
    doc: &mut InputMapDoc,
    s: &str,
    warn: bool,
    (x, y, w): (f32, f32, f32),
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let tx = x + m::SEG_INSET * 2.0;
    let tw = (w - m::SEG_INSET * 3.0).max(0.0);
    // Dois pesos (maquete B1): a 1.ª frase em SemiBold clara, o resto regular apagado a um espaço, na mesma linha.
    let (lead, rest) = match s.split_once(". ").filter(|_| !warn) {
        Some((l, r)) => (format!("{l}."), r),
        None => (String::new(), s),
    };
    let lead_w = if lead.is_empty() {
        0.0
    } else {
        (title_width(ts, &lead, m::VALUE_PX) + ts.prefix_width(" ", m::VALUE_PX)).min(tw)
    };
    let lines = wrap_after(ts, rest, m::VALUE_PX, (tw - lead_w).max(0.0), tw);
    let h = PHRASE_LINE_H * lines.len() as f32 + (m::FIELD_H - PHRASE_LINE_H);
    let r = Rect::new(x, y, w, h);
    let bg = if warn {
        ColorToken::InputMapWarningBg
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(scene, r, m::FIELD_RADIUS, ink.color(bg));
    if warn {
        let bar = Rect::new(r.x, r.y, m::SEG_PAD, r.h);
        fill_rounded_rect(
            scene,
            bar,
            m::FIELD_BORDER,
            ink.color(ColorToken::InputMapWarning),
        );
    }
    let mut ly = y + (m::FIELD_H - PHRASE_LINE_H) * 0.5;
    if !lead.is_empty() {
        semibold(
            &mut doc.census,
            ts,
            scene,
            &lead,
            Rect::new(tx, ly, lead_w, PHRASE_LINE_H),
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
    }
    let rest_ink = ink.color(if lead.is_empty() {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    });
    for (i, l) in lines.iter().enumerate() {
        let lx = if i == 0 { tx + lead_w } else { tx };
        if !l.is_empty() {
            line(
                &mut doc.census,
                ts,
                scene,
                l,
                Rect::new(lx, ly, (tx + tw - lx).max(0.0), PHRASE_LINE_H),
                m::VALUE_PX,
                rest_ink,
            );
        }
        ly += PHRASE_LINE_H;
    }
    h
}

/// Pinta o cartão da acção aberta a partir de `r.y` e devolve onde o conteúdo acaba (o cartão cabe nele).
pub(super) fn paint(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let x = r.x + m::CARD_PAD_X;
    let w = (r.w - m::CARD_PAD_X * 2.0).max(0.0);
    let mut y = r.y + m::CARD_PAD_TOP;
    let Some(a) = doc.kb.open.and_then(|id| doc.snap.map.get(id)).cloned() else {
        line(
            &mut doc.census,
            ts,
            scene,
            tr("input_map_keyboard.no_action"),
            Rect::new(x, y, w, TITLE_H),
            m::VALUE_PX,
            ink.color(ColorToken::Text2),
        );
        return y + TITLE_H + m::CARD_PAD_BOTTOM;
    };
    let rs = readers_of(&doc.snap, &a.name).to_vec();
    let right = if rs.is_empty() {
        tr("input_map_keyboard.read_by_nobody").to_owned()
    } else {
        tr("input_map_keyboard.read_by").replace("{who}", &reader_names(&rs))
    };
    let rw = ts.prefix_width(&right, m::LABEL_PX);
    let t = title(&a);
    let tw = ts
        .prefix_width(&t, m::CARD_TITLE_PX)
        .min((w - rw - m::FIELD_GAP).max(0.0));
    line(
        &mut doc.census,
        ts,
        scene,
        &t,
        Rect::new(x, y, tw, TITLE_H),
        m::CARD_TITLE_PX,
        ink.color(ColorToken::Text1),
    );
    line(
        &mut doc.census,
        ts,
        scene,
        &right,
        Rect::new(x + w - rw, y, rw, TITLE_H),
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += TITLE_H + m::FIELD_ROW_GAP;
    let mut tx = x;
    for (n, b) in a.bindings.iter().enumerate() {
        let (big, icon, cap) = face(*b);
        let bw = ts.prefix_width(&big, m::VALUE_PX)
            + icon.map_or(0.0, |_| super::keys::ARROW_BOX + m::SEG_PAD);
        let cw = ts.prefix_width(&cap, m::LABEL_PX);
        let tile = Rect::new(
            tx,
            y,
            (bw.max(cw) + m::CARD_PAD_X * 2.0).max(TILE_H),
            TILE_H,
        );
        fill_rounded_rect(
            scene,
            tile,
            m::FIELD_RADIUS,
            ink.color(ColorToken::CardField),
        );
        let kw = bw + m::SEG_INSET * 2.0;
        let key = Rect::new(
            tile.x + (tile.w - kw) * 0.5,
            tile.y + m::SEG_INSET,
            kw,
            CAP - m::SEG_INSET,
        );
        fill_rounded_rect(
            scene,
            key,
            m::SEG_RADIUS,
            ink.color(ColorToken::KanbanCardRaised),
        );
        let mut kx = key.x + m::SEG_INSET;
        if !big.is_empty() {
            let w0 = ts.prefix_width(&big, m::VALUE_PX);
            line(
                &mut doc.census,
                ts,
                scene,
                &big,
                Rect::new(kx, key.y, w0, key.h),
                m::VALUE_PX,
                ink.color(ColorToken::Text1),
            );
            kx += w0 + m::SEG_PAD;
        }
        if let Some(i) = icon {
            let s = super::keys::ARROW_BOX.min(key.h);
            paint_icon(
                scene,
                i,
                Rect::new(kx, key.y + (key.h - s) * 0.5, s, s),
                ink.color(ColorToken::Text1),
                StrokeToken::Default.px(),
            );
        }
        let cr = Rect::new(
            tile.x + (tile.w - cw) * 0.5,
            key.y + key.h + m::SEG_PAD,
            cw,
            CAP,
        );
        line(
            &mut doc.census,
            ts,
            scene,
            &cap,
            cr,
            m::LABEL_PX,
            ink.color(ColorToken::CardLabel),
        );
        let s = m::KEY_MARK_BOX;
        let xr = Rect::new(tile.x + tile.w - s - m::SEG_PAD, tile.y + m::SEG_PAD, s, s);
        if doc.kb.pointer.is_some_and(|(px, py)| tile.contains(px, py)) {
            paint_icon(
                scene,
                ph2d_editor_core::icons::IconId::Close,
                xr,
                ink.color(ColorToken::Text2),
                StrokeToken::Default.px(),
            );
            target(doc, KbHit::TileX(n), xr);
        }
        tx += tile.w + m::FIELD_GAP;
    }
    let listening = doc.snap.listening == Some(a.id);
    let msg = if listening {
        if doc.snap.gamepad_connected {
            tr("input_map_list.listen_pad")
        } else {
            tr("input_map_list.listen_key")
        }
    } else if a.bindings.is_empty() {
        tr("input_map_keyboard.press_first")
    } else if doc.kb.sub == Sub::Gamepad {
        tr("input_map_keyboard.press_button")
    } else {
        tr("input_map_keyboard.press_key")
    };
    let lw = ts.prefix_width(msg, m::LABEL_PX) + m::CARD_PAD_X * 2.0;
    let room = (x + w - tx).max(0.0);
    let (lx, ly) = if lw > room && tx > x {
        (x, y + TILE_H + m::FIELD_ROW_GAP)
    } else {
        (tx, y)
    };
    let tile = Rect::new(lx, ly, lw.max(TILE_H).min(w), TILE_H);
    // À escuta, ou a acção ainda sem ligação (maquete B4): o tracejado do acento.
    if listening || a.bindings.is_empty() {
        stroke_rounded_rect_dashed(
            scene,
            tile,
            m::FIELD_RADIUS,
            m::FIELD_BORDER * 2.0,
            m::LOOSE_DASH,
            ink.color(ColorToken::Accent),
        );
        if listening {
            doc.census.dashed.push(tile);
        }
    } else {
        stroke_rounded_rect_dashed(
            scene,
            tile,
            m::FIELD_RADIUS,
            m::FIELD_BORDER,
            m::LOOSE_DASH,
            ink.color(ColorToken::Text2),
        );
    }
    let mw = ts
        .prefix_width(msg, m::LABEL_PX)
        .min(tile.w - m::SEG_INSET * 2.0);
    // Em baixo, na linha da legenda dos outros ladrilhos (maquete B1/B4).
    let my = tile.y + m::SEG_INSET + CAP + m::SEG_PAD;
    line(
        &mut doc.census,
        ts,
        scene,
        msg,
        Rect::new(tile.x + (tile.w - mw) * 0.5, my, mw, CAP),
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    target(doc, KbHit::Listen, tile);
    y = tile.y + tile.h + m::FIELD_ROW_GAP;
    for (s, warn) in phrases(doc, &a) {
        y += phrase(doc, &s, warn, (x, y, w), scene, ts, ink) + m::FIELD_ROW_GAP;
    }
    if a.is_analog() {
        y = super::travel::paint(doc, &a, Rect::new(x, y, w, 0.0), scene, ts, ink)
            + m::FIELD_ROW_GAP;
    }
    y - m::FIELD_ROW_GAP + m::CARD_PAD_BOTTOM
}

/// As frases de baixo: a tecla repetida (de propósito explica; engano avisa), o Add recusado e ninguém lê.
fn phrases(doc: &InputMapDoc, a: &InputAction) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    // A frase do hover (no topo do teclado) já diz quem partilha a tecla sob o ponteiro: aqui não se repete.
    let hovered = |b: Binding| match doc.kb.hover {
        Some(super::state::KbHit::Key(k)) => b == Binding::Key(k),
        Some(super::state::KbHit::Pad(p)) => b == p,
        _ => false,
    };
    for s in shared_bindings(&doc.snap.map, &doc.snap.readers)
        .iter()
        .filter(|s| s.actions.contains(&a.id) && !hovered(s.binding))
    {
        let others: Vec<String> = s
            .actions
            .iter()
            .filter(|o| **o != a.id)
            .filter_map(|o| doc.snap.map.get(*o).map(|x| x.name.clone()))
            .collect();
        let who: Vec<ph2d_input::readers::Reader> = others
            .iter()
            .flat_map(|n| readers_of(&doc.snap, n).to_vec())
            .collect();
        let k = if s.on_purpose {
            "input_map_keyboard.shared_fine"
        } else {
            "input_map_keyboard.shared_warn"
        };
        out.push((
            tr(k)
                .replace("{key}", &binding_label(s.binding))
                .replace("{others}", &and_list(&others))
                .replace("{who}", &reader_names(&who)),
            !s.on_purpose,
        ));
    }
    if let Some(why) = doc.list.refused {
        out.push((tr(why).to_owned(), true));
    }
    if readers_of(&doc.snap, &a.name).is_empty() {
        out.push((
            tr("input_map_keyboard.nobody_reads").replace("{action}", &a.name),
            true,
        ));
    }
    out
}
