//! Controls (maquete C4): a tela do jogo emoldurada ao centro no TOPO, pintada pela prévia do painel do Play; por
//! baixo «Players can change keys», as caixas do «Shown in the game» em pílulas e, discretos, o «Opens with» e o
//! «Forget them».

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
use ph2d_input::{ActionId, PlayerControls};

use super::state::TryHit;
use super::target;

/// A tela do jogo da prévia (maquete C4): 16:9, o painel ao centro do céu, a faixa do chão por baixo.
const SCREEN_W: f32 = ROW * 16.0;
const SCREEN_H: f32 = SCREEN_W * 9.0 / 16.0;
/// O texto discreto do pé do cartão («Opens with», a frase do jogador e o «Forget them»).
const SUBTLE_PX: f32 = m::LABEL_PX;

/// A escala mínima da prévia: abaixo dela o texto do painel (11 linhas) deixa de se ler.
pub const PREVIEW_MIN_SCALE: f32 = 0.75;

/// A altura da tela do jogo para `rows`: 16:9 por omissão; com muitas linhas cresce para o painel inteiro caber a
/// [`PREVIEW_MIN_SCALE`] no céu (o recuo da prévia e a faixa do chão por cima disso).
#[must_use]
pub fn preview_frame_h(sw: f32, panel_h: f32) -> f32 {
    let need = panel_h * PREVIEW_MIN_SCALE + m::CARD_PAD_X * 2.0 + ROW;
    (sw * SCREEN_H / SCREEN_W).max(need)
}

/// O palco (relativo ao conteúdo, largura inteira) e a tela do jogo ao centro dele.
fn stage(doc: &InputMapDoc, ts: &mut TextSystem, w: f32) -> (Rect, Rect) {
    let iw = inner_w(w);
    let sw = SCREEN_W.min((iw - PAD * 2.0).max(0.0));
    let rows = ph2d_player_controls::rows(doc.snap.read_map());
    let panel = ph2d_player_controls::Layout::natural(0.0, 0.0, &rows, ts).panel;
    let base = sw * SCREEN_H / SCREEN_W;
    let sh = preview_frame_h(sw, panel.h).min(doc.tryit.screen_cap.max(base));
    let st = Rect::new(0.0, 0.0, iw, sh + PAD * 2.0);
    (st, Rect::new((iw - sw) * 0.5, PAD, sw, sh))
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

/// A pílula de uma caixa: a caixa, o nome e o recuo dos dois lados.
fn chip_w(ts: &mut TextSystem, name: &str) -> f32 {
    ts.prefix_width(name, m::VALUE_PX) + ROW + m::SEG_INSET * 4.0
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
        ROW
    } else {
        0.0
    };
    let chips = chips_rows(doc, ts, inner_w(w)) as f32 * (ROW + GAP);
    let below = (ROW + GAP) + refuse + line_h(m::LABEL_PX) + chips + ROW + forget;
    let rest = m::CARD_PAD_TOP + line_h(m::LABEL_PX) + GAP + below + m::CARD_PAD_BOTTOM;
    // A tela cresce só até ao que sobra na coluna: a vista não passa a rolar por causa dela.
    doc.tryit.screen_cap = (doc.tryit.controls_room - rest - PAD * 2.0).max(0.0);
    let (st, _) = stage(doc, ts, w);
    rest + st.h
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
    let (st, screen) = stage(doc, ts, r.w);
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
    preview(doc, (x, y), (st, screen), scene, ts, ink);
    y += st.h + GAP;
    let controls = doc.snap.map.controls().clone();
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
    y = shown(doc, (x, y, w), &controls, scene, ts, ink);
    y = open_key(doc, (x, y, w), &controls, scene, ts, ink);
    if doc.tryit.player_changed > 0 {
        forget(doc, (x, y, w), scene, ts, ink);
    }
}

fn check(on: bool) -> CheckboxValue {
    if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    }
}

/// O palco e a tela do jogo emoldurada, com o painel do Play pela prévia (sem linha escolhida, sem anel de foco).
fn preview(
    doc: &mut InputMapDoc,
    (x, y): (f32, f32),
    (st, screen): (Rect, Rect),
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let rad = m::FIELD_RADIUS;
    let bed = Rect::new(x + st.x, y + st.y, st.w, st.h);
    fill_rounded_rect(scene, bed, rad, ink.color(ColorToken::CardField));
    let back = Rect::new(x + screen.x, y + screen.y, screen.w, screen.h);
    fill_rounded_rect(scene, back, rad, ink.color(ColorToken::PreviewSky));
    let ground = Rect::new(back.x, back.y + back.h - ROW, back.w, ROW);
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
    let sky = Rect::new(back.x, back.y, back.w, back.h - ROW);
    let at = ph2d_player_controls::paint_preview(sky, &rows, scene, ts, doc.tryit.theme);
    let natural = ph2d_player_controls::Layout::natural(0.0, 0.0, &rows, ts).panel;
    doc.tryit.preview_scale = if natural.h > 0.0 {
        at.h / natural.h
    } else {
        1.0
    };
}

/// «Shown in the game» e uma caixa por acção, cada uma numa pílula; devolve o fundo.
fn shown(
    doc: &mut InputMapDoc,
    (x, y, w): (f32, f32, f32),
    controls: &PlayerControls,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let mut y = y;
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
        let pill = Rect::new(cx, y, cw, ROW);
        let bg = ink.color(ColorToken::CardField);
        fill_rounded_rect(scene, pill, m::FIELD_RADIUS, bg);
        let (px, pw) = (cx + m::SEG_INSET, cw - m::SEG_INSET * 2.0);
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, px, pw);
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
    y + ROW + GAP
}

/// «Opens with F1», discreto; devolve o fundo da linha.
fn open_key(
    doc: &mut InputMapDoc,
    (x, y, w): (f32, f32, f32),
    controls: &PlayerControls,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) -> f32 {
    let dim = ink.color(ColorToken::CardLabel);
    let lead = tr("input_map_try.opens_with");
    let lw = word(doc, ts, scene, lead, (x, y, w), SUBTLE_PX, dim);
    let key_text = if doc.tryit.listening_open_key {
        tr("input_map_try.press_open_key").to_owned()
    } else {
        controls.open_key.label()
    };
    let tw = ts.prefix_width(&key_text, SUBTLE_PX);
    let ch = line_h(SUBTLE_PX);
    let chip = Rect::new(
        x + lw + m::SEG_ICON_GAP,
        y + (ROW - ch) * 0.5,
        tw + m::SEG_INSET * 2.0,
        ch,
    );
    let (bg, fg) = if doc.tryit.listening_open_key {
        (ColorToken::Accent, ColorToken::Text1)
    } else {
        (ColorToken::CardField, ColorToken::Text2)
    };
    fill_rounded_rect(scene, chip, m::FIELD_RADIUS, ink.color(bg));
    line(
        &mut doc.census,
        ts,
        scene,
        &key_text,
        Rect::new(chip.x + m::SEG_INSET, y, tw, ROW),
        SUBTLE_PX,
        ink.color(fg),
    );
    target(doc, TryHit::OpenKey, chip);
    y + ROW
}

/// «On this computer, a player changed N keys. · Forget them», discreto; o «Forget them» é uma ligação no acento.
fn forget(
    doc: &mut InputMapDoc,
    (x, y, w): (f32, f32, f32),
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: CardInk,
) {
    let dim = ink.color(ColorToken::CardLabel);
    let msg = format!("{} \u{00B7}", changed_text(doc.tryit.player_changed));
    let mw = word(doc, ts, scene, &msg, (x, y, w), SUBTLE_PX, dim);
    let text = tr("input_map_try.forget");
    let fx = x + mw + m::SEG_ICON_GAP;
    let fw = ts.prefix_width(text, SUBTLE_PX).min((x + w - fx).max(0.0));
    let link = ink.color(ColorToken::Accent);
    word(doc, ts, scene, text, (fx, y, fw), SUBTLE_PX, link);
    target(doc, TryHit::Forget, Rect::new(fx, y, fw, ROW));
}
