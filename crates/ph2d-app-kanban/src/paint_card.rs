//! ⭐ **A frente de um cartão pintada** — a do `Card.dc.html` da maquete: capa de cor ou de imagem
//! (metade ou inteira), rótulos (com texto ou só barras), título, campos «na frente», distintivos
//! em filas (seguir, datas com a cor do estado, descrição, comentários, anexos, checklist, votos,
//! local), membros à direita; o anel do foco, o lápis ao pairar e as três sombras.

use crate::front::{self, Front, badges};
use crate::metrics as m;
use crate::paint::{c, label, shadow, text_top};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_block, paint_text_elided, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{Hue, LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

/// O que o pintor precisa de um cartão neste quadro de imagem.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CardLook {
    pub front: Front,
    pub hover: bool,
    pub focus: bool,
    /// W6 — o nível de envelhecimento `0..=3` (0 = fresco).
    pub age: u8,
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
/// A sombra de voo (do cartão e da lista, com o `radius` de cada).
pub(crate) fn flight_shadow(scene: &mut VectorScene, theme: Theme, r: Rect, radius: f32) {
    let far = if theme.is_dark() {
        m::SHADOW_FLIGHT_DARK
    } else {
        m::SHADOW_FLIGHT_LIGHT
    };
    shadow(scene, r, radius, c(theme, T::KanbanShadowFlight), far);
    shadow(
        scene,
        r,
        radius,
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
    let mut seen = probe::Painted::default();
    let bottom = if f.full() {
        cover::full(scene, ts, theme, f, r, &mut seen)
    } else {
        body(scene, ts, ink, f, r, label_text, &mut seen)
    };
    seen.h = bottom - r.y;
    probe::note(&f.title, seen);
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

/// O corpo de um cartão sem capa inteira: a capa metade, os rótulos, o título, os campos e os
/// distintivos — a MESMA sequência que [`front::measure`] soma. Devolve o fundo pintado.
#[allow(clippy::cast_precision_loss)]
fn body(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: &Ink,
    f: &Front,
    r: Rect,
    label_text: bool,
    seen: &mut probe::Painted,
) -> f32 {
    let theme = ink.theme;
    let ch = front::cover_h(f);
    if ch > 0.0 {
        cover::half(scene, theme, f, Rect::new(r.x, r.y, r.w, ch), seen);
    }
    let mut y = r.y + front::labels_top(f).unwrap_or(ch + m::CARD_PAD_Y);
    let x = r.x + m::CARD_PAD_X;
    let inner = r.w - 2.0 * m::CARD_PAD_X;
    if !f.labels.is_empty() {
        y = labels(scene, ts, theme, f, x, y, inner, label_text) + m::CARD_ROW_GAP;
    }
    let top = text_top(ts, y, m::LINE_H, m::CARD_TITLE_PX);
    paint_text_block(ts, scene, &f.title, x, top, m::CARD_TITLE_PX, inner, ink.fg);
    y += front::title_lines(ts, &f.title, inner) as f32 * m::LINE_H;
    if !f.fields.is_empty() {
        y = fields(scene, ts, theme, f, x, y + m::CARD_ROW_GAP, inner, seen);
    }
    let placed = badges::place(ts, f, inner);
    if placed.h > 0.0 {
        let y0 = y + m::CARD_ROW_GAP;
        badges(scene, ts, ink, f, &placed, x, y0, seen);
        y = y0 + placed.h;
    }
    y + m::CARD_PAD_Y
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

/// Os campos «na frente»: chips `Nome: valor` (a opção da lista suspensa com a cor dela). Devolve
/// o fundo deles.
#[allow(clippy::too_many_arguments)]
fn fields(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Front,
    x0: f32,
    y0: f32,
    width: f32,
    seen: &mut probe::Painted,
) -> f32 {
    let (mut x, mut y) = (x0, y0);
    for chip in &f.fields {
        let w = badges::field_w(ts, &chip.text, chip.check, width);
        if x > x0 && x + w > x0 + width {
            x = x0;
            y += badges::FIELD_H + m::LABEL_GAP;
        }
        let hue = chip.hue.unwrap_or(Hue::Black);
        let (fill, fg) = label(
            theme,
            LabelColor {
                hue,
                shade: Shade::Subtle,
            },
        );
        fill_rounded_rect(
            scene,
            Rect::new(x, y, w, badges::FIELD_H),
            m::LABEL_RADIUS,
            fill,
        );
        let mut tx = x + badges::FIELD_PAD_X;
        if chip.check {
            icon(
                scene,
                IconId::Check,
                Rect::new(tx, y, m::BADGE_ICON, badges::FIELD_H),
                fg,
            );
            tx += m::BADGE_ICON + m::BADGE_PAD;
        }
        let ty = text_top(ts, y, badges::FIELD_H, m::SMALL_PX);
        let room = x + w - badges::FIELD_PAD_X - tx;
        paint_text_elided(ts, scene, &chip.text, tx, ty, m::SMALL_PX, room, fg);
        seen.fields.push(chip.text.clone());
        x += w + m::LABEL_GAP;
    }
    y + badges::FIELD_H
}

/// Os distintivos já partidos em filas ([`badges::place`]) e os avatares à direita.
#[allow(clippy::too_many_arguments)]
fn badges(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    ink: &Ink,
    f: &Front,
    placed: &badges::Placed,
    x0: f32,
    y0: f32,
    seen: &mut probe::Painted,
) {
    let theme = ink.theme;
    for (b, at, tw) in &placed.badges {
        let r = Rect::new(x0 + at.x, y0 + at.y, at.w, at.h);
        let fg = match b.fill {
            Some((bg, fg)) => {
                fill_rounded_rect(
                    scene,
                    Rect::new(r.x, r.y + m::BADGE_PAD / 2.0, r.w, r.h - m::BADGE_PAD),
                    m::LABEL_RADIUS,
                    c(theme, bg),
                );
                c(theme, fg)
            }
            None => ink.meta,
        };
        icon(
            scene,
            b.icon,
            Rect::new(r.x + m::BADGE_PAD, r.y, m::BADGE_ICON, r.h),
            fg,
        );
        if let Some(t) = &b.text {
            let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
            let tx = r.x + m::BADGE_PAD + m::BADGE_ICON + m::BADGE_PAD;
            paint_text_elided(ts, scene, t, tx, ty, m::SMALL_PX, *tw, fg);
        }
        if b.icon == IconId::Timer {
            seen.due = Some((b.text.clone().unwrap_or_default(), b.fill.map(|f| f.0)));
        }
        seen.badges.push(b.icon);
    }
    let Some((group, n)) = placed.members else {
        return;
    };
    // Da direita para a esquerda, cada um a sobrepor o vizinho com um anel da cor do cartão.
    let mut ax = x0 + group.x + group.w - m::AVATAR;
    let y = y0 + group.y;
    for who in f.members.iter().rev().take(n) {
        let r = Rect::new(ax, y, m::AVATAR, m::AVATAR);
        let ring = Rect::new(
            r.x - m::AVATAR_RING,
            r.y - m::AVATAR_RING,
            r.w + 2.0 * m::AVATAR_RING,
            r.h + 2.0 * m::AVATAR_RING,
        );
        fill_rounded_rect(scene, ring, ring.w / 2.0, ink.card);
        let hue = front::avatar_hue(who);
        let (bg, _) = label(
            theme,
            LabelColor {
                hue,
                shade: Shade::Normal,
            },
        );
        fill_rounded_rect(scene, r, m::AVATAR / 2.0, bg);
        let s = front::initials(who);
        let w = ts.prefix_width(&s, m::AVATAR_PX);
        let ty = text_top(ts, r.y, r.h, m::AVATAR_PX);
        let avatar_ink = c(theme, T::KanbanAvatarInk);
        paint_text_elided(
            ts,
            scene,
            &s,
            r.x + (r.w - w) / 2.0,
            ty,
            m::AVATAR_PX,
            r.w,
            avatar_ink,
        );
        ax -= m::AVATAR - m::AVATAR_OVERLAP;
    }
}

mod cover;
pub(crate) mod probe;

#[cfg(test)]
mod tests;
