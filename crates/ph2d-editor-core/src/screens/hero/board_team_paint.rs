//! O desenho dos comentários (módulo filho de [`super`]): as bolhas no quadro (K6/K7), o cartão do fio
//! aberto (K4/K5), a lista (K8/K10), e a frase dos pedidos servidos. Os painéis das versões e do
//! exportar estão em `board_team_panels.rs`; a apresentação em `board_team_show.rs`.

use std::time::Instant;

use super::super::super::HeroScreen;
use super::{Hit, TeamButton, ago, me, node, now_unix, register};
use crate::documents::team::OpenThread;
use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{
    fill_rounded_rect, paint_text_block, paint_text_centered, paint_text_elided, resolve,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_model::{Board, Thread, ThreadId};
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O lado de uma bolha e de um botão (K7: tamanho de INTERFACE, o de um botão da barra).
fn btn() -> f32 {
    super::super::facilitate::btn()
}

/// K7 — o selo da contagem: 0,47 do Ø da bolha (medido, `21016057338514`).
const BADGE: f32 = 0.47; // LITERAL-PX-OK: fracção medida da bolha

/// A largura do cartão do fio e dos painéis, em botões.
const CARD_BTNS: f32 = 10.0; // LITERAL-PX-OK: largura em botões (uma mensagem curta numa linha)

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
    let list = visible(hero, area);
    let Some(b) = hero.documents.active_board() else {
        return;
    };
    let threads: Vec<Thread> = list
        .iter()
        .filter_map(|(id, _)| b.team.thread(*id).cloned())
        .collect();
    let d = btn();
    for (th, (_, at)) in threads.iter().zip(&list) {
        let r = bubble_rect(*at);
        let unread = th.unread_for(&mine);
        // K6 — novo azul, lido branco, resolvido com o visto.
        let (bg, fg) = if th.resolved {
            (ColorToken::Bg2, ColorToken::Text2)
        } else if unread {
            (ColorToken::Accent, ColorToken::AccentFg)
        } else {
            (ColorToken::BgElev, ColorToken::Text1)
        };
        // A gota: um círculo e o quadrante de baixo à esquerda em ponta (a ponta está no alvo).
        fill_rounded_rect(scene, r, d / 2.0, resolve(ColorToken::Border, theme));
        let inner = Rect::new(r.x + 1.0, r.y + 1.0, d - 2.0, d - 2.0); // LITERAL-PX-OK: o fio do contorno
        fill_rounded_rect(scene, inner, (d - 2.0) / 2.0, resolve(bg, theme)); // LITERAL-PX-OK: idem
        let tail = Rect::new(r.x, r.y + d / 2.0, d / 2.0, d / 2.0);
        fill_rounded_rect(scene, tail, 0.0, resolve(bg, theme));
        if th.resolved {
            let i = Rect::new(r.x + d * 0.25, r.y + d * 0.25, d * 0.5, d * 0.5);
            crate::paint::paint_icon(scene, IconId::Check, i, resolve(fg, theme), 2.0); // LITERAL-PX-OK: traço do ícone
        } else {
            let first = th.messages.first().map_or("?", |m| m.author.as_str());
            let initial: String = first
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect())
                .unwrap_or_default();
            paint_text_centered(
                ts,
                scene,
                &initial,
                r,
                TypeToken::Sm.px(),
                resolve(fg, theme),
            );
        }
        if th.messages.len() > 1 {
            let s = d * BADGE;
            let br = Rect::new(r.x + d - s * 0.6, r.y - s * 0.4, s, s);
            fill_rounded_rect(scene, br, s / 2.0, resolve(ColorToken::Accent, theme));
            let n = th.messages.len().to_string();
            paint_text_centered(
                ts,
                scene,
                &n,
                br,
                TypeToken::Xs.px(),
                resolve(ColorToken::AccentFg, theme),
            );
        }
        let id = node(Hit::Bubble(th.id));
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

/// ⭐ K4/K5 — o cartão do fio aberto, à direita da bolha (dentro da área).
fn paint_card(hero: &mut HeroScreen, area: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let Some((open, at)) = card_anchor(hero, area) else {
        return;
    };
    let theme = hero.theme;
    let mine = me(hero);
    let now = now_unix();
    let (s, pad) = (btn(), Spacing::Sm.px());
    let w = s * CARD_BTNS;
    let thread = match open {
        OpenThread::Thread(t) => board(hero).and_then(|b| b.team.thread(t).cloned()),
        OpenThread::New(_) => None,
    };
    if matches!(open, OpenThread::New(_)) {
        // O fio novo mostra onde vai ficar: a bolha vazia na ponta.
        let r = bubble_rect(at);
        fill_rounded_rect(scene, r, s / 2.0, resolve(ColorToken::Accent, theme));
        fill_rounded_rect(
            scene,
            Rect::new(r.x, r.y + s / 2.0, s / 2.0, s / 2.0),
            0.0,
            resolve(ColorToken::Accent, theme),
        );
    }
    let x = (at[0] + s + Spacing::Xs.px()).min(area.x + area.w - w - Spacing::Md.px());
    let y = (at[1] - s).clamp(
        area.y + Spacing::Md.px(),
        (area.y + area.h - s * 4.0).max(area.y),
    ); // CLAMP-OK: o cartão cabe na área (o máximo ≥ o mínimo pelo .max)
    // O conteúdo numa cena à parte: a altura só se sabe depois de quebrar as linhas.
    let mut body = VectorScene::new();
    let mut later = Deferred::new();
    let mut cy = y + pad;
    // A fila de cima: resolver (só num fio que existe) e fechar.
    let close = Rect::new(x + w - pad - s, cy, s, s);
    icon_button(
        &mut body,
        &mut hero.store,
        &mut later,
        node(Hit::Button(TeamButton::CardClose)),
        IconId::Close,
        close,
        false,
        theme,
    );
    if let Some(th) = &thread {
        let res = Rect::new(close.x - s - Spacing::Xs.px(), cy, s, s);
        icon_button(
            &mut body,
            &mut hero.store,
            &mut later,
            node(Hit::Button(TeamButton::CardResolve)),
            IconId::Check,
            res,
            th.resolved,
            theme,
        );
        let title = if th.resolved {
            tr("board.comment.resolved")
        } else {
            tr("board.comment.thread")
        };
        paint_text_elided(
            ts,
            &mut body,
            title,
            x + pad,
            cy + s * 0.25,
            TypeToken::Sm.px(),
            res.x - x - 2.0 * pad,
            resolve(ColorToken::Text2, theme),
        );
    } else {
        let t = tr("board.comment.new");
        paint_text_elided(
            ts,
            &mut body,
            t,
            x + pad,
            cy + s * 0.25,
            TypeToken::Sm.px(),
            w - 3.0 * pad - s,
            resolve(ColorToken::Text2, theme),
        );
    }
    cy += s + Spacing::Xs.px();
    for (n, m) in thread.iter().flat_map(|t| t.messages.iter().enumerate()) {
        let head = format!("{} · {}", m.author, ago(m.at, now));
        let own = m.author == mine;
        let room = if own {
            w - 2.0 * pad - 2.0 * s
        } else {
            w - 2.0 * pad
        };
        paint_text_elided(
            ts,
            &mut body,
            &head,
            x + pad,
            cy,
            TypeToken::Xs.px(),
            room,
            resolve(ColorToken::Text2, theme),
        );
        if own {
            let small = s * 0.75; // LITERAL-PX-OK: os botões de uma mensagem, ¾ de um botão
            let del = Rect::new(x + w - pad - small, cy - small * 0.2, small, small);
            let edit = Rect::new(del.x - small, del.y, small, small);
            icon_button(
                &mut body,
                &mut hero.store,
                &mut later,
                node(Hit::Edit(n)),
                IconId::VectorPencil,
                edit,
                false,
                theme,
            );
            icon_button(
                &mut body,
                &mut hero.store,
                &mut later,
                node(Hit::Delete(n)),
                IconId::Trash,
                del,
                false,
                theme,
            );
            hero.store
                .set_tooltip(node(Hit::Edit(n)), tr("board.comment.edit"));
            hero.store
                .set_tooltip(node(Hit::Delete(n)), tr("board.comment.delete"));
        }
        cy += TypeToken::Xs.px() * 1.6; // LITERAL-PX-OK: a altura da linha do cabeçalho (1,6 em)
        let mut text = m.text.clone();
        if m.edited {
            text.push_str(&format!(" {}", tr("board.comment.edited")));
        }
        cy += paint_text_block(
            ts,
            &mut body,
            &text,
            x + pad,
            cy,
            TypeToken::Sm.px(),
            w - 2.0 * pad,
            resolve(ColorToken::Text1, theme),
        );
        cy += Spacing::Sm.px();
    }
    let field = Rect::new(x + pad, cy, w - 2.0 * pad, s);
    let h = field.y + field.h + pad - y;
    let card = Rect::new(x, y, w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::BgElev, theme));
    // O fundo toma o clique (registado ANTES dos botões, que ganham por cima dele).
    hero.hit_index
        .register(node(Hit::Button(TeamButton::CardSurface)), card);
    for (id, r) in later {
        hero.hit_index.register(id, r);
    }
    scene.append_transformed(&body, ph2d_vector::Affine::IDENTITY);
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
        paint_text_elided(
            ts,
            scene,
            hint,
            field.x + pad,
            field.y + s * 0.25,
            TypeToken::Sm.px(),
            field.w - 2.0 * pad,
            resolve(ColorToken::Text3, theme),
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
    hero.documents.live.team.editing = Some(n);
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
    let (s, pad, g) = (btn(), Spacing::Sm.px(), Spacing::Xs.px());
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
    let room = at.y + at.h - Spacing::Md.px();
    let max_rows = (((room - at.y) / (s + g)).floor() as usize).saturating_sub(4);
    let shown = rows.len().min(max_rows);
    let h = pad * 2.0 + (s + g) * (3.0 + shown.max(1) as f32);
    let r = Rect::new(at.x, at.y, at.w, h);
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, r, radius, resolve(ColorToken::BgElev, theme));
    hero.hit_index
        .register(node(Hit::Button(TeamButton::ListSurface)), r);
    let mut y = at.y + pad;
    paint_text_elided(
        ts,
        scene,
        tr("board.comment.title"),
        at.x + pad,
        y + s * 0.25,
        TypeToken::Sm.px(),
        at.w - 3.0 * pad - s,
        resolve(ColorToken::Text1, theme),
    );
    let close = Rect::new(at.x + at.w - pad - s, y, s, s);
    super::super::super::board_nav::button(
        scene,
        &mut hero.hit_index,
        &hero.store,
        node(Hit::Button(TeamButton::ListClose)),
        IconId::Close,
        close,
        false,
        theme,
    );
    y += s + g;
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
            y + s * 0.25,
            TypeToken::Sm.px(),
            at.w - 2.0 * pad,
            resolve(ColorToken::Text3, theme),
        );
    }
    for (id, line, unread, resolved, n) in rows.into_iter().take(shown) {
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
            rr.y + s * 0.25,
            TypeToken::Sm.px(),
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
    let _ = Instant::now;
}
