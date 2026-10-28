//! **A linguagem das automações** — os cinco tipos do Butler (regra, botão de cartão, botão de
//! quadro, comando de calendário, comando de prazo), as condições e as acções. Viaja no documento
//! como o corpo opaco de um [`Automation`], com versão PRÓPRIA ([`SPEC_VERSION`]).

use ph2d_kanban_model::{
    Automation, AutomationId, ChecklistId, FieldId, FieldValue, FracKey, Kanban, KanbanId,
    KanbanOp, LabelId, ListId, Member, Meta, Moment, Record,
};
use serde::{Deserialize, Serialize};

/// A versão dos bytes de [`Spec::to_bytes`].
pub const SPEC_VERSION: u32 = 1;

/// Uma automação inteira.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spec {
    /// O nome de um botão; vazio numa regra (a frase é o nome).
    pub name: String,
    pub enabled: bool,
    /// Quem a criou (o «eu» de `By::Only` numa regra «por mim»).
    pub owner: Member,
    pub kind: Kind,
    pub actions: Vec<Action>,
}

/// Os cinco tipos do Butler.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Rule {
        trigger: Trigger,
        by: By,
        conditions: Vec<Condition>,
    },
    /// No verso do cartão; age no cartão aberto.
    CardButton,
    /// No topo do Kanban; age nos cartões do `scope`.
    BoardButton {
        scope: Scope,
    },
    Calendar {
        every: Every,
        at: TimeOfDay,
        scope: Scope,
    },
    DueDate {
        when: DueWhen,
        scope: Scope,
    },
}

/// O tipo sem os dados (o separador do painel de automações).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Family {
    Rule,
    CardButton,
    BoardButton,
    Calendar,
    DueDate,
}

impl Kind {
    #[must_use]
    pub fn family(&self) -> Family {
        match self {
            Kind::Rule { .. } => Family::Rule,
            Kind::CardButton => Family::CardButton,
            Kind::BoardButton { .. } => Family::BoardButton,
            Kind::Calendar { .. } => Family::Calendar,
            Kind::DueDate { .. } => Family::DueDate,
        }
    }
}

/// Os cartões em que um comando ou botão de quadro age: das listas `lists` (vazio = todas) e que
/// passam em TODAS as `conditions`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Scope {
    pub lists: Vec<ListId>,
    pub conditions: Vec<Condition>,
}

/// Como um cartão entrou numa lista.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Added {
    /// Criado nela ou movido para ela.
    Any,
    Created,
    MovedIn,
}

/// O gatilho de uma regra. `None` num filtro = qualquer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Trigger {
    CardAdded {
        list: Option<ListId>,
        how: Added,
    },
    MovedOut {
        list: Option<ListId>,
    },
    Label {
        label: Option<LabelId>,
        added: bool,
    },
    Member {
        member: Option<Member>,
        added: bool,
    },
    /// Uma data de entrega marcada ou mudada.
    DueSet,
    DueRemoved,
    DueDone {
        done: bool,
    },
    ChecklistComplete,
    ItemChecked {
        done: bool,
    },
    /// Um comentário novo; com `contains`, só se o texto o contém (sem caixa nem acentos).
    Commented {
        contains: Option<String>,
    },
    Archived {
        archived: bool,
    },
    FieldChanged {
        field: Option<FieldId>,
    },
}

/// Quem tem de fazer o gesto que dispara a regra.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum By {
    #[default]
    Anyone,
    Only(Member),
    Except(Member),
}

/// Uma condição; `not` nega-a («sem a etiqueta…»).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub test: Test,
    pub not: bool,
}

impl Condition {
    #[must_use]
    pub fn is(test: Test) -> Self {
        Self { test, not: false }
    }
}

/// O que se pergunta ao cartão — os predicados do filtro, mais lista e campo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Test {
    /// Tem uma (`all: false`) ou todas as etiquetas.
    Labels {
        labels: Vec<LabelId>,
        all: bool,
    },
    NoLabels,
    Members {
        members: Vec<Member>,
        all: bool,
    },
    /// O cartão é de quem fez o gesto.
    Mine,
    NoMembers,
    /// A entrega está marcada como concluída.
    Complete,
    Due(DueIn),
    /// A palavra-chave do filtro (título, campos, etiquetas, membros).
    Text(String),
    InList(Vec<ListId>),
    Field {
        field: FieldId,
        is: FieldTest,
    },
    /// Tem checklists e todas estão completas.
    ChecklistsDone,
}

/// A janela da entrega (as do filtro).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DueIn {
    NoDates,
    Overdue,
    NextDay,
    NextWeek,
    NextMonth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldTest {
    Set,
    Empty,
    Equals(FieldValue),
}

/// Uma lista, deste Kanban (`kanban: None`) ou de outro do projecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dest {
    pub kanban: Option<KanbanId>,
    pub list: ListId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum End {
    Top,
    Bottom,
}

/// Os critérios de «ordenar a lista».
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sort {
    Newest,
    Oldest,
    Name,
    Due,
}

/// Uma data relativa a «agora»: `days` dias depois, à hora `at` local (`None` = a mesma hora).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRule {
    pub days: i32,
    pub at: Option<TimeOfDay>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

/// Uma acção. As de cartão agem no cartão do gatilho (ou em cada cartão do `scope`); as de quadro
/// ([`Action::on_board`]) correm uma vez.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    MoveTo {
        to: Dest,
        at: End,
    },
    CopyTo {
        to: Dest,
        at: End,
    },
    MirrorTo {
        to: Dest,
    },
    Archive,
    Unarchive,
    AddLabel(LabelId),
    RemoveLabel(LabelId),
    ClearLabels,
    AddMember(Member),
    RemoveMember(Member),
    /// Junta ao cartão quem fez o gesto.
    JoinMe,
    ClearMembers,
    SetDue(DateRule),
    ClearDue,
    SetStart(DateRule),
    ClearStart,
    MarkDue {
        done: bool,
    },
    /// Uma checklist nova com estes itens.
    AddChecklist {
        title: String,
        items: Vec<String>,
    },
    /// Uma cópia da checklist `from` (de um cartão modelo do mesmo Kanban).
    CopyChecklist {
        from: ChecklistId,
    },
    CheckAll {
        done: bool,
    },
    /// Um comentário com variáveis (`{cardname}`, `{username}`, `{date}`… — ver `vars`).
    Comment(String),
    SetField {
        field: FieldId,
        value: Option<FieldValue>,
    },
    SortList {
        list: ListId,
        by: Sort,
    },
    CreateCard {
        to: Dest,
        title: String,
        at: End,
    },
}

impl Action {
    /// Corre uma vez, sem cartão.
    #[must_use]
    pub fn on_board(&self) -> bool {
        matches!(self, Action::SortList { .. } | Action::CreateCard { .. })
    }
}

/// Os dias de um comando de calendário. Dia do mês além do fim do mês = o último dia (31 → 30 de
/// abril, 29/fev → 28 nos anos comuns): todo tipo dispara pelo menos uma vez por ano.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Every {
    Day,
    /// Bit 0 = segunda … bit 6 = domingo.
    Weekdays(u8),
    MonthDay(u8),
    YearDay {
        month: u8,
        day: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    Days,
    Hours,
}

/// Quando um comando de prazo dispara, em relação à entrega de cada cartão.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DueWhen {
    Before {
        n: u32,
        unit: Unit,
    },
    After {
        n: u32,
        unit: Unit,
    },
    /// No dia da entrega, à hora `at` local.
    OnDay {
        at: TimeOfDay,
    },
}

impl Spec {
    /// Os bytes do corpo: a versão e a automação, em postcard.
    ///
    /// # Errors
    /// Uma agenda que nunca dispara ([`Self::check`]) ou o postcard a recusar.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.check()?;
        postcard::to_allocvec(&(SPEC_VERSION, self)).map_err(|e| e.to_string())
    }

    /// # Errors
    /// Versão desconhecida (de um PH2D mais novo), bytes corrompidos ou uma agenda que nunca dispara.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let (v, _) = postcard::take_from_bytes::<u32>(bytes).map_err(|e| e.to_string())?;
        if v != SPEC_VERSION {
            return Err(format!("automation version {v} != {SPEC_VERSION}"));
        }
        let spec = postcard::from_bytes::<(u32, Self)>(bytes)
            .map(|(_, s)| s)
            .map_err(|e| e.to_string())?;
        spec.check()?;
        Ok(spec)
    }

    /// Recusa o dia ou a hora que nunca chega — sem isto a automação ficava armada e muda. Dia do mês
    /// `1..=31` em qualquer mês (o que passa do fim cai no último dia), mês `1..=12`, pelo menos um
    /// dia da semana, horas `0..24` e minutos `0..60`.
    ///
    /// # Errors
    /// O primeiro campo fora da faixa.
    pub fn check(&self) -> Result<(), String> {
        let time = |t: &TimeOfDay| {
            if t.hour < 24 && t.minute < 60 {
                Ok(())
            } else {
                Err(format!("automation time {}:{} out of range", t.hour, t.minute))
            }
        };
        match &self.kind {
            Kind::Calendar { every, at, .. } => {
                let ok = match *every {
                    Every::Day => true,
                    Every::Weekdays(mask) => mask != 0 && mask < 0x80,
                    Every::MonthDay(d) => (1..=31).contains(&d),
                    Every::YearDay { month, day } => {
                        (1..=12).contains(&month) && (1..=31).contains(&day)
                    }
                };
                if !ok {
                    return Err(format!("automation day {every:?} never comes"));
                }
                time(at)?;
            }
            Kind::DueDate {
                when: DueWhen::OnDay { at },
                ..
            } => time(at)?,
            _ => {}
        }
        for a in &self.actions {
            if let Action::SetDue(r) | Action::SetStart(r) = a
                && let Some(t) = &r.at
            {
                time(t)?;
            }
        }
        Ok(())
    }

    /// O `Put` que cria esta automação no fim das de `k`. Os comandos agendados contam a partir de
    /// `now` (nunca correm o que passou antes de existirem).
    #[must_use]
    pub fn create(&self, k: &Kanban, now: Moment) -> KanbanOp {
        let last = k
            .automations
            .values()
            .filter(|a| !a.meta.deleted)
            .map(|a| &a.pos)
            .max();
        KanbanOp::put(Record::Automation(Automation {
            id: AutomationId(k.mint()),
            pos: FracKey::between(last, None),
            body: self.to_bytes().unwrap_or_default(),
            last_run: Some(now),
            meta: Meta::default(),
        }))
    }

    /// O `Put` que troca o corpo da automação `a` por este (sítio e último disparo ficam).
    #[must_use]
    pub fn update(&self, a: &Automation) -> KanbanOp {
        KanbanOp::put(Record::Automation(Automation {
            body: self.to_bytes().unwrap_or_default(),
            ..a.clone()
        }))
    }
}

/// As automações vivas de `k` que esta versão lê, pela ordem (`pos`, o id desempata) — a ordem em
/// que disparam. Um corpo ilegível (de um PH2D mais novo) fica no documento e não corre.
#[must_use]
pub fn specs(k: &Kanban) -> Vec<(AutomationId, Spec)> {
    let mut v: Vec<&Automation> = k.automations.values().filter(|a| !a.meta.deleted).collect();
    v.sort_by(|a, b| a.pos.cmp(&b.pos).then(a.id.cmp(&b.id)));
    v.into_iter()
        .filter_map(|a| Some((a.id, Spec::from_bytes(&a.body).ok()?)))
        .collect()
}
