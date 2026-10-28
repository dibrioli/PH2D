//! ⭐ **O verso pintado** — o véu, o modal (capa, cabeçalho, as duas colunas) e o popover por cima.
//! O conteúdo pinta-se numa cena à parte para o modal saber a altura dele antes do fundo; os alvos
//! ficam na ordem em que se pintam (o último por cima).

use super::ui::{self as u, Ui};
use super::{Back, BackHit, EditAt, ROW};
use crate::metrics as m;
use crate::paint::label;
use crate::tab::Tab;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, fill_rounded_rect_radii, rect_to_vello};
use ph2d_editor_core::widget::paint_rich_editor;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::DueTone;
use ph2d_kanban_model::{
    Card, CoverFill, FieldKind, FieldValue, Kanban, LabelColor, Member, Moment, Shade,
};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

/// O que o verso lê para se pintar.
pub(crate) struct Ctx<'a> {
    pub k: &'a Kanban,
    /// O conjunto (os bytes dos anexos de ficheiro e das capas).
    pub set: &'a ph2d_kanban_model::KanbanSet,
    pub card: &'a Card,
    pub me: &'a Member,
    pub now: Moment,
    /// Quem se pode pôr no cartão e chamar com `@` (equipas dos quadros + membros usados + «eu»).
    pub people: Vec<Member>,
    /// J2 — o Kanban está num Quadro (a vista é o elemento, ou ele está posto num): «Convert to
    /// sticky note».
    pub on_board: bool,
}

/// A caixa do modal na área (sem a rolagem): `(x, w)`.
pub(crate) fn modal_x(area: Rect) -> (f32, f32) {
    let w = u::MODAL_W.min(area.w - 2.0 * u::MODAL_GUTTER).max(0.0);
    (area.x + (area.w - w) / 2.0, w)
}

/// ⭐ Pinta o verso aberto da aba (se há) por cima do quadro, e o painel de automação por cima dele.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &mut Tab,
    set: &ph2d_kanban_model::KanbanSet,
    me: &Member,
    team: &[String],
    now: Moment,
) {
    card_back(scene, ts, theme, k, tab, set, me, team, now);
    crate::auto::paint(scene, ts, theme, k, tab, set, me, team);
}

#[allow(clippy::too_many_arguments)]
fn card_back(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &mut Tab,
    set: &ph2d_kanban_model::KanbanSet,
    me: &Member,
    team: &[String],
    now: Moment,
) {
    let on_board = tab.key.is_some() || !tab.placements.is_empty();
    let area = tab.layout.area;
    let Some(back) = tab.back.as_deref_mut() else {
        return;
    };
    let Some(card) = k.cards.get(&back.card).filter(|c| !c.meta.deleted) else {
        tab.back = None;
        return;
    };
    let cx = Ctx {
        k,
        set,
        card,
        me,
        now,
        people: super::people(k, team, me),
        on_board,
    };
    sync_description(back, card);
    back.area = area;
    let mut hits = Vec::new();
    hits.push((area, BackHit::Scrim));
    scene.fill_rect(
        rect_to_vello(area),
        crate::paint::c(theme, T::KanbanShadowFlight),
    );
    let (x, w) = modal_x(area);
    let top = area.y + u::MODAL_TOP - back.scroll;
    let mut body = VectorScene::new();
    let h = {
        let mut ui = Ui {
            scene: &mut body,
            ts,
            theme,
            hits: &mut hits,
            hover: back.hover,
        };
        modal_body(&mut ui, back, &cx, x, top, w)
    };
    let modal = Rect::new(x, top, w, h);
    back.modal = modal;
    back.content_h = h;
    hits.insert(1, (modal, BackHit::Inside));
    scene.push_clip(&rect_to_vello(area));
    crate::paint::shadow(
        scene,
        modal,
        u::MODAL_RADIUS,
        crate::paint::c(theme, T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_DARK,
    );
    fill_rounded_rect(
        scene,
        modal,
        u::MODAL_RADIUS,
        crate::paint::c(theme, T::KanbanCard),
    );
    scene.append_transformed(&body, ph2d_vector::Affine::IDENTITY);
    {
        let mut ui = Ui {
            scene,
            ts,
            theme,
            hits: &mut hits,
            hover: back.hover,
        };
        super::activity::mentions(&mut ui, back, &cx);
        super::pop::paint(&mut ui, back, &cx);
    }
    scene.pop_layer();
    back.hits = hits;
}

/// Fora da edição, a descrição segue o documento (o desfazer, outra janela).
fn sync_description(back: &mut Back, card: &Card) {
    if !back.desc.editing && back.desc_md != card.description {
        back.desc.set_doc(ph2d_kanban_md::parse(&card.description));
        back.desc_md.clone_from(&card.description);
    }
}

/// O modal inteiro a partir de `top`; devolve a altura.
fn modal_body(ui: &mut Ui<'_>, back: &mut Back, cx: &Ctx<'_>, x: f32, top: f32, w: f32) -> f32 {
    let mut y = top;
    let close = Rect::new(
        x + w - u::GAP - u::CLOSE - u::SMALL_GAP,
        top + u::POP_PAD,
        u::CLOSE,
        u::CLOSE,
    );
    if let Some(cover) = cx.card.cover {
        let r = Rect::new(x, y, w, u::COVER_H);
        let fill = match cover.fill {
            CoverFill::Color(hue) => {
                label(
                    ui.theme,
                    LabelColor {
                        hue,
                        shade: Shade::Normal,
                    },
                )
                .0
            }
            CoverFill::Image(_) => ui.c(T::KanbanCardRaised),
        };
        fill_rounded_rect_radii(
            ui.scene,
            r,
            (u::MODAL_RADIUS, u::MODAL_RADIUS, 0.0, 0.0),
            fill,
        );
        if let CoverFill::Image(f) = cover.fill {
            let radii = (u::MODAL_RADIUS, u::MODAL_RADIUS, 0.0, 0.0);
            if !super::thumb::draw(ui.scene, cx.set, f, r, radii, false) {
                crate::paint::icon(ui.scene, IconId::Image, r, ui.meta());
            }
        }
        let label = tr("kanban.back.cover");
        let bw = ui.button_w(label, true);
        let b = Rect::new(
            x + w - u::COVER_BTN_RIGHT.min(w) - bw,
            y + r.h - ROW - u::GAP - u::HAIR,
            bw,
            ROW,
        );
        let card_bg = ui.c(T::KanbanCard);
        fill_rounded_rect(ui.scene, b, m::BUTTON_RADIUS, card_bg);
        ui.button(b, label, Some(IconId::Image), BackHit::CoverButton, false);
        y += r.h;
    }
    let card_bg = ui.c(T::KanbanCard);
    ui.icon_button(close, IconId::Close, BackHit::Close, true, card_bg);
    y += u::PAD_TOP;
    if cx.card.archived {
        y = archived_banner(ui, x + u::PAD_L, y, w - u::PAD_L - u::PAD_R);
    }
    y = header(ui, back, cx, x, y, w, close);
    let main_x = x + u::PAD_L + u::MAIN_INDENT;
    let side_x = x + w - u::PAD_R - u::SIDE_W;
    let main_w = (side_x - u::COL_GAP - main_x).max(0.0);
    let side_end = super::side::paint(ui, cx, side_x, y);
    let mut ym = y;
    ym = chips(ui, cx, main_x, ym, main_w);
    ym = description(ui, back, main_x, ym, main_w);
    ym = fields(ui, back, cx, main_x, ym, main_w);
    ym = location(ui, back, cx, main_x, ym, main_w);
    ym = super::scene::links(ui, back, cx, main_x, ym, main_w);
    ym = super::lists::attachments(ui, cx, main_x, ym, main_w);
    ym = super::lists::checklists(ui, back, cx, main_x, ym, main_w);
    ym = super::activity::feed(ui, back, cx, main_x, ym, main_w);
    ym.max(side_end) + u::PAD_BOTTOM - top
}

fn archived_banner(ui: &mut Ui<'_>, x: f32, y: f32, w: f32) -> f32 {
    let r = Rect::new(x, y, w, ROW);
    let (bg, fg) = (ui.c(T::KanbanDueSoon), ui.c(T::KanbanDueSoonFg));
    fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, bg);
    crate::paint::icon(ui.scene, IconId::Warning, Rect::new(x, y, ROW, ROW), fg);
    ui.strong(
        tr("kanban.back.archived"),
        x + ROW,
        y,
        ROW,
        u::BODY_PX,
        w - ROW,
        fg,
    );
    y + ROW + u::GAP
}

/// O título (editável no lugar), «in list X · Kanban» e o seguir.
#[allow(clippy::too_many_arguments)]
fn header(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, x: f32, y: f32, w: f32, close: Rect) -> f32 {
    let meta = ui.meta();
    crate::paint::icon(
        ui.scene,
        IconId::Text,
        Rect::new(x + u::PAD_L - u::SMALL_GAP, y, ROW - u::SMALL_GAP, ROW),
        meta,
    );
    let tx = x + u::PAD_L + u::MAIN_INDENT;
    let right = close.x.min(x + w - u::PAD_R) - u::GAP;
    let watch = Rect::new(right - ROW, y, ROW, ROW);
    let tw = (watch.x - u::GAP - tx).max(0.0);
    let title_h = match &back.edit {
        Some(e) if e.at == EditAt::Title => {
            let r = Rect::new(tx, y, tw, ROW + u::SMALL_GAP);
            ui.field(r, &e.field, "");
            ui.hit(r, BackHit::EditField);
            r.h
        }
        _ => {
            let fg = ui.fg();
            let h = ui
                .text(&cx.card.title, tx, y, u::TITLE_PX, tw, fg)
                .max(u::TITLE_LINE);
            ui.hit(Rect::new(tx, y, tw, h), BackHit::Title);
            h
        }
    };
    let watching = cx.card.watchers.contains(cx.me);
    let fill = if watching {
        ui.c(T::KanbanHeaderBtnOn)
    } else {
        ui.c(T::KanbanCard)
    };
    ui.icon_button(
        watch,
        IconId::Eye,
        BackHit::Watch,
        false,
        fill,
    );
    let mut yy = y + title_h + u::HAIR;
    let list =
        cx.k.lists
            .get(&cx.card.list)
            .map_or("", |l| l.title.as_str());
    let pre = tr("kanban.back.in_list");
    let pw = ui.width(pre, u::BODY_PX) + u::SMALL_GAP;
    ui.line(pre, tx, yy, u::META_LINE, u::BODY_PX, tw, meta);
    let lw = ui.width(list, u::BODY_PX);
    let fg = ui.fg();
    ui.strong(list, tx + pw, yy, u::META_LINE, u::BODY_PX, tw - pw, fg);
    ui.hit(Rect::new(tx + pw, yy, lw, u::META_LINE), BackHit::ListLink);
    let rest = tr_with("kanban.back.in_kanban", &[("k", &cx.k.header.name)]);
    let rx = tx + pw + lw + u::SMALL_GAP;
    ui.line(
        &rest,
        rx,
        yy,
        u::META_LINE,
        u::BODY_PX,
        (tx + tw - rx).max(0.0),
        fg,
    );
    yy += u::META_LINE;
    yy.max(close.y + close.h) + u::CHIP_GAP_Y
}

/// O texto e as cores do distintivo de estado do prazo («Overdue», «Due soon», «Complete»).
pub(crate) fn due_badge(ui: &Ui<'_>, card: &Card, now: Moment) -> Option<(String, T, T)> {
    let d = card.due?;
    let tone = ph2d_kanban_dates::due_tone(Some(d.at.0), d.done, now.0)?;
    Some(match tone {
        DueTone::Overdue => (
            tr("kanban.back.overdue").to_owned(),
            T::KanbanDueOver,
            T::KanbanDueOverFg,
        ),
        DueTone::Soon => (
            tr("kanban.back.due_soon").to_owned(),
            T::KanbanDueSoon,
            T::KanbanDueSoonFg,
        ),
        DueTone::Done => (
            tr("kanban.back.complete").to_owned(),
            T::KanbanDueDone,
            T::KanbanDueDoneFg,
        ),
        DueTone::Neutral => {
            let _ = ui;
            return None;
        }
    })
}

/// Os grupos do topo (Members, Labels, Dates, Votes), que quebram de linha como na maquete.
fn chips(ui: &mut Ui<'_>, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let card = cx.card;
    let mut groups: Vec<(f32, Group)> = Vec::new();
    if !card.members.is_empty() {
        #[allow(clippy::cast_precision_loss)]
        let n = card.members.len() as f32 + 1.0;
        groups.push((n * (u::AVATAR + u::SMALL_GAP), Group::Members));
    }
    let labels: Vec<_> = card
        .labels
        .iter()
        .filter_map(|l| cx.k.labels.get(l))
        .filter(|l| !l.meta.deleted)
        .collect();
    if !labels.is_empty() {
        let mut lw = ROW;
        for l in &labels {
            lw += (ui.width(&l.name, u::BODY_PX) + 2.0 * m::CARD_PAD_X).max(m::LABEL_MIN_W)
                + u::SMALL_GAP;
        }
        groups.push((lw, Group::Labels));
    }
    let dates = super::dates::range_text(card.start, card.due.map(|d| d.at), cx.now);
    let mut badge = due_badge(ui, card, cx.now);
    if !dates.is_empty() {
        let bw = badge
            .as_ref()
            .map_or(0.0, |b| ui.width(&b.0, u::SMALL_PX) + 3.0 * u::GAP);
        let check = if card.due.is_some() {
            u::CHECK + u::GAP
        } else {
            0.0
        };
        groups.push((
            ui.width(&dates, u::BODY_PX) + bw + check + 2.0 * m::BUTTON_PAD_X,
            Group::Dates,
        ));
    }
    if groups.is_empty() {
        return y;
    }
    let row_h = u::META_LINE + u::SMALL_GAP + ROW;
    let (mut gx, mut gy) = (x, y);
    for (gw, g) in groups {
        if gx > x && gx + gw > x + w {
            gx = x;
            gy += row_h + u::CHIP_GAP_Y;
        }
        let cy = gy + u::META_LINE + u::SMALL_GAP;
        match g {
            Group::Members => {
                ui.caption(tr("kanban.back.members"), gx, gy, gw);
                let mut ax = gx;
                for who in &card.members {
                    ui.avatar(ax, cy, u::AVATAR, who);
                    ui.hit(Rect::new(ax, cy, u::AVATAR, u::AVATAR), BackHit::MemberAdd);
                    ax += u::AVATAR + u::SMALL_GAP;
                }
                let slot = ui.c(T::KanbanSlot);
                ui.icon_button(
                    Rect::new(ax, cy, u::AVATAR, u::AVATAR),
                    IconId::Plus,
                    BackHit::MemberAdd,
                    true,
                    slot,
                );
            }
            Group::Labels => {
                ui.caption(tr("kanban.back.labels"), gx, gy, gw);
                let mut lx = gx;
                for l in &labels {
                    let lw = ui.label_chip(lx, cy, &l.name, l.color);
                    ui.hit(Rect::new(lx, cy, lw, ROW), BackHit::LabelAdd);
                    lx += lw + u::SMALL_GAP;
                }
                let slot = ui.c(T::KanbanSlot);
                ui.icon_button(
                    Rect::new(lx, cy, ROW, ROW),
                    IconId::Plus,
                    BackHit::LabelAdd,
                    false,
                    slot,
                );
            }
            Group::Dates => {
                ui.caption(tr("kanban.back.dates"), gx, gy, gw);
                let r = Rect::new(gx, cy, gw, ROW);
                ui.button(r, "", None, BackHit::Dates, false);
                let mut tx = r.x + m::BUTTON_PAD_X;
                if let Some(d) = card.due {
                    let b = Rect::new(tx, cy + (ROW - u::CHECK) / 2.0, u::CHECK, u::CHECK);
                    ui.checkbox(b, d.done);
                    ui.hit(
                        Rect::new(r.x, cy, u::CHECK + m::BUTTON_PAD_X + u::GAP / 2.0, ROW),
                        BackHit::DueDone,
                    );
                    tx += u::CHECK + u::GAP;
                }
                let fg = ui.fg();
                ui.line(&dates, tx, cy, ROW, u::BODY_PX, r.x + r.w - tx, fg);
                if let Some((text, bg, ink)) = badge.take() {
                    let tw = ui.width(&text, u::SMALL_PX);
                    let bx = tx + ui.width(&dates, u::BODY_PX) + u::GAP;
                    let br = Rect::new(
                        bx,
                        cy + (ROW - u::META_LINE) / 2.0,
                        tw + 2.0 * (u::GAP - u::HAIR),
                        u::META_LINE - u::HAIR,
                    );
                    let (bg, ink) = (ui.c(bg), ui.c(ink));
                    fill_rounded_rect(ui.scene, br, m::LABEL_RADIUS - u::HAIR / 2.0, bg);
                    ui.strong(
                        &text,
                        br.x + (br.w - tw) / 2.0,
                        br.y,
                        br.h,
                        u::SMALL_PX,
                        br.w,
                        ink,
                    );
                }
            }
        }
        gx += gw + u::CHIP_GAP_X;
    }
    gy + row_h + u::SECTION_GAP
}

#[derive(Clone, Copy)]
enum Group {
    Members,
    Labels,
    Dates,
}

/// A descrição: o editor rico (ler; um clique entra em edição) e Save/Cancel.
fn description(ui: &mut Ui<'_>, back: &mut Back, x: f32, y: f32, w: f32) -> f32 {
    ui.heading(tr("kanban.back.description"), x, y, w);
    let empty = back.desc.doc().is_empty();
    if !back.desc.editing && !empty {
        let s = tr("kanban.back.edit");
        let bw = ui.button_w(s, false);
        ui.button(
            Rect::new(x + w - bw, y, bw, ROW),
            s,
            None,
            BackHit::DescEdit,
            false,
        );
    }
    let y = y + ROW + u::GAP;
    let pref = back.desc.preferred_height(ui.ts, w);
    let h = if back.desc.editing {
        pref.max(4.0 * ROW)
    } else if empty {
        2.0 * ROW
    } else {
        pref
    };
    let r = Rect::new(x, y, w, h);
    if empty && !back.desc.editing {
        let slot = ui.c(T::KanbanSlot);
        fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, slot);
    }
    paint_rich_editor(&mut back.desc, r, ui.scene, ui.ts, ui.theme);
    ui.hit(r, BackHit::DescArea);
    let mut y = y + h;
    if back.desc.editing {
        y += u::GAP;
        let save = tr("kanban.back.save");
        let sw = ui.button_w(save, false);
        ui.primary(Rect::new(x, y, sw, ROW), save, BackHit::DescSave);
        let cancel = tr("kanban.back.cancel");
        let cw = ui.button_w(cancel, false);
        ui.button(
            Rect::new(x + sw + u::GAP, y, cw, ROW),
            cancel,
            None,
            BackHit::DescCancel,
            false,
        );
        y += ROW;
    }
    y + u::SECTION_GAP
}

/// O texto de um valor de campo personalizado.
fn field_text(cx: &Ctx<'_>, kind: &FieldKind, v: Option<&FieldValue>) -> String {
    match (kind, v) {
        (_, Some(FieldValue::Text(s))) => s.clone(),
        (_, Some(FieldValue::Number(n))) => format!("{n}"),
        (_, Some(FieldValue::Date(d))) => super::dates::short(*d, cx.now),
        (FieldKind::Dropdown(opts), Some(FieldValue::Option(o))) => opts
            .iter()
            .find(|x| x.id == *o)
            .map(|x| x.text.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Os campos personalizados: uma grelha de três, o nome por cima e o valor numa caixa.
fn fields(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let mut defs: Vec<_> = cx.k.fields.values().filter(|f| !f.meta.deleted).collect();
    if defs.is_empty() {
        return y;
    }
    defs.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    ui.heading(tr("kanban.back.custom_fields"), x, y, w);
    let y0 = y + ROW + u::GAP;
    #[allow(clippy::cast_precision_loss)]
    let cw = (w - u::GAP * (u::FIELD_COLS as f32 - 1.0)) / u::FIELD_COLS as f32;
    let cell_h = u::META_LINE + u::SMALL_GAP + ROW;
    let mut bottom = y0;
    for (i, f) in defs.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let (col, row) = ((i % u::FIELD_COLS) as f32, (i / u::FIELD_COLS) as f32);
        let fx = x + col * (cw + u::GAP);
        let fy = y0 + row * (cell_h + u::GAP);
        ui.caption(&f.name, fx, fy, cw);
        let r = Rect::new(fx, fy + u::META_LINE + u::SMALL_GAP, cw, ROW);
        bottom = r.y + r.h;
        if let Some(e) = back.edit.as_ref().filter(|e| e.at == EditAt::Field(f.id)) {
            ui.field(r, &e.field, "");
            ui.hit(r, BackHit::EditField);
            continue;
        }
        ui.button(r, "", None, BackHit::FieldValue(f.id), false);
        let v = cx.card.fields.get(&f.id);
        let tx = r.x + m::BUTTON_PAD_X;
        match (&f.kind, v) {
            (FieldKind::Checkbox, v) => {
                let on = matches!(v, Some(FieldValue::Checkbox(true)));
                ui.checkbox(
                    Rect::new(tx, r.y + (ROW - u::CHECK) / 2.0, u::CHECK, u::CHECK),
                    on,
                );
            }
            (FieldKind::Dropdown(opts), Some(FieldValue::Option(o))) => {
                if let Some(opt) = opts.iter().find(|x| x.id == *o) {
                    let col = opt.color.map(|hue| LabelColor {
                        hue,
                        shade: Shade::Subtle,
                    });
                    let (bg, ink) =
                        col.map_or((ui.c(T::KanbanSlot), ui.fg()), |c| label(ui.theme, c));
                    let tw = ui.width(&opt.text, u::SMALL_PX) + 2.0 * u::GAP;
                    let chip = Rect::new(
                        tx,
                        r.y + (ROW - u::META_LINE) / 2.0,
                        tw,
                        u::META_LINE - u::HAIR,
                    );
                    fill_rounded_rect(ui.scene, chip, m::LABEL_RADIUS - u::HAIR / 2.0, bg);
                    ui.line(&opt.text, tx + u::GAP, chip.y, chip.h, u::SMALL_PX, tw, ink);
                }
            }
            (kind, v) => {
                let t = field_text(cx, kind, v);
                let fg = ui.fg();
                ui.line(
                    &t,
                    tx,
                    r.y,
                    ROW,
                    u::BODY_PX,
                    r.w - 2.0 * m::BUTTON_PAD_X,
                    fg,
                );
            }
        }
        if matches!(f.kind, FieldKind::Dropdown(_)) {
            let meta = ui.meta();
            crate::paint::icon(
                ui.scene,
                IconId::ChevronDown,
                Rect::new(r.x + r.w - ROW, r.y, ROW, ROW),
                meta,
            );
        }
    }
    bottom + u::SECTION_GAP
}

/// A localização em texto (o dono decidiu: sem mapa).
fn location(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    ui.heading(tr("kanban.back.location"), x, y, w);
    let r = Rect::new(x, y + ROW + u::GAP, w, ROW);
    if let Some(e) = back.edit.as_ref().filter(|e| e.at == EditAt::Location) {
        ui.field(r, &e.field, tr("kanban.back.location_placeholder"));
        ui.hit(r, BackHit::EditField);
    } else {
        ui.button(r, "", Some(IconId::Pin), BackHit::Location, false);
        let (t, ink) = if cx.card.location.is_empty() {
            (tr("kanban.back.location_placeholder").to_owned(), ui.meta())
        } else {
            (cx.card.location.clone(), ui.fg())
        };
        let tx = r.x + m::BUTTON_PAD_X + m::ICON_PX + u::GAP;
        ui.line(&t, tx, r.y, ROW, u::BODY_PX, r.x + r.w - tx, ink);
    }
    r.y + r.h + u::SECTION_GAP
}
