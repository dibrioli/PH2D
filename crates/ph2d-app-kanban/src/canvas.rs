//! ⭐ **O Kanban no canvas do Quadro** (W4, lote V, ADR-0195 §3.3) — os ganchos que o Quadro chama: a
//! pintura ([`ph2d_board_render::KanbanPainter`]) e a entrada
//! ([`ph2d_editor_core::screens::hero::board_video::kanban::KanbanElementInput`]).
//!
//! Cada elemento é uma VISTA como a aba ([`crate::tab::Tab`], id [`key_of`]): o mesmo `view::paint`
//! numa `Affine` (a câmara × o zoom interno) e o MESMO código do rato e das teclas, em px da vista. As
//! vistas dos elementos vivem aqui (o desenho do Quadro não chega à família); o rato e as teclas
//! trazem a vista para as abas da família pelo tempo do evento ([`with_element`]). O que é desta vista
//! (rolagem, recolhidas) volta ao `KanbanBoardView` do elemento, nunca ao documento.

use crate::docs::KanbanDocs;
use crate::tab::Tab;
use crate::view::{At, ViewCtx};
use ph2d_host::PointerKind;
use ph2d_board_model::{ElementId, KanbanBoardView, KanbanId};
use ph2d_board_render::{KanbanLod, KanbanOver, KanbanPaint, KanbanPainter};
use ph2d_editor_core::doc_surface::{Compose, KeyInput, KeyOutcome, PointerLook, Surfaces};
use ph2d_editor_core::documents::SharedKanbans;
use ph2d_editor_core::screens::hero::board_video::kanban::{
    KanbanElementInput, KanbanPointer, KanbanWheel,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::Affine;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// O id de uma vista de elemento: fora do espaço dos `KanbanId` (contam de 1) e do início (`u64::MAX`).
const ELEMENT_KEY: u64 = 1 << 62;

pub(crate) fn key_of(el: ElementId) -> u64 {
    ELEMENT_KEY | el.0
}

/// A vista `id` é a de um elemento do Quadro (não uma aba nem o início).
pub(crate) fn on_board(id: u64) -> bool {
    id != crate::docs::HOME && id & ELEMENT_KEY != 0
}

/// As vistas dos Kanbans do canvas e o que o quadro de imagem lhes dá.
#[derive(Default)]
pub(crate) struct Canvas {
    pub tabs: BTreeMap<ElementId, Tab>,
    pub cx: ViewCtx,
    /// Os Kanbans do projecto (o passe por cima não os recebe do Quadro).
    pub set: Option<SharedKanbans>,
}

pub(crate) type SharedCanvas = Rc<RefCell<Canvas>>;

/// A vista do elemento lê o `KanbanBoardView` se ele mudou fora dela (desfazer, um colega, abrir).
fn adopt(tab: &mut Tab, view: &KanbanBoardView) {
    if tab.board_seen.as_ref() != Some(view) {
        tab.view.scroll_x = view.scroll_x;
        tab.view.scroll_y.clone_from(&view.scroll_y);
        tab.view.collapsed.clone_from(&view.collapsed);
        tab.board_seen = Some(view.clone());
    }
}

/// O que é desta vista volta ao elemento (a rolagem do quadro e de cada lista, as recolhidas).
fn write_back(tab: &mut Tab, view: &mut KanbanBoardView) {
    view.scroll_x = tab.view.scroll_x;
    view.scroll_y.clone_from(&tab.view.scroll_y);
    view.scroll_y.retain(|_, y| *y > 0.0);
    view.collapsed.clone_from(&tab.view.collapsed);
    tab.board_seen = Some(view.clone());
}

fn take(canvas: &SharedCanvas, el: ElementId, kanban: KanbanId) -> Tab {
    canvas
        .borrow_mut()
        .tabs
        .remove(&el)
        .filter(|t| t.view.kanban == kanban)
        .unwrap_or_else(|| Tab::on_board(kanban, key_of(el)))
}

/// ⭐ Corre `f` com a vista do elemento entre as abas da família (o código da aba, intacto).
fn with_element<R>(
    docs: &mut KanbanDocs,
    canvas: &SharedCanvas,
    (el, kanban): (ElementId, KanbanId),
    view: &mut KanbanBoardView,
    f: impl FnOnce(&mut KanbanDocs, u64) -> R,
) -> R {
    let key = key_of(el);
    let mut tab = take(canvas, el, kanban);
    adopt(&mut tab, view);
    docs.tabs.push(tab);
    let out = f(docs, key);
    if let Some(i) = docs.tabs.iter().position(|t| t.id() == key) {
        let mut tab = docs.tabs.remove(i);
        write_back(&mut tab, view);
        canvas.borrow_mut().tabs.insert(el, tab);
    }
    out
}

/// Px de ecrã → px da vista.
fn zoom_of(view: &KanbanBoardView) -> f64 {
    if view.zoom > 0.0 { view.zoom } else { 1.0 }
}

#[allow(clippy::cast_possible_truncation)]
fn to_view(at: [f64; 2], z: f64) -> (f32, f32) {
    ((at[0] / z) as f32, (at[1] / z) as f32)
}

/// O gancho de pintura.
pub(crate) struct Painter(pub SharedCanvas);

impl KanbanPainter for Painter {
    fn paint(&mut self, p: KanbanPaint<'_>) -> bool {
        if p.lod == KanbanLod::Low {
            return false;
        }
        let z = zoom_of(p.view);
        #[allow(clippy::cast_possible_truncation)]
        let area = Rect::new(
            0.0,
            0.0,
            (p.frame.width() / z) as f32,
            (p.frame.height() / z) as f32,
        );
        let at = At {
            to_screen: p.to_screen * Affine::scale(z),
            area,
            lod: p.lod,
            modal: false,
        };
        if p.export {
            // Um export não mexe no estado da vista viva (nem nas molas: sem lei, saltam).
            let mut tab = Tab::on_board(p.kanban, key_of(p.element));
            adopt(&mut tab, p.view);
            let cx = ViewCtx::default();
            crate::view::paint(p.scene, p.ts, p.theme, at, &mut tab, p.set, &cx);
            return true;
        }
        let mut c = self.0.borrow_mut();
        let cx = c.cx.clone();
        let mut tab = c
            .tabs
            .remove(&p.element)
            .filter(|t| t.view.kanban == p.kanban)
            .unwrap_or_else(|| Tab::on_board(p.kanban, key_of(p.element)));
        adopt(&mut tab, p.view);
        crate::view::paint(p.scene, p.ts, p.theme, at, &mut tab, p.set, &cx);
        c.tabs.insert(p.element, tab);
        true
    }

    fn paint_over(&mut self, o: KanbanOver<'_>) {
        let Some(el) = o.active else {
            return;
        };
        let mut c = self.0.borrow_mut();
        let (Some(set), me, team) = (c.set.clone(), c.cx.me.clone(), c.cx.team.clone()) else {
            return;
        };
        let Some(tab) = c.tabs.get_mut(&el).filter(|t| t.back.is_some() || t.auto.is_some()) else {
            return;
        };
        let set = set.borrow();
        let Some(k) = set.get(tab.view.kanban) else {
            return;
        };
        // O verso é o modal do Trello: em ecrã, centrado na área do quadro.
        #[allow(clippy::cast_possible_truncation)]
        let screen = Rect::new(
            o.area.x0 as f32,
            o.area.y0 as f32,
            o.area.width() as f32,
            o.area.height() as f32,
        );
        let board_area = std::mem::replace(&mut tab.layout.area, screen);
        let now = crate::actions::now();
        crate::back::paint(o.scene, o.ts, o.theme, k, tab, &set, &me, &team, now);
        // O diálogo aberto do verso (mover, copiar, espelhar) vai por cima dele, em ecrã.
        crate::w6::paint_dialog(o.scene, o.ts, o.theme, &set, tab, screen);
        tab.layout.area = board_area;
    }
}

/// O gancho de entrada.
pub(crate) struct Input(pub SharedCanvas);

impl KanbanElementInput for Input {
    fn pointer(
        &mut self,
        surfaces: &mut Surfaces,
        ts: &mut TextSystem,
        target: (ElementId, KanbanId),
        ev: KanbanPointer,
        view: &mut KanbanBoardView,
    ) -> bool {
        let Some(docs) = surfaces.find_mut::<KanbanDocs>() else {
            return false;
        };
        let z = zoom_of(view);
        with_element(docs, &self.0, target, view, |docs, id| {
            let modal = crate::back::is_open(docs, id);
            #[allow(clippy::cast_possible_truncation)]
            let scr = |s: [f64; 2]| (s[0] as f32, s[1] as f32);
            // A W6 primeiro: com o verso aberto, o diálogo dele está em ecrã; senão, na vista.
            let (kind, at, screen) = match ev {
                KanbanPointer::Down { at, screen, .. } => (PointerKind::Down, at, screen),
                KanbanPointer::Move { at, screen } => (PointerKind::Move, at, screen),
                KanbanPointer::Up { at, screen, .. } => (PointerKind::Up, at, screen),
            };
            let (wx, wy) = if modal { scr(screen) } else { to_view(at, z) };
            if let Some(taken) = crate::w6::pointer(docs, id, kind, wx, wy) {
                return taken;
            }
            match ev {
                KanbanPointer::Down {
                    at,
                    screen,
                    button,
                    mods,
                    now_ns,
                } => {
                    if modal {
                        return crate::back::down(docs, id, ts, scr(screen), mods, now_ns);
                    }
                    let (x, y) = to_view(at, z);
                    crate::pointer::down(docs, id, ts, button, x, y)
                }
                KanbanPointer::Move { at, screen } => {
                    if modal {
                        let (x, y) = scr(screen);
                        return crate::back::moved(docs, id, ts, x, y);
                    }
                    let (x, y) = to_view(at, z);
                    crate::pointer::moved(docs, id, ts, x, y)
                }
                KanbanPointer::Up { at, screen, now_ns } => {
                    if modal {
                        let (x, y) = scr(screen);
                        return crate::back::up(docs, id, x, y);
                    }
                    let (x, y) = to_view(at, z);
                    crate::pointer::up(docs, id, ts, now_ns, x, y)
                }
            }
        })
    }

    fn key(
        &mut self,
        surfaces: &mut Surfaces,
        ts: &mut TextSystem,
        target: (ElementId, KanbanId),
        input: KeyInput<'_>,
        view: &mut KanbanBoardView,
    ) -> KeyOutcome {
        let Some(docs) = surfaces.find_mut::<KanbanDocs>() else {
            return KeyOutcome::default();
        };
        with_element(docs, &self.0, target, view, |docs, id| {
            crate::keys::key(docs, id, ts, input)
        })
    }

    fn wheel(
        &mut self,
        surfaces: &mut Surfaces,
        target: (ElementId, KanbanId),
        at: [f64; 2],
        w: KanbanWheel,
        view: &mut KanbanBoardView,
    ) -> bool {
        let Some(docs) = surfaces.find_mut::<KanbanDocs>() else {
            return false;
        };
        let z = zoom_of(view);
        let (x, y) = to_view(at, z);
        with_element(docs, &self.0, target, view, |docs, id| {
            let modal = crate::back::is_open(docs, id);
            if !modal
                && let KanbanWheel::Wheel { dy, shift } = w
                && crate::w6::wheel(docs, id, x, y, crate::pointer::wheel_px(dy), shift)
            {
                return true;
            }
            #[allow(clippy::cast_possible_truncation)]
            match w {
                KanbanWheel::Wheel { dy, .. } if modal => {
                    crate::back::wheel(docs, id, crate::pointer::wheel_px(dy))
                }
                KanbanWheel::Pan { screen, .. } if modal => {
                    crate::back::wheel(docs, id, -screen[1] as f32)
                }
                KanbanWheel::Wheel { dy, shift } => {
                    crate::pointer::wheel(docs, id, x, y, dy, shift)
                }
                // O conteúdo anda com os dedos: a rolagem anda ao contrário.
                KanbanWheel::Pan { local, .. } => {
                    let d = [(-local[0] / z) as f32, (-local[1] / z) as f32];
                    crate::pointer::scroll(docs, id, (x, y), d)
                }
            }
        })
    }

    fn compose(
        &mut self,
        surfaces: &mut Surfaces,
        target: (ElementId, KanbanId),
        c: Compose<'_>,
        view: &mut KanbanBoardView,
    ) -> bool {
        let Some(docs) = surfaces.find_mut::<KanbanDocs>() else {
            return false;
        };
        with_element(docs, &self.0, target, view, |docs, id| {
            crate::keys::compose(docs, id, c)
        })
    }

    fn look(&mut self, el: ElementId, inside: bool) -> Option<PointerLook> {
        let c = self.0.borrow();
        let tab = c.tabs.get(&el)?;
        (inside || tab.back.is_some() || tab.w6.open()).then(|| crate::surface::look_of(tab))
    }

    fn captures(&mut self, _surfaces: &mut Surfaces, el: ElementId) -> bool {
        self.0
            .borrow()
            .tabs
            .get(&el)
            .is_some_and(|t| t.back.is_some() || t.auto.is_some() || t.w6.open())
    }
}

/// Instala os dois ganchos neste fio e devolve as vistas (a família segura-as).
pub(crate) fn install() -> SharedCanvas {
    let canvas = SharedCanvas::default();
    ph2d_board_render::install_kanban_painter(Rc::new(RefCell::new(Painter(canvas.clone()))));
    ph2d_editor_core::screens::hero::board_video::kanban::install_kanban_input(Rc::new(
        RefCell::new(Input(canvas.clone())),
    ));
    ph2d_editor_core::screens::hero::board_video::kanban::install_tab_opener();
    canvas
}
