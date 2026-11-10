//! **C · Try it** (plano `docs/InputMap/01_plano.md` §3, maquete `04_C_bancada.png`): os cartões Start from ·
//! Try it · The stick · Controls — duas colunas na janela larga, uma coluna que rola na estreita.

mod cards;
mod controls;
pub mod state;
pub mod stick;

pub use cards::{METER_TRACK, meter_fill, recipe_cols, tile_w};
pub use controls::{KEY_CHIP_H, SHOWN_GAP, preview_frame_h};

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::doc_surface::KeyInput;
use ph2d_editor_core::paint::fill_rounded_rect;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, GamepadState, InputAction, InputState, Key};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::doc::InputMapDoc;
use crate::edit::InputMapEdit;
use crate::interact::{Control, ControlRole};
// O corpo da C no peso Regular (como a A e a B); só os destaques e os nomes de acção vão em SemiBold.
use crate::view_list::text::body as line;
use state::TryHit;

/// A largura mínima de uma coluna para a vista ir a duas.
const TWO_COLUMNS_MIN: f32 = m::FIELD_H * 20.0;

/// O cru de `a` só no comando (meia manche, gatilho, botão) — o MESMO que o resolvedor lê.
#[must_use]
pub fn pad_raw(pad: &GamepadState, a: &InputAction) -> f32 {
    let mut dev = InputState::default();
    dev.gamepad.clone_from(pad);
    let only_pad = InputAction {
        bindings: a
            .bindings
            .iter()
            .copied()
            .filter(|b| !matches!(b, Binding::Key(_)))
            .collect(),
        dead_zone: 0.0,
        press_point: 1.0,
        ..a.clone()
    };
    let mut map = ph2d_input::InputMap::new();
    map.insert(only_pad);
    let mut st = ph2d_input::ActionState::new();
    st.tick(&map, &dev);
    st.sample(a.id).strength
}

/// O quadro do bench: parado = teclas da aba + comando ao vivo; a correr = o estado do jogo, sem tomar teclas.
pub(crate) fn tick(doc: &mut InputMapDoc, live: &InputState) {
    let idle = doc.view() != crate::doc::View::TryIt || doc.snap.play_running;
    let t = &mut doc.tryit;
    t.live_pad.clone_from(&live.gamepad);
    if idle {
        t.bench.keyboard.release_all();
    }
    t.bench.gamepad.clone_from(&live.gamepad);
    let dev = if doc.snap.play_running {
        live
    } else {
        &t.bench
    };
    t.actions.tick(doc.snap.read_map(), dev);
}

/// Lê o ficheiro do jogador (ao abrir a vista, ao trocar de jogo, depois do Forget) — nunca a cada quadro.
pub(crate) fn reread_player_file(doc: &mut InputMapDoc) {
    if !doc.tryit.reread || doc.view() != crate::doc::View::TryIt {
        return;
    }
    doc.tryit.reread = false;
    doc.tryit.player_changed = match &doc.tryit.player_override {
        Some(keys) => keys.changed(&doc.snap.map),
        None => doc.player_file().map_or(0, |p| {
            ph2d_player_controls::file::load(&p).changed(&doc.snap.map)
        }),
    };
}

/// A tecla física: à escuta da tecla que abre a tela, ela; senão, com o jogo parado, o bench (desce E sobe).
pub(crate) fn raw_key(doc: &mut InputMapDoc, key: Key, pressed: bool) -> bool {
    if doc.snap.play_running || doc.store.focus_id().is_some() || doc.list.typing.is_some() {
        return false;
    }
    let kb = &mut doc.tryit.bench.keyboard;
    if pressed {
        kb.handle_key_down(key);
    } else {
        kb.handle_key_up(key);
    }
    // Ctrl/Alt, os acordes e o F5 passam: o Ctrl+Z desfaz e o F5 dá Play mesmo com o Try it aberto.
    !crate::raw_keys::never_binds(key, kb.iter_held())
}

/// **Antes do foco da aba** (o `doc.rs` chama-a primeiro): à escuta da tecla que abre a tela, TODA tecla é dela
/// (o Tab também; o Esc cancela a escuta; o F5 fica recusado com o porquê — a lei de `try_it.rs` — e a escuta continua); com a pergunta da receita aberta, o diálogo. `true` = tomada.
pub(crate) fn before_focus(doc: &mut InputMapDoc, key: Key, pressed: bool) -> bool {
    if doc.view() != crate::doc::View::TryIt || doc.snap.play_running {
        return false;
    }
    if doc.tryit.listening_open_key {
        if pressed {
            take_open_key(doc, key);
        }
        return true;
    }
    modal_key(doc, key, pressed)
}

/// A mesma porta pelo caminho do texto (a escuta já tomou a tecla física; aqui só não deixa o foco mexer).
pub(crate) fn before_focus_text(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    if doc.view() != crate::doc::View::TryIt || doc.snap.play_running {
        return false;
    }
    doc.tryit.listening_open_key || modal_text_key(doc, input)
}

fn take_open_key(doc: &mut InputMapDoc, key: Key) {
    let esc = Key::named_in_script("escape").unwrap_or_default();
    let f5 = Key::named_in_script("f5").unwrap_or_default();
    if esc.contains(&key) {
        // Esc cancela a escuta, como em toda a aba («Esc cancels»): a tecla fica a que era.
        doc.tryit.refused = None;
        doc.tryit.listening_open_key = false;
        return;
    }
    if f5.contains(&key) {
        doc.tryit.refused = Some(key);
        return;
    }
    doc.tryit.refused = None;
    doc.tryit.listening_open_key = false;
    doc.pending.push(InputMapEdit::SetOpenKey(key));
}

/// A porta do texto: os números Dead/Press abertos (os da vista A).
pub(crate) fn key(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    crate::view_list::key(doc, input)
}

/// O polegar agarrado segue o ponteiro (a conversão da barra do cartão, como na A).
pub(crate) fn drag_thumb(doc: &mut InputMapDoc, y: f32) -> bool {
    let (Some((y0, s0)), Some((track_h, total, view_h))) = (doc.tryit.thumb_drag, doc.tryit.bar)
    else {
        return false;
    };
    let d = ph2d_editor_core::widget::scrollbar_delta_for_drag(y - y0, track_h, total, view_h);
    let before = doc.tryit.scroll;
    doc.tryit.reveal = None;
    doc.tryit.scroll = (s0 + d).clamp(0.0, (total - view_h).max(0.0));
    (doc.tryit.scroll - before).abs() > f32::EPSILON
}

pub(crate) fn wheel(doc: &mut InputMapDoc, dy: f32) -> bool {
    let before = doc.tryit.scroll;
    doc.tryit.reveal = None;
    doc.tryit.scroll = (doc.tryit.scroll - dy).max(0.0);
    (doc.tryit.scroll - before).abs() > f32::EPSILON
}

/// Carregar em `(x, y)` dentro do cartão da vista.
pub(crate) fn press(doc: &mut InputMapDoc, x: f32, y: f32) -> bool {
    crate::view_list::commit_typing(doc);
    let Some(id) = doc.hits.hit(x, y) else {
        doc.tryit.listening_open_key = false;
        return true;
    };
    if id == THUMB {
        doc.tryit.thumb_drag = Some((y, doc.tryit.scroll));
        return true;
    }
    if let Some(a) = stick::stick_action(doc) {
        for (h, zone) in [
            (
                crate::ids::ListHit::Dead(a),
                crate::view_list::state::Zone::Dead,
            ),
            (
                crate::ids::ListHit::Press(a),
                crate::view_list::state::Zone::Press,
            ),
        ] {
            if h.id() == id {
                let w = doc.hits.rect_for(id).map_or(1.0, |r| r.w);
                crate::view_keyboard::input::scrub(doc, Some(a), zone, x, w, true);
                return true;
            }
        }
    }
    let map = &doc.snap.map;
    match doc.tryit.hit_of(id) {
        Some(TryHit::Recipe(r)) if map.is_empty() => doc.pending.push(InputMapEdit::StartFrom(r)),
        Some(TryHit::Recipe(r)) => {
            doc.tryit.confirm = Some(r);
            // O foco nasce no Cancel (o seguro), como no diálogo do SO; o Enter continua a ser o Replace.
            doc.store.set_focus(None);
            doc.ui.focus = Some(TryHit::Cancel.id());
        }
        Some(TryHit::Replace) => {
            if let Some(r) = doc.tryit.confirm.take() {
                doc.pending.push(InputMapEdit::StartFrom(r));
            }
            doc.ui.focus = None;
        }
        Some(TryHit::Cancel) => close_confirm(doc),
        Some(TryHit::PlayersCanChange) => {
            let on = map.controls().players_can_change_keys;
            doc.pending.push(InputMapEdit::SetPlayersCanChange(!on));
        }
        Some(TryHit::OpenKey) => {
            doc.tryit.listening_open_key = !doc.tryit.listening_open_key;
            doc.tryit.refused = None;
        }
        Some(TryHit::Shown(a)) => {
            let on = map.controls().shown_in_game(a);
            doc.pending.push(InputMapEdit::SetShown(a, !on));
        }
        Some(TryHit::Forget) => {
            doc.pending.push(InputMapEdit::ForgetPlayerKeys);
            doc.tryit.forget_session = true;
            doc.tryit.player_override = None;
            doc.tryit.reread = true;
        }
        None => {}
    }
    true
}

/// A dica (Plain English) de um alvo da vista C; os números Dead/Press usam a da vista A.
#[must_use]
pub fn tip_of(doc: &InputMapDoc, id: ph2d_a11y::NodeId) -> Option<&'static str> {
    let Some(hit) = doc.tryit.hit_of(id) else {
        return crate::view_list::tip_of(doc, id);
    };
    let key = match hit {
        TryHit::Recipe(_) => "input_map_try.recipe_tip",
        TryHit::Replace => "input_map_try.replace_tip",
        TryHit::Cancel => "input_map_try.cancel_tip",
        TryHit::PlayersCanChange => "input_map_try.players_can_change_tip",
        TryHit::OpenKey => "input_map_try.opens_with_tip",
        TryHit::Shown(_) => "input_map_try.shown_tip",
        TryHit::Forget => "input_map_try.forget_tip",
    };
    Some(ph2d_i18n::tr(key))
}

/// O papel de um alvo da vista C no registo de controlos.
const fn role_of(h: TryHit) -> ControlRole {
    match h {
        TryHit::Recipe(_) | TryHit::Replace | TryHit::Cancel | TryHit::OpenKey => {
            ControlRole::Button
        }
        TryHit::PlayersCanChange | TryHit::Shown(_) => ControlRole::Check,
        TryHit::Forget => ControlRole::Link,
    }
}

/// Regista um alvo da vista C pintado à mão: o alvo de clique E o controlo (premido, foco, a11y, cursor).
pub(crate) fn target(
    doc: &mut InputMapDoc,
    h: TryHit,
    r: Rect,
    label: impl Into<String>,
    off: Option<&'static str>,
) {
    doc.hits.register(h.id(), r);
    doc.tryit.targets.push(h);
    doc.control(Control::new(h.id(), r, role_of(h), label).disabled(off));
}

/// Regista como controlo um alvo que o `CardCtx` acabou de pintar (o rect é o que ele registou); premido, o anel.
pub(crate) fn card_target(
    doc: &mut InputMapDoc,
    h: TryHit,
    label: impl Into<String>,
    scene: &mut VectorScene,
    ink: CardInk,
) {
    doc.tryit.targets.push(h);
    if let Some(r) = doc.hits.rect_for(h.id()) {
        if doc.is_pressed(h.id()) {
            crate::view_list::ring(scene, r, ink);
        }
        doc.control(Control::new(h.id(), r, role_of(h), label));
    }
}

/// O fundo de um alvo pintado à mão: premido = `AccentSoft`, pairado (instantâneo, D32) ou aceso = `CardFieldHover`.
pub(crate) fn face(doc: &InputMapDoc, id: ph2d_a11y::NodeId, on: bool) -> ColorToken {
    if doc.is_pressed(id) {
        ColorToken::AccentSoft
    } else if on || doc.is_hovered(id) {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    }
}

/// O polegar da barra da vista C.
pub const THUMB: ph2d_a11y::NodeId =
    ph2d_editor_core::registry::hash_node_id("input_map_try.thumb");

/// A tecla do diálogo da receita, ANTES do foco: Esc = Cancel, Enter = Replace (o botão de omissão, como no
/// diálogo do SO), Tab/Shift+Tab giram só entre Replace e Cancel (o foco preso); o Space continua a activar o
/// focado (o Cancel, onde o foco nasce). `true` = tomada.
pub(crate) fn modal_key(doc: &mut InputMapDoc, key: Key, pressed: bool) -> bool {
    if doc.view() != crate::doc::View::TryIt || doc.tryit.confirm.is_none() {
        return false;
    }
    let tab = Key::named_in_script("tab").unwrap_or_default();
    if tab.contains(&key) {
        if pressed {
            let back = doc.ui.shift;
            doc.trap_focus(&[TryHit::Replace.id(), TryHit::Cancel.id()], back);
        }
        return true;
    }
    if !pressed {
        return false;
    }
    let esc = Key::named_in_script("escape").unwrap_or_default();
    if esc.contains(&key) {
        close_confirm(doc);
        return true;
    }
    if key.is_enter_like() {
        if let Some(r) = doc.tryit.confirm.take() {
            doc.pending.push(InputMapEdit::StartFrom(r));
        }
        doc.ui.focus = None;
        return true;
    }
    false
}

/// A porta do texto do mesmo diálogo (Esc / Enter pelo caminho do `key`).
pub(crate) fn modal_text_key(doc: &mut InputMapDoc, input: &KeyInput<'_>) -> bool {
    use ph2d_editor_core::doc_surface::Key as K;
    if input.key == K::Tab && doc.tryit.confirm.is_some() {
        if input.pressed {
            let ids = [TryHit::Replace.id(), TryHit::Cancel.id()];
            doc.trap_focus(&ids, input.mods.shift);
        }
        return true;
    }
    let raw = match input.key {
        K::Escape => Key::named_in_script("escape"),
        K::Enter => Key::named_in_script("enter"),
        _ => None,
    };
    raw.and_then(|v| v.first().copied())
        .is_some_and(|k| modal_key(doc, k, input.pressed))
}

/// Fecha a confirmação; o foco que estava nos botões dela sai com ela.
fn close_confirm(doc: &mut InputMapDoc) {
    doc.tryit.confirm = None;
    let ids = [TryHit::Replace.id(), TryHit::Cancel.id()];
    if doc.ui.focus.is_some_and(|f| ids.contains(&f)) {
        doc.ui.focus = None;
    }
}

/// Quebra `s` em linhas que cabem em `w` (palavra a palavra).
pub(crate) fn wrap(ts: &mut TextSystem, s: &str, px: f32, w: f32) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        let cand = if cur.is_empty() {
            word.to_owned()
        } else {
            format!("{cur} {word}")
        };
        if ts.prefix_width(&cand, px) > w && !cur.is_empty() {
            out.push(std::mem::replace(&mut cur, word.to_owned()));
        } else {
            cur = cand;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// A altura de uma linha de texto de `px`.
pub(crate) fn line_h(px: f32) -> f32 {
    px + m::LABEL_BOTTOM * 2.0
}

/// Pinta `s` quebrado em `w`, a partir de `y`; devolve o fundo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn para(
    doc: &mut InputMapDoc,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    (x, y, w): (f32, f32, f32),
    px: f32,
    color: Color,
) -> f32 {
    let mut y = y;
    for l in wrap(ts, s, px, w) {
        line(
            &mut doc.census,
            ts,
            scene,
            &l,
            Rect::new(x, y, w, line_h(px)),
            px,
            color,
        );
        y += line_h(px);
    }
    y
}

/// A altura de `s` quebrado em `w`.
pub(crate) fn para_h(ts: &mut TextSystem, s: &str, px: f32, w: f32) -> f32 {
    wrap(ts, s, px, w).len() as f32 * line_h(px)
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
    doc.tryit.targets.clear();
    doc.tryit.theme = theme;
    doc.kb.targets.clear();
    doc.kb.bars.clear();
    let inner = Rect::new(
        body.x + m::CARD_PAD_X,
        body.y + m::CARD_PAD_TOP,
        (body.w - m::CARD_PAD_X * 2.0).max(0.0),
        (body.h - m::CARD_PAD_TOP - m::CARD_PAD_BOTTOM).max(0.0),
    );
    let two = inner.w >= TWO_COLUMNS_MIN * 2.0 + m::CARD_GAP;
    let col_w = if two {
        (inner.w - m::CARD_GAP) * 0.5
    } else {
        inner.w
    };
    type Card = fn(&mut InputMapDoc, &mut TextSystem, f32) -> f32;
    type Paint = fn(&mut InputMapDoc, Rect, &mut VectorScene, &mut TextSystem, CardInk);
    let order: [(Card, Paint); 4] = [
        (cards::recipes_h, cards::recipes),
        (cards::live_h, cards::live),
        (stick::stick_h, stick::stick),
        (controls::controls_h, controls::controls),
    ];
    let col_of = |i: usize| usize::from(two && i >= 2);
    let mut heights: Vec<f32> = Vec::with_capacity(order.len());
    for (i, (h, _)) in order.iter().enumerate() {
        let above: f32 = (0..i)
            .filter(|&j| col_of(j) == col_of(i))
            .map(|j| heights[j] + m::CARD_GAP)
            .sum();
        // Duas colunas: a tela do jogo só cresce até ao que sobra na coluna. Uma coluna: a página já rola, e
        // a tela cresce com as linhas (com o tecto, as 11 acções liam-se a 0,47 na aba de 1600, foto c1_1600).
        doc.tryit.controls_room = if two {
            (inner.h - above).max(0.0)
        } else {
            f32::INFINITY
        };
        heights.push(h(doc, ts, col_w));
    }
    let mut tops = [inner.y, inner.y];
    let mut bottom = inner.y;
    for i in 0..order.len() {
        tops[col_of(i)] += heights[i] + m::CARD_GAP;
        bottom = bottom.max(tops[col_of(i)]);
    }
    let overflow = (bottom - m::CARD_GAP - (inner.y + inner.h)).max(0.0);
    if let Some(i) = doc.tryit.reveal.map(|c| c as usize) {
        // O cartão inteiro à vista: o fundo dele no fundo da vista, sem passar do topo.
        let top: f32 = (0..i)
            .filter(|&j| col_of(j) == col_of(i))
            .map(|j| heights[j] + m::CARD_GAP)
            .sum();
        doc.tryit.scroll = (top + heights[i] - inner.h).clamp(0.0, top);
    }
    doc.tryit.scroll = doc.tryit.scroll.min(overflow);
    let mut ys = [inner.y - doc.tryit.scroll; 2];
    // A vista rola: os cartões cortam-se na linha do enchimento (a 1280×720 o último descia ao HUD, foto A1).
    let clip = Rect::new(body.x, inner.y, body.w, inner.h);
    scene.push_clip(&ph2d_editor_core::paint::rect_to_vello(clip));
    doc.hits.push_clip(clip);
    doc.census.clip = Some(clip);
    for (i, (_, p)) in order.iter().enumerate() {
        let c = col_of(i);
        let r = Rect::new(
            inner.x + (col_w + m::CARD_GAP) * c as f32,
            ys[c],
            col_w,
            heights[i],
        );
        fill_rounded_rect(scene, r, m::CARD_RADIUS, ink.color(ColorToken::KanbanList));
        p(doc, r, scene, ts, ink);
        ys[c] += heights[i] + m::CARD_GAP;
    }
    doc.hits.pop_clip();
    scene.pop_layer();
    scrollbar(doc, clip, bottom - m::CARD_GAP - inner.y, scene, ink);
}

/// A barra discreta do cartão (a da vista A): só o polegar fino, sem trilho, na margem direita da vista.
fn scrollbar(doc: &mut InputMapDoc, view: Rect, total: f32, scene: &mut VectorScene, ink: CardInk) {
    use ph2d_editor_core::widget::{
        scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
    };
    doc.tryit.bar = None;
    if !scrollbar_is_needed(total, view.h) {
        return;
    }
    let lane = Rect::new(view.x, view.y, view.w - m::SEG_INSET, view.h);
    let track = scrollbar_track_rect(lane);
    doc.tryit.bar = Some((track.h, total, view.h));
    let thumb = scrollbar_thumb_rect(track, doc.tryit.scroll, total, view.h);
    let pill = Rect::new(
        thumb.x + thumb.w - m::BAR_TRACK,
        thumb.y,
        m::BAR_TRACK,
        thumb.h,
    );
    fill_rounded_rect(
        scene,
        pill,
        m::BAR_TRACK * 0.5,
        ink.color(ColorToken::BorderEmph),
    );
    // Quente, o polegar acende por cima no tom do texto secundário, pelo desvanecer do registo (`Role::Fade`).
    let fade = doc.thumb_fade(THUMB);
    if fade > 0.0 {
        let hot = ink.color(ColorToken::Text2).multiply_alpha(fade);
        fill_rounded_rect(scene, pill, m::BAR_TRACK * 0.5, hot);
    }
    let label = ph2d_i18n::tr("input_map_try.scrollbar");
    doc.control(Control::new(THUMB, thumb, ControlRole::ScrollThumb, label));
}

impl InputMapDoc {
    /// ⭐ O «Forget them» com o Play a correr chega à SESSÃO: o mapa que o jogo lê volta ao do projecto e a próxima
    /// troca do jogador já não regrava as esquecidas. A shell chama-a depois do `frame` da família (ela só vê a
    /// sessão por `&`). A tela do jogo aberta continua aberta e o relógio parado. Fora do Play, nada: o ficheiro
    /// já saiu.
    pub fn forget_reaches(
        &mut self,
        session: &mut ph2d_player_controls::PlaySession,
        project: &ph2d_input::InputMap,
    ) {
        if std::mem::take(&mut self.tryit.forget_session) {
            session.forget(project);
        }
    }
}
