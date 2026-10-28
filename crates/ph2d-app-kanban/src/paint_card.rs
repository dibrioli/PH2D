//! ⭐ **A frente de um cartão pintada** — a do `Card.dc.html` da maquete: capa de cor, rótulos (com
//! texto ou só barras), título, distintivos (descrição, comentários, anexos, checklist, votos,
//! local), membros à direita; o anel do foco, o lápis ao pairar e as três sombras (repouso, pairar,
//! voo).

use crate::front::{self, Front};
use crate::metrics as m;
use crate::paint::{c, label, shadow, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_block, paint_text_elided, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// O que o pintor precisa de um cartão neste quadro de imagem.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CardLook {
    pub front: Front,
    pub hover: bool,
    pub focus: bool,
}

/// As tintas do cartão no tema.
pub(crate) struct Ink {
    pub card: Color,
    pub fg: Color,
    pub meta: Color,
    pub dark: bool,
    pub theme: Theme,
}

impl Ink {
    pub fn of(theme: Theme) -> Self {
        Self {
            card: c(theme, T::KanbanCard),
            fg: c(theme, T::KanbanCardFg),
            meta: c(theme, T::KanbanCardMeta),
            dark: theme.is_dark(),
            theme,
        }
    }
}

/// A sombra de repouso (ou a de pairar) por baixo do cartão.
fn rest_shadow(scene: &mut VectorScene, ink: &Ink, r: Rect, hover: bool) {
    let t = ink.theme;
    if hover && !ink.dark {
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            c(t, T::KanbanShadowHover),
            m::SHADOW_HOVER_LIGHT,
        );
    } else {
        shadow(
            scene,
            r,
            m::CARD_RADIUS,
            c(t, T::KanbanShadowNear),
            m::SHADOW_NEAR,
        );
    }
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        c(t, T::KanbanShadowRim),
        m::SHADOW_RIM,
    );
}

/// A sombra de voo (o cartão agarrado descola do quadro).
pub(crate) fn flight_shadow(scene: &mut VectorScene, theme: Theme, r: Rect) {
    let far = if theme.is_dark() {
        m::SHADOW_FLIGHT_DARK
    } else {
        m::SHADOW_FLIGHT_LIGHT
    };
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        c(theme, T::KanbanShadowFlight),
        far,
    );
    shadow(
        scene,
        r,
        m::CARD_RADIUS,
        c(theme, T::KanbanFlightNear),
        m::SHADOW_FLIGHT_NEAR,
    );
}

/// ⭐ Um cartão em `r`.
pub(crate) fn card(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    ink: &Ink,
    look: &CardLook,
    r: Rect,
    label_text: bool,
) {
    let f = &look.front;
    rest_shadow(scene, ink, r, look.hover);
    fill_rounded_rect(scene, r, m::CARD_RADIUS, ink.card);
    if look.hover && ink.dark {
        stroke_rounded_rect(
            scene,
            r,
            m::CARD_RADIUS,
            m::HOVER_RING,
            c(theme, T::KanbanShadowHover),
        );
    }
    let mut y = r.y;
    if let Some(h) = f.cover {
        let cover = Rect::new(r.x, r.y, r.w, m::COVER_COLOR_H);
        let (fill, _) = label(
            theme,
            LabelColor {
                hue: h,
                shade: Shade::Normal,
            },
        );
        ph2d_editor_core::paint::fill_rounded_rect_radii(
            scene,
            cover,
            (m::CARD_RADIUS, m::CARD_RADIUS, 0.0, 0.0),
            fill,
        );
        y += m::COVER_COLOR_H;
    }
    y += m::CARD_PAD_Y;
    let x = r.x + m::CARD_PAD_X;
    let inner = r.w - 2.0 * m::CARD_PAD_X;
    if !f.labels.is_empty() {
        y = labels(scene, ts, theme, f, x, y, inner, label_text) + m::CARD_ROW_GAP;
    }
    let top = text_top(ts, y, m::LINE_H, m::CARD_TITLE_PX);
    paint_text_block(ts, scene, &f.title, x, top, m::CARD_TITLE_PX, inner, ink.fg);
    #[allow(clippy::cast_precision_loss)]
    let lines = front::title_lines(ts, &f.title, inner) as f32;
    y += lines * m::LINE_H;
    if f.badges() {
        badges(scene, ts, ink, f, x, y + m::CARD_ROW_GAP, inner);
    }
    if look.focus {
        let o = m::FOCUS_OFFSET;
        let ring = Rect::new(r.x - o, r.y - o, r.w + 2.0 * o, r.h + 2.0 * o);
        stroke_rounded_rect(
            scene,
            ring,
            m::CARD_RADIUS + o,
            m::FOCUS_RING,
            c(theme, T::Accent),
        );
    }
    if look.hover {
        let b = edit_button(r);
        shadow(
            scene,
            b,
            m::EDIT_BTN / 2.0,
            c(theme, T::KanbanShadowNear),
            m::SHADOW_NEAR,
        );
        fill_rounded_rect(scene, b, m::EDIT_BTN / 2.0, c(theme, T::KanbanCardRaised));
        icon(scene, IconId::EyePencil, b, ink.fg);
    }
}

/// O lápis no canto do cartão (pairar): editar o título no lugar.
pub(crate) fn edit_button(r: Rect) -> Rect {
    Rect::new(
        r.x + r.w - m::EDIT_INSET - m::EDIT_BTN,
        r.y + m::EDIT_INSET,
        m::EDIT_BTN,
        m::EDIT_BTN,
    )
}

fn icon(scene: &mut VectorScene, id: IconId, around: Rect, color: Color) {
    let s = m::BADGE_ICON;
    let r = Rect::new(
        around.x + (around.w - s) / 2.0,
        around.y + (around.h - s) / 2.0,
        s,
        s,
    );
    paint_icon(scene, id, r, color, StrokeToken::Default.px());
}

/// Os rótulos (chips com texto, ou barras finas). Devolve o fundo deles.
#[allow(clippy::too_many_arguments)]
fn labels(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Front,
    x0: f32,
    y0: f32,
    width: f32,
    text: bool,
) -> f32 {
    let row_h = if text { m::LABEL_H } else { m::LABEL_H / 2.0 };
    let (mut x, mut y) = (x0, y0);
    for (col, name) in &f.labels {
        let w = if text {
            front::chip_w(ts, name)
        } else {
            m::LABEL_MIN_W
        };
        if x > x0 && x + w > x0 + width {
            x = x0;
            y += row_h + m::LABEL_GAP;
        }
        let (fill, fg) = label(theme, *col);
        let chip = Rect::new(x, y, w, row_h);
        fill_rounded_rect(scene, chip, m::LABEL_RADIUS, fill);
        if text && !name.is_empty() {
            let ty = text_top(ts, y, row_h, m::SMALL_PX);
            paint_text_elided(
                ts,
                scene,
                name,
                x + m::LABEL_PAD_X,
                ty,
                m::SMALL_PX,
                w - 2.0 * m::LABEL_PAD_X,
                fg,
            );
        }
        x += w + m::LABEL_GAP;
    }
    y + row_h
}

/// A fila dos distintivos e, à direita, os avatares.
fn badges(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: &Ink,
    f: &Front,
    x0: f32,
    y: f32,
    width: f32,
) {
    let theme = ink.theme;
    let mut x = x0;
    let row = |x: f32, w: f32| Rect::new(x, y, w, m::BADGE_ROW_H);
    let badge = |scene: &mut VectorScene,
                 ts: &mut TextSystem,
                 x: &mut f32,
                 id: IconId,
                 text: Option<String>,
                 fill: Option<(Color, Color)>| {
        let tw = text
            .as_deref()
            .map_or(0.0, |t| ts.prefix_width(t, m::SMALL_PX) + m::BADGE_PAD);
        let w = m::BADGE_ICON + tw + 2.0 * m::BADGE_PAD;
        let r = row(*x, w);
        let fg = match fill {
            Some((bg, fg)) => {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y + m::BADGE_PAD / 2.0, r.w, r.h - m::BADGE_PAD),
                    m::LABEL_RADIUS,
                    bg,
                );
                fg
            }
            None => ink.meta,
        };
        icon(
            scene,
            id,
            Rect::new(r.x + m::BADGE_PAD, r.y, m::BADGE_ICON, r.h),
            fg,
        );
        if let Some(t) = text {
            let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
            paint_text_elided(
                ts,
                scene,
                &t,
                r.x + m::BADGE_PAD + m::BADGE_ICON + m::BADGE_PAD,
                ty,
                m::SMALL_PX,
                tw,
                fg,
            );
        }
        *x += w + m::BADGE_GAP;
    };
    if f.description {
        badge(scene, ts, &mut x, IconId::Menu, None, None);
    }
    if f.comments > 0 {
        badge(
            scene,
            ts,
            &mut x,
            IconId::MessageSquare,
            Some(f.comments.to_string()),
            None,
        );
    }
    if f.attachments > 0 {
        badge(
            scene,
            ts,
            &mut x,
            IconId::Link,
            Some(f.attachments.to_string()),
            None,
        );
    }
    if let Some((done, all)) = f.check {
        let full = done == all && all > 0;
        let fill = full.then(|| (c(theme, T::KanbanDueDone), c(theme, T::KanbanDueDoneFg)));
        badge(
            scene,
            ts,
            &mut x,
            IconId::Check,
            Some(format!("{done}/{all}")),
            fill,
        );
    }
    if f.votes > 0 {
        badge(
            scene,
            ts,
            &mut x,
            IconId::Vote,
            Some(f.votes.to_string()),
            None,
        );
    }
    if f.location {
        badge(scene, ts, &mut x, IconId::Place, None, None);
    }
    // Os membros, da direita para a esquerda, cada um a sobrepor o vizinho com um anel da cor do
    // cartão.
    let mut ax = x0 + width - m::AVATAR;
    for who in f.members.iter().rev() {
        let r = Rect::new(ax, y, m::AVATAR, m::AVATAR);
        let ring = Rect::new(
            r.x - m::AVATAR_RING,
            r.y - m::AVATAR_RING,
            r.w + 2.0 * m::AVATAR_RING,
            r.h + 2.0 * m::AVATAR_RING,
        );
        fill_rounded_rect(scene, ring, ring.w / 2.0, ink.card);
        let (bg, _) = label(
            theme,
            LabelColor {
                hue: front::avatar_hue(who),
                shade: Shade::Normal,
            },
        );
        fill_rounded_rect(scene, r, m::AVATAR / 2.0, bg);
        let s = front::initials(who);
        let w = ts.prefix_width(&s, m::AVATAR_PX);
        let ty = text_top(ts, r.y, r.h, m::AVATAR_PX);
        paint_text_elided(
            ts,
            scene,
            &s,
            r.x + (r.w - w) / 2.0,
            ty,
            m::AVATAR_PX,
            r.w,
            c(theme, T::KanbanAvatarInk),
        );
        ax -= m::AVATAR - m::AVATAR_OVERLAP;
        if ax < x {
            break;
        }
    }
}
