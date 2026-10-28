//! **A pista** — o que uma corrida de script alcança: uma CÓPIA de trabalho do projecto (cada
//! escrita aplica-se logo, a leitura seguinte vê-a) e as escritas por ordem, com o autor de cada
//! uma. Vive nos dados da VM só durante a corrida ([`crate::run`]); fora dela as funções dizem porquê.

use std::fmt::Display;

use mlua::Lua;
use ph2d_kanban_dates::Zone;
use ph2d_kanban_filter::normalize;
use ph2d_kanban_model::{
    Card, CardId, CardRef, Hue, Kanban, KanbanId, KanbanOp, KanbanSet, Label, LabelColor, List,
    ListId, Member, Moment, Plan, Shade,
};

/// O autor das escritas de um script na actividade.
#[must_use]
pub fn script_member(name: &str) -> Member {
    Member(format!("script: {name}"))
}

pub(crate) struct Lane {
    pub set: KanbanSet,
    /// `(autor, Kanban, lote)` pela ordem em que se aplicaram à cópia.
    pub writes: Vec<(Member, KanbanId, Vec<KanbanOp>)>,
    pub by: Member,
    /// A pessoa que corre o script (o `@me` das pesquisas).
    pub me: Member,
    pub now: Moment,
    pub zone: Zone,
    /// Os `ph2d.kanban.on` desta corrida.
    pub handlers: Vec<(String, mlua::Function)>,
}

const NOT_HERE: &str = "the Kanbans are not open to this script right now";

pub(crate) fn err(f: &str, msg: impl Display) -> mlua::Error {
    mlua::Error::RuntimeError(format!("{f}: {msg}"))
}

/// `body` com a pista; sem corrida, o erro.
pub(crate) fn with<R>(
    lua: &Lua,
    f: &str,
    body: impl FnOnce(&mut Lane) -> mlua::Result<R>,
) -> mlua::Result<R> {
    let mut lane = lua.app_data_mut::<Lane>().ok_or_else(|| err(f, NOT_HERE))?;
    body(&mut lane)
}

/// O nome de uma cor de etiqueta, como o Trello o exporta (`green`, `green_light`, `green_dark`).
#[must_use]
pub(crate) fn color_name(c: LabelColor) -> String {
    let hue = match c.hue {
        Hue::Green => "green",
        Hue::Yellow => "yellow",
        Hue::Orange => "orange",
        Hue::Red => "red",
        Hue::Purple => "purple",
        Hue::Blue => "blue",
        Hue::Sky => "sky",
        Hue::Lime => "lime",
        Hue::Pink => "pink",
        Hue::Black => "black",
    };
    match c.shade {
        Shade::Subtle => format!("{hue}_light"),
        Shade::Normal => hue.to_owned(),
        Shade::Bold => format!("{hue}_dark"),
    }
}

impl Lane {
    pub fn write(&mut self, k: KanbanId, ops: Vec<KanbanOp>) {
        if ops.is_empty() {
            return;
        }
        self.set.apply(k, ops.clone());
        self.writes.push((self.by.clone(), k, ops));
    }

    pub fn write_plan(&mut self, p: Plan, by: &Member) {
        for (k, ops) in p.into_steps() {
            if !ops.is_empty() {
                self.set.apply(k, ops.clone());
                self.writes.push((by.clone(), k, ops));
            }
        }
    }

    pub fn kanban(&self, f: &str, k: KanbanId) -> mlua::Result<&Kanban> {
        self.set
            .get(k)
            .filter(|_| k != KanbanId::AGENDA)
            .ok_or_else(|| err(f, "this Kanban is no longer in the project"))
    }

    pub fn list(&self, f: &str, k: KanbanId, l: ListId) -> mlua::Result<&List> {
        self.kanban(f, k)?
            .lists
            .get(&l)
            .filter(|l| !l.meta.deleted)
            .ok_or_else(|| err(f, "this list was deleted"))
    }

    pub fn card(&self, f: &str, k: KanbanId, c: CardId) -> mlua::Result<&Card> {
        self.set
            .card(CardRef { kanban: k, card: c })
            .filter(|c| !c.meta.deleted)
            .ok_or_else(|| err(f, "this card was deleted"))
    }

    /// A primeira lista viva e aberta com este título (sem caixa nem acentos).
    pub fn list_named(&self, f: &str, k: KanbanId, name: &str) -> mlua::Result<Option<ListId>> {
        let want = normalize(name);
        Ok(self
            .kanban(f, k)?
            .lists_in_order()
            .into_iter()
            .find(|l| normalize(&l.title) == want)
            .map(|l| l.id))
    }

    /// A etiqueta pelo nome, ou pela cor numa etiqueta sem nome; o erro diz as que há.
    pub fn label_named(&self, f: &str, k: KanbanId, name: &str) -> mlua::Result<&Label> {
        let kb = self.kanban(f, k)?;
        let want = normalize(name);
        let live = || kb.labels.values().filter(|l| !l.meta.deleted);
        live()
            .find(|l| !l.name.is_empty() && normalize(&l.name) == want)
            .or_else(|| {
                live().find(|l| l.name.is_empty() && l.color.is_some_and(|c| color_name(c) == want))
            })
            .ok_or_else(|| {
                let has: Vec<String> = live()
                    .map(|l| match l.color {
                        _ if !l.name.is_empty() => format!("\"{}\"", l.name),
                        Some(c) => format!("\"{}\"", color_name(c)),
                        None => String::new(),
                    })
                    .filter(|s| !s.is_empty())
                    .collect();
                err(
                    f,
                    format!(
                        "no label \"{name}\" in \"{}\" (it has {})",
                        kb.header.name,
                        if has.is_empty() {
                            "none".to_owned()
                        } else {
                            has.join(", ")
                        }
                    ),
                )
            })
    }
}
