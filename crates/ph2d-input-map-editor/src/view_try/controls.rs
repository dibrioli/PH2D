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
// O corpo da C no peso Regular (como a A e a B); só os destaques e os nomes de acção vão em SemiBold.
use crate::view_list::text::body as line;
use ph2d_editor_core::paint::{fill_rounded_rect, fill_rounded_rect_radii, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::{CardCtx, Widget};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_input::{ActionId, PlayerControls};

use super::state::TryHit;
use super::{card_target, target};

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

/// O vão da grelha do «Shown in the game», nas duas direcções.
pub const SHOWN_GAP: f32 = m::CARD_GAP;

/// A grelha do «Shown in the game» em `w`: colunas IGUAIS (a maior caixa cabe em cada uma), as filas equilibradas
/// (11 em 6 + 5, nunca 10 + 1) e a largura repartida por igual. Devolve `(colunas, filas, largura da coluna)`.
#[must_use]
pub fn shown_grid(names: &[&str], ts: &mut TextSystem, w: f32) -> (usize, usize, f32) {
    let n = names.len().max(1);
    let widest = names.iter().map(|s| chip_w(ts, s)).fold(0.0_f32, f32::max);
    let max_cols = (((w + SHOWN_GAP) / (widest + SHOWN_GAP)).floor() as usize).clamp(1, n);
    let rows = n.div_ceil(max_cols);
    let cols = n.div_ceil(rows);
    let cw = ((w - SHOWN_GAP * (cols as f32 - 1.0)) / cols as f32).max(0.0);
    (cols, rows, cw)
}

fn chips_rows(doc: &InputMapDoc, ts: &mut TextSystem, w: f32) -> usize {
    let names: Vec<&str> = doc
        .snap
        .map
        .actions()
        .iter()
        .map(|a| a.name.as_str())
        .collect();
    shown_grid(&names, ts, w).1
}

/// A pílula de uma caixa: a moldura com ✓ e o nome dentro, e o losango ao lado.
fn chip_w(ts: &mut TextSystem, name: &str) -> f32 {
    ph2d_editor_core::widget::card_check_in_row_w(ts, name)
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
    let chips = chips_rows(doc, ts, inner_w(w)) as f32 * (ROW + SHOWN_GAP) - SHOWN_GAP + GAP;
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
    let label = tr("input_map_try.players_can_change");
    card_target(doc, TryHit::PlayersCanChange, label, scene, ink);
    if doc.tryit.refused.is_some() {
        let why = tr("input_map_try.refused_f5");
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
    let actions: Vec<(ActionId, String)> = doc
        .snap
        .map
        .actions()
        .iter()
        .map(|a| (a.id, a.name.clone()))
        .collect();
    let names: Vec<&str> = actions.iter().map(|a| a.1.as_str()).collect();
    let (cols, rows, cw) = shown_grid(&names, ts, w);
    for (i, (id, name)) in actions.into_iter().enumerate() {
        let cx = x + (cw + SHOWN_GAP) * (i % cols) as f32;
        let y = y + (ROW + SHOWN_GAP) * (i / cols) as f32;
        // A pílula É a moldura da caixa em fila (à Godot, dono 13/11): o alvo é o que se pinta.
        let pill = Rect::new(cx, y, cw, ROW);
        let mut c = CardCtx::flutuante(scene, ts, &doc.store, &mut doc.hits, ink, cx, cw);
        let hid = TryHit::Shown(id).id();
        let valor = check(controls.shown_in_game(id));
        c.marca_em_fila_em(pill, (hid, Rotulo::vista_de(name.as_str())), valor);
        card_target(doc, TryHit::Shown(id), name.as_str(), scene, ink);
    }
    y + (ROW + SHOWN_GAP) * rows.max(1) as f32 - SHOWN_GAP + GAP
}

/// O chip da tecla no MESMO molde dos chips de tecla da vista A (`view_list::rows::paint_chip`): a altura, o raio,
/// o recuo dos lados e o texto no tamanho do valor.
pub const KEY_CHIP_H: f32 = m::FIELD_H - m::SEG_INSET;
pub const KEY_CHIP_RADIUS: f32 = m::SEG_RADIUS;
pub const KEY_CHIP_SIDE: f32 = m::KEY_MARK_BOX + m::SEG_INSET + m::SEG_PAD * 2.0;

/// «Opens with F1»: o rótulo discreto e a tecla num chip de tecla; devolve o fundo da linha.
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
    let tw = ts.prefix_width(&key_text, m::VALUE_PX);
    let chip = Rect::new(
        x + lw + m::SEG_ICON_GAP,
        y + (ROW - KEY_CHIP_H) * 0.5,
        tw + KEY_CHIP_SIDE * 2.0,
        KEY_CHIP_H,
    );
    let id = TryHit::OpenKey.id();
    let listening = doc.tryit.listening_open_key;
    let (bg, fg) = if listening {
        (ColorToken::Accent, ColorToken::Text1)
    } else {
        (super::face(doc, id, false), ColorToken::Text1)
    };
    fill_rounded_rect(scene, chip, KEY_CHIP_RADIUS, ink.color(bg));
    if listening {
        // À escuta, o anel tracejado do foco (o da caixa à escuta da vista A) respira pelo pulso do registo;
        // estático (inteiro) em Discreto e com reduced motion.
        let k = doc.pulse(id).map_or(1.0, |p| 0.5 + 0.5 * p);
        let ring = ink.color(ColorToken::FocusRing).multiply_alpha(k);
        let outer = Rect::new(
            chip.x - m::FIELD_BORDER,
            chip.y - m::FIELD_BORDER,
            chip.w + m::FIELD_BORDER * 2.0,
            chip.h + m::FIELD_BORDER * 2.0,
        );
        ph2d_editor_core::paint::stroke_rounded_rect_dashed(
            scene,
            outer,
            KEY_CHIP_RADIUS,
            m::FIELD_BORDER,
            m::LOOSE_DASH,
            ring,
        );
        doc.census.dashed.push(outer);
    }
    line(
        &mut doc.census,
        ts,
        scene,
        &key_text,
        Rect::new(chip.x + KEY_CHIP_SIDE, chip.y, tw, chip.h),
        m::VALUE_PX,
        ink.color(fg),
    );
    let named = tr("input_map_try.opens_with_named").replace("{key}", &key_text);
    target(doc, TryHit::OpenKey, chip, named, None);
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
    let id = TryHit::Forget.id();
    let off = (doc.tryit.player_changed == 0).then_some("input_map_try.forget_off");
    if off.is_none() && (doc.is_hovered(id) || doc.is_pressed(id)) {
        // A ligação sublinha sob o ponteiro (no quadro, D32): o fio no pé da letra.
        let base = y + (ROW + SUBTLE_PX) * 0.5;
        let rule = Rect::new(fx, base, fw, StrokeToken::Hairline.px());
        fill_rounded_rect(scene, rule, 0.0, link);
    }
    target(doc, TryHit::Forget, Rect::new(fx, y, fw, ROW), text, off);
}
