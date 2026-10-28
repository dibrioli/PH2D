//! J2 — **o cartão que abre o objecto** (os «Power-Ups» do PH2D no verso): a secção «Project links»
//! (cada ligação com o ícone do tipo, o nome ACTUAL — ou «quebrada» com o nome de reserva —, clicar
//! abre-a pelo dono dela, `×` tira-a), o popover «Scene object» (o catálogo do projecto, com
//! pesquisa), os botões do Butler e «Convert to sticky note».

use super::click::{back_mut, read};
use super::paint::Ctx;
use super::ui::{self as u, Ui};
use super::{Back, BackHit, PopHit, ROW, Side, write};
use crate::docs::KanbanDocs;
use crate::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::paint_icon;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_links::{Catalog, Kind, Target, kind_of, link_op, resolve};
use ph2d_kanban_model::{CardId, CardRef, KanbanOp, RecordId};
use ph2d_tokens::StrokeToken;

/// Há um verso aberto (só então o catálogo se monta: ele lê os elementos de cada Quadro).
pub(crate) fn any_open(docs: &KanbanDocs) -> bool {
    docs.tabs.iter().any(|t| t.back.is_some())
        || docs.canvas.borrow().tabs.values().any(|t| t.back.is_some())
}

/// ⭐ O catálogo do projecto chega a cada verso aberto (a família chama-o a cada quadro).
pub(crate) fn follow(docs: &mut KanbanDocs, cat: &Catalog) {
    let put = |b: &mut Back| {
        if b.catalog != *cat {
            b.catalog.clone_from(cat);
        }
    };
    for t in &mut docs.tabs {
        if let Some(b) = t.back.as_deref_mut() {
            put(b);
        }
    }
    for t in docs.canvas.borrow_mut().tabs.values_mut() {
        if let Some(b) = t.back.as_deref_mut() {
            put(b);
        }
    }
}

fn icon_of(k: Kind) -> IconId {
    match k {
        Kind::Object => IconId::Cube,
        Kind::Script => IconId::Script,
        Kind::Macro => IconId::Macro,
        Kind::Span => IconId::Film,
        Kind::Board => IconId::LayoutGrid,
        Kind::Element => IconId::Place,
    }
}

fn kind_key(k: Kind) -> &'static str {
    match k {
        Kind::Object => "kanban.links.kind.object",
        Kind::Script => "kanban.links.kind.script",
        Kind::Macro => "kanban.links.kind.macro",
        Kind::Span => "kanban.links.kind.span",
        Kind::Board => "kanban.links.kind.board",
        Kind::Element => "kanban.links.kind.element",
    }
}

/// O nome mostrado (o trecho leva os quadros).
fn shown(t: &Target, name: &str) -> String {
    match t {
        Target::Span { from, to, .. } if to > from => {
            let f = tr_with("kanban.links.span_frames", &[("from", from), ("to", to)]);
            format!("{name} \u{b7} {f}")
        }
        _ => name.to_owned(),
    }
}

/// «Project links»: as ligações do cartão, resolvidas contra o catálogo.
pub(crate) fn links(ui: &mut Ui<'_>, back: &Back, cx: &Ctx<'_>, x: f32, y: f32, w: f32) -> f32 {
    let ls = cx.k.links_of(cx.card.id);
    if ls.is_empty() {
        return y;
    }
    ui.heading(tr("kanban.links.title"), x, y, w);
    let mut y = y + ROW + u::GAP;
    for l in ls {
        let r = resolve(&back.catalog, l);
        let row = Rect::new(x, y, w - ROW - u::GAP, ROW);
        let (fg, meta) = (ui.fg(), ui.meta());
        let ink = if r.live { fg } else { meta };
        let ir = Rect::new(
            row.x + m::BUTTON_PAD_X,
            y + (ROW - m::ICON_PX) / 2.0,
            m::ICON_PX,
            m::ICON_PX,
        );
        paint_icon(
            ui.scene,
            icon_of(r.kind),
            ir,
            ink,
            StrokeToken::Default.px(),
        );
        let name = if r.live {
            shown(&l.target, &r.name)
        } else {
            tr_with("kanban.links.broken", &[("name", &r.name)])
        };
        let kind = tr(kind_key(r.kind));
        let kw = ui.width(kind, u::SMALL_PX);
        let tx = ir.x + m::ICON_PX + u::GAP;
        let tw = (row.x + row.w - tx - kw - u::GAP).max(0.0);
        ui.strong(&name, tx, y, ROW, u::BODY_PX, tw, ink);
        ui.line(
            kind,
            row.x + row.w - kw - m::BUTTON_PAD_X / 2.0,
            y,
            ROW,
            u::SMALL_PX,
            kw + u::GAP,
            meta,
        );
        ui.hit(row, BackHit::LinkOpen(l.id));
        let x_btn = Rect::new(x + w - ROW, y, ROW, ROW);
        let bg = ui.c(ph2d_tokens::ColorToken::KanbanCard);
        ui.icon_button(x_btn, IconId::Close, BackHit::LinkRemove(l.id), false, bg);
        y += ROW + u::SMALL_GAP;
    }
    y + u::SECTION_GAP
}

/// O que o catálogo oferece a `card` com a pesquisa `q` (sem caixa), pela ordem dos tipos; as
/// ligações que o cartão já tem saem.
pub(crate) fn entries(cat: &Catalog, have: &[Target], q: &str) -> Vec<(Target, String)> {
    let q = q.trim().to_lowercase();
    let objects = cat
        .objects
        .iter()
        .map(|(i, n)| (Target::Object(*i), n.clone()));
    let scripts = cat
        .scripts
        .iter()
        .map(|(i, n)| (Target::Script(*i), n.clone()));
    let macros = cat
        .macros
        .iter()
        .map(|n| (Target::Macro(n.clone()), n.clone()));
    let strips = cat.strips.iter().map(|(i, n)| {
        let t = Target::Span {
            object: None,
            strip: Some(*i),
            from: 0,
            to: 0,
        };
        (t, n.clone())
    });
    let boards = cat.boards.iter().map(|(i, b)| {
        let t = Target::Board {
            board: *i,
            element: None,
        };
        (t, b.name.clone())
    });
    objects
        .chain(scripts)
        .chain(macros)
        .chain(strips)
        .chain(boards)
        .filter(|(t, n)| !have.contains(t) && (q.is_empty() || n.to_lowercase().contains(&q)))
        .collect()
}

/// O popover «Scene object»: a pesquisa e as entradas que cabem na área (o ecrã é a régua).
pub(crate) fn pop(
    ui: &mut Ui<'_>,
    back: &Back,
    cx: &Ctx<'_>,
    f: &ph2d_editor_core::widget::LineField,
    x: f32,
    y: f32,
    w: f32,
) -> f32 {
    ui.caption(tr("kanban.links.title"), x, y, w);
    let r = Rect::new(x, y + u::META_LINE + u::SMALL_GAP, w, ROW);
    ui.field(r, f, tr("kanban.links.add"));
    ui.hit(r, BackHit::Pop(PopHit::Input));
    let mut y = r.y + r.h + u::GAP;
    let have: Vec<Target> =
        cx.k.links_of(cx.card.id)
            .into_iter()
            .map(|l| l.target.clone())
            .collect();
    let room = ((back.area.h - 2.0 * u::MODAL_TOP - (y - r.y) - ROW) / ROW)
        .floor()
        .max(1.0);
    let fit = room as usize;
    for (i, (t, name)) in entries(&back.catalog, &have, &f.text)
        .into_iter()
        .take(fit)
        .enumerate()
    {
        let row = Rect::new(x, y, w, ROW);
        let h = BackHit::Pop(PopHit::Scene(u16::try_from(i).unwrap_or(u16::MAX)));
        if ui.hover == Some(h) {
            let raised = ui.c(ph2d_tokens::ColorToken::KanbanCardRaised);
            ph2d_editor_core::paint::fill_rounded_rect(ui.scene, row, m::LABEL_RADIUS, raised);
        }
        let fg = ui.fg();
        let ir = Rect::new(
            x + u::SMALL_GAP,
            y + (ROW - m::ICON_PX) / 2.0,
            m::ICON_PX,
            m::ICON_PX,
        );
        paint_icon(
            ui.scene,
            icon_of(kind_of(&t)),
            ir,
            fg,
            StrokeToken::Default.px(),
        );
        let tx = ir.x + m::ICON_PX + u::GAP;
        ui.line(&name, tx, y, ROW, u::BODY_PX, x + w - tx, fg);
        ui.hit(row, h);
        y += ROW;
    }
    y
}

/// ⭐ Os cliques de J2 no verso.
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, card: CardId, h: BackHit) {
    match h {
        BackHit::Side(Side::SceneObject) => {
            if let Some(b) = back_mut(docs, id) {
                let same = matches!(b.pop, Some(super::Pop::Scene(_)));
                b.pop = (!same).then(|| super::Pop::Scene(Default::default()));
            }
        }
        BackHit::Side(Side::AddButton) => {
            crate::auto::open(docs, id, ph2d_kanban_auto::Family::CardButton, true);
        }
        BackHit::Side(Side::ToNote) => {
            if let Some(kanban) = docs.kanban_of(id) {
                ph2d_editor_core::kanban_links::request_card_to_note(CardRef { kanban, card });
                super::close(docs, id);
            }
        }
        BackHit::CardButton(a) => {
            crate::auto::buttons::press_card(docs, id, a, card);
        }
        BackHit::LinkOpen(l) => {
            if let Some(Some(t)) = read(docs, id, |k, _| {
                k.project_links.get(&l).map(|l| l.target.clone())
            }) {
                ph2d_editor_core::kanban_links::open(&t);
            }
        }
        BackHit::LinkRemove(l) => {
            write::logged(docs, id, |_| {
                Some(vec![KanbanOp::Delete(RecordId::ProjectLink(l))])
            });
        }
        BackHit::Pop(PopHit::Scene(n)) => pick(docs, id, card, usize::from(n)),
        _ => {}
    }
}

/// A entrada `n` do popover vira uma ligação (com o nome de agora como reserva).
fn pick(docs: &mut KanbanDocs, id: u64, card: CardId, n: usize) {
    let Some(b) = back_mut(docs, id) else { return };
    let Some(super::Pop::Scene(f)) = b.pop.as_ref() else {
        return;
    };
    let q = f.text.clone();
    let cat = b.catalog.clone();
    let have: Vec<Target> = read(docs, id, |k, _| {
        k.links_of(card)
            .into_iter()
            .map(|l| l.target.clone())
            .collect()
    })
    .unwrap_or_default();
    let Some((t, name)) = entries(&cat, &have, &q).into_iter().nth(n) else {
        return;
    };
    let now = crate::actions::now();
    write::logged(docs, id, |k| Some(vec![link_op(k, card, t, &name, now)]));
    if let Some(b) = back_mut(docs, id) {
        b.pop = None;
    }
}
