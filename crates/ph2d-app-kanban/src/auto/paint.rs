//! ⭐ **O painel de automação pintado** — o véu, o modal (título, separadores, e a lista OU o
//! construtor) e, por cima, o menu ou a caixa do chip aberto. As peças são as do verso
//! ([`crate::back::ui`]): o mesmo botão, a mesma caixa, as mesmas medidas da maquete.

use super::choices::{self, Shown};
use super::input::{of_family, visible_suggestions};
use super::sentence::{Names, Seg, conditions_mut, segs};
use super::{AutoHit, FAMILIES, Panel, family_key};
use crate::back::ui::{self as u, Ui};
use crate::back::{BackHit, ROW};
use crate::metrics as m;
use crate::tab::Tab;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_auto::{Family, Kind, Part, Spec, Trigger};
use ph2d_kanban_model::{Kanban, KanbanSet, Member};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::VectorScene;

const A: fn(AutoHit) -> BackHit = BackHit::Auto;

/// O que o painel lê para se pintar.
struct Cx<'a> {
    k: &'a Kanban,
    names: Names<'a>,
    me: &'a Member,
    people: Vec<Member>,
}

/// ⭐ Pinta o painel aberto da vista (se há) por cima do quadro e do verso.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    k: &Kanban,
    tab: &mut Tab,
    set: &KanbanSet,
    me: &Member,
    team: &[String],
) {
    let area = tab.layout.area;
    let Some(p) = tab.auto.as_deref_mut() else {
        return;
    };
    let cx = Cx {
        k,
        names: Names { k, set },
        me,
        people: crate::back::people(k, team, me),
    };
    p.area = area;
    p.lines.clear();
    let mut hits = vec![(area, A(AutoHit::Scrim))];
    scene.fill_rect(
        rect_to_vello(area),
        crate::paint::c(theme, T::KanbanShadowFlight),
    );
    let (x, w) = crate::back::paint::modal_x(area);
    let top = area.y + u::MODAL_TOP - p.scroll;
    let mut body = VectorScene::new();
    let h = {
        let mut ui = Ui {
            scene: &mut body,
            ts,
            theme,
            hits: &mut hits,
            hover: p.hover,
        };
        modal_body(&mut ui, p, &cx, x, top, w)
    };
    let modal = Rect::new(x, top, w, h);
    p.content_h = h;
    hits.insert(1, (modal, A(AutoHit::Inside)));
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
            hover: p.hover,
        };
        chooser(&mut ui, p, &cx, area);
    }
    scene.pop_layer();
    p.hits = hits;
}

fn modal_body(ui: &mut Ui<'_>, p: &mut Panel, cx: &Cx<'_>, x: f32, top: f32, w: f32) -> f32 {
    let ix = x + u::PAD_L;
    let iw = w - u::PAD_L - u::PAD_R;
    let mut y = top + u::PAD_TOP;
    let close = Rect::new(x + w - u::PAD_R - u::CLOSE, y, u::CLOSE, u::CLOSE);
    let bg = ui.c(T::KanbanCard);
    ui.icon_button(close, IconId::Close, A(AutoHit::Close), true, bg);
    let ir = Rect::new(
        ix,
        y + (u::CLOSE - m::ICON_PX) / 2.0,
        m::ICON_PX,
        m::ICON_PX,
    );
    let fg = ui.fg();
    ph2d_editor_core::paint::paint_icon(ui.scene, IconId::Bolt, ir, fg, StrokeToken::Default.px());
    ui.strong(
        tr("kanban.auto.ui.title"),
        u::Band::new(ix + m::ICON_PX + u::GAP, y, iw - u::CLOSE, u::CLOSE),
        u::TITLE_PX,
        fg,
    );
    y += u::CLOSE + u::GAP;
    let mut tx = ix;
    for f in FAMILIES {
        let label = tr(&format!("kanban.auto.ui.tab.{}", family_key(f)));
        let bw = ui.button_w(label, false);
        if tx + bw > ix + iw && tx > ix {
            tx = ix;
            y += ROW + u::SMALL_GAP;
        }
        ui.button(
            Rect::new(tx, y, bw, ROW),
            label,
            None,
            A(AutoHit::Tab(f)),
            f == p.family,
        );
        tx += bw + u::SMALL_GAP;
    }
    y += ROW + u::SECTION_GAP;
    y = if p.draft.is_some() {
        builder(ui, p, cx, ix, y, iw)
    } else {
        list(ui, p, cx, ix, y, iw)
    };
    y + u::PAD_BOTTOM - top
}

/// A lista do separador: as sugestões (nas regras), cada automação e o «Create…».
fn list(ui: &mut Ui<'_>, p: &mut Panel, cx: &Cx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let mut y = y;
    if p.family == Family::Rule {
        let sugs = visible_suggestions(cx.k, cx.me, &p.dismissed);
        if !sugs.is_empty() {
            ui.caption(tr("kanban.auto.ui.suggested"), x, y, w);
            y += u::META_LINE + u::SMALL_GAP;
        }
        for (i, s) in sugs.iter().enumerate() {
            let list = match &s.spec.kind {
                Kind::Rule {
                    trigger: Trigger::CardAdded { list: Some(l), .. },
                    ..
                } => cx.names.text(&Part::List(*l)),
                _ => String::new(),
            };
            let n = u16::try_from(i).unwrap_or(u16::MAX);
            let box_top = y;
            y += u::GAP;
            let fg = ui.fg();
            let say = tr_with(
                "kanban.auto.ui.suggestion",
                &[("n", &s.times), ("list", &list)],
            );
            y += ui.text(&say, x + u::GAP, y, u::BODY_PX, w - 2.0 * u::GAP, fg) + u::SMALL_GAP;
            y = flow(
                ui,
                &cx.names,
                &segs(&s.spec),
                x + u::GAP,
                y,
                w - 2.0 * u::GAP,
                None,
                None,
            );
            y += u::SMALL_GAP;
            let create = tr("kanban.auto.ui.suggest_create");
            let cw = ui.button_w(create, false);
            ui.primary(
                Rect::new(x + u::GAP, y, cw, ROW),
                create,
                A(AutoHit::Suggest(n, true)),
            );
            let dismiss = tr("kanban.auto.ui.suggest_dismiss");
            let dw = ui.button_w(dismiss, false);
            let r = Rect::new(x + u::GAP + cw + u::GAP, y, dw, ROW);
            ui.button(r, dismiss, None, A(AutoHit::Suggest(n, false)), false);
            y += ROW + u::GAP;
            let ring = ui.accent();
            let frame = Rect::new(x, box_top, w, y - box_top);
            stroke_rounded_rect(
                ui.scene,
                frame,
                m::CARD_RADIUS,
                StrokeToken::Default.px(),
                ring,
            );
            p.lines.push(format!("suggest:{}", cx.names.line(&s.spec)));
            y += u::GAP;
        }
    }
    let rows = of_family(cx.k, p.family);
    if rows.is_empty() {
        let meta = ui.meta();
        y += ui.text(tr("kanban.auto.ui.empty"), x, y, u::BODY_PX, w, meta) + u::GAP;
    }
    for (a, s) in &rows {
        y = row(ui, cx, *a, s, x, y, w);
        p.lines.push(format!(
            "{}:{}:{}",
            family_key(p.family),
            if s.enabled { "on" } else { "off" },
            cx.names.line(s)
        ));
    }
    let create = tr(&format!("kanban.auto.ui.create.{}", family_key(p.family)));
    let cw = ui.button_w(create, true);
    let r = Rect::new(x, y + u::GAP, cw, ROW);
    fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, ui.accent());
    let ink = ui.c(T::KanbanLabelInkBold);
    ph2d_editor_core::paint::paint_icon(
        ui.scene,
        IconId::Plus,
        Rect::new(
            r.x + m::BUTTON_PAD_X,
            r.y + (r.h - m::ICON_PX) / 2.0,
            m::ICON_PX,
            m::ICON_PX,
        ),
        ink,
        StrokeToken::Default.px(),
    );
    let tx = r.x + m::BUTTON_PAD_X + m::ICON_PX + u::GAP;
    ui.strong(
        create,
        u::Band::new(tx, r.y, r.x + r.w - tx, r.h),
        u::BODY_PX,
        ink,
    );
    ui.hit(r, A(AutoHit::Create));
    r.y + r.h
}

/// Uma automação da lista: a frase e, à direita, ligar/desligar, (correr), editar, apagar.
fn row(
    ui: &mut Ui<'_>,
    cx: &Cx<'_>,
    a: ph2d_kanban_model::AutomationId,
    s: &Spec,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    let on_off = tr(if s.enabled {
        "kanban.auto.ui.on"
    } else {
        "kanban.auto.ui.off"
    });
    let mut buttons = vec![(on_off.to_owned(), AutoHit::Toggle(a), s.enabled)];
    if matches!(s.kind, Kind::BoardButton { .. }) {
        buttons.push((tr("kanban.auto.ui.run").to_owned(), AutoHit::Run(a), false));
    }
    buttons.push((
        tr("kanban.auto.ui.edit").to_owned(),
        AutoHit::Edit(a),
        false,
    ));
    buttons.push((
        tr("kanban.auto.ui.delete").to_owned(),
        AutoHit::Delete(a),
        false,
    ));
    let widths: Vec<f32> = buttons.iter().map(|b| ui.button_w(&b.0, false)).collect();
    let bw: f32 = widths.iter().sum::<f32>() + u::SMALL_GAP * (widths.len() as f32);
    let text_w = (w - bw - 2.0 * u::GAP).max(w / 2.0);
    let mut scratch = VectorScene::new();
    let mut none = Vec::new();
    let h = {
        let mut s2 = Ui {
            scene: &mut scratch,
            ts: ui.ts,
            theme: ui.theme,
            hits: &mut none,
            hover: None,
        };
        flow(&mut s2, &cx.names, &segs(s), 0.0, 0.0, text_w, None, None)
    }
    .max(ROW);
    let card = Rect::new(x, y, w, h + 2.0 * u::GAP);
    let bg = ui.c(T::KanbanCardRaised);
    fill_rounded_rect(ui.scene, card, m::CARD_RADIUS, bg);
    flow(
        ui,
        &cx.names,
        &segs(s),
        x + u::GAP,
        y + u::GAP,
        text_w,
        None,
        None,
    );
    let mut bx = x + w - u::GAP - bw + u::SMALL_GAP;
    for ((label, hit, on), bw) in buttons.iter().zip(widths) {
        ui.button(
            Rect::new(bx, y + u::GAP, bw, ROW),
            label,
            None,
            A(*hit),
            *on,
        );
        bx += bw + u::SMALL_GAP;
    }
    card.y + card.h + u::GAP
}

/// ⭐ O construtor em frase: os chips, «Add condition»/«Add action», «Save»/«Cancel».
fn builder(ui: &mut Ui<'_>, p: &mut Panel, cx: &Cx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let Some(d) = p.draft.as_ref() else {
        return y;
    };
    let meta = ui.meta();
    let mut y = y + ui.text(tr("kanban.auto.ui.builder"), x, y, u::SMALL_PX, w, meta) + u::GAP;
    let sg = segs(&d.spec);
    let open = d.open.or(d.typed.as_ref().map(|t| t.0));
    y = flow(ui, &cx.names, &sg, x, y, w, Some(()), open) + u::GAP;
    p.lines.push(format!("draft:{}", cx.names.line(&d.spec)));
    let mut bx = x;
    let mut spec = d.spec.clone();
    let has_conditions = conditions_mut(&mut spec).is_some();
    for (key, hit, show) in [
        (
            "kanban.auto.ui.add_condition",
            AutoHit::AddCondition,
            has_conditions,
        ),
        ("kanban.auto.ui.add_action", AutoHit::AddAction, true),
    ] {
        if !show {
            continue;
        }
        let label = tr(key);
        let bw = ui.button_w(label, true);
        ui.button(
            Rect::new(bx, y, bw, ROW),
            label,
            Some(IconId::Plus),
            A(hit),
            false,
        );
        bx += bw + u::GAP;
    }
    y += ROW + u::SECTION_GAP;
    let save = tr("kanban.auto.ui.save");
    let sw = ui.button_w(save, false);
    ui.primary(Rect::new(x, y, sw, ROW), save, A(AutoHit::Save));
    let cancel = tr("kanban.auto.ui.cancel");
    let cw = ui.button_w(cancel, false);
    ui.button(
        Rect::new(x + sw + u::GAP, y, cw, ROW),
        cancel,
        None,
        A(AutoHit::Cancel),
        false,
    );
    y + ROW
}

/// A frase a correr da esquerda para a direita com quebra: as palavras soltas, os valores em chip.
/// `live` = os chips clicam (o construtor); `open` = o chip com o menu aberto (o anel).
#[allow(clippy::too_many_arguments)]
fn flow(
    ui: &mut Ui<'_>,
    names: &Names<'_>,
    sg: &[Seg],
    x: f32,
    y: f32,
    w: f32,
    live: Option<()>,
    open: Option<super::Slot>,
) -> f32 {
    let space = ui.width(" ", u::BODY_PX);
    let line = ROW;
    let chip_h = line - 2.0 * u::SMALL_GAP;
    let (mut cx, mut cy) = (x, y);
    let fg = ui.fg();
    for g in sg {
        let text = names.text(&g.part);
        match g.slot {
            Some(slot) => {
                let tw = (ui.width(&text, u::BODY_PX) + 2.0 * m::LABEL_PAD_X).min(w);
                if cx + tw > x + w && cx > x {
                    (cx, cy) = (x, cy + line);
                }
                let r = Rect::new(cx, cy + u::SMALL_GAP, tw, chip_h);
                let hit = A(AutoHit::Chip(slot));
                let fill = if live.is_some() && ui.hover == Some(hit) {
                    T::KanbanHeaderBtn
                } else {
                    T::KanbanSlot
                };
                fill_rounded_rect(ui.scene, r, m::LABEL_RADIUS, ui.c(fill));
                if open == Some(slot) {
                    let ring = ui.accent();
                    stroke_rounded_rect(ui.scene, r, m::LABEL_RADIUS, m::FOCUS_RING, ring);
                }
                ui.line(
                    &text,
                    u::Band::new(r.x + m::LABEL_PAD_X, cy, tw - m::LABEL_PAD_X, line),
                    u::BODY_PX,
                    fg,
                );
                if live.is_some() {
                    ui.hit(r, hit);
                }
                cx += tw + space;
            }
            None => {
                for word in text.split_whitespace() {
                    let ww = ui.width(word, u::BODY_PX);
                    if cx + ww > x + w && cx > x {
                        (cx, cy) = (x, cy + line);
                    }
                    ui.line(word, u::Band::new(cx, cy, ww + space, line), u::BODY_PX, fg);
                    cx += ww + space;
                }
            }
        }
    }
    cy + line
}

/// O menu (ou a caixa) do chip aberto, ancorado por baixo dele e preso à área.
fn chooser(ui: &mut Ui<'_>, p: &mut Panel, cx: &Cx<'_>, area: Rect) {
    p.picks.clear();
    let Some(d) = p.draft.as_ref() else { return };
    let Some(slot) = d.open.or(d.typed.as_ref().map(|t| t.0)) else {
        return;
    };
    let anchor = ui
        .hits
        .iter()
        .rev()
        .find(|(_, h)| *h == A(AutoHit::Chip(slot)))
        .map(|(r, _)| *r);
    let Some(a) = anchor else { return };
    let pad = u::POP_PAD;
    if let Some((_, _, f)) = d.typed.as_ref() {
        let h = 2.0 * pad + u::META_LINE + u::SMALL_GAP + 2.0 * ROW + u::GAP;
        let r = place(a, area, u::POP_W, h);
        backdrop(ui, r);
        ui.hit(r, A(AutoHit::Menu));
        let iw = r.w - 2.0 * pad;
        ui.caption(tr("kanban.auto.ui.text"), r.x + pad, r.y + pad, iw);
        let fr = Rect::new(r.x + pad, r.y + pad + u::META_LINE + u::SMALL_GAP, iw, ROW);
        ui.field(fr, f, "");
        ui.hit(fr, A(AutoHit::Typed));
        let save = tr("kanban.auto.ui.save");
        let sw = ui.button_w(save, false);
        ui.primary(
            Rect::new(r.x + pad, fr.y + fr.h + u::GAP, sw, ROW),
            save,
            A(AutoHit::TypedSave),
        );
        return;
    }
    let sg = segs(&d.spec);
    let opts = choices::options(slot, &d.spec, &sg, cx.k, &cx.people, cx.me);
    if opts.is_empty() {
        return;
    }
    // Colunas quando as linhas não cabem na altura da área (a régua é o ecrã, não um número).
    let room = (area.h - 2.0 * u::GAP - 2.0 * pad).max(m::MENU_ROW_H);
    let per_col = ((room / m::MENU_ROW_H).floor() as usize).max(1);
    let cols = opts.len().div_ceil(per_col);
    let rows = opts.len().min(per_col);
    let col_w = u::POP_W - 2.0 * pad;
    let w = col_w * cols as f32 + 2.0 * pad;
    let h = rows as f32 * m::MENU_ROW_H + 2.0 * pad;
    let r = place(a, area, w, h);
    backdrop(ui, r);
    ui.hit(r, A(AutoHit::Menu));
    let fg = ui.fg();
    for (i, (shown, pick)) in opts.into_iter().enumerate() {
        let (col, rowi) = (i / per_col, i % per_col);
        let rr = Rect::new(
            r.x + pad + col as f32 * col_w,
            r.y + pad + rowi as f32 * m::MENU_ROW_H,
            col_w,
            m::MENU_ROW_H,
        );
        let hit = A(AutoHit::Choice(u16::try_from(i).unwrap_or(u16::MAX)));
        if ui.hover == Some(hit) {
            let raised = ui.c(T::KanbanCardRaised);
            fill_rounded_rect(ui.scene, rr, m::LABEL_RADIUS, raised);
        }
        let text = match &shown {
            Shown::Word(w) => tr(w.key()).to_owned(),
            Shown::Part(p) => cx.names.text(p),
            Shown::Remove => tr("kanban.auto.ui.remove").to_owned(),
        };
        let ink = if matches!(shown, Shown::Remove) {
            ui.c(T::KanbanDueOver)
        } else {
            fg
        };
        ui.line(
            &text,
            u::Band::new(rr.x + m::MENU_PAD / 2.0, rr.y, rr.w - m::MENU_PAD, rr.h),
            u::BODY_PX,
            ink,
        );
        ui.hit(rr, hit);
        p.lines.push(format!("choice:{text}"));
        p.picks.push(pick);
    }
}

fn backdrop(ui: &mut Ui<'_>, r: Rect) {
    crate::paint::shadow(
        ui.scene,
        r,
        m::MENU_RADIUS,
        ui.c(T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    let bg = ui.c(T::KanbanCard);
    fill_rounded_rect(ui.scene, r, m::MENU_RADIUS, bg);
    let ring = ui.c(T::KanbanSlot);
    stroke_rounded_rect(ui.scene, r, m::MENU_RADIUS, StrokeToken::Default.px(), ring);
}

fn place(a: Rect, area: Rect, w: f32, h: f32) -> Rect {
    let x = a.x.min(area.x + area.w - w - u::GAP).max(area.x + u::GAP);
    let below = a.y + a.h + u::SMALL_GAP;
    let y = if below + h <= area.y + area.h - u::GAP {
        below
    } else {
        (area.y + area.h - h - u::GAP).max(area.y + u::GAP)
    };
    Rect::new(x, y, w, h)
}
