//! ⭐ **W6 — o que o Trello faz com os cartões sem os abrir**, na aba E no elemento do Quadro (a
//! vista partilhada): o FILTRO (`F`, o botão «Filters» com o número), a PESQUISA (`/`), a EDIÇÃO
//! RÁPIDA (`E`), copiar/mover/espelhar, os MODELOS, o ENVELHECIMENTO e os atalhos que faltavam.
//!
//! O estado mora na vista ([`W6`], na [`crate::tab::Tab`]) — o filtro é da janela, nunca do
//! documento. Cada painel é uma [`sheet::Sheet`] (dados); o pintor regista os alvos e o clique
//! seguinte lê-os, como a disposição VISTA do quadro. Os gestos escrevem pelas portas puras do
//! `ph2d-kanban-model`/`ph2d-kanban-filter` ([`gesture`]).

pub(crate) mod aging;
pub(crate) mod complete;
pub(crate) mod copy_move;
mod filter_bar;
pub(crate) mod gesture;
mod help;
mod probe;
pub(crate) mod quick_edit;
mod search;
pub(crate) mod sheet;
pub(crate) mod shortcuts;

use crate::docs::KanbanDocs;
use crate::edit::{Field, Outcome, Target};
use crate::tab::{Hit, Tab};
use ph2d_board_render::KanbanLod;
use ph2d_editor_core::doc_surface::{Compose, KeyInput, KeyOutcome};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_kanban_filter::{ActivityWindow, DueWindow, Filter, MatchMode, SortBy};
use ph2d_kanban_model::{CardId, CardRef, Kanban, KanbanId, KanbanSet, LabelId, ListId, Member};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::cell::{Cell, RefCell};

pub(crate) use copy_move::{Dialog, Kind as DialogKind};

/// O painel aberto por cima do quadro (um de cada vez).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Over {
    Filter,
    Search,
    Help,
}

/// A caixa de texto com o teclado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Boxed {
    Keyword,
    Query,
    Title,
}

/// Uma linha do menu da edição rápida (o idioma do Trello).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Quick {
    Open,
    Labels,
    Members,
    Cover,
    Dates,
    Move,
    Copy,
    Mirror,
    Archive,
}

/// Qual lista suspensa de um diálogo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Pick {
    Kanban,
    List,
    Position,
}

/// O que se acerta num diálogo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dlg {
    Open(Pick),
    Kanban(KanbanId),
    List(ListId),
    Position(usize),
    /// Manter: 0 checklists · 1 etiquetas · 2 membros · 3 anexos · 4 comentários.
    Keep(u8),
    KeepCards,
    Title,
    Confirm,
    Sort(SortBy),
    /// O envelhecimento: 0 desligado · 1 regular · 2 pirata.
    Aging(u8),
    Template(CardRef),
}

/// Um alvo da W6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Act {
    /// Dentro de um painel, sem alvo (o clique não o fecha).
    Inside,
    Close,
    FilterButton,
    ClearAll,
    SearchBox,
    Keyword,
    Member(u16),
    NoMembers,
    Mine,
    Complete,
    Incomplete,
    Due(DueWindow),
    Activity(ActivityWindow),
    Label(LabelId),
    NoLabels,
    Mode(MatchMode),
    Result(CardRef),
    Quick(Quick),
    QuickSave,
    Dlg(Dlg),
    /// Uma sugestão do `@` `#` `^` no título do cartão novo.
    Suggest(u8),
}

/// O estado W6 de uma vista.
#[derive(Debug, Default)]
pub(crate) struct W6 {
    pub filter: Filter,
    pub over: Option<Over>,
    pub input: Option<(Boxed, Field)>,
    pub query: String,
    pub found: Vec<ph2d_kanban_filter::Hit>,
    pub pick: usize,
    /// A linha com a edição rápida aberta.
    pub quick: Option<CardId>,
    pub dialog: Option<Dialog>,
    /// A rolagem do painel aberto, e o fundo dela no último quadro.
    pub scroll: f32,
    pub scroll_max: Cell<f32>,
    pub hover: Option<Act>,
    pub press: Option<Act>,
    /// Os alvos pintados no último quadro, por ordem (o último por cima), e onde começam os do
    /// diálogo (que, aberto, é o único que se acerta).
    pub hits: RefCell<Vec<(Rect, Act)>>,
    pub dialog_from: Cell<Option<usize>>,
    /// Os painéis pintados (a roda rola o que está por baixo dela).
    pub sheets: RefCell<Vec<Rect>>,
}

impl W6 {
    /// Há algo da W6 aberto por cima do quadro.
    pub fn open(&self) -> bool {
        self.over.is_some() || self.quick.is_some() || self.dialog.is_some()
    }

    /// O que está em `(x, y)` (o mais de cima primeiro).
    pub fn hit(&self, x: f32, y: f32) -> Option<Act> {
        let hits = self.hits.borrow();
        let from = if self.dialog.is_some() {
            self.dialog_from.get().unwrap_or(hits.len())
        } else {
            0
        };
        hits[from.min(hits.len())..]
            .iter()
            .rev()
            .find(|(r, _)| r.contains(x, y))
            .map(|(_, a)| *a)
    }

    /// O texto da caixa `b` vai para onde ela escreve.
    fn sync(&mut self, set: &KanbanSet, me: &Member) {
        let Some((b, f)) = self.input.as_ref() else {
            return;
        };
        let text = f.text.clone();
        match b {
            Boxed::Keyword => self.filter.keyword = text,
            Boxed::Query => {
                if text != self.query {
                    let zone = ph2d_kanban_dates::Zone::system();
                    self.found = if text.trim().is_empty() {
                        Vec::new()
                    } else {
                        ph2d_kanban_filter::search(set, &text, crate::actions::now(), &zone, me)
                    };
                    self.query = text;
                    self.pick = 0;
                }
            }
            Boxed::Title => {
                if let Some(d) = self.dialog.as_mut() {
                    d.title = text;
                }
            }
        }
    }

    /// Foca a caixa `b` com `text` (o cursor no fim).
    pub fn focus(&mut self, b: Boxed, text: &str) {
        let mut f = Field::open(Target::Scratch, text);
        f.anchor = f.caret;
        self.input = Some((b, f));
    }

    /// Fecha o que está por cima (o diálogo primeiro). `true` = havia.
    pub fn close_top(&mut self) -> bool {
        if self.dialog.take().is_some() {
            if matches!(self.input, Some((Boxed::Title, _))) {
                self.input = None;
            }
            return true;
        }
        if self.over.take().is_some() {
            self.input = None;
            self.scroll = 0.0;
            return true;
        }
        false
    }
}

/// O avatar de uma pessoa (o do verso) numa folha.
pub(crate) fn sheet_avatar(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    (x, y): (f32, f32),
    who: &Member,
) {
    let d = sheet::AVATAR;
    let hue = crate::front::avatar_hue(who);
    let (fill, _) = crate::paint::label(
        theme,
        ph2d_kanban_model::LabelColor {
            hue,
            shade: ph2d_kanban_model::Shade::Normal,
        },
    );
    ph2d_editor_core::paint::fill_rounded_rect(scene, Rect::new(x, y, d, d), d / 2.0, fill);
    let t = crate::front::initials(who);
    let w = crate::docs::title_width(ts, &t, crate::metrics::SMALL_PX);
    let ty = crate::paint::text_top(ts, y, d, crate::metrics::SMALL_PX);
    ph2d_editor_core::paint::paint_text_title_elided(
        ts,
        scene,
        &t,
        x + (d - w) / 2.0,
        ty,
        crate::metrics::SMALL_PX,
        d,
        crate::paint::c(theme, ph2d_tokens::ColorToken::KanbanAvatarInk),
    );
}

/// As pessoas que o filtro oferece: as das equipas e as que estão nos cartões, por nome.
pub(crate) fn people(k: &Kanban, team: &[String]) -> Vec<Member> {
    let mut v: Vec<Member> = team.iter().map(|t| Member(t.clone())).collect();
    for c in k.cards.values().filter(|c| !c.meta.deleted) {
        v.extend(c.members.iter().cloned());
    }
    v.sort();
    v.dedup();
    v
}

/// ⭐ Pinta o que é da W6 por baixo do verso: os botões do cabeçalho, a edição rápida e o painel.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    (k, set): (&Kanban, &KanbanSet),
    tab: &mut Tab,
    (me, team, floor): (&Member, &[String], Option<f32>),
    lod: KanbanLod,
) {
    tab.w6.hits.borrow_mut().clear();
    tab.w6.sheets.borrow_mut().clear();
    tab.w6.dialog_from.set(None);
    if lod != KanbanLod::High {
        return;
    }
    let anchors = filter_bar::header(scene, ts, theme, tab);
    let area = above(tab.layout.area, floor);
    if tab.w6.quick.is_some() {
        quick_edit::paint(scene, ts, theme, k, tab);
    }
    // As sugestões do `@` `#` `^` por baixo do compositor do cartão novo.
    let composer = tab.layout.lists.iter().find_map(|l| l.composer);
    if let (Some(at), Some(s)) = (composer, complete::sheet(tab, k, me, team)) {
        show(scene, ts, theme, tab, &s, at, false, area);
    }
    if let Some(over) = tab.w6.over {
        let (sheet, anchor, right) = match over {
            Over::Filter => (filter_bar::sheet(k, tab, team), anchors.0, true),
            Over::Search => (search::sheet(tab, set), anchors.1, true),
            Over::Help => (help::sheet(), help::anchor(tab.layout.area), false),
        };
        show(scene, ts, theme, tab, &sheet, anchor, right, area);
    }
}

/// A área sem o chrome do hero por baixo (o HUD): as folhas param acima dele, como as listas.
pub(crate) fn above(area: Rect, floor: Option<f32>) -> Rect {
    match floor {
        Some(f) if f > area.y && f < area.y + area.h => Rect::new(area.x, area.y, area.w, f - area.y),
        _ => area,
    }
}

/// Uma folha pintada e os alvos dela registados.
#[allow(clippy::too_many_arguments)]
pub(crate) fn show(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tab: &Tab,
    s: &sheet::Sheet,
    anchor: Rect,
    right: bool,
    area: Rect,
) {
    let r = sheet::place(s, anchor, area, right);
    let max = (s.content_h() - r.h).max(0.0);
    let scroll = tab.w6.scroll.clamp(0.0, max);
    tab.w6.scroll_max.set(max);
    let hits = sheet::paint(scene, ts, theme, s, r, scroll, tab.w6.hover);
    tab.w6.hits.borrow_mut().extend(hits);
    tab.w6.sheets.borrow_mut().push(r);
}

/// ⭐ O diálogo (copiar, mover, espelhar, ordenar…) — por cima de tudo, até do verso.
pub(crate) fn paint_dialog(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    set: &KanbanSet,
    tab: &Tab,
    area: Rect,
) {
    let Some(d) = tab.w6.dialog.as_ref() else {
        return;
    };
    let input = match &tab.w6.input {
        Some((Boxed::Title, f)) => Some(f),
        _ => None,
    };
    let s = copy_move::sheet(d, set, input);
    tab.w6.dialog_from.set(Some(tab.w6.hits.borrow().len()));
    show(scene, ts, theme, tab, &s, d.anchor, false, area);
}

/// ⭐ O rato. `Some(tomado)` = é da W6; `None` = segue para o quadro (ou o verso).
pub(crate) fn pointer(
    docs: &mut KanbanDocs,
    id: u64,
    kind: PointerKind,
    x: f32,
    y: f32,
) -> Option<bool> {
    let tab = docs.tab_mut(id)?;
    if tab.back.is_some() && tab.w6.dialog.is_none() {
        return None;
    }
    let hit = tab.w6.hit(x, y);
    let open = tab.w6.open();
    match kind {
        PointerKind::Move => {
            tab.w6.hover = hit;
            if open || hit.is_some() || tab.w6.press.is_some() {
                tab.hover = hit.map(Hit::W6);
                return Some(true);
            }
            None
        }
        PointerKind::Down => {
            if let Some(a) = hit {
                tab.w6.press = Some(a);
                return Some(true);
            }
            if open {
                close_top(docs, id);
                return Some(true);
            }
            None
        }
        PointerKind::Up => {
            let p = tab.w6.press.take()?;
            if hit == Some(p) {
                act(docs, id, p);
            }
            Some(true)
        }
    }
}

/// Fecha o que está por cima; a edição rápida grava o título (clicar fora, o Trello).
pub(crate) fn close_top(docs: &mut KanbanDocs, id: u64) {
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    if t.w6.close_top() {
        return;
    }
    if t.w6.quick.take().is_some() {
        crate::pointer::commit_field(docs, id);
    }
}

/// A roda sobre um painel rola-o. `true` = tomada.
pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, x: f32, y: f32, dy: f32) -> bool {
    let Some(t) = docs.tab_mut(id) else {
        return false;
    };
    if !t.w6.sheets.borrow().iter().any(|r| r.contains(x, y)) {
        return t.w6.open() && t.back.is_none();
    }
    let max = t.w6.scroll_max.get();
    t.w6.scroll = (t.w6.scroll.min(max) + dy).clamp(0.0, max);
    true
}

/// ⭐ Uma ação acertada.
pub(crate) fn act(docs: &mut KanbanDocs, id: u64, a: Act) {
    match a {
        Act::Inside => {}
        Act::Close => close_top(docs, id),
        Act::FilterButton => toggle_over(docs, id, Over::Filter),
        Act::SearchBox | Act::Result(_) => search::click(docs, id, a),
        Act::Quick(_) | Act::QuickSave => quick_edit::click(docs, id, a),
        Act::Dlg(d) => copy_move::click(docs, id, d),
        Act::Suggest(i) => complete::pick(docs, id, i),
        _ => filter_bar::click(docs, id, a),
    }
}

/// Abre (ou fecha) um painel; o filtro e a pesquisa abrem com o teclado na caixa deles.
pub(crate) fn toggle_over(docs: &mut KanbanDocs, id: u64, o: Over) {
    let Some(t) = docs.tab_mut(id) else {
        return;
    };
    let was = t.w6.over;
    t.w6.close_top();
    t.menu = None;
    if was == Some(o) {
        return;
    }
    t.w6.over = Some(o);
    t.w6.scroll = 0.0;
    match o {
        Over::Filter => {
            let kw = t.w6.filter.keyword.clone();
            t.w6.focus(Boxed::Keyword, &kw);
        }
        Over::Search => {
            let q = t.w6.query.clone();
            t.w6.focus(Boxed::Query, &q);
        }
        Over::Help => {}
    }
}

/// O texto escrito na caixa com o teclado. `None` = não há caixa.
fn type_into(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> Option<Outcome> {
    let me = docs.me.clone();
    let set = docs.set();
    let t = docs.tab_mut(id)?;
    let (_, f) = t.w6.input.as_mut()?;
    let out = f.key(input.key, input.mods, input.text, input.paste.as_deref());
    t.w6.sync(&set.borrow(), &me);
    Some(out)
}

/// ⭐ O teclado. `Some` = é da W6 (tomado ou não); `None` = segue (o verso, o quadro).
pub(crate) fn key(
    docs: &mut KanbanDocs,
    id: u64,
    ts: &mut TextSystem,
    input: &KeyInput<'_>,
) -> Option<KeyOutcome> {
    let t = docs.tab_mut(id)?;
    if t.w6.dialog.is_some() {
        return Some(copy_move::key(docs, id, input));
    }
    if t.back.is_some() {
        return None;
    }
    if let Some(o) = t.w6.over {
        return Some(match o {
            Over::Filter => filter_bar::key(docs, id, input),
            Over::Search => search::key(docs, id, input),
            Over::Help => help::key(docs, id, input),
        });
    }
    if t.w6.quick.is_some() {
        return quick_edit::key(docs, id, ts, input);
    }
    None
}

/// A composição (IME, carácter confirmado) para a caixa com o teclado. `true` = tomada.
pub(crate) fn compose(docs: &mut KanbanDocs, id: u64, c: &Compose<'_>) -> bool {
    let me = docs.me.clone();
    let set = docs.set();
    let Some(t) = docs.tab_mut(id) else {
        return false;
    };
    let Some((_, f)) = t.w6.input.as_mut() else {
        return false;
    };
    if let Compose::Commit(ch) = c {
        f.insert(ch.encode_utf8(&mut [0; 4]));
        t.w6.sync(&set.borrow(), &me);
    }
    true
}

/// O que um campo de painel fez com uma tecla, como resposta ao teclado.
pub(crate) fn outcome(o: Option<Outcome>) -> KeyOutcome {
    match o {
        Some(Outcome::Copy(s)) => KeyOutcome {
            consumed: true,
            copy: Some(s),
        },
        Some(Outcome::Ignored) | None => KeyOutcome::default(),
        Some(_) => taken(),
    }
}

pub(crate) fn taken() -> KeyOutcome {
    KeyOutcome {
        consumed: true,
        copy: None,
    }
}
