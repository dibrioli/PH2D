//! ⭐ **Os distintivos da frente e onde caem** — a lista (pela ordem do Trello: seguir, datas,
//! descrição, comentários, anexos, checklist, votos, local) e a quebra em filas com os membros à
//! direita da última. A ALTURA do cartão e a PINTURA passam pela MESMA [`place`]: a medida nunca
//! diverge do que se pinta.

use super::Front;
use crate::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::DueTone;
use ph2d_text::TextSystem;
use ph2d_tokens::ColorToken as T;

/// O espaço entre duas filas de distintivos (o `gap: 4px 6px` da maquete).
pub(crate) const ROW_GAP: f32 = 4.0;
/// A altura de um campo personalizado na frente e o recuo do texto dele.
pub(crate) const FIELD_H: f32 = 20.0;
pub(crate) const FIELD_PAD_X: f32 = 6.0;

/// Um distintivo: o ícone, o texto (se tem) e o fundo com a tinta (se é preenchido).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Badge {
    pub icon: IconId,
    pub text: Option<String>,
    pub fill: Option<(T, T)>,
}

/// O fundo e a tinta do distintivo de datas no tom `t` (`None` = sem fundo, a cor «meta»).
pub(crate) fn due_fill(t: DueTone) -> Option<(T, T)> {
    match t {
        DueTone::Neutral => None,
        DueTone::Soon => Some((T::KanbanDueSoon, T::KanbanDueSoonFg)),
        DueTone::Overdue => Some((T::KanbanDueOver, T::KanbanDueOverFg)),
        DueTone::Done => Some((T::KanbanDueDone, T::KanbanDueDoneFg)),
    }
}

/// Os distintivos de `f`, pela ordem em que se pintam.
pub(crate) fn items(f: &Front) -> Vec<Badge> {
    let b = |icon, text: Option<String>, fill| Badge { icon, text, fill };
    let mut out = Vec::new();
    if f.template {
        let text = Some(ph2d_i18n::tr("kanban.w6.template_badge").to_owned());
        out.push(b(
            IconId::File,
            text,
            Some((T::KanbanBlueSubtle, T::KanbanLabelInk)),
        ));
    }
    if let Some(from) = &f.mirror {
        let text = if from.is_empty() {
            ph2d_i18n::tr("kanban.w6.mirror_badge").to_owned()
        } else {
            ph2d_i18n::tr_with("kanban.w6.mirror_from", &[("kanban", from)])
        };
        out.push(b(IconId::Clone, Some(text), None));
    }
    if f.watching {
        out.push(b(IconId::Eye, None, None));
    }
    if let Some(d) = &f.due {
        out.push(b(IconId::Timer, Some(d.text.clone()), due_fill(d.tone)));
    }
    if f.description {
        out.push(b(IconId::Menu, None, None));
    }
    if f.comments > 0 {
        out.push(b(IconId::MessageSquare, Some(f.comments.to_string()), None));
    }
    if f.attachments > 0 {
        out.push(b(IconId::Link, Some(f.attachments.to_string()), None));
    }
    if let Some((done, all)) = f.check {
        let full = done == all && all > 0;
        let fill = full.then_some((T::KanbanDueDone, T::KanbanDueDoneFg));
        out.push(b(IconId::Check, Some(format!("{done}/{all}")), fill));
    }
    if f.votes > 0 {
        out.push(b(IconId::Vote, Some(f.votes.to_string()), None));
    }
    if !f.location.is_empty() {
        out.push(b(IconId::Place, Some(f.location.clone()), None));
    }
    out
}

/// A largura de um distintivo e a do texto dentro dele, cortadas a `max`.
pub(crate) fn badge_w(ts: &mut TextSystem, b: &Badge, max: f32) -> (f32, f32) {
    let frame = m::BADGE_ICON + 2.0 * m::BADGE_PAD;
    let tw = b
        .text
        .as_deref()
        .map_or(0.0, |t| ts.prefix_width(t, m::SMALL_PX) + m::BADGE_PAD);
    let w = (frame + tw).min(max);
    (w, (w - frame).max(0.0))
}

/// A largura dos `n` avatares sobrepostos.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn members_w(n: usize) -> f32 {
    if n == 0 {
        0.0
    } else {
        m::AVATAR + (n - 1) as f32 * (m::AVATAR - m::AVATAR_OVERLAP)
    }
}

/// Onde cada distintivo cai, relativo ao canto da faixa: `(rect, largura do texto)`; os membros
/// (rect, quantos cabem); a altura da faixa.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Placed {
    pub badges: Vec<(Badge, Rect, f32)>,
    pub members: Option<(Rect, usize)>,
    pub h: f32,
}

/// ⭐ Parte os distintivos de `f` em filas de `width`; os membros à direita da última fila (ou
/// numa fila sua, se não cabem ao lado).
#[allow(clippy::cast_precision_loss)]
pub(crate) fn place(ts: &mut TextSystem, f: &Front, width: f32) -> Placed {
    let mut out = Placed::default();
    let (mut x, mut row) = (0.0_f32, 0usize);
    let y = |row: usize| row as f32 * (m::BADGE_ROW_H + ROW_GAP);
    for b in items(f) {
        let (w, tw) = badge_w(ts, &b, width);
        if x > 0.0 && x + w > width {
            row += 1;
            x = 0.0;
        }
        out.badges
            .push((b, Rect::new(x, y(row), w, m::BADGE_ROW_H), tw));
        x += w + m::BADGE_GAP;
    }
    if !f.members.is_empty() {
        let mut n = f.members.len();
        while n > 1 && members_w(n) > width {
            n -= 1;
        }
        let gw = members_w(n);
        if x > 0.0 && x + gw > width {
            row += 1;
        }
        let r = Rect::new(width - gw, y(row), gw, m::AVATAR);
        out.members = Some((r, n));
    }
    let rows = row + 1;
    if !out.badges.is_empty() || out.members.is_some() {
        out.h = rows as f32 * m::BADGE_ROW_H + (rows - 1) as f32 * ROW_GAP;
    }
    out
}

/// O recuo do texto de um campo à esquerda (o visto da caixa marcada, quando há).
fn field_icon(check: bool) -> f32 {
    if check {
        m::BADGE_ICON + m::BADGE_PAD
    } else {
        0.0
    }
}

/// A altura do texto de um campo QUEBRADO a `room` — pela pintura de verdade (numa cena à parte),
/// para a medida ser a do pintor.
fn wrapped_h(ts: &mut TextSystem, text: &str, room: f32) -> f32 {
    let mut scratch = ph2d_vector::VectorScene::new();
    let ink = ph2d_vector::Color::TRANSPARENT;
    ph2d_editor_core::paint::paint_text_block(
        ts,
        &mut scratch,
        text,
        0.0,
        0.0,
        m::SMALL_PX,
        room,
        ink,
    )
}

/// Um campo posto: o chip (relativo à faixa) e se o texto quebra em linhas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FieldAt {
    pub rect: Rect,
    pub wraps: bool,
    /// O orçamento do texto. Na linha única é a largura MEDIDA, nunca o chip menos o respiro: somar
    /// e subtrair perde um ulp e a elisão cortava o nome que o chip foi feito para levar
    /// (`Priority: Option 3` na Noto: `98,12399` para um texto de `98,124`).
    pub room: f32,
}

/// ⭐ **Onde cai cada campo personalizado da frente** (a ALTURA e a PINTURA passam por aqui). O
/// nome e o valor NUNCA se cortam enquanto couberem no cartão (dono 30/10: *«evitar cortar nomes
/// sempre que possível»*): o chip tem a largura do texto; o que não cabe numa linha ocupa a largura
/// do cartão e QUEBRA em linhas. Devolve os chips e a altura da faixa.
pub(crate) fn place_fields(ts: &mut TextSystem, f: &Front, width: f32) -> (Vec<FieldAt>, f32) {
    let (mut x, mut y, mut row_h) = (0.0_f32, 0.0_f32, 0.0_f32);
    let mut out = Vec::with_capacity(f.fields.len());
    for c in &f.fields {
        let pad = field_icon(c.check) + 2.0 * FIELD_PAD_X;
        let text_w = ts.prefix_width(&c.text, m::SMALL_PX);
        let one = text_w + pad;
        let wraps = one > width;
        let (w, h, room) = if wraps {
            let room = (width - pad).max(0.0);
            let line = wrapped_h(ts, "M", room);
            let text = wrapped_h(ts, &c.text, room);
            (width, FIELD_H + (text - line).max(0.0), room)
        } else {
            (one, FIELD_H, text_w)
        };
        if x > 0.0 && x + w > width {
            x = 0.0;
            y += row_h + m::LABEL_GAP;
            row_h = 0.0;
        }
        out.push(FieldAt {
            rect: Rect::new(x, y, w, h),
            wraps,
            room,
        });
        row_h = row_h.max(h);
        x += w + m::LABEL_GAP;
    }
    let h = if out.is_empty() { 0.0 } else { y + row_h };
    (out, h)
}

/// A altura da faixa dos campos personalizados a `width`.
pub(crate) fn fields_h(ts: &mut TextSystem, f: &Front, width: f32) -> f32 {
    place_fields(ts, f, width).1
}
