//! ⭐ **As outras vistas na aba e no elemento do Quadro** — a Tabela, o Calendário, a Linha do tempo e
//! o Painel da `ph2d-kanban-views` pintam-se no corpo (por baixo do cabeçalho), nos px da VISTA que a
//! `Affine` da aba ou do canvas leva ao ecrã; o rato, a roda e o teclado chegam-lhes nos mesmos px. O
//! que um gesto muda volta num `Plan` aplicado pela porta da W6 ([`crate::w6::gesture::apply`]).

use crate::docs::KanbanDocs;
use crate::tab::Tab;
use ph2d_editor_core::doc_surface::{KeyInput, KeyOutcome};
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_kanban_dates::{Weekday, Zone};
use ph2d_kanban_model::{KanbanSet, Member};
use ph2d_kanban_views::{Ctx, Reply};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

/// O 1.º dia da semana pela região do sistema (`LANG=pt_BR.UTF-8` → `BR` → domingo).
pub(crate) fn week_start() -> Weekday {
    let lang = std::env::var("LC_TIME")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    let region = lang
        .split(['.', '@'])
        .next()
        .and_then(|l| l.split('_').nth(1))
        .unwrap_or("US");
    ph2d_kanban_dates::week_start_for_region(region)
}

/// O corpo da vista (por baixo do cabeçalho, acima do HUD).
pub(crate) fn body(tab: &Tab, floor: Option<f32>) -> Rect {
    crate::w6::above(tab.layout.board, floor)
}

/// ⭐ Pinta a vista à vista no corpo (px da vista; a `Affine` da aba leva tudo junto ao ecrã).
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    tab: &mut Tab,
    set: &KanbanSet,
    (me, team, floor): (&Member, &[String], Option<f32>),
) {
    let rect = body(tab, floor);
    let team: Vec<Member> = team.iter().map(|t| Member(t.clone())).collect();
    let zone = Zone::system();
    let kanban = tab.view.kanban;
    let Tab { views, w6, .. } = tab;
    let cx = Ctx {
        now: crate::actions::now(),
        zone: &zone,
        me,
        team: &team,
        filter: &w6.filter,
        week_start: week_start(),
    };
    scene.push_clip(&ph2d_editor_core::paint::rect_to_vello(rect));
    views.paint(scene, Affine::IDENTITY, rect, set, kanban, ts, theme, &cx);
    scene.pop_layer();
}

/// Corre `f` sobre as vistas da aba `id` (com o contexto do quadro) e aplica a resposta.
fn with_views(
    docs: &mut KanbanDocs,
    id: u64,
    f: impl FnOnce(
        &mut ph2d_kanban_views::Views,
        &KanbanSet,
        ph2d_kanban_model::KanbanId,
        &Ctx<'_>,
    ) -> Reply,
) -> Option<Reply> {
    let me = docs.me.clone();
    let team: Vec<Member> = docs.team.iter().map(|t| Member(t.clone())).collect();
    let set = docs.set();
    let reply = {
        let set = set.borrow();
        let tab = docs.tab_mut(id)?;
        tab.other?;
        let zone = Zone::system();
        let kanban = tab.view.kanban;
        let Tab { views, w6, .. } = tab;
        let cx = Ctx {
            now: crate::actions::now(),
            zone: &zone,
            me: &me,
            team: &team,
            filter: &w6.filter,
            week_start: week_start(),
        };
        f(views, &set, kanban, &cx)
    };
    let kanban = docs.kanban_of(id);
    crate::w6::gesture::apply(docs, Some(reply.plan.clone()));
    if let Some(card) = reply.open
        && Some(card.kanban) == kanban
    {
        crate::back::open(docs, id, card.card);
    }
    Some(reply)
}

/// O rato numa vista que não é o quadro. `None` = segue (o cabeçalho, o quadro).
pub(crate) fn pointer(
    docs: &mut KanbanDocs,
    id: u64,
    kind: PointerKind,
    x: f32,
    y: f32,
) -> Option<bool> {
    let floor = docs.over.map(|o| o.y);
    let inside = {
        let tab = docs.tab_mut(id)?;
        tab.other?;
        body(tab, floor).contains(x, y)
    };
    // Largar fora do corpo ainda é da vista (o arrasto de uma barra que saiu).
    if !inside && kind != PointerKind::Up {
        return None;
    }
    let k = match kind {
        PointerKind::Down => ph2d_kanban_views::PointerKind::Down,
        PointerKind::Move => ph2d_kanban_views::PointerKind::Move,
        PointerKind::Up => ph2d_kanban_views::PointerKind::Up,
    };
    let r = with_views(docs, id, |v, set, kid, cx| v.pointer(set, kid, cx, k, x, y))?;
    Some(r.taken || inside)
}

/// A roda numa vista que não é o quadro. `true` = tomada.
pub(crate) fn wheel(docs: &mut KanbanDocs, id: u64, x: f32, y: f32, dy: f32, shift: bool) -> bool {
    let floor = docs.over.map(|o| o.y);
    let inside = docs
        .tab_mut(id)
        .is_some_and(|t| t.other.is_some() && body(t, floor).contains(x, y));
    if !inside {
        return false;
    }
    with_views(docs, id, |v, set, kid, cx| {
        v.wheel(set, kid, cx, (x, y), dy, (shift, false))
    });
    true
}

/// O teclado numa vista que não é o quadro. `None` = segue (os atalhos do Kanban).
pub(crate) fn key(docs: &mut KanbanDocs, id: u64, input: &KeyInput<'_>) -> Option<KeyOutcome> {
    let tab = docs.tab_mut(id)?;
    if tab.other.is_none() || tab.back.is_some() {
        return None;
    }
    let r = with_views(docs, id, |v, set, kid, cx| v.key(set, kid, cx, input))?;
    r.taken.then(crate::w6::taken)
}

impl KanbanDocs {
    /// ⚠️ SONDA (J1): a vista à vista da aba `id` (`None` = o quadro).
    #[must_use]
    pub fn probe_view_kind(&self, id: u64) -> Option<ph2d_kanban_views::ViewKind> {
        self.tabs.iter().find(|t| t.id() == id)?.other
    }

    /// ⚠️ SONDA (J1): o cartão da linha `row` da Tabela e o rect da célula da coluna `col` (px da
    /// vista), como o último quadro os pintou.
    #[must_use]
    pub fn probe_table_cell(
        &self,
        id: u64,
        row: usize,
        col: ph2d_kanban_views::table::Column,
    ) -> Option<(ph2d_kanban_model::CardId, Rect)> {
        let st = &self.tabs.iter().find(|t| t.id() == id)?.views.table;
        Some((*st.rows().get(row)?, st.grid().cell(row, col)?))
    }
}
