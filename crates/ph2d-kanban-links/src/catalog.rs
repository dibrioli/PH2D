//! O catálogo do projecto e a resolução de uma ligação contra ele.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_kanban_model::{
    CardId, Kanban, KanbanOp, Moment, ProjectLink, ProjectLinkId, Record, Target,
};

/// Um Quadro do projecto: o nome e os elementos vivos.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoardEntry {
    pub name: String,
    pub elements: BTreeSet<u64>,
}

/// O que o projecto tem AGORA, pelas chaves duráveis de cada família.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalog {
    /// `StableId` → nome do objecto da cena.
    pub objects: BTreeMap<u64, String>,
    /// `ScriptId` → nome do script.
    pub scripts: BTreeMap<u64, String>,
    /// Os nomes da biblioteca de macros.
    pub macros: BTreeSet<String>,
    /// `StripId` → nome do clip da timeline.
    pub strips: BTreeMap<u64, String>,
    /// `BoardId` → o Quadro.
    pub boards: BTreeMap<u64, BoardEntry>,
}

impl Catalog {
    /// Os Quadros do projecto, com os elementos vivos de cada um.
    pub fn set_boards(&mut self, boards: &ph2d_board_model::BoardSet) {
        self.boards = boards
            .boards()
            .iter()
            .map(|b| {
                let entry = BoardEntry {
                    name: b.name.clone(),
                    elements: b.doc.live().map(|e| e.id.0).collect(),
                };
                (b.id.0, entry)
            })
            .collect();
    }
}

/// O tipo de uma ligação (o verso escolhe o ícone por ele).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Object,
    Script,
    Macro,
    Span,
    Board,
    /// Um elemento de um Quadro.
    Element,
}

/// Uma ligação resolvida.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub kind: Kind,
    /// O nome actual, ou o de reserva se o alvo já não existe.
    pub name: String,
    /// `false` = «quebrada»: o alvo foi apagado.
    pub live: bool,
}

#[must_use]
pub fn kind_of(t: &Target) -> Kind {
    match t {
        Target::Object(_) => Kind::Object,
        Target::Script(_) => Kind::Script,
        Target::Macro(_) => Kind::Macro,
        Target::Span { .. } => Kind::Span,
        Target::Board { element: None, .. } => Kind::Board,
        Target::Board {
            element: Some(_), ..
        } => Kind::Element,
    }
}

/// O nome actual de `t` (`None` = já não existe).
fn current(cat: &Catalog, t: &Target) -> Option<String> {
    match t {
        Target::Object(s) => cat.objects.get(s).cloned(),
        Target::Script(s) => cat.scripts.get(s).cloned(),
        Target::Macro(n) => cat.macros.get(n).cloned(),
        Target::Span { object, strip, .. } => {
            let owner = match object {
                Some(o) => Some(cat.objects.get(o)?),
                None => None,
            };
            let clip = match strip {
                Some(s) => Some(cat.strips.get(s)?),
                None => None,
            };
            clip.or(owner).cloned().or_else(|| Some(String::new()))
        }
        Target::Board { board, element } => {
            let b = cat.boards.get(board)?;
            match element {
                Some(e) if !b.elements.contains(e) => None,
                _ => Some(b.name.clone()),
            }
        }
    }
}

/// ⭐ A ligação contra o catálogo: viva com o nome actual, ou quebrada com o de reserva.
#[must_use]
pub fn resolve(cat: &Catalog, link: &ProjectLink) -> Resolved {
    let kind = kind_of(&link.target);
    match current(cat, &link.target) {
        Some(n) if !n.is_empty() => Resolved {
            kind,
            name: n,
            live: true,
        },
        Some(_) => Resolved {
            kind,
            name: link.name.clone(),
            live: true,
        },
        None => Resolved {
            kind,
            name: link.name.clone(),
            live: false,
        },
    }
}

/// Os `Put` que gravam o nome actual como reserva nas ligações vivas do cartão `card` cujo alvo
/// mudou de nome (vazio = nada a gravar). Sem isto, um objecto renomeado e depois apagado
/// mostrar-se-ia pelo nome antigo.
#[must_use]
pub fn refresh(cat: &Catalog, k: &Kanban, card: CardId) -> Vec<KanbanOp> {
    k.links_of(card)
        .into_iter()
        .filter_map(|l| {
            let r = resolve(cat, l);
            (r.live && r.name != l.name).then(|| {
                let mut n = l.clone();
                n.name = r.name;
                KanbanOp::put(Record::ProjectLink(n))
            })
        })
        .collect()
}

/// O `Put` de uma ligação NOVA de `card` a `target`, no fim das dele, com o nome actual como reserva.
#[must_use]
pub fn link_op(k: &Kanban, card: CardId, target: Target, name: &str, at: Moment) -> KanbanOp {
    let id = ProjectLinkId(k.mint());
    let l = ProjectLink::new(id, card, k.pos_after_links(card), target, name, at);
    KanbanOp::put(Record::ProjectLink(l))
}

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;
