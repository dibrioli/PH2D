//! W4 do Kanban ([ADR-0195] §3) — **o Kanban do quadro, do lado do editor**: pô-lo (cria o documento
//! no `KanbanSet` partilhado e a referência no quadro), pedi-lo numa aba, e o GANCHO de entrada que a
//! família Kanban instala ([`install_kanban_input`]; a pintura é o `KanbanPainter` do board-render).
//!
//! Com o gancho, o Kanban activo (o seleccionado sozinho em que se clica de novo, a nível alto) recebe o
//! rato e as teclas; clicar fora ou `Esc` desactiva-o. Sem gancho o Kanban é uma caixa como um vídeo
//! parado. O duplo clique no cabeçalho pede-o numa aba ([`take_open_requests`]); sem quem abra abas de
//! Kanban ([`install_tab_opener`]) o pedido diz-o num aviso, nunca um clique mudo.
//!
//! [ADR-0195]: ../../../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use ph2d_board_edit::Tool;
use ph2d_board_model::{Board, ElementId, KanbanBoardView, KanbanId};
use ph2d_board_render::KanbanLod;
use ph2d_host::{Modifiers, PointerButton};
use ph2d_text::TextSystem;

use crate::screens::hero::board_keys::BoardKey;
use crate::screens::hero::board_view::{area, area_of};
use crate::doc_surface::KeyInput;
use crate::icons::IconId;
use crate::screens::hero::board_bar::Item;
use crate::documents::BoardLive;
use crate::screens::hero::HeroScreen;

/// O que o rato faz dentro de um Kanban activo; `at` em coordenadas LOCAIS do elemento (`0..w × 0..h`,
/// unidades do mundo — a vista do Kanban escala-as por `view.zoom`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KanbanPointer {
    Down {
        at: [f64; 2],
        button: PointerButton,
        mods: Modifiers,
        now_ns: u128,
    },
    Move {
        at: [f64; 2],
    },
    Up {
        at: [f64; 2],
        now_ns: u128,
    },
}

/// ⭐ O gancho de ENTRADA (a família Kanban implementa-o com o `KanbanPainter` do board-render).
pub trait KanbanElementInput {
    /// O rato no Kanban `kanban` do elemento `el`. `true` = consumido.
    fn pointer(
        &mut self,
        el: ElementId,
        kanban: KanbanId,
        ev: KanbanPointer,
        view: &mut KanbanBoardView,
    ) -> bool;
    /// Uma tecla. `true` = consumida (`false` com `Esc` desactiva o Kanban).
    fn key(
        &mut self,
        el: ElementId,
        kanban: KanbanId,
        ts: &mut TextSystem,
        input: KeyInput<'_>,
        view: &mut KanbanBoardView,
    ) -> bool;
}

thread_local! {
    static INPUT: RefCell<Option<Rc<RefCell<dyn KanbanElementInput>>>> = const { RefCell::new(None) };
    static OPENER: Cell<bool> = const { Cell::new(false) };
    static OPEN: RefCell<Vec<KanbanId>> = const { RefCell::new(Vec::new()) };
}

/// O gancho de entrada deste fio (a família Kanban instala-o ao arrancar).
pub fn install_kanban_input(input: Rc<RefCell<dyn KanbanElementInput>>) {
    INPUT.with(|s| *s.borrow_mut() = Some(input));
}

/// Quem abre abas de Kanban existe neste fio (esvazia [`take_open_requests`] a cada quadro).
pub fn install_tab_opener() {
    OPENER.with(|o| o.set(true));
}

/// Os Kanbans pedidos numa aba desde a última vez, por ordem.
#[must_use]
pub fn take_open_requests() -> Vec<KanbanId> {
    OPEN.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Há quem abra abas de Kanban («Open in tab» fica aceso).
#[must_use]
pub fn can_open_tabs() -> bool {
    OPENER.with(Cell::get)
}

/// A frase quando não há quem abra a aba.
pub(crate) const NO_TAB: &str = "board.kanban.no_tab";

/// Pede o Kanban `k` numa aba; sem quem a abra, o aviso no quadro. `true` = pedido.
pub(crate) fn request_open(live: &mut BoardLive, k: KanbanId) -> bool {
    if !can_open_tabs() {
        live.notice = Some((NO_TAB, std::time::Instant::now()));
        return false;
    }
    OPEN.with(|q| q.borrow_mut().push(k));
    true
}

fn hook() -> Option<Rc<RefCell<dyn KanbanElementInput>>> {
    INPUT.with(|s| s.borrow().clone())
}

/// ⭐ Um Kanban novo no quadro activo: o documento no `KanbanSet` do projecto (o MESMO `Rc` da aba) e a
/// referência no meio da vista, seleccionada; UM passo de desfazer (que tira só a referência).
pub(crate) fn insert(hero: &mut HeroScreen) -> Option<ElementId> {
    let view = hero.last_layout.map(|l| area_of(area(&l)));
    let theme = hero.theme;
    let kanbans = hero.documents.kanbans_shared();
    let (board, live) = hero.documents.active_parts()?;
    let kanban = kanbans
        .borrow_mut()
        .create(ph2d_i18n::tr("board.kanban.new_name"));
    let z = board.camera.zoom;
    let fit = view.map_or([f64::INFINITY; 2], |v| [v[2] / z / 2.0, v[3] / z / 2.0]);
    let center = [board.camera.center_x, board.camera.center_y];
    let history = live.histories.entry(board.id).or_default();
    let ed = crate::screens::hero::board_view::editor(&mut live.editor, theme);
    Some(ed.insert_kanban(&mut board.doc, history, kanban, center, fit))
}

/// «Open in tab» da barra: o Kanban seleccionado (o primeiro).
pub(crate) fn open_selected(hero: &mut HeroScreen) -> bool {
    let theme = hero.theme;
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let ed = crate::screens::hero::board_view::editor(&mut live.editor, theme);
    let k = ed
        .selection()
        .iter()
        .find_map(|id| board.doc.get(*id)?.kanban().map(|k| k.kanban));
    k.is_some_and(|k| request_open(live, k))
}

/// Os dois botões do Kanban nas barras: pôr um (a barra curta) e abri-lo na aba (a barra dele).
pub(crate) fn apply(hero: &mut HeroScreen, it: Item) -> bool {
    match it {
        Item::Kanban => {
            insert(hero);
            true
        }
        Item::OpenKanban => {
            open_selected(hero);
            true
        }
        _ => false,
    }
}

/// O balão dos dois botões; «Open in tab» sem quem abra a aba diz porquê está apagado.
pub(crate) fn tooltip_key(it: Item) -> Option<&'static str> {
    match it {
        Item::Kanban => Some("board.tool.kanban"),
        Item::OpenKanban if can_open_tabs() => Some("board.kanban.open_tab"),
        Item::OpenKanban => Some(NO_TAB),
        _ => None,
    }
}

/// Os ícones: o Kanban (a grelha de colunas) e «abrir»; apagado sem quem abra a aba.
pub(crate) fn paint_icon(
    scene: &mut ph2d_vector::VectorScene,
    theme: ph2d_tokens::Theme,
    it: Item,
    inner: crate::zones::Rect,
    fg: ph2d_vector::Color,
) -> bool {
    let (icon, fg) = match it {
        Item::Kanban => (IconId::LayoutGrid, fg),
        Item::OpenKanban if can_open_tabs() => (IconId::Open, fg),
        Item::OpenKanban => (
            IconId::Open,
            crate::paint::resolve(ph2d_tokens::ColorToken::TextDisabled, theme),
        ),
        _ => return false,
    };
    let line = ph2d_tokens::StrokeToken::Default.px();
    crate::paint::paint_icon(scene, icon, inner, fg, line);
    true
}

/// O duplo clique em `p` (ecrã) no CABEÇALHO de um Kanban que não está activo pede-o numa aba.
pub(crate) fn double_click(board: &Board, live: &mut BoardLive, p: [f64; 2]) -> bool {
    let frames = &live.render_cache.kanbans;
    let hit = frames.on_screen.keys().rev().copied().find(|id| {
        Some(*id) != frames.active
            && board.doc.get(*id).and_then(|e| e.kanban()).is_some_and(|k| {
                frames.on_header(*id, k.view.zoom, p)
            })
    });
    let Some(k) = hit.and_then(|id| board.doc.get(id)?.kanban().map(|k| k.kanban)) else {
        return false;
    };
    request_open(live, k);
    true
}

/// O rato para o gancho, no Kanban `id` do quadro activo. `None` = não há gancho ou o Kanban sumiu.
fn deliver(hero: &mut HeroScreen, id: ElementId, ev: KanbanPointer) -> Option<bool> {
    let h = hook()?;
    let (board, _) = hero.documents.active_parts()?;
    let kanban = board.doc.get(id)?.kanban()?.kanban;
    let view = board.doc.kanban_view_mut(id)?;
    Some(h.borrow_mut().pointer(id, kanban, ev, view))
}

/// O carregar no quadro, antes do editor. `true` = era do Kanban activo (ou o activou).
pub(crate) fn pointer_down(
    hero: &mut HeroScreen,
    x: f32,
    y: f32,
    button: PointerButton,
    now_ns: u128,
) -> bool {
    if hook().is_none() || hero.documents.live.space {
        return false;
    }
    let mods = Modifiers {
        shift: hero.store.shift_held(),
        ..Modifiers::default()
    };
    let p = [f64::from(x), f64::from(y)];
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let tool = live.editor.as_ref().map(|e| e.tool);
    if button == PointerButton::Middle || !matches!(tool, None | Some(Tool::Select)) {
        return false;
    }
    let frames = &mut live.render_cache.kanbans;
    let target = match frames.active {
        Some(id) => match frames.local(id, p) {
            Some(at) => Some((id, at)),
            None => {
                frames.active = None;
                None
            }
        },
        None => None,
    };
    let target = target.or_else(|| {
        // Entra no Kanban seleccionado SOZINHO em que se clica de novo, a nível alto.
        let ed = live.editor.as_ref()?;
        let [id] = ed.selection().iter().copied().collect::<Vec<_>>()[..] else {
            return None;
        };
        board.doc.get(id)?.kanban()?;
        let pl = frames.on_screen.get(&id)?;
        if pl.lod != KanbanLod::High || button != PointerButton::Primary {
            return None;
        }
        let at = frames.local(id, p)?;
        frames.active = Some(id);
        Some((id, at))
    });
    let Some((id, at)) = target else {
        return false;
    };
    live.render_cache.kanbans.pressed = true;
    let ev = KanbanPointer::Down {
        at,
        button,
        mods,
        now_ns,
    };
    deliver(hero, id, ev);
    true
}

/// O mover: dentro do Kanban activo vai ao gancho; `true` = um carregar dele está em curso.
pub(crate) fn pointer_move(hero: &mut HeroScreen, x: f32, y: f32) -> bool {
    let p = [f64::from(x), f64::from(y)];
    let Some((_, live)) = hero.documents.active_parts() else {
        return false;
    };
    let frames = &live.render_cache.kanbans;
    let pressed = frames.pressed;
    let Some(id) = frames.active else {
        return false;
    };
    let Some(pl) = frames.on_screen.get(&id) else {
        return pressed;
    };
    let l = pl.to_screen.inverse() * ph2d_vector::Point::new(p[0], p[1]);
    let inside = frames.local(id, p).is_some();
    if inside || pressed {
        deliver(hero, id, KanbanPointer::Move { at: [l.x, l.y] });
    }
    pressed
}

/// O largar de um carregar que foi do Kanban activo (`true` = era dele).
pub(crate) fn pointer_up(hero: &mut HeroScreen, x: f32, y: f32, now_ns: u128) -> bool {
    let Some((_, live)) = hero.documents.active_parts() else {
        return false;
    };
    let frames = &mut live.render_cache.kanbans;
    if !std::mem::take(&mut frames.pressed) {
        return false;
    }
    let Some(id) = frames.active else {
        return true;
    };
    let at = frames.on_screen.get(&id).map_or([-1.0; 2], |pl| {
        let l = pl.to_screen.inverse() * ph2d_vector::Point::new(f64::from(x), f64::from(y));
        [l.x, l.y]
    });
    deliver(hero, id, KanbanPointer::Up { at, now_ns });
    true
}

/// As teclas com um Kanban activo são do gancho; o `Esc` que ele não toma desactiva-o. `true` = tomadas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn key(
    hero: &mut HeroScreen,
    ts: &mut TextSystem,
    k: BoardKey,
    mods: Modifiers,
    pressed: bool,
    text: Option<&str>,
    paste: Option<&str>,
) -> bool {
    let Some(h) = hook() else { return false };
    let Some((board, live)) = hero.documents.active_parts() else {
        return false;
    };
    let Some(id) = live.render_cache.kanbans.active else {
        return false;
    };
    let Some(kanban) = board.doc.get(id).and_then(|e| e.kanban()).map(|k| k.kanban) else {
        live.render_cache.kanbans.active = None;
        return false;
    };
    let Some(view) = board.doc.kanban_view_mut(id) else {
        return false;
    };
    let input = KeyInput {
        key: crate::doc_surface::Key::from(k),
        mods,
        pressed,
        text,
        paste: paste.map(str::to_owned),
    };
    if h.borrow_mut().key(id, kanban, ts, input, view) {
        return true;
    }
    if k == BoardKey::Escape {
        if pressed {
            live.render_cache.kanbans.active = None;
        }
        return true;
    }
    false
}
