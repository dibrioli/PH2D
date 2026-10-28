//! ⭐ **O painel de automação** (o Butler do Trello, J2) — separadores Rules · Card buttons · Board
//! buttons · Calendar · Due date; a lista de cada um com liga/desliga, editar, apagar (e correr um
//! botão de quadro); o CONSTRUTOR EM FRASE (a frase de `ph2d_kanban_auto::describe` em chips: a
//! cabeça troca o tipo, os nomes escolhem-se, os textos escrevem-se); e as SUGESTÕES do que se
//! repete (`ph2d_kanban_auto::suggest`), com Criar/Dispensar.
//!
//! É um modal da VISTA ([`crate::tab::Tab::auto`]) e entra pelas portas do verso: o verso pinta-o por
//! cima de si e encaminha-lhe o rato e o teclado ([`crate::back::is_open`]). O que escreve vai por
//! [`crate::back::write::logged`] (um passo de desfazer cada); o que um botão FAZ vai em nome do
//! Butler ([`buttons`]).

pub(crate) mod buttons;
mod choices;
mod input;
mod paint;
mod sentence;

pub(crate) use input::{compose, down, key, moved, up, wheel};
pub(crate) use paint::paint;

use crate::back::BackHit;
use crate::docs::KanbanDocs;
use ph2d_editor_core::widget::LineField;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_auto::{Family, Kind, Spec};
use ph2d_kanban_model::{AutomationId, KanbanOp, Member, RecordId};

/// Uma secção da frase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sec {
    Trigger,
    /// O nome de um botão.
    Name,
    Cond(u16),
    Action(u16),
    /// O «quando» de um comando (calendário ou prazo).
    Schedule,
    /// As listas em que um botão de quadro ou um comando age.
    Scope,
}

/// Um chip da frase: a cabeça de uma secção, o valor na peça `n`, ou a palavra `n` que alterna.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Slot {
    Head(Sec),
    Arg(Sec, u8),
    Word(Sec, u8),
}

/// O que o rato acerta no painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AutoHit {
    Scrim,
    Inside,
    Close,
    Tab(Family),
    Create,
    Toggle(AutomationId),
    Edit(AutomationId),
    Delete(AutomationId),
    Run(AutomationId),
    /// A sugestão `n`: criar (`true`) ou dispensar.
    Suggest(u16, bool),
    Chip(Slot),
    /// A linha `n` do menu aberto de um chip.
    Choice(u16),
    /// A caixa de texto de um chip, e o «Save» dela.
    Typed,
    TypedSave,
    AddCondition,
    AddAction,
    Save,
    Cancel,
    /// Dentro do menu de um chip, sem linha.
    Menu,
}

/// Uma automação a ser montada (`id = None` = nova).
#[derive(Debug)]
pub(crate) struct Draft {
    pub id: Option<AutomationId>,
    pub spec: Spec,
    /// O chip com o menu aberto.
    pub open: Option<Slot>,
    /// A caixa de texto aberta de um chip.
    pub typed: Option<(Slot, choices::Typed, LineField)>,
}

/// O painel aberto.
#[derive(Debug)]
pub(crate) struct Panel {
    pub family: Family,
    pub draft: Option<Draft>,
    /// As sugestões dispensadas nesta vista.
    pub dismissed: Vec<Spec>,
    pub hits: Vec<(Rect, BackHit)>,
    pub hover: Option<BackHit>,
    pub press: Option<BackHit>,
    pub scroll: f32,
    pub area: Rect,
    pub content_h: f32,
    /// As escolhas do menu aberto, pela ordem pintada.
    pub picks: Vec<choices::Pick>,
    /// O que o último quadro pintou (as sondas): cada frase da lista, as sugestões e o construtor.
    pub lines: Vec<String>,
}

impl Panel {
    fn hit(&self, x: f32, y: f32) -> Option<AutoHit> {
        self.hits.iter().rev().find(|(r, _)| r.contains(x, y)).and_then(|(_, h)| match h {
            BackHit::Auto(a) => Some(*a),
            _ => None,
        })
    }

    fn typing(&self) -> bool {
        self.draft.as_ref().is_some_and(|d| d.typed.is_some())
    }
}

/// O separador de cada família, pela ordem do Trello.
pub(crate) const FAMILIES: [Family; 5] = [
    Family::Rule,
    Family::CardButton,
    Family::BoardButton,
    Family::Calendar,
    Family::DueDate,
];

/// A chave da i18n de um separador.
pub(crate) fn family_key(f: Family) -> &'static str {
    match f {
        Family::Rule => "rules",
        Family::CardButton => "card_buttons",
        Family::BoardButton => "board_buttons",
        Family::Calendar => "calendar",
        Family::DueDate => "due",
    }
}

/// A automação nova de cada separador (a frase por onde o construtor começa).
pub(crate) fn blank(f: Family, k: &ph2d_kanban_model::Kanban, me: &Member) -> Spec {
    use ph2d_kanban_auto::{Added, By, DueWhen, Every, Scope, TimeOfDay, Trigger, Unit};
    let nine = TimeOfDay { hour: 9, minute: 0 };
    let kind = match f {
        Family::Rule => Kind::Rule {
            trigger: Trigger::CardAdded {
                list: k.lists_in_order().first().map(|l| l.id),
                how: Added::Any,
            },
            by: By::Anyone,
            conditions: Vec::new(),
        },
        Family::CardButton => Kind::CardButton,
        Family::BoardButton => Kind::BoardButton {
            scope: Scope::default(),
        },
        Family::Calendar => Kind::Calendar {
            every: Every::Day,
            at: nine,
            scope: Scope::default(),
        },
        Family::DueDate => Kind::DueDate {
            when: DueWhen::Before {
                n: 1,
                unit: Unit::Days,
            },
            scope: Scope::default(),
        },
    };
    Spec {
        name: String::new(),
        enabled: true,
        owner: me.clone(),
        kind,
        actions: Vec::new(),
    }
}

pub(crate) fn panel_mut(docs: &mut KanbanDocs, id: u64) -> Option<&mut Panel> {
    docs.tab_mut(id)?.auto.as_deref_mut()
}

/// O painel está aberto na vista `id`.
pub(crate) fn is_open(docs: &mut KanbanDocs, id: u64) -> bool {
    panel_mut(docs, id).is_some()
}

/// ⭐ Abre o painel no separador `family` (o «…» do Kanban: as regras); `new` abre já o construtor
/// de uma automação nova (o «Add button» do verso).
pub(crate) fn open(docs: &mut KanbanDocs, id: u64, family: Family, new: bool) {
    let me = docs.me.clone();
    let spec = docs
        .kanban_of(id)
        .and_then(|kid| docs.set.borrow().get(kid).map(|k| blank(family, k, &me)));
    let Some(t) = docs.tab_mut(id) else { return };
    t.menu = None;
    t.auto = Some(Box::new(Panel {
        family,
        draft: spec.filter(|_| new).map(|spec| Draft {
            id: None,
            spec,
            open: None,
            typed: None,
        }),
        dismissed: Vec::new(),
        hits: Vec::new(),
        hover: None,
        press: None,
        scroll: 0.0,
        area: Rect::new(0.0, 0.0, 0.0, 0.0),
        content_h: 0.0,
        picks: Vec::new(),
        lines: Vec::new(),
    }));
}

pub(crate) fn close(docs: &mut KanbanDocs, id: u64) {
    if let Some(t) = docs.tab_mut(id) {
        t.auto = None;
    }
}

/// Grava a automação montada (nova ou editada) — um passo de desfazer. Sem acções (o Butler pede
/// pelo menos uma) ou com uma agenda que nunca dispara não grava e o rascunho fica aberto.
pub(crate) fn save(docs: &mut KanbanDocs, id: u64) -> bool {
    let Some(d) = panel_mut(docs, id).and_then(|p| p.draft.take()) else {
        return false;
    };
    if d.spec.actions.is_empty() || d.spec.check().is_err() {
        if let Some(p) = panel_mut(docs, id) {
            p.draft = Some(d);
        }
        return false;
    }
    let now = crate::actions::now();
    crate::back::write::logged(docs, id, |k| {
        Some(vec![match d.id.and_then(|a| k.automations.get(&a)) {
            Some(a) => d.spec.update(a).ok()?,
            None => d.spec.create(k, now).ok()?,
        }])
    })
}

/// A automação `a` mudada por `f` (liga/desliga) — um passo de desfazer.
pub(crate) fn change(docs: &mut KanbanDocs, id: u64, a: AutomationId, f: impl FnOnce(&mut Spec)) -> bool {
    crate::back::write::logged(docs, id, |k| {
        let rec = k.automations.get(&a)?;
        let mut s = Spec::from_bytes(&rec.body).ok()?;
        f(&mut s);
        Some(vec![s.update(rec).ok()?])
    })
}

pub(crate) fn delete(docs: &mut KanbanDocs, id: u64, a: AutomationId) -> bool {
    crate::back::write::logged(docs, id, |_| {
        Some(vec![KanbanOp::Delete(RecordId::Automation(a))])
    })
}
