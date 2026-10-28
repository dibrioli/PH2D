//! ⭐ **Os colegas no quadro (J1, W5)** — o anel e o avatar no cartão que alguém tem aberto, e o cursor
//! de cada um. O cursor viaja nas coordenadas do CONTEÚDO do Kanban (desde a origem das listas, sem a
//! rolagem de quem o mandou): aqui volta pela MINHA rolagem, nos px da vista — a `Affine` da aba ou
//! do canvas leva-o ao ecrã com o resto (sobrevive ao zoom).

use crate::metrics as m;
use crate::paint::{c, text_top};
use crate::share::Seen;
use crate::tab::Tab;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_title_elided, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{LabelColor, Member, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, VectorScene};

/// O avatar no canto do cartão aberto por alguém.
const CARD_AVATAR: f32 = 20.0;
/// A seta do cursor (px da vista) e a etiqueta com o nome.
const CURSOR: f64 = 14.0;
const NAME_PAD: f32 = 4.0;

/// A cor de uma pessoa (a do avatar do verso: o matiz pelo nome).
pub(crate) fn color(theme: Theme, who: &Member) -> Color {
    let hue = crate::front::avatar_hue(who);
    crate::paint::label(
        theme,
        LabelColor {
            hue,
            shade: Shade::Normal,
        },
    )
    .0
}

/// Um avatar redondo com as iniciais em `r`.
pub(crate) fn avatar(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    r: Rect,
    who: &Member,
) {
    fill_rounded_rect(scene, r, r.w / 2.0, color(theme, who));
    let t = crate::front::initials(who);
    let w = crate::docs::title_width(ts, &t, m::SMALL_PX);
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_title_elided(
        ts,
        scene,
        &t,
        r.x + (r.w - w) / 2.0,
        ty,
        m::SMALL_PX,
        r.w,
        c(theme, T::KanbanAvatarInk),
    );
}

/// O ponto do conteúdo `(x, y)` (desde a origem das listas) nos px da vista desta aba.
pub(crate) fn to_view(tab: &Tab, p: [f64; 2]) -> (f32, f32) {
    let (ox, oy) = crate::tab::origin(tab, tab.layout.board);
    #[allow(clippy::cast_possible_truncation)]
    (ox + p[0] as f32, oy + p[1] as f32)
}

/// O ponto da vista `(x, y)` no conteúdo (o inverso de [`to_view`]).
pub(crate) fn to_content(tab: &Tab, (x, y): (f32, f32)) -> [f64; 2] {
    let (ox, oy) = crate::tab::origin(tab, tab.layout.board);
    [f64::from(x - ox), f64::from(y - oy)]
}

/// ⭐ Os anéis nos cartões abertos e os cursores, por cima do quadro.
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tab: &Tab,
    seen: &Seen,
) {
    for (card, who) in &seen.cards {
        let Some(first) = who.first() else {
            continue;
        };
        let Some((_, cb)) = tab.layout.card(*card) else {
            continue;
        };
        let r = cb.rect;
        stroke_rounded_rect(scene, r, m::CARD_RADIUS, m::FOCUS_RING, color(theme, first));
        for (i, p) in who.iter().enumerate().take(crate::header::AVATARS_SHOWN) {
            #[allow(clippy::cast_precision_loss)]
            let x = r.x + r.w
                - CARD_AVATAR
                - m::AVATAR_RING
                - i as f32 * (CARD_AVATAR - m::AVATAR_OVERLAP);
            let a = Rect::new(x, r.y - CARD_AVATAR / 2.0, CARD_AVATAR, CARD_AVATAR);
            let halo = Rect::new(
                a.x - m::AVATAR_RING,
                a.y - m::AVATAR_RING,
                a.w + 2.0 * m::AVATAR_RING,
                a.h + 2.0 * m::AVATAR_RING,
            );
            fill_rounded_rect(scene, halo, halo.w / 2.0, c(theme, T::KanbanCard));
            avatar(scene, ts, theme, a, p);
        }
    }
    if seen.reconnecting {
        reconnecting(scene, ts, theme, tab.layout.board);
    }
    if tab.other.is_some() {
        return;
    }
    let board = tab.layout.board;
    scene.push_clip(&ph2d_editor_core::paint::rect_to_vello(board));
    for (who, at) in &seen.cursors {
        let (x, y) = to_view(tab, *at);
        if board.contains(x, y) {
            cursor(scene, ts, theme, (x, y), who);
        }
    }
    scene.pop_layer();
}

/// A seta e o nome de um colega em `(x, y)`.
fn cursor(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    (x, y): (f32, f32),
    who: &Member,
) {
    let col = color(theme, who);
    let mut p = BezPath::new();
    p.move_to((0.0, 0.0));
    p.line_to((0.0, CURSOR));
    p.line_to((CURSOR * 0.3, CURSOR * 0.72));
    p.line_to((CURSOR * 0.72, CURSOR * 0.72));
    p.close_path();
    let t = Affine::translate((f64::from(x), f64::from(y)));
    scene.fill_path(&p, &Brush::Solid(col), t);
    #[allow(clippy::cast_possible_truncation)]
    let (nx, ny) = (x + (CURSOR * 0.6) as f32, y + CURSOR as f32);
    let w = crate::docs::title_width(ts, &who.0, m::SMALL_PX) + 2.0 * NAME_PAD;
    let r = Rect::new(nx, ny, w, m::COUNT_H);
    fill_rounded_rect(scene, r, m::LABEL_RADIUS, col);
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_title_elided(
        ts,
        scene,
        &who.0,
        r.x + NAME_PAD,
        ty,
        m::SMALL_PX,
        w,
        c(theme, T::KanbanAvatarInk),
    );
}

/// J4 — o selo «Reconnecting…» no canto do quadro: a sala caiu e está a voltar sozinha.
fn reconnecting(scene: &mut VectorScene, ts: &mut TextSystem, theme: Theme, board: Rect) {
    let t = ph2d_i18n::tr("kanban.share.reconnecting_badge");
    let w = crate::docs::title_width(ts, t, m::SMALL_PX) + 2.0 * NAME_PAD * 2.0;
    let r = Rect::new(
        board.x + board.w - w - m::HEADER_GAP,
        board.y + m::HEADER_GAP,
        w,
        m::COUNT_H,
    );
    fill_rounded_rect(scene, r, r.h / 2.0, c(theme, T::KanbanHeaderBtnOn));
    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
    paint_text_title_elided(
        ts,
        scene,
        t,
        r.x + NAME_PAD * 2.0,
        ty,
        m::SMALL_PX,
        w,
        c(theme, T::KanbanBtnOnFg),
    );
}
