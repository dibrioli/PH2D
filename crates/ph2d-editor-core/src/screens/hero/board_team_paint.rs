//! O desenho dos comentários (módulo filho de [`super`]): as bolhas no quadro (K6/K7), o cartão do fio
//! aberto (K4/K5), a lista (K8/K10), e a frase dos pedidos servidos. Os painéis das versões e do
//! exportar estão em `board_team_panels.rs`; a apresentação em `board_team_show.rs`.

use super::super::super::HeroScreen;
use super::{Hit, TeamButton, ago, me, node, now_unix, register, text_y};
use crate::documents::team::{Confirm, OpenThread};
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_text_block, paint_text_centered, paint_text_elided, resolve,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, Message, ThreadId};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O lado de uma bolha e de um botão (K7: tamanho de INTERFACE, o de um botão da barra).
fn btn() -> f32 {
    super::super::facilitate::btn()
}

/// K7 — o selo da contagem: 0,47 do Ø da bolha (medido, `21016057338514`).
const BADGE: f32 = 0.47; // LITERAL-PX-OK: fracção medida da bolha

/// A largura do cartão do fio e dos painéis, em botões.
const CARD_BTNS: f32 = 10.0; // LITERAL-PX-OK: largura em botões (uma mensagem curta numa linha)

/// Os botões de uma mensagem (editar, apagar) e a fila do cabeçalho dela: ¾ de um botão.
const SMALL: f32 = 0.75; // LITERAL-PX-OK: fracção de um botão

/// O quadro activo, para ler.
fn board(hero: &HeroScreen) -> Option<&Board> {
    hero.documents.active_board()
}

/// ⭐ K7 — a bolha do fio cuja ponta (o canto de baixo à esquerda) está em `at` (px de ecrã).
#[must_use]
pub fn bubble_rect(at: [f32; 2]) -> Rect {
    let d = btn();
    Rect::new(at[0], at[1] - d, d, d)
}

/// Os fios à vista no quadro: os não resolvidos (e os resolvidos com «Show resolved») cujo alvo
/// existe (K3), com a ponta em px de ecrã.
fn visible(hero: &HeroScreen, area: Rect) -> Vec<(ThreadId, [f32; 2])> {
    let Some(b) = board(hero) else {
        return Vec::new();
    };
    let t = &hero.documents.live.team;
    if t.hide_comments {
        return Vec::new();
    }
    let a = super::super::super::board_view::area_of(area);
    b.team
        .threads
        .iter()
        .filter(|th| !th.resolved || t.show_resolved)
        .filter_map(|th| {
            let p = b.camera.to_screen(a, th.anchor(&b.doc)?);
            Some((th.id, [p[0] as f32, p[1] as f32]))
        })
        .filter(|(_, p)| area.contains(p[0], p[1]))
        .collect()
}

/// Um botão de ícone desenhado e registado depois (o fundo do cartão regista-se ANTES dele).
type Deferred = Vec<(NodeId, Rect)>;

#[allow(clippy::too_many_arguments)]
fn icon_button(
    scene: &mut VectorScene,
    store: &mut WidgetStore,
    later: &mut Deferred,
    id: NodeId,
    icon: IconId,
    r: Rect,
    on: bool,
    theme: Theme,
) {
    register(store, id);
    let mut sink = HitIndex::default();
    super::super::super::board_nav::button(scene, &mut sink, store, id, icon, r, on, theme);
    later.push((id, r));
}

/// ⭐ Pinta a equipa do quadro activo: as bolhas, o cartão do fio aberto, a lista, os painéis, e a
/// frase de um pedido servido.
pub fn paint(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    if hero.documents.live.team.show.is_some() {
        return;
    }
    paint_bubbles(hero, area, scene, ts);
    paint_card(hero, area, scene, ts);
    super::panels::paint(hero, area, scene, ts);
    paint_message(hero, area, scene, ts);
}

fn paint_bubbles(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let theme = hero.theme;
    let mine = me(hero);
    let d = btn();
    // O contorno da gota é um ANEL: pela porta do tema (o plano não o tem em repouso).
    let ring = crate::paint::fill_ring(
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Thin.px(),
        resolve(ColorToken::Border, theme),
    );
    for (id, at) in visible(hero, area) {
        let Some(th) = board(hero).and_then(|b| b.team.thread(id)) else {
            continue;
        };
        let (resolved, unread, count) = (th.resolved, th.unread_for(&mine), th.messages.len());
        let initial: String = th
            .messages
            .first()
            .and_then(|m| m.author.chars().next())
            .map(|c| c.to_uppercase().collect())
            .unwrap_or_default();
        let r = bubble_rect(at);
        // K6 — novo azul, lido branco, resolvido com o visto.
        let (bg, fg) = if resolved {
            (ColorToken::Bg2, ColorToken::Text2)
        } else if unread {
            (ColorToken::Accent, ColorToken::AccentFg)
        } else {
            (ColorToken::BgElev, ColorToken::Text1)
        };
        // A gota: um círculo e o quadrante de baixo à esquerda em ponta (a ponta está no alvo).
        let inset = ring.map_or(0.0, |(w, c)| {
            fill_rounded_rect(scene, r, d / 2.0, c);
            w
        });
        let inner = Rect::new(r.x + inset, r.y + inset, d - 2.0 * inset, d - 2.0 * inset);
        fill_rounded_rect(scene, inner, inner.w / 2.0, resolve(bg, theme));
        let tail = Rect::new(r.x, r.y + d / 2.0, d / 2.0, d / 2.0);
        fill_rounded_rect(scene, tail, 0.0, resolve(bg, theme));
        if resolved {
            let i = Spacing::Sm.px();
            let ir = Rect::new(r.x + i, r.y + i, d - 2.0 * i, d - 2.0 * i);
            let stroke = StrokeToken::Thick.px();
            crate::paint::paint_icon(scene, IconId::Check, ir, resolve(fg, theme), stroke);
        } else {
            let font = TypeToken::Sm.px();
            paint_text_centered(ts, scene, &initial, r, font, resolve(fg, theme));
        }
        if count > 1 {
            // K7 — o selo da contagem centrado no canto de cima à direita.
            let s = d * BADGE;
            let br = Rect::new(r.x + d - s / 2.0, r.y - s / 2.0, s, s);
            fill_rounded_rect(scene, br, s / 2.0, resolve(ColorToken::Accent, theme));
            let fg = resolve(ColorToken::AccentFg, theme);
            paint_text_centered(ts, scene, &count.to_string(), br, TypeToken::Xs.px(), fg);
        }
        let id = node(Hit::Bubble(id));
        register(&mut hero.store, id);
        hero.store.set_tooltip(id, tr("board.comment.open"));
        hero.hit_index.register(id, r);
    }
}

/// O alvo do cartão aberto em px de ecrã (a ponta da bolha), se há cartão.
fn card_anchor(hero: &HeroScreen, area: Rect) -> Option<(OpenThread, [f32; 2])> {
    let b = board(hero)?;
    let open = *hero.documents.live.team.open.get(&b.id)?;
    let world = match open {
        OpenThread::New(pin) => pin.anchor(&b.doc)?,
        OpenThread::Thread(t) => b.team.thread(t)?.anchor(&b.doc)?,
    };
    let a = super::super::super::board_view::area_of(area);
    let p = b.camera.to_screen(a, world);
    Some((open, [p[0] as f32, p[1] as f32]))
}

/// O texto de uma mensagem como o cartão o mostra.
fn shown_text(m: &Message) -> String {
    if m.edited {
        format!("{} {}", m.text, tr("board.comment.edited"))
    } else {
        m.text.clone()
    }
}

/// ⭐ K4/K5 — o cartão do fio aberto, à direita da bolha e SEMPRE dentro da área: um fio comprido
/// mostra as últimas mensagens que cabem e «N earlier» por cima delas; o alvo fora da vista esconde
/// o cartão com ele (volta quando o alvo volta).
#[allow(clippy::too_many_lines)]
fn paint_card(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let Some((open, at)) = card_anchor(hero, area) else {
        return;
    };
    if !area.contains(at[0], at[1]) {
        return;
    }
    let theme = hero.theme;
    let mine = me(hero);
    let now = now_unix();
    let confirm = hero.documents.live.team.confirm;
    let (s, pad, g, m) = (btn(), Spacing::Sm.px(), Spacing::Xs.px(), Spacing::Md.px());
    let (xs, sm) = (TypeToken::Xs.px(), TypeToken::Sm.px());
    let (w, small) = (s * CARD_BTNS, s * SMALL);
    let thread = match open {
        OpenThread::Thread(t) => board(hero).and_then(|b| b.team.thread(t).cloned()),
        OpenThread::New(_) => None,
    };
    if matches!(open, OpenThread::New(_)) {
        // O fio novo mostra onde vai ficar: a bolha vazia na ponta.
        let r = bubble_rect(at);
        let accent = resolve(ColorToken::Accent, theme);
        fill_rounded_rect(scene, r, s / 2.0, accent);
        let tail = Rect::new(r.x, r.y + s / 2.0, s / 2.0, s / 2.0);
        fill_rounded_rect(scene, tail, 0.0, accent);
    }
    // As alturas primeiro (a mesma passada que pinta, numa cena descartada): escolhe-se o que cabe.
    let msgs: &[Message] = thread.as_ref().map_or(&[], |t| &t.messages);
    let text1 = resolve(ColorToken::Text1, theme);
    let mut probe = VectorScene::new();
    let heights: Vec<f32> = msgs
        .iter()
        .map(|msg| {
            let body = paint_text_block(
                ts,
                &mut probe,
                &shown_text(msg),
                0.0,
                0.0,
                sm,
                w - 2.0 * pad,
                text1,
            );
            small + body + pad
        })
        .collect();
    let fixed = pad + s + g + s + pad;
    let max_h = area.h - 2.0 * m;
    let (mut first, mut used) = (msgs.len(), fixed);
    while first > 0 {
        let more = heights[first - 1] + if first > 1 { small } else { 0.0 };
        // A última mensagem mostra-se sempre (mesmo maior que a área).
        if used + more > max_h && first < msgs.len() {
            break;
        }
        used += heights[first - 1];
        first -= 1;
    }
    let h = used + if first > 0 { small } else { 0.0 };
    let x = (at[0] + s + g).min(area.x + area.w - w - m).max(area.x + m);
    let y = (at[1] - s).min(area.y + area.h - m - h).max(area.y + m);
    let card = Rect::new(x, y, w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::BgElev, theme));
    // O fundo toma o clique (registado ANTES dos botões, que ganham por cima dele).
    hero.hit_index
        .register(node(Hit::Button(TeamButton::CardSurface)), card);
    let mut later = Deferred::new();
    let mut cy = y + pad;
    // A fila de cima: resolver (só num fio que existe) e fechar.
    let close = Rect::new(x + w - pad - s, cy, s, s);
    icon_button(
        scene,
        &mut hero.store,
        &mut later,
        node(Hit::Button(TeamButton::CardClose)),
        IconId::Close,
        close,
        false,
        theme,
    );
    let mut title_end = close.x;
    let title = match &thread {
        Some(th) => {
            let res = Rect::new(close.x - s - g, cy, s, s);
            icon_button(
                scene,
                &mut hero.store,
                &mut later,
                node(Hit::Button(TeamButton::CardResolve)),
                IconId::Check,
                res,
                th.resolved,
                theme,
            );
            title_end = res.x;
            if th.resolved {
                tr("board.comment.resolved")
            } else {
                tr("board.comment.thread")
            }
        }
        None => tr("board.comment.new"),
    };
    let text2 = resolve(ColorToken::Text2, theme);
    paint_text_elided(
        ts,
        scene,
        title,
        x + pad,
        text_y(cy, s, sm),
        sm,
        title_end - g - x - pad,
        text2,
    );
    cy += s + g;
    if first > 0 {
        let n = first.to_string();
        let earlier = tr_with("board.comment.earlier", &[("n", &n)]);
        let text3 = resolve(ColorToken::Text3, theme);
        paint_text_elided(
            ts,
            scene,
            &earlier,
            x + pad,
            text_y(cy, small, xs),
            xs,
            w - 2.0 * pad,
            text3,
        );
        cy += small;
    }
    let tid = thread.as_ref().map(|t| t.id);
    for (n, msg) in msgs.iter().enumerate().skip(first) {
        let head = format!("{} · {}", msg.author, ago(msg.at, now));
        let own = msg.author == mine;
        let room = if own {
            w - 2.0 * pad - 2.0 * small
        } else {
            w - 2.0 * pad
        };
        paint_text_elided(
            ts,
            scene,
            &head,
            x + pad,
            text_y(cy, small, xs),
            xs,
            room,
            text2,
        );
        if let (true, Some(t)) = (own, tid) {
            let del = Rect::new(x + w - pad - small, cy, small, small);
            let edit = Rect::new(del.x - small, cy, small, small);
            let armed = confirm == Some(Confirm::Message(t, n));
            icon_button(
                scene,
                &mut hero.store,
                &mut later,
                node(Hit::Edit(n)),
                IconId::VectorPencil,
                edit,
                false,
                theme,
            );
            icon_button(
                scene,
                &mut hero.store,
                &mut later,
                node(Hit::Delete(n)),
                IconId::Trash,
                del,
                armed,
                theme,
            );
            let tip = if armed {
                "board.comment.confirm"
            } else {
                "board.comment.delete"
            };
            hero.store
                .set_tooltip(node(Hit::Edit(n)), tr("board.comment.edit"));
            hero.store.set_tooltip(node(Hit::Delete(n)), tr(tip));
        }
        cy += small;
        cy += paint_text_block(
            ts,
            scene,
            &shown_text(msg),
            x + pad,
            cy,
            sm,
            w - 2.0 * pad,
            text1,
        );
        cy += pad;
    }
    for (id, r) in later {
        hero.hit_index.register(id, r);
    }
    let field = Rect::new(x + pad, cy, w - 2.0 * pad, s);
    let fid = crate::ids::BOARD_COMMENT_INPUT;
    super::super::super::board_nav::paint_field(
        fid,
        field,
        scene,
        ts,
        theme,
        &mut hero.hit_index,
        &hero.store,
    );
    if super::super::super::board_nav::field_text(&hero.store, fid).is_empty() {
        let hint = if thread.is_some() {
            tr("board.comment.reply")
        } else {
            tr("board.comment.write")
        };
        let text3 = resolve(ColorToken::Text3, theme);
        paint_text_elided(
            ts,
            scene,
            hint,
            field.x + pad,
            text_y(field.y, s, sm),
            sm,
            field.w - 2.0 * pad,
            text3,
        );
    }
}

/// K4 — editar a minha mensagem `n`: o compositor com o texto dela.
pub(super) fn begin_edit(hero: &mut HeroScreen, n: usize) {
    let mine = me(hero);
    let Some(b) = board(hero) else {
        return;
    };
    let Some(OpenThread::Thread(t)) = hero.documents.live.team.open.get(&b.id).copied() else {
        return;
    };
    let Some(m) = b
        .team
        .thread(t)
        .and_then(|th| th.messages.get(n))
        .filter(|m| m.author == mine)
    else {
        return;
    };
    let text = m.text.clone();
    hero.documents.live.team.editing = Some((t, n));
    // Editar continua o texto (o cursor no fim, nada seleccionado): uma tecla nunca apaga a mensagem.
    let id = crate::ids::BOARD_COMMENT_INPUT;
    let caret = text.len();
    hero.store.register(
        id,
        crate::interaction::InteractiveState::TextInput {
            state: crate::widget::TextInputState::Focused,
            text,
            caret,
            selection_anchor: None,
        },
    );
    hero.store.set_focus(Some(id));
    hero.store.mark_cancel_on_escape(id);
}

/// K8 — clicar numa linha da lista enquadra o alvo do fio (com a vizinhança).
pub(super) fn go_to_thread(hero: &mut HeroScreen, t: ThreadId) {
    let area = super::super::super::board_view::area_of(hero.last_canvas);
    let Some(b) = hero.documents.active_board_mut() else {
        return;
    };
    let Some(p) = b.team.thread(t).and_then(|th| th.anchor(&b.doc)) else {
        return;
    };
    // Uma janela de um ecrã à volta do ponto, ao zoom de agora (só se desloca a vista).
    let (hw, hh) = (area[2] / 2.0 / b.camera.zoom, area[3] / 2.0 / b.camera.zoom);
    b.camera
        .fit(area, [p[0] - hw, p[1] - hh, p[0] + hw, p[1] + hh], 0.0);
}

/// ⭐ K8 — a lista dos comentários (o botão «Comments» do canto): o meu nome (K10), «Show resolved»,
/// e os fios do mais novo ao mais velho com a bolha de novo.
pub(super) fn paint_list(
    hero: &mut HeroScreen,
    at: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) -> f32 {
    let theme = hero.theme;
    let mine = me(hero);
    let (s, pad, g) = (btn(), Spacing::Sm.px(), ph2d_tokens::control_gap_px());
    let sm = TypeToken::Sm.px();
    let Some(b) = board(hero) else {
        return 0.0;
    };
    let show_resolved = hero.documents.live.team.show_resolved;
    let editing = hero.documents.live.team.editing_author;
    let mut rows: Vec<(ThreadId, String, bool, bool, usize)> = b
        .team
        .threads
        .iter()
        .filter(|t| !t.resolved || show_resolved)
        .map(|t| {
            let m = &t.messages[0];
            let line = format!("{}: {}", m.author, m.text.lines().next().unwrap_or(""));
            (
                t.id,
                line,
                t.unread_for(&mine),
                t.resolved,
                t.messages.len(),
            )
        })
        .collect();
    rows.reverse();
    let fit = super::panels::fit_rows(at.h);
    let (from, up, down) = super::panels::window(hero, rows.len(), fit);
    let shown = rows.len().saturating_sub(from).min(fit);
    let h = pad * 2.0 + (s + g) * (super::panels::HEAD_ROWS + shown.max(1)) as f32;
    let r = Rect::new(at.x, at.y, at.w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    hero.hit_index
        .register(node(Hit::Button(TeamButton::ListSurface)), r);
    let mut y = super::panels::header(
        hero,
        at,
        scene,
        ts,
        "board.comment.title",
        TeamButton::ListClose,
        (up, down),
    );
    // K10 — «Name: …» (clicar abre o campo).
    let name_r = Rect::new(at.x + pad, y, at.w - 2.0 * pad, s);
    if editing {
        super::super::super::board_nav::paint_field(
            crate::ids::BOARD_AUTHOR_INPUT,
            name_r,
            scene,
            ts,
            theme,
            &mut hero.hit_index,
            &hero.store,
        );
    } else {
        let label = tr_with("board.comment.name", &[("name", &mine)]);
        text_button(
            scene,
            ts,
            &mut hero.hit_index,
            &hero.store,
            node(Hit::Button(TeamButton::ListAuthor)),
            &label,
            name_r,
            false,
            theme,
        );
    }
    y += s + g;
    let tog = Rect::new(at.x + pad, y, at.w - 2.0 * pad, s);
    text_button(
        scene,
        ts,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ListShowResolved)),
        tr("board.comment.show_resolved"),
        tog,
        show_resolved,
        theme,
    );
    y += s + g;
    if rows.is_empty() {
        paint_text_elided(
            ts,
            scene,
            tr("board.comment.none"),
            at.x + pad,
            text_y(y, s, sm),
            sm,
            at.w - 2.0 * pad,
            resolve(ColorToken::Text3, theme),
        );
    }
    for (id, line, unread, resolved, n) in rows.into_iter().skip(from).take(shown) {
        let rr = Rect::new(at.x + pad, y, at.w - 2.0 * pad, s);
        let nid = node(Hit::Row(id));
        register(&mut hero.store, nid);
        let dot = s * 0.3; // LITERAL-PX-OK: o ponto de «novo», 0,3 de uma linha
        let mark = if resolved {
            ColorToken::Text3
        } else if unread {
            ColorToken::Accent
        } else {
            ColorToken::Bg3
        };
        fill_rounded_rect(
            scene,
            Rect::new(rr.x, rr.y + (s - dot) / 2.0, dot, dot),
            dot / 2.0,
            resolve(mark, theme),
        );
        let count = if n > 1 {
            format!(" ({n})")
        } else {
            String::new()
        };
        paint_text_elided(
            ts,
            scene,
            &format!("{line}{count}"),
            rr.x + dot + g,
            text_y(rr.y, s, sm),
            sm,
            rr.w - dot - g,
            resolve(ColorToken::Text1, theme),
        );
        hero.hit_index.register(nid, rr);
        y += s + g;
    }
    h
}

/// Um botão de texto (o idioma do painel da votação).
#[allow(clippy::too_many_arguments)]
pub(super) fn text_button(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    hit: &mut HitIndex,
    store: &WidgetStore,
    id: NodeId,
    label: &str,
    r: Rect,
    on: bool,
    theme: Theme,
) {
    let hover = matches!(
        store.button_state(id),
        Some(
            crate::widget::ButtonState::Hovered
                | crate::widget::ButtonState::Pressed
                | crate::widget::ButtonState::Focused
        )
    );
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let bg = if on {
        ColorToken::Accent
    } else if hover {
        ColorToken::Bg3
    } else {
        ColorToken::Bg2
    };
    fill_rounded_rect(scene, r, radius, resolve(bg, theme));
    let fg = resolve(
        if on {
            ColorToken::AccentFg
        } else {
            ColorToken::Text1
        },
        theme,
    );
    paint_text_centered(ts, scene, label, r, TypeToken::Sm.px(), fg);
    hit.register(id, r);
}

/// A frase de um pedido servido (`Exported to …`), no topo do quadro enquanto é fresca.
fn paint_message(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let Some((text, at)) = hero.documents.live.team.message.clone() else {
        return;
    };
    if at.elapsed().as_secs_f32() >= crate::toast::Toast::DEFAULT_TTL_S {
        hero.documents.live.team.message = None;
        return;
    }
    let theme = hero.theme;
    let font = TypeToken::Sm.px();
    let w = (ts.prefix_width(&text, font) + 2.0 * Spacing::Lg.px()).min(area.w);
    let h = Spacing::Xl2.px();
    let y = area.y + Spacing::Md.px() + h + Spacing::Xs.px();
    let r = Rect::new(area.x + (area.w - w) / 2.0, y, w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    paint_text_centered(ts, scene, &text, r, font, resolve(ColorToken::Text1, theme));
}
