//! ⭐ **O cabeçalho do Kanban (J1)** — a maquete `Main.dc.html`: o seletor de vistas («Board ▾») ao
//! lado da estrela; à direita, antes do «…», o «Share» e os avatares de quem está a ver. O filtro
//! ([`crate::w6`]) continua à esquerda deles. Os alvos registam-se na lista da W6 (o clique seguinte
//! lê-os), e os painéis abrem-se como as outras folhas.

pub(crate) mod other;

use crate::metrics as m;
use crate::paint::{c, text_top};
use crate::share::Look;
use crate::tab::Tab;
use crate::w6::sheet::{Item, Sheet};
use crate::w6::{Act, Over};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_title_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{Kanban, Member};
use ph2d_kanban_views::ViewKind;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// O diâmetro de um avatar do cabeçalho (a maquete: 28) e quanto se sobrepõem (6).
pub(crate) const AVATAR: f32 = 28.0;
pub(crate) const AVATAR_STEP: f32 = 22.0;
/// Quantos avatares se mostram antes do «+n» (a maquete mostra quatro).
pub(crate) const AVATARS_SHOWN: usize = 4;

/// Onde ficaram os botões do cabeçalho neste quadro.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Head {
    pub views: Rect,
    pub share: Rect,
    /// O filtro e a pesquisa ficam à esquerda daqui.
    pub right: f32,
}

/// O nome da vista: `None` = o quadro.
pub(crate) fn view_name(v: Option<ViewKind>) -> &'static str {
    v.map_or_else(|| tr("kanban.views.board"), ViewKind::label)
}

fn text_w(ts: &mut TextSystem, s: &str) -> f32 {
    crate::docs::title_width(ts, s, m::BUTTON_TEXT_PX)
}

/// ⭐ Pinta o seletor, o «Share» e os avatares; regista os alvos.
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tab: &Tab,
    (k, look): (&Kanban, &Look),
) -> Head {
    let kanban = k.id;
    let l = &tab.layout;
    let fg = c(theme, T::KanbanHeaderFg);
    let hover = |a: Act| tab.w6.hover == Some(a);
    let mut hits = Vec::new();
    // «Board ▾» depois da estrela (aceso: é a vista à vista).
    let name = view_name(tab.other);
    let w = m::BUTTON_PAD_X * 2.0 + m::ICON_PX * 2.0 + m::LIST_ICON_GAP * 2.0 + text_w(ts, name);
    let views = Rect::new(
        l.star.x + l.star.w + m::HEADER_GAP,
        l.star.y,
        w,
        m::BUTTON_H,
    );
    let open = tab.w6.over == Some(Over::Views);
    let t = if open || hover(Act::ViewsButton) {
        T::KanbanHeaderBtnOn
    } else {
        T::KanbanHeaderBtn
    };
    fill_rounded_rect(scene, views, m::BUTTON_RADIUS, c(theme, t));
    let ink = if open || hover(Act::ViewsButton) {
        c(theme, T::KanbanBtnOnFg)
    } else {
        fg
    };
    let mut x = views.x + m::BUTTON_PAD_X;
    crate::paint::icon(
        scene,
        IconId::LayoutGrid,
        Rect::new(x, views.y, m::ICON_PX, views.h),
        ink,
    );
    x += m::ICON_PX + m::LIST_ICON_GAP;
    let ty = text_top(ts, views.y, views.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(ts, scene, name, x, ty, m::BUTTON_TEXT_PX, w, ink);
    let chev = Rect::new(
        views.x + views.w - m::BUTTON_PAD_X - m::ICON_PX,
        views.y,
        m::ICON_PX,
        views.h,
    );
    crate::paint::icon(scene, IconId::ChevronDown, chev, ink);
    hits.push((views, Act::ViewsButton));
    // «Share» à esquerda do «…» (o botão de destaque da maquete).
    let label = tr("kanban.share.button");
    let sw = text_w(ts, label) + 2.0 * m::BUTTON_PAD_X;
    let share = Rect::new(
        l.board_menu.x - m::HEADER_GAP - sw,
        l.board_menu.y,
        sw,
        m::BUTTON_H,
    );
    let on = hover(Act::ShareButton) || tab.w6.over == Some(Over::Share);
    fill_rounded_rect(
        scene,
        share,
        m::BUTTON_RADIUS,
        c(theme, if on { T::AccentHover } else { T::Accent }),
    );
    let sty = text_top(ts, share.y, share.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        label,
        share.x + m::BUTTON_PAD_X,
        sty,
        m::BUTTON_TEXT_PX,
        sw,
        c(theme, T::AccentFg),
    );
    hits.push((share, Act::ShareButton));
    let people = look
        .seen
        .get(&kanban)
        .map(|s| s.people.as_slice())
        .unwrap_or_default();
    let right = avatars(scene, ts, theme, share.x - m::HEADER_GAP, l.header, people);
    // J5 — os botões de quadro do Butler à esquerda dos avatares (o clique: `Act::BoardButton`).
    let lit = match tab.w6.hover {
        Some(Act::BoardButton(a)) => Some(a),
        _ => None,
    };
    let (buttons, right) = crate::auto::buttons::paint_board_buttons(
        scene,
        ts,
        theme,
        k,
        right - m::HEADER_GAP,
        l.board_menu.y,
        m::BUTTON_H,
        lit,
    );
    hits.extend(buttons.into_iter().map(|(r, a)| (r, Act::BoardButton(a))));
    tab.w6.hits.borrow_mut().extend(hits);
    Head {
        views,
        share,
        right: right - m::HEADER_GAP,
    }
}

/// A pilha de avatares que acaba em `right` (o 1.º à esquerda, os outros por baixo dele, como na
/// maquete), com «+n» além de [`AVATARS_SHOWN`]. Devolve onde começa.
fn avatars(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    right: f32,
    header: Rect,
    people: &[Member],
) -> f32 {
    if people.is_empty() {
        return right + m::HEADER_GAP;
    }
    let shown = people.len().min(AVATARS_SHOWN);
    let extra = people.len() - shown;
    let more = (extra > 0).then(|| format!("+{extra}"));
    #[allow(clippy::cast_precision_loss)]
    let n = shown + usize::from(more.is_some());
    #[allow(clippy::cast_precision_loss)]
    let w = AVATAR + (n.saturating_sub(1)) as f32 * AVATAR_STEP;
    let x0 = right - w;
    let y = header.y + (header.h - AVATAR) / 2.0;
    let ring = c(theme, T::KanbanHeaderBg);
    // Da direita para a esquerda: o da esquerda fica por cima.
    for i in (0..n).rev() {
        #[allow(clippy::cast_precision_loss)]
        let x = x0 + i as f32 * AVATAR_STEP;
        let halo = Rect::new(
            x - m::AVATAR_RING,
            y - m::AVATAR_RING,
            AVATAR + 2.0 * m::AVATAR_RING,
            AVATAR + 2.0 * m::AVATAR_RING,
        );
        fill_rounded_rect(scene, halo, halo.w / 2.0, ring);
        match (people.get(i).filter(|_| i < shown), &more) {
            (Some(p), _) => {
                crate::presence::avatar(scene, ts, theme, Rect::new(x, y, AVATAR, AVATAR), p)
            }
            (None, Some(t)) => {
                let r = Rect::new(x, y, AVATAR, AVATAR);
                fill_rounded_rect(scene, r, AVATAR / 2.0, c(theme, T::KanbanHeaderBtnOn));
                let tw = crate::docs::title_width(ts, t, m::SMALL_PX);
                let ty = text_top(ts, y, AVATAR, m::SMALL_PX);
                paint_text_title_elided(
                    ts,
                    scene,
                    t,
                    x + (AVATAR - tw) / 2.0,
                    ty,
                    m::SMALL_PX,
                    AVATAR,
                    c(theme, T::KanbanBtnOnFg),
                );
            }
            (None, None) => {}
        }
    }
    x0
}

/// A folha do seletor: o quadro e as quatro vistas, a de agora marcada.
pub(crate) fn views_sheet(tab: &Tab) -> Sheet {
    let mut s = Sheet::new(crate::w6::sheet::W);
    s.title("kanban.share.views_title");
    let all = std::iter::once(None).chain(ViewKind::ALL.into_iter().map(Some));
    for (i, v) in all.enumerate() {
        s.push(
            Item::Entry {
                text: view_name(v).to_owned(),
                icon: None,
                on: tab.other == v,
            },
            Act::View(u8::try_from(i).unwrap_or(0)),
        );
    }
    s
}

/// A vista `i` da folha (0 = o quadro).
pub(crate) fn kind_at(i: u8) -> Option<ViewKind> {
    usize::from(i)
        .checked_sub(1)
        .and_then(|k| ViewKind::ALL.get(k).copied())
}

/// Escolhe a vista `i` (fecha a folha).
pub(crate) fn pick_view(docs: &mut crate::docs::KanbanDocs, id: u64, i: u8) {
    if let Some(t) = docs.tab_mut(id) {
        t.w6.close_top();
        let k = kind_at(i);
        t.other = k;
        if let Some(k) = k {
            t.views.kind = k;
        }
        t.drag = None;
        t.press = None;
    }
}
