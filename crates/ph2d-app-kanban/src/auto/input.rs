//! ⭐ **O rato e o teclado no painel de automação** — carregar guarda o alvo, largar no MESMO alvo é o
//! clique. Um menu de chip aberto: o clique fora só o fecha. `Esc` desce um degrau (caixa → menu →
//! construtor → painel); `Enter` grava a caixa aberta.

use super::choices;
use super::sentence::{conditions_mut, segs};
use super::{AutoHit, Draft, Sec, Slot, panel_mut};
use crate::back::BackHit;
use crate::docs::KanbanDocs;
use ph2d_editor_core::doc_surface::{Compose, Key, KeyInput, KeyOutcome};
use ph2d_editor_core::widget::{FieldKey, LineField};
use ph2d_kanban_auto::{Family, Spec, specs, suggest};

pub(crate) fn down(docs: &mut KanbanDocs, id: u64, (x, y): (f32, f32)) -> bool {
    let Some(p) = panel_mut(docs, id) else {
        return false;
    };
    if !p.area.contains(x, y) {
        return false;
    }
    p.press = p.hit(x, y).map(BackHit::Auto);
    true
}

pub(crate) fn moved(docs: &mut KanbanDocs, id: u64, x: f32, y: f32) -> bool {
    let Some(p) = panel_mut(docs, id) else {
        return false;
    };
    if !p.area.contains(x, y) {
        return false;
    }
    p.hover = p.hit(x, y).map(BackHit::Auto);
    true
}

pub(crate) fn up(docs: &mut KanbanDocs, id: u64, x: f32, y: f32) -> bool {
    let Some(p) = panel_mut(docs, id) else {
        return false;
    };
    let press = p.press.take();
    let now = p.hit(x, y);
    if let (Some(BackHit::Auto(a)), Some(b)) = (press, now)
        && a == b
    {
        click(docs, id, a);
    }
    true
}

pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, dy: f32) -> bool {
    let Some(p) = panel_mut(docs, id) else {
        return false;
    };
    let max = (p.content_h + 2.0 * crate::back::ui::MODAL_TOP - p.area.h).max(0.0);
    p.scroll = (p.scroll + dy).clamp(0.0, max);
    true
}

fn taken() -> KeyOutcome {
    KeyOutcome {
        consumed: true,
        copy: None,
    }
}

/// Uma tecla com o painel aberto (modal: nenhuma tecla sem `Ctrl` passa para baixo).
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> KeyOutcome {
    let Some(p) = panel_mut(docs, id) else {
        return KeyOutcome::default();
    };
    if !input.pressed {
        return KeyOutcome {
            consumed: p.typing(),
            copy: None,
        };
    }
    if let Some(d) = p.draft.as_mut()
        && let Some((_, _, f)) = d.typed.as_mut()
    {
        let text = input.paste.as_deref().or(input.text);
        match f.key(input.key, text) {
            FieldKey::Commit => commit_typed(docs, id),
            FieldKey::Cancel => d.typed = None,
            FieldKey::Ignored if input.mods.ctrl => return KeyOutcome::default(),
            _ => {}
        }
        return taken();
    }
    if input.key == Key::Escape {
        match p.draft.as_mut() {
            Some(d) if d.open.is_some() => d.open = None,
            Some(_) => p.draft = None,
            None => super::close(docs, id),
        }
        return taken();
    }
    KeyOutcome {
        consumed: !input.mods.ctrl,
        copy: None,
    }
}

/// A composição (IME, tecla morta) vai para a caixa aberta.
pub(crate) fn compose(docs: &mut KanbanDocs, id: u64, c: Compose<'_>) -> bool {
    // A tecla morta compõe-se no xkb e a letra vem na tecla: tomá-la aqui engolia o teclado.
    if matches!(c, Compose::DeadKey(_)) {
        return false;
    }
    let Some(f) = panel_mut(docs, id)
        .and_then(|p| p.draft.as_mut())
        .and_then(|d| d.typed.as_mut())
        .map(|t| &mut t.2)
    else {
        return panel_mut(docs, id).is_some();
    };
    if let Compose::Commit(ch) = c {
        f.insert(ch.encode_utf8(&mut [0; 4]));
    }
    true
}

fn commit_typed(docs: &mut KanbanDocs, id: u64) {
    let Some(d) = panel_mut(docs, id).and_then(|p| p.draft.as_mut()) else {
        return;
    };
    if let Some((slot, t, f)) = d.typed.take() {
        choices::write(&mut d.spec, slot, t, &f.text);
    }
}

fn draft(docs: &mut KanbanDocs, id: u64) -> Option<&mut Draft> {
    panel_mut(docs, id)?.draft.as_mut()
}

/// O Kanban da vista, «eu» e quem se pode escolher.
fn context(
    docs: &KanbanDocs,
    id: u64,
) -> Option<(
    ph2d_kanban_model::Kanban,
    ph2d_kanban_model::Member,
    Vec<ph2d_kanban_model::Member>,
)> {
    let kid = docs.kanban_of(id)?;
    let set = docs.set.borrow();
    let k = set.get(kid)?.clone();
    let people = crate::back::people(&k, &docs.team, &docs.me);
    Some((k, docs.me.clone(), people))
}

/// ⭐ Um clique no painel.
pub(crate) fn click(docs: &mut KanbanDocs, id: u64, h: AutoHit) {
    // Um menu aberto: o clique fora dele só o fecha.
    if let Some(d) = draft(docs, id)
        && (d.open.is_some() || d.typed.is_some())
        && !matches!(
            h,
            AutoHit::Choice(_) | AutoHit::Menu | AutoHit::Typed | AutoHit::TypedSave
        )
    {
        let was = d.open.take().or(d.typed.take().map(|t| t.0));
        if was.map(AutoHit::Chip) == Some(h) || matches!(h, AutoHit::Inside | AutoHit::Scrim) {
            return;
        }
    }
    match h {
        AutoHit::Scrim | AutoHit::Close => super::close(docs, id),
        AutoHit::Inside | AutoHit::Menu | AutoHit::Typed => {}
        AutoHit::Tab(f) => {
            if let Some(p) = panel_mut(docs, id) {
                p.family = f;
                p.draft = None;
                p.scroll = 0.0;
            }
        }
        AutoHit::Create => {
            let Some((k, me, _)) = context(docs, id) else {
                return;
            };
            if let Some(p) = panel_mut(docs, id) {
                p.draft = Some(Draft {
                    id: None,
                    spec: super::blank(p.family, &k, &me),
                    open: None,
                    typed: None,
                });
            }
        }
        AutoHit::Toggle(a) => {
            super::change(docs, id, a, |s| s.enabled = !s.enabled);
        }
        AutoHit::Edit(a) => {
            let Some((k, ..)) = context(docs, id) else {
                return;
            };
            let Some(spec) = specs(&k).into_iter().find(|(i, _)| *i == a).map(|x| x.1) else {
                return;
            };
            if let Some(p) = panel_mut(docs, id) {
                p.draft = Some(Draft {
                    id: Some(a),
                    spec,
                    open: None,
                    typed: None,
                });
            }
        }
        AutoHit::Delete(a) => {
            super::delete(docs, id, a);
        }
        AutoHit::Run(a) => {
            super::buttons::press_board(docs, id, a);
        }
        AutoHit::Suggest(n, create) => suggestion(docs, id, usize::from(n), create),
        AutoHit::Chip(slot) => chip(docs, id, slot),
        AutoHit::Choice(n) => {
            let Some(p) = panel_mut(docs, id) else { return };
            let pick = p.picks.get(usize::from(n)).cloned();
            if let (Some(d), Some(pick)) = (p.draft.as_mut(), pick)
                && let Some(slot) = d.open.take()
            {
                choices::apply(&mut d.spec, slot, pick);
            }
        }
        AutoHit::TypedSave => commit_typed(docs, id),
        AutoHit::AddCondition => {
            let Some((k, me, _)) = context(docs, id) else {
                return;
            };
            if let Some(d) = draft(docs, id)
                && let Some(c) = conditions_mut(&mut d.spec)
            {
                c.push(choices::new_condition(&k, &me));
            }
        }
        AutoHit::AddAction => {
            let Some((k, me, _)) = context(docs, id) else {
                return;
            };
            if let Some(d) = draft(docs, id) {
                d.spec.actions.push(choices::new_action(&k, &me));
            }
        }
        AutoHit::Save => {
            super::save(docs, id);
        }
        AutoHit::Cancel => {
            if let Some(p) = panel_mut(docs, id) {
                p.draft = None;
            }
        }
    }
}

/// Um chip: a palavra que alterna roda; o que se escreve abre a caixa; o resto abre o menu.
fn chip(docs: &mut KanbanDocs, id: u64, slot: Slot) {
    let Some(d) = draft(docs, id) else { return };
    match slot {
        Slot::Word(sec, n) => choices::toggle_word(&mut d.spec, sec, n),
        Slot::Arg(..) => {
            let sg = segs(&d.spec);
            match choices::typed(slot, &sg) {
                Some(t) => {
                    let now = choices::typed_now(slot, &sg);
                    d.typed = Some((slot, t, LineField::new(now)));
                }
                None => d.open = Some(slot),
            }
        }
        Slot::Head(Sec::Name | Sec::Scope) => {}
        Slot::Head(_) => d.open = Some(slot),
    }
}

/// Criar (a regra entra já gravada, como no Butler) ou dispensar a sugestão `n`.
fn suggestion(docs: &mut KanbanDocs, id: u64, n: usize, create: bool) {
    let Some((k, me, _)) = context(docs, id) else {
        return;
    };
    let dismissed = panel_mut(docs, id)
        .map(|p| p.dismissed.clone())
        .unwrap_or_default();
    let Some(s) = visible_suggestions(&k, &me, &dismissed).into_iter().nth(n) else {
        return;
    };
    if create {
        let now = crate::actions::now();
        crate::back::write::logged(docs, id, |k| Some(vec![s.spec.create(k, now).ok()?]));
    } else if let Some(p) = panel_mut(docs, id) {
        p.dismissed.push(s.spec);
    }
}

/// As sugestões que o painel mostra (as dispensadas nesta vista saem).
pub(crate) fn visible_suggestions(
    k: &ph2d_kanban_model::Kanban,
    me: &ph2d_kanban_model::Member,
    dismissed: &[Spec],
) -> Vec<ph2d_kanban_auto::Suggestion> {
    suggest(k, me)
        .into_iter()
        .filter(|s| !dismissed.contains(&s.spec))
        .collect()
}

/// As automações do separador `f`, pela ordem.
pub(crate) fn of_family(
    k: &ph2d_kanban_model::Kanban,
    f: Family,
) -> Vec<(ph2d_kanban_model::AutomationId, Spec)> {
    specs(k)
        .into_iter()
        .filter(|(_, s)| s.kind.family() == f)
        .collect()
}
