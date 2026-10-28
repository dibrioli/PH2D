//! ⭐ **O rato no verso** — carregar guarda o alvo (os editores e o seletor de datas tomam-no já, para
//! o cursor e a selecção); largar no MESMO alvo é o clique. Cada clique é um verbo do Trello.

use super::outside::Outside;
use super::{Back, BackHit, Edit, EditAt, Pop, PopHit, Side, write};
use crate::docs::KanbanDocs;
use ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS;
use ph2d_editor_core::widget::{DatePickerEvent, LineField, RichEditorEvent};
use ph2d_host::Modifiers;
use ph2d_i18n::tr;
use ph2d_kanban_model::{
    Card, CardId, Cover, CoverFill, FieldKind, FieldValue, Hue, Kanban, LabelColor, Link, Member,
    Moment, RecordId, Shade,
};
use ph2d_text::TextSystem;

pub(crate) fn back_mut(docs: &mut KanbanDocs, id: u64) -> Option<&mut Back> {
    docs.tab_mut(id)?.back.as_deref_mut()
}

/// Lê o Kanban da aba e o cartão do verso.
pub(crate) fn read<R>(
    docs: &KanbanDocs,
    id: u64,
    f: impl FnOnce(&Kanban, &Card) -> R,
) -> Option<R> {
    let t = docs.tabs.iter().find(|t| t.id() == id)?;
    let c = t.back.as_ref()?.card;
    let set = docs.set.borrow();
    let k = set.get(t.view.kanban)?;
    let card = k.cards.get(&c)?;
    Some(f(k, card))
}

fn card_of(docs: &mut KanbanDocs, id: u64) -> Option<CardId> {
    back_mut(docs, id).map(|b| b.card)
}

pub(crate) fn people(docs: &KanbanDocs, id: u64) -> Vec<Member> {
    let (team, me) = (docs.team.clone(), docs.me.clone());
    read(docs, id, |k, _| super::people(k, &team, &me)).unwrap_or_default()
}

/// Quantos cliques seguidos no alvo `h` (1, 2 = palavra, 3 = bloco; o 4.º recomeça), à janela do
/// duplo clique do sistema.
fn clicks(b: &mut Back, h: BackHit, now_ns: u128) -> u8 {
    let n = match b.last_down {
        Some((at, t, n)) if at == h && now_ns.saturating_sub(t) <= DOUBLE_CLICK_WINDOW_NS => {
            n % 3 + 1
        }
        _ => 1,
    };
    b.last_down = Some((h, now_ns, n));
    n
}

/// Carregar. `true` = tomado (o verso está aberto e o rato na área). `mods` e `now_ns` chegam ao
/// editor rico: o Shift estende a selecção, o 2.º e o 3.º clique escolhem a palavra e o bloco.
pub(crate) fn down(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    (x, y): (f32, f32),
    mods: Modifiers,
    now_ns: u128,
) -> bool {
    if crate::auto::is_open(docs, id) {
        return crate::auto::down(docs, id, (x, y));
    }
    let Some(b) = back_mut(docs, id) else {
        return false;
    };
    if !b.area.contains(x, y) {
        return false;
    }
    let h = b.hit(x, y);
    b.press = h;
    let keeps_edit = matches!(
        h,
        Some(BackHit::EditField | BackHit::EditSave | BackHit::EditCancel)
    );
    if !keeps_edit && b.edit.is_some() {
        commit_edit(docs, id, false);
    }
    let Some(b) = back_mut(docs, id) else {
        return true;
    };
    if b.desc.editing
        && !matches!(
            h,
            Some(BackHit::DescArea | BackHit::DescSave | BackHit::DescCancel)
        )
    {
        super::keys::save_description(docs, id);
    }
    let Some(b) = back_mut(docs, id) else {
        return true;
    };
    if b.comment.editing
        && b.comment.doc().is_empty()
        && b.comment_edit.is_none()
        && !matches!(h, Some(BackHit::CommentArea | BackHit::Mention(_)))
    {
        b.comment.editing = false;
    }
    match h {
        Some(BackHit::DescArea) => {
            let n = clicks(b, BackHit::DescArea, now_ns);
            if let Some(r) = b.rect_of(BackHit::DescArea) {
                let ev = b.desc.pointer_down(ts, r, x, y, mods, n);
                if let Some(RichEditorEvent::OpenLink(u)) = ev {
                    b.opened.push(Outside::Link(u));
                }
            }
        }
        Some(BackHit::CommentArea) => {
            let n = clicks(b, BackHit::CommentArea, now_ns);
            if let Some(r) = b.rect_of(BackHit::CommentArea) {
                let _ = b.comment.pointer_down(ts, r, x, y, mods, n);
            }
        }
        Some(BackHit::Pop(PopHit::Picker)) => {
            let r = b.rect_of(BackHit::Pop(PopHit::Picker));
            let events = match (&mut b.pop, r) {
                (Some(Pop::Dates(_, p)), Some(r)) => p.widget.pointer_down(r, x, y),
                _ => Vec::new(),
            };
            picker_events(docs, id, events);
        }
        _ => {}
    }
    true
}

/// Movimento: o realce, e o arrasto da selecção num editor.
pub(crate) fn moved(docs: &mut KanbanDocs, id: u64, ts: &mut TextSystem, x: f32, y: f32) -> bool {
    if crate::auto::is_open(docs, id) {
        return crate::auto::moved(docs, id, x, y);
    }
    let Some(b) = back_mut(docs, id) else {
        return false;
    };
    if !b.area.contains(x, y) {
        return false;
    }
    match b.press {
        Some(BackHit::DescArea) => {
            if let Some(r) = b.rect_of(BackHit::DescArea) {
                b.desc.pointer_drag(ts, r, x, y);
            }
        }
        Some(BackHit::CommentArea) => {
            if let Some(r) = b.rect_of(BackHit::CommentArea) {
                b.comment.pointer_drag(ts, r, x, y);
            }
        }
        _ => {}
    }
    b.hover = b.hit(x, y);
    if let Some(r) = b.rect_of(BackHit::DescArea) {
        b.desc.pointer_move(r, x, y);
    }
    let picker = b.rect_of(BackHit::Pop(PopHit::Picker));
    if let (Some(Pop::Dates(_, p)), Some(r)) = (&mut b.pop, picker) {
        p.widget.pointer_move(r, x, y);
    }
    true
}

/// Largar: o clique, se acabou no alvo onde começou.
pub(crate) fn up(docs: &mut KanbanDocs, id: u64, x: f32, y: f32) -> bool {
    if crate::auto::is_open(docs, id) {
        return crate::auto::up(docs, id, x, y);
    }
    let Some(b) = back_mut(docs, id) else {
        return false;
    };
    b.desc.pointer_up();
    b.comment.pointer_up();
    let press = b.press.take();
    let now = b.hit(x, y);
    if press.is_some()
        && press == now
        && let Some(h) = now
    {
        click(docs, id, h);
    }
    true
}

/// A roda rola o verso inteiro (o modal anda com a página, o Trello): `dy` px de ecrã, positivo =
/// ver mais abaixo ([`crate::pointer::wheel_px`] converte a roda).
pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, dy: f32) -> bool {
    if crate::auto::is_open(docs, id) {
        return crate::auto::wheel(docs, id, dy);
    }
    let Some(b) = back_mut(docs, id) else {
        return false;
    };
    let max = (b.content_h + 2.0 * super::ui::MODAL_TOP - b.area.h).max(0.0);
    b.scroll = (b.scroll + dy).clamp(0.0, max);
    true
}

fn open_pop(docs: &mut KanbanDocs, id: u64, pop: Pop) {
    if let Some(b) = back_mut(docs, id) {
        let same = b
            .pop
            .as_ref()
            .is_some_and(|p| std::mem::discriminant(p) == std::mem::discriminant(&pop));
        b.pop_focus = false;
        b.pop = if same { None } else { Some(pop) };
    }
}

fn edit(docs: &mut KanbanDocs, id: u64, at: EditAt, text: &str) {
    if let Some(b) = back_mut(docs, id) {
        b.pop = None;
        b.edit = Some(Edit {
            at,
            field: LineField::new(text),
        });
    }
}

/// O seletor de datas do cartão (`None`) ou de um item.
pub(crate) fn dates_pop(
    docs: &mut KanbanDocs,
    id: u64,
    item: Option<ph2d_kanban_model::CheckItemId>,
) {
    let now = crate::actions::now();
    let Some((card, start, due)) = read(docs, id, |k, c| match item {
        None => (c.id, c.start, c.due.map(|d| (d.at, d.remind_before_min))),
        Some(i) => (
            c.id,
            None,
            k.items.get(&i).and_then(|i| i.due).map(|d| (d, None)),
        ),
    }) else {
        return;
    };
    let picker = super::dates::picker(card, start, due, now);
    open_pop(docs, id, Pop::Dates(item, Box::new(picker)));
}

pub(crate) fn picker_events(docs: &mut KanbanDocs, id: u64, events: Vec<DatePickerEvent>) {
    for ev in events {
        let Some(b) = back_mut(docs, id) else {
            return;
        };
        let card = b.card;
        let Some(Pop::Dates(item, p)) = b.pop.as_mut() else {
            return;
        };
        let item = *item;
        match ev {
            DatePickerEvent::Month(e) => super::dates::month_event(p, e),
            DatePickerEvent::Pick { slot, date } => p.picked(slot, date),
            DatePickerEvent::TextCommitted { field, text } => super::dates::typed(p, field, &text),
            DatePickerEvent::Toggled { .. } | DatePickerEvent::Reminder(_) => {}
            DatePickerEvent::Save | DatePickerEvent::Remove => {
                let (start, due) = if ev == DatePickerEvent::Save {
                    super::dates::read(p)
                } else {
                    (None, None)
                };
                b.pop = None;
                match item {
                    None => {
                        write::set_dates(docs, id, card, start, due);
                    }
                    Some(i) => {
                        write::item(docs, id, i, |it| it.due = due.map(|d| d.0));
                    }
                }
            }
            DatePickerEvent::Close => b.pop = None,
        }
    }
}

/// Grava o campo aberto (`again` = um item novo deixa o compositor aberto para o seguinte).
pub(crate) fn commit_edit(docs: &mut KanbanDocs, id: u64, again: bool) {
    let Some(b) = back_mut(docs, id) else {
        return;
    };
    let Some(e) = b.edit.take() else {
        return;
    };
    let card = b.card;
    let text = e.field.text.trim().to_owned();
    let kind = read(docs, id, |k, _| match &e.at {
        EditAt::Field(f) => k.fields.get(f).map(|d| d.kind.clone()),
        _ => None,
    })
    .flatten();
    match e.at {
        EditAt::Title if !text.is_empty() => {
            write::card(docs, id, card, |c| c.title = text);
        }
        EditAt::Location => {
            write::card(docs, id, card, |c| c.location = text);
        }
        EditAt::ClTitle(cl) if !text.is_empty() => {
            write::checklist(docs, id, cl, |c| c.title = text);
        }
        EditAt::Item(i) if !text.is_empty() => {
            write::item(docs, id, i, |it| it.text = text);
        }
        EditAt::NewItem(cl) => {
            if !text.is_empty() {
                write::add_item(docs, id, cl, text);
            }
            if again && let Some(b) = back_mut(docs, id) {
                b.edit = Some(Edit {
                    at: EditAt::NewItem(cl),
                    field: LineField::default(),
                });
            }
        }
        EditAt::Field(f) => {
            let today =
                ph2d_kanban_dates::civil(crate::actions::now().0, &super::dates::zone()).date;
            let v = match kind {
                _ if text.is_empty() => None,
                Some(FieldKind::Number) => text
                    .replace(',', ".")
                    .parse::<f64>()
                    .ok()
                    .map(FieldValue::Number),
                Some(FieldKind::Date) => super::dates::parse_date(&text, today)
                    .and_then(|d| ph2d_kanban_dates::start_of_day(d, &super::dates::zone()))
                    .map(|ms| FieldValue::Date(Moment(ms))),
                _ => Some(FieldValue::Text(text)),
            };
            write::set_field(docs, id, card, f, v);
        }
        _ => {}
    }
}

/// O texto de um campo para abrir em edição.
fn field_text(docs: &KanbanDocs, id: u64, f: ph2d_kanban_model::FieldId) -> String {
    read(docs, id, |_, c| match c.fields.get(&f) {
        Some(FieldValue::Text(s)) => s.clone(),
        Some(FieldValue::Number(n)) => format!("{n}"),
        Some(FieldValue::Date(d)) => {
            let c = ph2d_kanban_dates::civil(d.0, &super::dates::zone()).date;
            format!("{}/{}/{}", c.month, c.day, c.year)
        }
        _ => String::new(),
    })
    .unwrap_or_default()
}

/// Um valor de campo que se muda num clique (caixa, lista suspensa) — senão abre a edição.
fn field_click(docs: &mut KanbanDocs, id: u64, card: CardId, f: ph2d_kanban_model::FieldId) {
    let Some((kind, cur)) = read(docs, id, |k, c| {
        (
            k.fields.get(&f).map(|d| d.kind.clone()),
            c.fields.get(&f).cloned(),
        )
    }) else {
        return;
    };
    match kind {
        Some(FieldKind::Checkbox) => {
            let on = matches!(cur, Some(FieldValue::Checkbox(true)));
            write::set_field(docs, id, card, f, Some(FieldValue::Checkbox(!on)));
        }
        Some(FieldKind::Dropdown(opts)) => {
            // O próximo da lista; depois do último, nenhum.
            let at = match cur {
                Some(FieldValue::Option(o)) => {
                    opts.iter().position(|x| x.id == o).map_or(0, |i| i + 1)
                }
                _ => 0,
            };
            let v = opts.get(at).map(|o| FieldValue::Option(o.id));
            write::set_field(docs, id, card, f, v);
        }
        Some(_) => {
            let t = field_text(docs, id, f);
            edit(docs, id, EditAt::Field(f), &t);
        }
        None => {}
    }
}

/// ⭐ Um clique no verso.
#[allow(clippy::too_many_lines)]
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, h: BackHit) {
    let Some(card) = card_of(docs, id) else {
        return;
    };
    // Um popover aberto: um clique fora dele só o fecha.
    if let Some(b) = back_mut(docs, id)
        && b.pop.is_some()
        && !matches!(h, BackHit::Pop(_))
    {
        let anchor = b.pop.as_ref().map(Pop::anchor);
        b.pop = None;
        if anchor == Some(h) || h == BackHit::Inside || h == BackHit::Scrim {
            return;
        }
    }
    match h {
        BackHit::Scrim | BackHit::Close => super::close(docs, id),
        BackHit::Inside
        | BackHit::EditField
        | BackHit::DescArea
        | BackHit::DescMenu
        | BackHit::CommentArea
        | BackHit::CommentMenu => {}
        BackHit::Title => {
            let t = read(docs, id, |_, c| c.title.clone()).unwrap_or_default();
            edit(docs, id, EditAt::Title, &t);
        }
        BackHit::ListLink => open_pop(docs, id, Pop::Move),
        BackHit::Side(s @ (Side::Move | Side::Copy | Side::Mirror | Side::FromTemplate)) => {
            dialog(docs, id, card, s);
        }
        BackHit::Watch | BackHit::Side(Side::Watch) => {
            write::toggle_watch(docs, id, card);
        }
        BackHit::CoverButton | BackHit::Side(Side::Cover) => open_pop(docs, id, Pop::Cover),
        BackHit::MemberAdd | BackHit::Side(Side::Members) => open_pop(docs, id, Pop::Members(None)),
        BackHit::LabelAdd | BackHit::Side(Side::Labels) => open_pop(docs, id, Pop::Labels),
        BackHit::Dates | BackHit::Side(Side::Dates) => dates_pop(docs, id, None),
        BackHit::DueDone => {
            write::toggle_done(docs, id, card);
        }
        BackHit::FieldValue(f) => field_click(docs, id, card, f),
        BackHit::DescEdit => {
            if let Some(b) = back_mut(docs, id) {
                b.desc.editing = true;
                let end = b.desc.doc().end();
                b.desc.set_selection(ph2d_richdoc::Selection::caret(end));
            }
        }
        BackHit::DescSave => super::keys::save_description(docs, id),
        BackHit::DescCancel => super::keys::cancel_description(docs, id),
        BackHit::ClTitle(cl) => {
            let t = read(docs, id, |k, _| {
                k.checklists.get(&cl).map(|c| c.title.clone())
            })
            .flatten()
            .unwrap_or_default();
            edit(docs, id, EditAt::ClTitle(cl), &t);
        }
        BackHit::ClHide(cl) => {
            write::checklist(docs, id, cl, |c| c.hide_checked = !c.hide_checked);
        }
        BackHit::ClDelete(cl) => {
            write::delete_checklist(docs, id, cl);
        }
        BackHit::ItemBox(i) => {
            write::item(docs, id, i, |it| it.done = !it.done);
        }
        BackHit::ItemText(i) => {
            let t = read(docs, id, |k, _| k.items.get(&i).map(|x| x.text.clone()))
                .flatten()
                .unwrap_or_default();
            edit(docs, id, EditAt::Item(i), &t);
        }
        BackHit::ItemMember(i) => open_pop(docs, id, Pop::Members(Some(i))),
        BackHit::ItemDue(i) => dates_pop(docs, id, Some(i)),
        BackHit::ItemConvert(i) => {
            write::convert(docs, id, i);
        }
        BackHit::ItemDelete(i) => {
            write::delete(docs, id, RecordId::CheckItem(i));
        }
        BackHit::AddItem(cl) => edit(docs, id, EditAt::NewItem(cl), ""),
        BackHit::EditSave => {
            let again = back_mut(docs, id).is_some_and(|b| {
                matches!(b.edit.as_ref().map(|e| &e.at), Some(EditAt::NewItem(_)))
            });
            commit_edit(docs, id, again);
        }
        BackHit::EditCancel => {
            if let Some(b) = back_mut(docs, id) {
                b.edit = None;
            }
        }
        BackHit::AttOpen(a) => open_attachment(docs, id, a),
        BackHit::AttCover(a) => {
            let Some(Some((file, cur))) = read(docs, id, |k, c| {
                match k.attachments.get(&a).map(|x| &x.link) {
                    Some(Link::File(f)) => Some((*f, c.cover)),
                    _ => None,
                }
            }) else {
                return;
            };
            let on = cur.is_some_and(|c| c.fill == CoverFill::Image(file));
            let cover = (!on).then_some(Cover {
                fill: CoverFill::Image(file),
                full: false,
            });
            write::set_cover(docs, id, card, cover);
        }
        BackHit::AttDelete(a) => {
            write::delete(docs, id, RecordId::Attachment(a));
        }
        BackHit::AttAdd | BackHit::Side(Side::Attachment) => {
            open_pop(
                docs,
                id,
                Pop::Attach(LineField::default(), LineField::default()),
            );
        }
        BackHit::Location => {
            let t = read(docs, id, |_, c| c.location.clone()).unwrap_or_default();
            edit(docs, id, EditAt::Location, &t);
        }
        BackHit::Details => {
            if let Some(b) = back_mut(docs, id) {
                b.details = !b.details;
            }
        }
        BackHit::CommentSave => super::keys::save_comment(docs, id),
        BackHit::CommentCancel => super::keys::cancel_comment(docs, id),
        BackHit::CommentEdit(c) => {
            let t = read(docs, id, |k, _| k.comments.get(&c).map(|x| x.text.clone()))
                .flatten()
                .unwrap_or_default();
            if let Some(b) = back_mut(docs, id) {
                b.comment_edit = Some(c);
                b.comment.set_doc(ph2d_kanban_md::parse(&t));
                b.comment.editing = true;
                let end = b.comment.doc().end();
                b.comment.set_selection(ph2d_richdoc::Selection::caret(end));
            }
        }
        BackHit::CommentDelete(c) => {
            write::delete(docs, id, RecordId::Comment(c));
        }
        BackHit::React(c, i) => {
            if let Some(e) = super::REACTIONS.get(usize::from(i)) {
                write::react(docs, id, c, e);
            }
        }
        BackHit::Mention(i) => {
            let people = people(docs, id);
            if let Some(b) = back_mut(docs, id)
                && let Some((_, q)) = super::activity::mention_query(&b.comment)
                && let Some(who) = super::activity::candidates(&people, &q).get(usize::from(i))
            {
                super::activity::pick_mention(&mut b.comment, who);
            }
        }
        BackHit::Side(Side::Checklist) => {
            open_pop(
                docs,
                id,
                Pop::Checklist(LineField::new(tr("kanban.back.checklist_default"))),
            );
        }
        BackHit::Side(Side::Fields) => {
            open_pop(
                docs,
                id,
                Pop::Fields(LineField::default(), 0, LineField::default()),
            );
        }
        BackHit::Side(Side::Template) => {
            write::card(docs, id, card, |c| c.template = !c.template);
        }
        BackHit::Side(Side::Archive) => {
            let archived = read(docs, id, |_, c| c.archived).unwrap_or(false);
            // Arquivar leva os espelhos do cartão (o Trello); restaurar só o devolve.
            if archived {
                write::card(docs, id, card, |c| c.archived = false);
            } else {
                crate::w6::gesture::archive_row(docs, id, card);
            }
        }
        BackHit::Side(Side::Delete) => open_pop(docs, id, Pop::Delete),
        BackHit::Side(Side::Share) => open_pop(docs, id, Pop::Share),
        BackHit::Side(Side::Vote) => {
            write::toggle_vote(docs, id, card);
        }
        BackHit::Pop(p) => super::keys::pop_click(docs, id, card, p),
        BackHit::Side(Side::SceneObject | Side::AddButton | Side::ToNote)
        | BackHit::CardButton(_)
        | BackHit::LinkOpen(_)
        | BackHit::LinkRemove(_) => super::scene::click(docs, id, card, h),
        BackHit::Auto(_) => {}
    }
}

/// W6 — os diálogos de mover, copiar, espelhar e «Create card from template», ancorados no botão.
fn dialog(docs: &mut KanbanDocs, id: u64, card: CardId, s: Side) {
    use crate::w6::DialogKind as K;
    let Some(here) = docs.kanban_of(id) else {
        return;
    };
    let r = ph2d_kanban_model::CardRef { kanban: here, card };
    let anchor = back_mut(docs, id)
        .and_then(|b| {
            b.hits
                .iter()
                .rev()
                .find(|(_, h)| *h == BackHit::Side(s))
                .map(|(r, _)| *r)
        })
        .unwrap_or(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 0.0, 0.0));
    let list = read(docs, id, |_, c| c.list);
    let kind = match s {
        Side::Move => K::MoveCard {
            src: r,
            mirror: None,
        },
        Side::Copy => K::CopyCard(r),
        Side::Mirror => K::MirrorCard(r),
        _ => match list {
            Some(l) => K::FromTemplate(l),
            None => return,
        },
    };
    crate::w6::copy_move::open(docs, id, kind, anchor);
    if s == Side::FromTemplate {
        crate::w6::copy_move::click(docs, id, crate::w6::Dlg::Template(r));
    }
}

fn open_attachment(docs: &mut KanbanDocs, id: u64, a: ph2d_kanban_model::AttachmentId) {
    let Some(Some((link, name, here))) = read(docs, id, |k, _| {
        k.attachments
            .get(&a)
            .map(|x| (x.link.clone(), x.name.clone(), k.id))
    }) else {
        return;
    };
    match link {
        Link::Url(u) => {
            if let Some(b) = back_mut(docs, id) {
                b.opened.push(Outside::Link(u));
            }
        }
        Link::File(f) => {
            if let Some(b) = back_mut(docs, id) {
                b.opened.push(Outside::File(f, name));
            }
        }
        Link::Card(k, c) if k == here => super::open(docs, id, c),
        Link::Card(k, _) | Link::Kanban(k) => docs.open(k),
    }
}

/// A capa por cor (o tamanho que já tinha).
pub(crate) fn cover_color(docs: &mut KanbanDocs, id: u64, card: CardId, hue: Hue) {
    let full = read(docs, id, |_, c| c.cover.is_some_and(|c| c.full)).unwrap_or(false);
    write::set_cover(
        docs,
        id,
        card,
        Some(Cover {
            fill: CoverFill::Color(hue),
            full,
        }),
    );
}

/// A cor de etiqueta com que o «Create label» abre (a 1.ª da paleta normal).
pub(crate) fn default_color() -> LabelColor {
    LabelColor {
        hue: Hue::Green,
        shade: Shade::Normal,
    }
}
