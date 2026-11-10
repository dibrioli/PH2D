//! Controls (C4): a prévia da tela do jogador pela MESMA pintura do Play, as caixas e o «Forget them».

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

use super::cards::{GAP, PAD, ROW, inner_w, word};
use super::{line_h, para};
use crate::doc::InputMapDoc;
use crate::view_list::text::line;
use ph2d_editor_core::paint::{fill_rounded_rect, fill_rounded_rect_radii, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_input::ActionId;

use super::state::TryHit;
use super::target;

/// A tela do jogo da prévia (maquete C4): 16:9, o painel ao centro do céu, a faixa do chão por baixo.
const SCREEN_W: f32 = ROW * 16.0;
const SCREEN_H: f32 = SCREEN_W * 9.0 / 16.0;
/// A coluna dos controlos só fica AO LADO da tela com esta largura; abaixo dela, empilha.
const SIDE_MIN: f32 = ROW * 14.0;

/// A tela (relativa ao conteúdo) e a coluna dos controlos `(dx, w)`: lado a lado no largo (o cartão inteiro cabe a
/// 1600×1000), empilhadas no estreito com a tela ao centro.
fn split(w: f32) -> (Rect, f32, f32) {
    let iw = inner_w(w);
    let sw = SCREEN_W.min(iw);
    let sh = sw * SCREEN_H / SCREEN_W;
    if iw >= SCREEN_W + PAD + SIDE_MIN {
        (Rect::new(0.0, 0.0, sw, sh), sw + PAD, iw - sw - PAD)
    } else {
        (Rect::new((iw - sw) * 0.5, 0.0, sw, sh), 0.0, iw)
    }
}

fn chips_rows(doc: &InputMapDoc, ts: &mut TextSystem, col_w: f32) -> usize {
    let mut rows = 1;
    let mut x = 0.0;
    for a in doc.snap.map.actions() {
        let cw = chip_w(ts, &a.name);
        if x + cw > col_w && x > 0.0 {
            rows += 1;
            x = 0.0;
        }
        x += cw + m::FIELD_GAP;
    }
    rows
}

fn chip_w(ts: &mut TextSystem, name: &str) -> f32 {
    ts.prefix_width(name, m::VALUE_PX) + ROW + m::SEG_INSET * 2.0
}

fn changed_text(n: usize) -> String {
    let k = if n == 1 {
        "input_map_try.changed_one"
    } else {
        "input_map_try.changed_many"
    };
    tr(k).replace("{n}", &n.to_string())
}

pub(super) fn controls_h(doc: &mut InputMapDoc, ts: &mut TextSystem, w: f32) -> f32 {
    let refuse = doc.tryit.refused.map_or(0.0, |_| line_h(m::LABEL_PX));
    let forget = if doc.tryit.player_changed > 0 {
        ROW + GAP
    } else {
        0.0
    };
    let (screen, dx, col_w) = split(w);
    let chips = chips_rows(doc, ts, col_w) as f32 * (ROW + GAP);
    let column = (ROW + GAP) * 2.0 + refuse + line_h(m::LABEL_PX) + chips + forget;
    let body = if dx > 0.0 {
        screen.h.max(column)
    } else {
        screen.h + GAP + column
    };
    m::CARD_PAD_TOP + line_h(m::LABEL_PX) + body + m::CARD_PAD_BOTTOM
}

/// C·4: a prévia pela MESMA pintura da tela do jogador, as caixas e o «Forget them».
pub(super) fn controls(
    doc: &mut InputMapDoc,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let (x, w) = (r.x + PAD, inner_w(r.w));
    let mut y = r.y + m::CARD_PAD_TOP;
    let (screen, dx, col_w) = split(r.w);
    let label = Rect::new(x, y, w, line_h(m::LABEL_PX));
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_try.controls"),
        label,
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += line_h(m::LABEL_PX);
    let back = Rect::new(x + screen.x, y, screen.w, screen.h);
    // A prévia é uma TELA DE JOGO emoldurada (maquete C4): céu claro, a faixa do chão, o painel ao centro do céu.
    fill_rounded_rect(
        scene,
        back,
        m::FIELD_RADIUS,
        ink.color(ColorToken::PreviewSky),
    );
    let ground = Rect::new(back.x, back.y + back.h - ROW, back.w, ROW);
    let rad = m::FIELD_RADIUS;
    fill_rounded_rect_radii(
        scene,
        ground,
        (0.0, 0.0, rad, rad),
        ink.color(ColorToken::PreviewGround),
    );
    stroke_rounded_rect(
        scene,
        back,
        rad,
        StrokeToken::Hairline.px(),
        ink.color(ColorToken::Border),
    );
    let rows = ph2d_player_controls::rows(doc.snap.read_map());
    let shown = ph2d_player_controls::ControlsScreen::default();
    let sky = Rect::new(back.x, back.y, back.w, back.h - ROW);
    ph2d_player_controls::paint(sky, &rows, &shown, scene, ts, doc.tryit.theme);
    y += if dx > 0.0 { 0.0 } else { screen.h + GAP };
    let (x, w) = (x + dx, col_w);
    let controls = doc.snap.map.controls().clone();
    let check = |on: bool| {
        if on {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        }
    };
    {
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, x, w);
        let id = TryHit::PlayersCanChange.id();
        let label = Rotulo::vista_de(tr("input_map_try.players_can_change"));
        y = c.full(
            None,
            Widget::Check {
                id,
                label,
                value: check(controls.players_can_change_keys),
                off: false,
            },
            y,
        ) + GAP;
    }
    doc.tryit.targets.push(TryHit::PlayersCanChange);
    let lead = tr("input_map_try.opens_with");
    let lw = word(
        doc,
        ts,
        scene,
        lead,
        (x, y, w),
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    let key_text = if doc.tryit.listening_open_key {
        tr("input_map_try.press_open_key").to_owned()
    } else {
        controls.open_key.label()
    };
    let tw = ts.prefix_width(&key_text, m::VALUE_PX);
    let chip = Rect::new(x + lw + m::SEG_ICON_GAP, y, tw + m::SEG_INSET * 2.0, ROW);
    let bg = if doc.tryit.listening_open_key {
        ColorToken::Accent
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(scene, chip, m::FIELD_RADIUS, ink.color(bg));
    line(
        &mut doc.census,
        ts,
        scene,
        &key_text,
        Rect::new(chip.x + m::SEG_INSET, y, tw, ROW),
        m::VALUE_PX,
        ink.color(ColorToken::Text1),
    );
    target(doc, TryHit::OpenKey, chip);
    y += ROW + GAP;
    if let Some(k) = doc.tryit.refused {
        let esc = ph2d_input::Key::named_in_script("escape").unwrap_or_default();
        let why = tr(if esc.contains(&k) {
            "input_map_try.refused_escape"
        } else {
            "input_map_try.refused_f5"
        });
        y = para(
            doc,
            ts,
            scene,
            why,
            (x, y, w),
            m::LABEL_PX,
            ink.color(ColorToken::Warn),
        );
    }
    let sl = Rect::new(x, y, w, line_h(m::LABEL_PX));
    line(
        &mut doc.census,
        ts,
        scene,
        tr("input_map_try.shown"),
        sl,
        m::LABEL_PX,
        ink.color(ColorToken::CardLabel),
    );
    y += line_h(m::LABEL_PX);
    let mut cx = x;
    let actions: Vec<(ActionId, String)> = doc
        .snap
        .map
        .actions()
        .iter()
        .map(|a| (a.id, a.name.clone()))
        .collect();
    for (id, name) in actions {
        let cw = chip_w(ts, &name);
        if cx + cw > x + w && cx > x {
            cx = x;
            y += ROW + GAP;
        }
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, cx, cw);
        let hid = TryHit::Shown(id).id();
        c.full(
            None,
            Widget::Check {
                id: hid,
                label: Rotulo::vista_de(name.as_str()),
                value: check(controls.shown_in_game(id)),
                off: false,
            },
            y,
        );
        doc.tryit.targets.push(TryHit::Shown(id));
        cx += cw + m::FIELD_GAP;
    }
    y += ROW + GAP;
    if doc.tryit.player_changed > 0 {
        let msg = changed_text(doc.tryit.player_changed);
        let mw = word(
            doc,
            ts,
            scene,
            &msg,
            (x, y, w),
            m::VALUE_PX,
            ink.color(ColorToken::Text1),
        );
        let fx = x + mw + m::SEG_ICON_GAP;
        let fw = (x + w - fx).max(0.0);
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, fx, fw);
        c.full(
            None,
            Widget::Button {
                id: TryHit::Forget.id(),
                text: tr("input_map_try.forget"),
                primary: false,
            },
            y,
        );
        doc.tryit.targets.push(TryHit::Forget);
    }
}
