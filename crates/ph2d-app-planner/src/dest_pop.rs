//! ⭐ **«Move to Kanban»** — o destino de um item da Inbox: a pesquisa, os Kanbans com as listas
//! deles (escolhe-se a lista), a posição (topo ou fundo) e «Move». O idioma é o do diálogo «Move»
//! do Kanban (`ph2d-app-kanban/src/w6/copy_move.rs`: Kanban → lista → posição); aqui a escolha é
//! uma lista só, filtrada pela pesquisa, porque o item ainda não mora em lugar nenhum.
//!
//! ⚠️ Duplicação registada: a folha do W6 é do `ph2d-app-kanban` (privada); quando outro consumidor
//! aparecer, o seletor de destino sobe para o `ph2d-editor-core`.

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, stroke_rounded_rect};
use ph2d_editor_core::widget::{FieldKey, LineField, Popover, paint_popover};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{InboxId, KanbanId, KanbanSet, ListRef};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::VectorScene;

use crate::event_pop::chip;
use crate::metrics as m;
use crate::ui::{c, text_top};

/// Um sítio do popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestHit {
    Search,
    List(ListRef),
    Top(bool),
    Move,
}

/// Uma linha da lista de destinos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    /// O nome do Kanban (não se escolhe).
    Kanban(String),
    List(ListRef, String),
}

/// O popover aberto.
#[derive(Clone, Debug)]
pub struct DestPop {
    pub item: InboxId,
    pub search: LineField,
    pub pick: Option<ListRef>,
    pub top: bool,
    /// A primeira linha à vista (a roda anda por elas).
    pub scroll: usize,
    pub rect: Rect,
    pub anchor: Rect,
    pub hover: Option<DestHit>,
}

/// O que um gesto fez ao popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DestDone {
    Stay,
    Close,
    Move { to: ListRef, top: bool },
}

/// As linhas que a pesquisa `q` deixa: um Kanban cujo nome casa mostra todas as listas; senão, só
/// as listas que casam (e o Kanban delas por cima).
#[must_use]
pub fn rows(set: &KanbanSet, q: &str) -> Vec<Row> {
    let q = q.trim().to_lowercase();
    let mut out = Vec::new();
    for k in set.iter() {
        let all = q.is_empty() || k.header.name.to_lowercase().contains(&q);
        let lists: Vec<Row> = k
            .lists_in_order()
            .into_iter()
            .filter(|l| !l.archived && (all || l.title.to_lowercase().contains(&q)))
            .map(|l| {
                Row::List(
                    ListRef {
                        kanban: k.id,
                        list: l.id,
                    },
                    l.title.clone(),
                )
            })
            .collect();
        if !lists.is_empty() {
            out.push(Row::Kanban(k.header.name.clone()));
            out.extend(lists);
        }
    }
    out
}

fn first_pick(rows: &[Row]) -> Option<ListRef> {
    rows.iter().find_map(|r| match r {
        Row::List(l, _) => Some(*l),
        Row::Kanban(_) => None,
    })
}

impl DestPop {
    /// O popover do item `item`, ancorado no botão `anchor`; a escolha começa na 1.ª lista.
    #[must_use]
    pub fn new(set: &KanbanSet, item: InboxId, anchor: Rect) -> Self {
        Self {
            item,
            search: LineField::new(""),
            pick: first_pick(&rows(set, "")),
            top: false,
            scroll: 0,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            anchor,
            hover: None,
        }
    }

    /// Quantas linhas de destino cabem em `h` de altura total.
    fn fit(h: f32) -> usize {
        let fixed = Self::height(0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((h - fixed) / m::ROW_H).floor().max(1.0) as usize;
        n
    }

    /// A altura com `n` linhas de destino.
    #[must_use]
    pub fn height(n: usize) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        let list = n as f32 * m::ROW_H;
        m::PAD
            + m::SECTION_H
            + m::FIELD_H
            + m::GAP
            + list
            + m::GAP
            + m::SECTION_H
            + m::BUTTON_H
            + m::GAP
            + m::BUTTON_H
            + m::PAD
    }

    /// Põe o popover debaixo da âncora, dentro de `area`, com as linhas que couberem.
    pub fn place(&mut self, set: &KanbanSet, area: Rect) {
        let n = rows(set, &self.search.text).len();
        let room = area.y + area.h - (self.anchor.y + self.anchor.h) - m::PAD;
        let shown = n.min(Self::fit(room)).max(1);
        self.scroll = self.scroll.min(n.saturating_sub(shown));
        let h = Self::height(shown);
        let x = self
            .anchor
            .x
            .min(area.x + area.w - m::POP_W - m::PAD)
            .max(area.x + m::PAD);
        let y = (self.anchor.y + self.anchor.h)
            .min(area.y + area.h - h)
            .max(area.y);
        self.rect = Rect::new(x, y, m::POP_W, h);
    }

    fn shown(&self) -> usize {
        let fixed = Self::height(0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let n = ((self.rect.h - fixed) / m::ROW_H).round().max(0.0) as usize;
        n
    }

    /// Os sítios de clique e as linhas pintadas (`(hit, rect, row)`).
    fn layout(&self, set: &KanbanSet) -> (Rect, Vec<(Rect, Row)>, [Rect; 2], Rect) {
        let r = self.rect;
        let x = r.x + m::PAD;
        let w = r.w - 2.0 * m::PAD;
        let search = Rect::new(x, r.y + m::PAD + m::SECTION_H, w, m::FIELD_H);
        let mut y = search.y + search.h + m::GAP;
        let all = rows(set, &self.search.text);
        let mut lines = Vec::new();
        for row in all.into_iter().skip(self.scroll).take(self.shown()) {
            lines.push((Rect::new(x, y, w, m::ROW_H), row));
            y += m::ROW_H;
        }
        #[allow(clippy::cast_precision_loss)]
        let list_end = search.y + search.h + m::GAP + self.shown() as f32 * m::ROW_H;
        let seg_y = list_end + m::GAP + m::SECTION_H;
        let half = (w - m::GAP) / 2.0;
        let seg = [
            Rect::new(x, seg_y, half, m::BUTTON_H),
            Rect::new(x + half + m::GAP, seg_y, half, m::BUTTON_H),
        ];
        let go = Rect::new(x, seg_y + m::BUTTON_H + m::GAP, w, m::BUTTON_H);
        (search, lines, seg, go)
    }

    /// Os sítios de clique.
    #[must_use]
    pub fn hits(&self, set: &KanbanSet) -> Vec<(DestHit, Rect)> {
        let (search, lines, seg, go) = self.layout(set);
        let mut v = vec![(DestHit::Search, search)];
        v.extend(lines.into_iter().filter_map(|(r, row)| match row {
            Row::List(l, _) => Some((DestHit::List(l), r)),
            Row::Kanban(_) => None,
        }));
        v.push((DestHit::Top(true), seg[0]));
        v.push((DestHit::Top(false), seg[1]));
        v.push((DestHit::Move, go));
        v
    }

    pub fn paint(
        &self,
        set: &KanbanSet,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: Theme,
    ) {
        let (search, lines, seg, go) = self.layout(set);
        paint_popover(&Popover::new(crate::POP_NODE), self.rect, scene, theme);
        let x = search.x;
        let w = search.w;
        let caption = |ts: &mut TextSystem, scene: &mut VectorScene, y: f32, key: &str| {
            let ty = text_top(ts, y, m::SECTION_H, m::SMALL_PX);
            paint_text_elided(
                ts,
                scene,
                tr(key),
                x,
                ty,
                m::SMALL_PX,
                w,
                c(theme, T::Text2),
            );
        };
        caption(ts, scene, self.rect.y + m::PAD, "planner.move.title");
        fill_rounded_rect(scene, search, m::RADIUS, c(theme, T::Bg2));
        stroke_rounded_rect(scene, search, m::RADIUS, 1.0, c(theme, T::Accent));
        let (text, ink) = if self.search.text.is_empty() {
            (tr("planner.move.search"), T::Text2)
        } else {
            (self.search.text.as_str(), T::Text1)
        };
        let ty = text_top(ts, search.y, search.h, m::TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            text,
            search.x + m::BUTTON_PAD_X,
            ty,
            m::TEXT_PX,
            search.w - 2.0 * m::BUTTON_PAD_X,
            c(theme, ink),
        );
        if lines.is_empty() {
            let ty = text_top(ts, search.y + search.h + m::GAP, m::ROW_H, m::TEXT_PX);
            paint_text_elided(
                ts,
                scene,
                tr("planner.move.none"),
                x,
                ty,
                m::TEXT_PX,
                w,
                c(theme, T::Text2),
            );
        }
        for (r, row) in &lines {
            match row {
                Row::Kanban(name) => {
                    let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
                    paint_text_elided(
                        ts,
                        scene,
                        name,
                        r.x,
                        ty,
                        m::SMALL_PX,
                        r.w,
                        c(theme, T::KanbanListMeta),
                    );
                }
                Row::List(l, title) => {
                    let on = self.pick == Some(*l);
                    let hover = self.hover == Some(DestHit::List(*l));
                    let (bg, fg) = match (on, hover) {
                        (true, _) => (Some(T::Accent), T::AccentFg),
                        (false, true) => (Some(T::Bg3), T::Text1),
                        (false, false) => (None, T::Text1),
                    };
                    if let Some(bg) = bg {
                        fill_rounded_rect(scene, *r, m::RADIUS, c(theme, bg));
                    }
                    let ty = text_top(ts, r.y, r.h, m::TEXT_PX);
                    let tx = r.x + m::BUTTON_PAD_X;
                    paint_text_elided(
                        ts,
                        scene,
                        title,
                        tx,
                        ty,
                        m::TEXT_PX,
                        r.w - 2.0 * m::BUTTON_PAD_X,
                        c(theme, fg),
                    );
                }
            }
        }
        caption(ts, scene, seg[0].y - m::SECTION_H, "planner.move.position");
        chip(scene, ts, theme, seg[0], tr("planner.move.top"), self.top);
        chip(
            scene,
            ts,
            theme,
            seg[1],
            tr("planner.move.bottom"),
            !self.top,
        );
        let ready = self.pick.is_some();
        let (bg, fg) = if ready {
            (T::Accent, T::AccentFg)
        } else {
            (T::Bg2, T::Text2)
        };
        fill_rounded_rect(scene, go, m::RADIUS, c(theme, bg));
        let label = tr("planner.move.go");
        let lw = ts.prefix_width(label, m::TEXT_PX).min(go.w);
        let ty = text_top(ts, go.y, go.h, m::TEXT_PX);
        paint_text_elided(
            ts,
            scene,
            label,
            go.x + (go.w - lw) / 2.0,
            ty,
            m::TEXT_PX,
            go.w,
            c(theme, fg),
        );
    }

    /// Um clique (dentro ou fora: fora fecha).
    pub fn click(&mut self, set: &KanbanSet, x: f32, y: f32) -> DestDone {
        if !self.rect.contains(x, y) {
            return DestDone::Close;
        }
        let hit = self
            .hits(set)
            .into_iter()
            .find(|(_, r)| r.contains(x, y))
            .map(|(h, _)| h);
        match hit {
            Some(DestHit::List(l)) => self.pick = Some(l),
            Some(DestHit::Top(t)) => self.top = t,
            Some(DestHit::Move) => return self.go(),
            Some(DestHit::Search) | None => {}
        }
        DestDone::Stay
    }

    fn go(&self) -> DestDone {
        self.pick
            .map_or(DestDone::Stay, |to| DestDone::Move { to, top: self.top })
    }

    /// A pesquisa mudou: a escolha passa para a 1.ª lista que ficou, se a dela saiu.
    fn searched(&mut self, set: &KanbanSet) {
        let all = rows(set, &self.search.text);
        let kept = all
            .iter()
            .any(|r| matches!(r, Row::List(l, _) if Some(*l) == self.pick));
        if !kept {
            self.pick = first_pick(&all);
        }
        self.scroll = 0;
    }

    /// Texto composto (a pesquisa tem sempre o teclado).
    pub fn insert(&mut self, set: &KanbanSet, s: &str) {
        self.search.insert(s);
        self.searched(set);
    }

    /// Uma tecla: Enter move, Esc fecha, as setas andam pelas listas, o resto escreve na pesquisa.
    pub fn key(&mut self, set: &KanbanSet, input: &KeyInput<'_>) -> DestDone {
        if !input.pressed {
            return DestDone::Stay;
        }
        if matches!(input.key, Key::Down | Key::Up) {
            let lists: Vec<ListRef> = rows(set, &self.search.text)
                .into_iter()
                .filter_map(|r| match r {
                    Row::List(l, _) => Some(l),
                    Row::Kanban(_) => None,
                })
                .collect();
            let at = lists.iter().position(|l| Some(*l) == self.pick);
            let next = match (input.key, at) {
                (Key::Down, Some(i)) => (i + 1).min(lists.len().saturating_sub(1)),
                (Key::Up, Some(i)) => i.saturating_sub(1),
                _ => 0,
            };
            self.pick = lists.get(next).copied();
            return DestDone::Stay;
        }
        match self.search.key(input.key, input.text) {
            FieldKey::Commit => self.go(),
            FieldKey::Cancel => DestDone::Close,
            FieldKey::Edited => {
                self.searched(set);
                DestDone::Stay
            }
            _ => DestDone::Stay,
        }
    }

    /// A roda sobre a lista.
    pub fn wheel(&mut self, set: &KanbanSet, dy: f32) {
        let n = rows(set, &self.search.text).len();
        let max = n.saturating_sub(self.shown());
        self.scroll = if dy > 0.0 {
            (self.scroll + 1).min(max)
        } else {
            self.scroll.saturating_sub(1)
        };
    }

    /// O Kanban da escolha (as fotos e os testes).
    #[must_use]
    pub fn kanban(&self) -> Option<KanbanId> {
        self.pick.map(|l| l.kanban)
    }
}
