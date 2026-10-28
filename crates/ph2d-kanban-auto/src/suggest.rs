//! **Sugestões** (o Butler sugere a partir do que se repete): «moveste 3 cartões para Done e
//! marcaste a entrega como concluída de cada vez» → a regra «quando um cartão entra em Done, por
//! mim, marcar a entrega como concluída». Puro, sobre a atividade do Kanban.
//!
//! A lei: depois de mover um cartão para a lista L, o que a MESMA pessoa fez a esse cartão nos
//! [`SUGGEST_WINDOW_MS`] seguintes (até ao próximo movimento dele) é o seguimento. Cada acção de
//! seguimento repetida em pelo menos [`SUGGEST_MIN`] movimentos para L entra na sugestão. Uma regra
//! que já faz isso (mesmo gatilho e já com todas essas acções) cala a sugestão.

use std::collections::BTreeMap;

use ph2d_kanban_model::{Activity, Happened, Kanban, KanbanId, ListId, Member, Subject};

use crate::spec::{Action, Added, By, Kind, Spec, Trigger, specs};

/// Quantas vezes a mesma sequência tem de se repetir (o exemplo do Butler: três).
pub const SUGGEST_MIN: usize = 3;
/// A janela do seguimento depois de um movimento: cinco minutos.
pub const SUGGEST_WINDOW_MS: i64 = 5 * 60 * 1000;

/// Uma regra sugerida e quantos movimentos a mostram.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    pub kanban: KanbanId,
    pub spec: Spec,
    pub times: usize,
}

/// A acção que repete o acontecimento `h` feito por `who` (só as que não dependem de um valor
/// escolhido na hora).
fn follow(h: &Happened, who: &Member) -> Option<Action> {
    Some(match h {
        Happened::DueDone(done) => Action::MarkDue { done: *done },
        Happened::Archived => Action::Archive,
        Happened::MemberAdded(m) if m == who => Action::JoinMe,
        Happened::MemberAdded(m) => Action::AddMember(m.clone()),
        Happened::MemberRemoved(m) => Action::RemoveMember(m.clone()),
        _ => return None,
    })
}

/// A ordem das acções na regra: marcar e atribuir antes de arquivar (arquivar por último, como o
/// Butler escreve).
fn rank(a: &Action) -> u8 {
    match a {
        Action::MarkDue { .. } => 0,
        Action::JoinMe | Action::AddMember(_) | Action::RemoveMember(_) => 1,
        _ => 2,
    }
}

/// As sugestões para `me` no Kanban `k`, da mais repetida para a menos (empate: a lista).
#[must_use]
pub fn suggest(k: &Kanban, me: &Member) -> Vec<Suggestion> {
    let mut by_card: BTreeMap<_, Vec<&Activity>> = BTreeMap::new();
    for a in k.activities.values() {
        if let Subject::Card(c) = a.subject
            && !a.meta.deleted
            && a.by == *me
        {
            by_card.entry(c).or_default().push(a);
        }
    }
    // lista → (movimentos, acção → em quantos movimentos apareceu)
    let mut seen: BTreeMap<ListId, (usize, Vec<(Action, usize)>)> = BTreeMap::new();
    for acts in by_card.values_mut() {
        acts.sort_by(|a, b| a.at.cmp(&b.at).then(a.id.cmp(&b.id)));
        for (i, a) in acts.iter().enumerate() {
            let Happened::Moved { to, .. } = a.what else {
                continue;
            };
            let entry = seen.entry(to).or_default();
            entry.0 += 1;
            let mut here: Vec<Action> = Vec::new();
            for b in &acts[i + 1..] {
                if matches!(b.what, Happened::Moved { .. }) || b.at.0 - a.at.0 > SUGGEST_WINDOW_MS {
                    break;
                }
                if let Some(x) = follow(&b.what, me)
                    && !here.contains(&x)
                {
                    here.push(x);
                }
            }
            for x in here {
                match entry.1.iter_mut().find(|(y, _)| *y == x) {
                    Some((_, n)) => *n += 1,
                    None => entry.1.push((x, 1)),
                }
            }
        }
    }
    let rules = specs(k);
    let mut out = Vec::new();
    for (list, (_, actions)) in seen {
        let mut picked: Vec<(Action, usize)> = actions
            .into_iter()
            .filter(|(_, n)| *n >= SUGGEST_MIN)
            .collect();
        if picked.is_empty() {
            continue;
        }
        picked.sort_by_key(|(a, _)| rank(a));
        let times = picked.iter().map(|(_, n)| *n).min().unwrap_or(0);
        let actions: Vec<Action> = picked.into_iter().map(|(a, _)| a).collect();
        let covered = rules.iter().any(|(_, s)| {
            matches!(&s.kind, Kind::Rule { trigger: Trigger::CardAdded { list: Some(l), .. }, .. } if *l == list)
                && actions.iter().all(|a| s.actions.contains(a))
        });
        if covered {
            continue;
        }
        out.push(Suggestion {
            kanban: k.id,
            spec: Spec {
                name: String::new(),
                enabled: true,
                owner: me.clone(),
                kind: Kind::Rule {
                    trigger: Trigger::CardAdded {
                        list: Some(list),
                        how: Added::MovedIn,
                    },
                    by: By::Only(me.clone()),
                    conditions: Vec::new(),
                },
                actions,
            },
            times,
        });
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.times));
    out
}
