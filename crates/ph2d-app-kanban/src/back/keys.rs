//! ⭐ **O teclado no verso** — com um texto aberto (o campo, a descrição, o comentário, a caixa de um
//! popover), o texto; senão os atalhos do Trello dentro do cartão (plano §1.7): `D` datas,
//! `L` etiquetas, `M` membros, `Space` atribuir-me, `C` arquivar, `S` seguir, `V` votar, `T` título,
//! `Esc` fecha. O verso é modal: nenhuma tecla sem `Ctrl` chega ao quadro por baixo.

use super::click::{back_mut, commit_edit, people, read};
use super::{Back, BackHit, EditAt, Pop, PopHit, write};
use crate::docs::KanbanDocs;
use ph2d_editor_core::doc_surface::{Compose, Key, KeyInput, KeyOutcome};
use ph2d_editor_core::widget::{FieldKey, LineField, RichEditorEvent};
use ph2d_kanban_model::{CardId, FieldKind, FieldOption, Hue, LabelColor, OptionId, RecordId};
use ph2d_text::TextSystem;

fn taken() -> KeyOutcome {
    KeyOutcome {
        consumed: true,
        copy: None,
    }
}

/// A caixa de texto do popover que tem o teclado.
fn pop_field(b: &mut Back) -> Option<&mut LineField> {
    let second = b.pop_focus;
    match b.pop.as_mut()? {
        Pop::Label(_, f, _) | Pop::Checklist(f) => Some(f),
        Pop::Attach(a, n) => Some(if second { n } else { a }),
        Pop::Fields(n, _, o) => Some(if second { o } else { n }),
        _ => None,
    }
}

/// ⭐ Uma tecla com o verso aberto.
pub(crate) fn key(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    input: &KeyInput<'_>,
) -> KeyOutcome {
    let Some(b) = back_mut(docs, id) else {
        return KeyOutcome::default();
    };
    if !input.pressed {
        return KeyOutcome {
            consumed: b.typing(),
            copy: None,
        };
    }
    // O seletor de datas.
    if let Some(Pop::Dates(_, p, _)) = b.pop.as_mut() {
        let (took, ev) = p.key(input);
        super::click::picker_events(docs, id, ev.into_iter().collect());
        return KeyOutcome {
            consumed: took || !input.mods.ctrl,
            copy: None,
        };
    }
    if matches!(b.pop, Some(Pop::Share)) && input.mods.ctrl && input.key == Key::Char('c') {
        let link = read(docs, id, super::pop::card_link);
        return KeyOutcome {
            consumed: true,
            copy: link,
        };
    }
    if let Some(f) = pop_field(b) {
        return match input.key {
            Key::Escape => {
                b.pop = None;
                taken()
            }
            Key::Tab => {
                b.pop_focus = !b.pop_focus;
                taken()
            }
            Key::Enter => {
                let card = b.card;
                pop_click(docs, id, card, PopHit::Save);
                taken()
            }
            k => {
                let text = input.paste.as_deref().or(input.text);
                match f.key(k, text) {
                    FieldKey::Ignored if input.mods.ctrl => KeyOutcome::default(),
                    _ => taken(),
                }
            }
        };
    }
    if b.pop.is_some() && input.key == Key::Escape {
        b.pop = None;
        return taken();
    }
    if let Some(e) = b.edit.as_mut() {
        let text = input.paste.as_deref().or(input.text);
        return match e.field.key(input.key, text) {
            FieldKey::Commit => {
                let again = matches!(e.at, EditAt::NewItem(_));
                commit_edit(docs, id, again);
                taken()
            }
            FieldKey::Cancel => {
                b.edit = None;
                taken()
            }
            FieldKey::Ignored if input.mods.ctrl => KeyOutcome::default(),
            _ => taken(),
        };
    }
    if b.desc.editing {
        let Some(r) = b.rect_of(BackHit::DescArea) else {
            return taken();
        };
        let out = b.desc.key(ts, r, input);
        match out.event {
            Some(RichEditorEvent::Submit | RichEditorEvent::ExitEdit) => save_description(docs, id),
            Some(RichEditorEvent::OpenLink(u)) => b.opened.push(super::outside::Outside::Link(u)),
            _ => {}
        }
        return KeyOutcome {
            consumed: out.consumed,
            copy: out.copy,
        };
    }
    if b.comment.editing {
        return comment_key(docs, id, ts, input);
    }
    shortcut(docs, id, input)
}

fn comment_key(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    input: &KeyInput<'_>,
) -> KeyOutcome {
    let people = people(docs, id);
    let Some(b) = back_mut(docs, id) else {
        return KeyOutcome::default();
    };
    let list = super::activity::mention_query(&b.comment)
        .map(|(_, q)| super::activity::candidates(&people, &q))
        .unwrap_or_default();
    if !list.is_empty() {
        match input.key {
            Key::Down => {
                b.mention_pick = (b.mention_pick + 1) % list.len();
                return taken();
            }
            Key::Up => {
                b.mention_pick = (b.mention_pick + list.len() - 1) % list.len();
                return taken();
            }
            Key::Enter | Key::Tab if !input.mods.ctrl => {
                let who = list[b.mention_pick.min(list.len() - 1)].clone();
                super::activity::pick_mention(&mut b.comment, &who);
                b.mention_pick = 0;
                return taken();
            }
            _ => {}
        }
    }
    let Some(r) = b.rect_of(BackHit::CommentArea) else {
        return taken();
    };
    let out = b.comment.key(ts, r, input);
    match out.event {
        Some(RichEditorEvent::Submit) => save_comment(docs, id),
        Some(RichEditorEvent::ExitEdit) => {
            if b.comment.doc().is_empty() || b.comment_edit.is_some() {
                cancel_comment(docs, id);
            } else {
                b.comment.editing = false;
            }
        }
        _ => {}
    }
    KeyOutcome {
        consumed: out.consumed,
        copy: out.copy,
    }
}

/// Os atalhos do cartão aberto (nenhum texto aberto).
fn shortcut(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    if input.mods.ctrl || input.mods.meta || input.mods.alt {
        return KeyOutcome::default();
    }
    let ch = match input.key {
        Key::Char(c) => Some(c.to_ascii_lowercase()),
        _ => input
            .text
            .and_then(|t| t.chars().next())
            .map(|c| c.to_ascii_lowercase()),
    };
    let Some(card) = back_mut(docs, id).map(|b| b.card) else {
        return KeyOutcome::default();
    };
    let hit = match (input.key, ch) {
        (Key::Escape, _) => Some(BackHit::Close),
        (Key::Space, _) | (_, Some(' ')) => {
            let me = docs.me.clone();
            write::toggle_member(docs, id, card, &me);
            None
        }
        (_, Some('d')) => Some(BackHit::Side(super::Side::Dates)),
        (_, Some('l')) => Some(BackHit::Side(super::Side::Labels)),
        (_, Some('m')) => Some(BackHit::Side(super::Side::Members)),
        (_, Some('c')) => Some(BackHit::Side(super::Side::Archive)),
        (_, Some('s')) => Some(BackHit::Side(super::Side::Watch)),
        (_, Some('v')) => Some(BackHit::Side(super::Side::Vote)),
        (_, Some('t')) => Some(BackHit::Title),
        _ => None,
    };
    if let Some(h) = hit {
        super::click::click(docs, id, h);
    }
    taken()
}

/// A composição (IME, tecla morta) vai para o texto aberto.
pub(crate) fn compose(docs: &mut KanbanDocs, id: u64, c: Compose<'_>) -> bool {
    let Some(b) = back_mut(docs, id) else {
        return false;
    };
    if b.desc.editing {
        b.desc.compose(c);
        return true;
    }
    if b.comment.editing {
        b.comment.compose(c);
        return true;
    }
    let Compose::Commit(ch) = c else {
        return false;
    };
    let s = ch.encode_utf8(&mut [0; 4]).to_owned();
    if let Some(e) = b.edit.as_mut() {
        e.field.insert(&s);
        return true;
    }
    if let Some(f) = pop_field(b) {
        f.insert(&s);
        return true;
    }
    false
}

/// Grava a descrição em edição (markdown pelo `ph2d-kanban-md`) e sai da edição.
pub(crate) fn save_description(docs: &mut KanbanDocs, id: u64) {
    let Some(b) = back_mut(docs, id) else {
        return;
    };
    if !b.desc.editing {
        return;
    }
    b.desc.editing = false;
    let card = b.card;
    let md = ph2d_kanban_md::render(b.desc.doc());
    let stored = read(docs, id, |_, c| c.description.clone()).unwrap_or_default();
    if ph2d_kanban_md::normalize(&stored) != md {
        write::card(docs, id, card, |c| c.description.clone_from(&md));
        if let Some(b) = back_mut(docs, id) {
            b.desc_md = md;
        }
    }
}

/// Larga o que se escreveu na descrição.
pub(crate) fn cancel_description(docs: &mut KanbanDocs, id: u64) {
    let stored = read(docs, id, |_, c| c.description.clone()).unwrap_or_default();
    if let Some(b) = back_mut(docs, id) {
        b.desc.editing = false;
        b.desc.set_doc(ph2d_kanban_md::parse(&stored));
        b.desc_md = stored;
    }
}

/// Publica o comentário (novo, ou o que se editava).
pub(crate) fn save_comment(docs: &mut KanbanDocs, id: u64) {
    let Some(b) = back_mut(docs, id) else {
        return;
    };
    let md = ph2d_kanban_md::render(b.comment.doc());
    let (card, editing) = (b.card, b.comment_edit);
    cancel_comment(docs, id);
    if md.trim().is_empty() {
        return;
    }
    match editing {
        Some(c) => write::edit_comment(docs, id, c, md),
        None => write::add_comment(docs, id, card, md),
    };
}

pub(crate) fn cancel_comment(docs: &mut KanbanDocs, id: u64) {
    if let Some(b) = back_mut(docs, id) {
        b.comment.set_doc(ph2d_richdoc::RichDoc::new());
        b.comment.editing = false;
        b.comment_edit = None;
        b.mention_pick = 0;
    }
}

/// As opções de uma lista suspensa escritas «Low, Medium, High».
fn options(text: &str, first_id: u64) -> Vec<FieldOption> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .zip(first_id..)
        .map(|(s, i)| FieldOption {
            id: OptionId(i),
            text: s.to_owned(),
            color: None,
        })
        .collect()
}

/// ⭐ Um clique dentro de um popover.
#[allow(clippy::too_many_lines)]
pub(crate) fn pop_click(docs: &mut KanbanDocs, id: u64, card: CardId, p: PopHit) {
    let people = people(docs, id);
    let Some(b) = back_mut(docs, id) else {
        return;
    };
    match p {
        PopHit::Inside | PopHit::Picker => {}
        PopHit::Close => b.pop = None,
        PopHit::Input => b.pop_focus = false,
        PopHit::Input2 => b.pop_focus = true,
        PopHit::Member(i) => {
            let Some(who) = people.get(usize::from(i)).cloned() else {
                return;
            };
            match b.pop {
                Some(Pop::Members(Some(item))) => {
                    write::item(docs, id, item, |it| {
                        it.member = if it.member.as_ref() == Some(&who) {
                            None
                        } else {
                            Some(who)
                        };
                    });
                }
                _ => {
                    write::toggle_member(docs, id, card, &who);
                }
            }
        }
        PopHit::List(l) => {
            b.pop = None;
            write::move_to(docs, id, card, l);
        }
        PopHit::Label(l) => {
            write::toggle_label(docs, id, card, l);
        }
        PopHit::LabelEdit(l) => {
            let Some((name, color)) = read(docs, id, |k, _| {
                k.labels.get(&l).map(|x| (x.name.clone(), x.color))
            })
            .flatten() else {
                return;
            };
            if let Some(b) = back_mut(docs, id) {
                b.pop = Some(Pop::Label(Some(l), LineField::new(name), color));
            }
        }
        PopHit::LabelCreate => {
            b.pop = Some(Pop::Label(
                None,
                LineField::default(),
                Some(super::click::default_color()),
            ));
        }
        PopHit::Color(i) => {
            if let Some(Pop::Label(_, _, c)) = b.pop.as_mut() {
                *c = LabelColor::all().get(usize::from(i)).copied();
            }
        }
        PopHit::NoColor => {
            if let Some(Pop::Label(_, _, c)) = b.pop.as_mut() {
                *c = None;
            }
        }
        PopHit::Save => match b.pop.take() {
            Some(Pop::Label(l, name, color)) => {
                let name = name.text.trim().to_owned();
                match l {
                    Some(l) => {
                        write::edit_label(docs, id, l, name, color);
                    }
                    None => {
                        write::create_label(docs, id, card, name, color);
                    }
                }
                if let Some(b) = back_mut(docs, id) {
                    b.pop = Some(Pop::Labels);
                }
            }
            Some(Pop::Checklist(t)) => {
                let t = t.text.trim().to_owned();
                if !t.is_empty() {
                    write::add_checklist(docs, id, card, t);
                }
            }
            Some(Pop::Attach(url, name)) => {
                let url = url.text.trim().to_owned();
                if url.is_empty() {
                    if let Some(b) = back_mut(docs, id) {
                        b.pop = Some(Pop::Attach(LineField::new(url), name));
                    }
                    return;
                }
                let name = Some(name.text.trim().to_owned())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| url.clone());
                write::add_link(docs, id, card, url, name);
            }
            Some(Pop::Fields(name, kind, opts)) => {
                let n = name.text.trim().to_owned();
                if n.is_empty() {
                    if let Some(b) = back_mut(docs, id) {
                        b.pop = Some(Pop::Fields(name, kind, opts));
                    }
                    return;
                }
                let kind = match kind {
                    1 => FieldKind::Number,
                    2 => FieldKind::Date,
                    3 => FieldKind::Checkbox,
                    4 => FieldKind::Dropdown(options(&opts.text, 1)),
                    _ => FieldKind::Text,
                };
                write::create_field(docs, id, n, kind);
                if let Some(b) = back_mut(docs, id) {
                    b.pop = Some(Pop::Fields(LineField::default(), 0, LineField::default()));
                }
            }
            Some(Pop::Delete) => {
                let plan = docs.kanban_of(id).and_then(|kanban| {
                    let r = ph2d_kanban_model::CardRef { kanban, card };
                    ph2d_kanban_model::remove_card(&docs.set.borrow(), r, false)
                });
                if crate::w6::gesture::apply(docs, plan)
                    && let Some(t) = docs.tab_mut(id)
                {
                    t.back = None;
                    t.focus = None;
                }
            }
            other => b.pop = other,
        },
        PopHit::Remove => match b.pop {
            Some(Pop::Label(Some(l), ..)) => {
                b.pop = Some(Pop::Labels);
                write::delete_label(docs, id, l);
            }
            Some(Pop::Cover) => {
                write::set_cover(docs, id, card, None);
            }
            _ => {}
        },
        PopHit::CoverHue(i) => {
            if let Some(h) = Hue::ALL.get(usize::from(i)) {
                super::click::cover_color(docs, id, card, *h);
            }
        }
        PopHit::CoverImage(a) => {
            let file = read(docs, id, |k, _| {
                match k.attachments.get(&a).map(|x| &x.link) {
                    Some(ph2d_kanban_model::Link::File(f)) => Some(*f),
                    _ => None,
                }
            })
            .flatten();
            if let Some(f) = file {
                let full = read(docs, id, |_, c| c.cover.is_some_and(|c| c.full)).unwrap_or(false);
                let cover = ph2d_kanban_model::Cover {
                    fill: ph2d_kanban_model::CoverFill::Image(f),
                    full,
                };
                write::set_cover(docs, id, card, Some(cover));
            }
        }
        PopHit::CoverSize(full) => {
            write::card(docs, id, card, |c| {
                if let Some(cv) = c.cover.as_mut() {
                    cv.full = full;
                }
            });
        }
        PopHit::FieldKind(k) => {
            if let Some(Pop::Fields(_, kind, _)) = b.pop.as_mut() {
                *kind = k;
            }
        }
        PopHit::FieldFront(f) => {
            write::field_front(docs, id, f);
        }
        PopHit::FieldDelete(f) => {
            write::delete(docs, id, RecordId::Field(f));
        }
        PopHit::PickFile => {
            b.pop = None;
            b.pick = true;
        }
        PopHit::CopyLink => {
            let link = read(docs, id, super::pop::card_link);
            if let Some(b) = back_mut(docs, id) {
                b.copy = link;
                b.pop = None;
            }
        }
    }
}
