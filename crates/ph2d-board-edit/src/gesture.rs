//! O ponteiro: carregar, arrastar, largar — seleccionar, mover (e duplicar com `Alt`),
//! redimensionar, rodar, criar formas e seleccionar por arrasto.

use std::collections::BTreeSet;

use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Element, ElementId, End, History, ShapeType, rotate_about,
};
use ph2d_text::TextSystem;

use crate::resize::{axis_angle, drag_box, resize_frame, resized};
use crate::snap::{Guide, snap_box};
use crate::{Dir, Editor, Frame, Handle, Mods, Pointer, Tool, live, union_aabb};

/// O que o carregar pede a quem chama.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Down {
    /// O editor tomou o gesto.
    Taken,
    /// É a mão: quem chama arrasta a VISTA.
    Pan,
}

/// O passo do ângulo com `Shift` a rodar.
const ROTATE_STEP: f64 = std::f64::consts::PI / 12.0;
/// Lado mínimo de uma forma, em unidades do mundo.
pub(crate) const MIN_SIDE: f64 = 1.0;

pub(crate) enum Gesture {
    Marquee {
        start: [f64; 2],
        cur: [f64; 2],
        base: BTreeSet<ElementId>,
        active: bool,
    },
    Move {
        start: [f64; 2],
        originals: Vec<Element>,
        targets: Vec<[f64; 4]>,
        active: bool,
        /// As cópias de um arrasto com `Alt` (os originais ficam onde estavam).
        copies: Vec<ElementId>,
        /// Clicar (sem arrastar) num já seleccionado de uma selecção de vários fica só com ele.
        collapse_to: Option<ElementId>,
        guides: Vec<Guide>,
    },
    Resize {
        dir: Dir,
        frame: Frame,
        originals: Vec<Element>,
        targets: Vec<[f64; 4]>,
        guides: Vec<Guide>,
    },
    Rotate {
        frame: Frame,
        start: [f64; 2],
        originals: Vec<Element>,
    },
    Create {
        kind: ShapeType,
        start: [f64; 2],
        id: Option<ElementId>,
        targets: Vec<[f64; 4]>,
        guides: Vec<Guide>,
    },
    /// Arrastar dentro do texto em edição: selecciona texto.
    Text,
    /// Uma seta nova: da ferramenta Seta, ou de um ponto azul (`dot` = a direcção dele — largar sem
    /// arrastar cria a forma seguinte).
    Connect {
        start: End,
        from: [f64; 2],
        id: Option<ElementId>,
        /// A forma onde a ponta de fim se vai prender (o realce).
        target: Option<(ElementId, Anchor)>,
        /// A forma de onde a seta sai: a ponta de fim não se prende de volta a ela.
        origin: Option<ElementId>,
        dot: Option<[f64; 2]>,
    },
    /// Arrastar (ou criar, a partir do meio de um trecho) um ponto de ajuste de uma seta.
    Bend {
        id: ElementId,
        index: usize,
        start: [f64; 2],
        moved: bool,
        original: Element,
    },
    /// Arrastar uma ponta de uma seta seleccionada.
    EndDrag {
        id: ElementId,
        which: usize,
        original: Element,
        target: Option<(ElementId, Anchor)>,
    },
    /// A pega de arrumar em grelha: as notas em ordem de leitura, quantas colunas o ponteiro pede.
    Grid {
        originals: Vec<Element>,
        ordered: Vec<Element>,
        cols: usize,
    },
    /// Carregar numa pilha de notas (não seleccionada): arrastar tira uma nota nova; largar sem
    /// arrastar selecciona a pilha.
    Peel {
        stack: Element,
        start: [f64; 2],
        targets: Vec<[f64; 4]>,
    },
}

impl Gesture {
    pub(crate) fn marquee(&self) -> Option<[f64; 4]> {
        match self {
            Gesture::Marquee {
                start,
                cur,
                active: true,
                ..
            } => Some(rect(*start, *cur)),
            _ => None,
        }
    }

    pub(crate) fn guides(&self) -> Vec<Guide> {
        match self {
            Gesture::Move { guides, .. }
            | Gesture::Resize { guides, .. }
            | Gesture::Create { guides, .. } => guides.clone(),
            _ => Vec::new(),
        }
    }
}

pub(crate) fn rect(a: [f64; 2], b: [f64; 2]) -> [f64; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[0].max(b[0]),
        a[1].max(b[1]),
    ]
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn intersects(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

/// As caixas das formas vivas, menos `skip`, que tocam a vista — os alvos das guias.
fn targets(doc: &BoardDoc, skip: &BTreeSet<ElementId>, view: [f64; 4]) -> Vec<[f64; 4]> {
    doc.live()
        .filter(|el| el.shape().is_some() && !skip.contains(&el.id))
        .map(Element::aabb)
        .filter(|b| intersects(b, &view))
        .collect()
}

impl Editor {
    /// O elemento de cima em `p` (o que um clique apanha): uma forma pelo contorno, uma seta pela
    /// linha (a menos de meia pega, mais meia espessura).
    pub fn hit(&mut self, doc: &BoardDoc, p: Pointer) -> Option<ElementId> {
        let tol = self.metrics.handle / 2.0 * p.px;
        self.routes.sync(doc);
        let routes = &self.routes;
        doc.live_in_z_order()
            .into_iter()
            .rev()
            .find(|el| match el.connector() {
                Some(c) => routes
                    .get(el.id)
                    .is_some_and(|r| r.distance(p.world) <= tol + c.style.stroke_width / 2.0),
                None if el.ink().is_some() => crate::ink::ink_hit(el, p.world, tol),
                None => ph2d_board_geom::hit(el, p.world, tol),
            })
            .map(|el| el.id)
    }

    /// A pega da moldura em `p`, se há selecção.
    #[must_use]
    pub fn handle_at(&self, doc: &BoardDoc, p: Pointer) -> Option<Handle> {
        let frame = self.frame(doc)?;
        let reach = self.metrics.handle * p.px;
        std::iter::once(Handle::Rotate)
            .chain(Dir::ALL.into_iter().map(Handle::Resize))
            .find(|h| dist(frame.handle(*h, p.px, &self.metrics), p.world) <= reach)
    }

    /// Carregar o botão principal. `view` = o rectângulo do mundo à vista (os alvos das guias).
    pub fn pointer_down(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ts: &mut TextSystem,
        p: Pointer,
        view: [f64; 4],
    ) -> Down {
        if self.text_pointer_down(doc, history, ts, p) {
            self.gesture = Some(Gesture::Text);
            return Down::Taken;
        }
        if self.ink_down(doc, p) {
            return Down::Taken;
        }
        match self.tool {
            Tool::Hand => return Down::Pan,
            Tool::Shape(kind) => {
                self.gesture = Some(Gesture::Create {
                    kind,
                    start: p.world,
                    id: None,
                    targets: targets(doc, &BTreeSet::new(), view),
                    guides: Vec::new(),
                });
                return Down::Taken;
            }
            Tool::Connector => {
                self.gesture = Some(self.begin_connect(doc, p, None, None));
                return Down::Taken;
            }
            Tool::Select | Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser => {}
        }
        if let Some((shape, dir)) = self.dot_at(doc, p) {
            self.gesture = Some(self.begin_connect(doc, p, Some(shape), Some(dir)));
            return Down::Taken;
        }
        if let Some((id, h)) = self.wire_handle_at(doc, p) {
            self.gesture = match h {
                crate::wire::WireHandle::End(which) => {
                    let original = doc.get(id).cloned().expect("a pega é de uma seta viva");
                    Some(Gesture::EndDrag {
                        id,
                        which,
                        original,
                        target: None,
                    })
                }
                _ => self.begin_bend(doc, id, h, p),
            };
            return Down::Taken;
        }
        if let Some(g) = self.grid_handle(doc, p.px)
            && dist(g, p.world) <= self.metrics.handle * p.px
        {
            let originals = self.selected_notes(doc).expect("há pega, há notas");
            let ordered = crate::notes::reading_order(&originals);
            self.gesture = Some(Gesture::Grid {
                originals,
                ordered,
                cols: 0,
            });
            return Down::Taken;
        }
        if let Some(h) = self.handle_at(doc, p) {
            let frame = self.frame(doc).expect("há pega, há moldura");
            let originals: Vec<Element> = self
                .selected(doc)
                .into_iter()
                .filter(|el| el.connector().is_none())
                .cloned()
                .collect();
            self.gesture = Some(match h {
                Handle::Rotate => Gesture::Rotate {
                    frame,
                    start: p.world,
                    originals,
                },
                Handle::Resize(dir) => Gesture::Resize {
                    dir,
                    frame,
                    originals,
                    targets: targets(doc, &self.selection, view),
                    guides: Vec::new(),
                },
            });
            return Down::Taken;
        }
        match self.hit(doc, p) {
            Some(id)
                if !p.mods.shift
                    && !self.selection.contains(&id)
                    && doc
                        .get(id)
                        .and_then(Element::shape)
                        .is_some_and(|s| s.kind == ShapeType::StickyStack) =>
            {
                let stack = doc.get(id).cloned().expect("acabou de ser apanhada");
                self.gesture = Some(Gesture::Peel {
                    stack,
                    start: p.world,
                    targets: targets(doc, &BTreeSet::new(), view),
                });
            }
            Some(id) => {
                let was = self.selection.contains(&id);
                let mut collapse_to = None;
                if p.mods.shift {
                    if was {
                        self.selection.remove(&id);
                        return Down::Taken;
                    }
                    self.selection.insert(id);
                } else if !was {
                    self.selection = BTreeSet::from([id]);
                } else if self.selection.len() > 1 {
                    collapse_to = Some(id);
                }
                self.gesture = Some(Gesture::Move {
                    start: p.world,
                    originals: self.selected(doc).into_iter().cloned().collect(),
                    targets: targets(doc, &self.selection, view),
                    active: false,
                    copies: Vec::new(),
                    collapse_to,
                    guides: Vec::new(),
                });
            }
            None => {
                if !p.mods.shift {
                    self.selection.clear();
                }
                self.gesture = Some(Gesture::Marquee {
                    start: p.world,
                    cur: p.world,
                    base: self.selection.clone(),
                    active: false,
                });
            }
        }
        Down::Taken
    }

    /// O ponteiro andou (com o botão em baixo).
    pub fn pointer_move(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem, p: Pointer) {
        if self.ink_move(doc, p) {
            return;
        }
        let Some(mut g) = self.gesture.take() else {
            return;
        };
        let drag = self.metrics.drag * p.px;
        let snap = (!p.mods.ctrl).then_some(self.metrics.snap * p.px);
        match &mut g {
            Gesture::Peel {
                stack,
                start,
                targets,
            } => {
                if dist(*start, p.world) <= drag {
                    self.gesture = Some(g);
                    return;
                }
                // Tira a nota de cima da pilha e passa a arrastá-la (largar = UM passo: apagá-la).
                let note = self.peel(doc, stack);
                let id = note.id;
                live(doc, note.clone());
                self.selection = BTreeSet::from([id]);
                self.gesture = Some(Gesture::Move {
                    start: *start,
                    originals: vec![note],
                    targets: std::mem::take(targets),
                    active: true,
                    copies: vec![id],
                    collapse_to: None,
                    guides: Vec::new(),
                });
                return self.pointer_move(doc, ts, p);
            }
            Gesture::Grid { ordered, cols, .. } => {
                let n = Editor::grid_columns(ordered, p.world[0]);
                if n != *cols {
                    *cols = n;
                    self.arrange(doc, ordered, n);
                }
            }
            Gesture::Text => self.text_drag(ts, doc, p),
            Gesture::Connect { .. } => self.connect_move(doc, &mut g, p),
            Gesture::EndDrag { .. } => self.end_drag_move(doc, &mut g, p),
            Gesture::Bend { .. } => self.bend_move(doc, &mut g, p),
            Gesture::Marquee {
                start,
                cur,
                base,
                active,
            } => {
                *cur = p.world;
                *active |= dist(*start, *cur) > drag;
                if *active {
                    let r = rect(*start, *cur);
                    self.routes.sync(doc);
                    let routes = &self.routes;
                    let inside = doc.live().filter(|el| {
                        let b = match el.connector() {
                            Some(_) => routes.get(el.id).map_or([f64::NAN; 4], |r| r.bbox),
                            None => el.aabb(),
                        };
                        b[0] >= r[0] && b[1] >= r[1] && b[2] <= r[2] && b[3] <= r[3]
                    });
                    self.selection = base.iter().copied().chain(inside.map(|el| el.id)).collect();
                }
            }
            Gesture::Move {
                start,
                originals,
                targets,
                active,
                copies,
                guides,
                ..
            } => {
                if !*active && dist(*start, p.world) <= drag {
                    self.gesture = Some(g);
                    return;
                }
                if !*active {
                    *active = true;
                    if p.mods.alt {
                        *copies = self.duplicate_live(doc, originals);
                        let copy_state: Vec<Element> = copies
                            .iter()
                            .filter_map(|id| doc.get(*id).cloned())
                            .collect();
                        *originals = copy_state;
                    }
                }
                let mut d = [p.world[0] - start[0], p.world[1] - start[1]];
                if p.mods.shift {
                    if d[0].abs() > d[1].abs() {
                        d[1] = 0.0;
                    } else {
                        d[0] = 0.0;
                    }
                }
                guides.clear();
                let shapes = originals.iter().filter(|o| o.shape().is_some());
                if let Some(tol) = snap.filter(|_| shapes.clone().next().is_some()) {
                    let b = union_aabb(shapes);
                    let moved = [b[0] + d[0], b[1] + d[1], b[2] + d[0], b[3] + d[1]];
                    let (s, g2) = snap_box(moved, targets, tol);
                    d = [d[0] + s[0], d[1] + s[1]];
                    *guides = g2;
                }
                for o in originals.iter() {
                    let mut el = o.clone();
                    el.translate(d);
                    live(doc, el);
                }
            }
            Gesture::Resize {
                dir,
                frame,
                originals,
                targets,
                guides,
            } => {
                let mut world = p.world;
                guides.clear();
                if let Some(tol) = snap.filter(|_| frame.angle == 0.0) {
                    let (s, g2) = snap_box([world[0], world[1], world[0], world[1]], targets, tol);
                    world = [world[0] + s[0], world[1] + s[1]];
                    *guides = g2;
                }
                // As notas escalam inteiras, com a letra (o Miro): a proporção não muda.
                let uniform = (originals.len() > 1
                    && originals.iter().any(|o| !axis_angle(o.angle)))
                    || originals.iter().any(is_note);
                let mods = Mods {
                    shift: p.mods.shift || uniform,
                    ..p.mods
                };
                let nf = resize_frame(*frame, *dir, frame.local(world), mods);
                for el in resized(originals, *frame, nf) {
                    live(doc, el);
                }
            }
            Gesture::Rotate {
                frame,
                start,
                originals,
            } => {
                let c = frame.center;
                let a0 = (start[1] - c[1]).atan2(start[0] - c[0]);
                let a1 = (p.world[1] - c[1]).atan2(p.world[0] - c[0]);
                let mut delta = a1 - a0;
                if p.mods.shift {
                    let target = ((frame.angle + delta) / ROTATE_STEP).round() * ROTATE_STEP;
                    delta = target - frame.angle;
                }
                for o in originals.iter() {
                    let mut el = o.clone();
                    let nc = rotate_about(o.center(), c, delta);
                    el.x = nc[0] - el.w / 2.0;
                    el.y = nc[1] - el.h / 2.0;
                    el.angle = o.angle + delta;
                    live(doc, el);
                }
            }
            Gesture::Create {
                kind,
                start,
                id,
                targets,
                guides,
            } => {
                if id.is_none() && dist(*start, p.world) <= drag {
                    self.gesture = Some(g);
                    return;
                }
                let mut world = p.world;
                guides.clear();
                if let Some(tol) = snap {
                    let (s, g2) = snap_box([world[0], world[1], world[0], world[1]], targets, tol);
                    world = [world[0] + s[0], world[1] + s[1]];
                    *guides = g2;
                }
                let mut bx = drag_box(*start, world, p.mods);
                if let Some(a) = kind.note_aspect() {
                    bx[3] = bx[2] * a;
                }
                let el = match *id {
                    Some(existing) => doc.get(existing).cloned().map(|mut el| {
                        [el.x, el.y, el.w, el.h] = bx;
                        if kind.is_note() {
                            el.style_mut().font_size = crate::notes::note_font(*kind, bx[2]);
                        }
                        el
                    }),
                    None => {
                        let el = self.new_shape(doc, *kind, bx);
                        *id = Some(el.id);
                        Some(el)
                    }
                };
                if let Some(el) = el {
                    live(doc, el);
                }
            }
        }
        self.gesture = Some(g);
    }

    /// Largar o botão: o gesto acaba e vira UM passo de desfazer.
    pub fn pointer_up(&mut self, doc: &mut BoardDoc, history: &mut History, p: Pointer) {
        if self.ink_up(doc, history, p) {
            return;
        }
        let Some(g) = self.gesture.take() else {
            return;
        };
        match g {
            Gesture::Text | Gesture::Marquee { .. } => {}
            Gesture::Peel { stack, .. } => self.selection = BTreeSet::from([stack.id]),
            Gesture::Grid { originals, .. } => history.record(undo_ops(doc, &originals)),
            g @ Gesture::Connect { .. } => self.connect_up(doc, history, g),
            g @ Gesture::Bend { .. } => self.bend_up(doc, history, g, p),
            Gesture::EndDrag { original, .. } => {
                history.record(undo_ops(doc, std::slice::from_ref(&original)));
            }
            Gesture::Move {
                originals,
                active,
                copies,
                collapse_to,
                ..
            } => {
                if !active {
                    if let Some(id) = collapse_to {
                        self.selection = BTreeSet::from([id]);
                    }
                } else if copies.is_empty() {
                    history.record(undo_ops(doc, &originals));
                } else {
                    history.record(copies.into_iter().rev().map(BoardOp::Delete).collect());
                }
            }
            Gesture::Resize { originals, .. } | Gesture::Rotate { originals, .. } => {
                history.record(undo_ops(doc, &originals));
            }
            Gesture::Create {
                kind, start, id, ..
            } => {
                let id = id.unwrap_or_else(|| {
                    let [w, h] = self.metrics.click_size;
                    let bx = if kind.is_note() {
                        self.note_box(kind, start)
                    } else {
                        [start[0] - w / 2.0, start[1] - h / 2.0, w, h]
                    };
                    let el = self.new_shape(doc, kind, bx);
                    let id = el.id;
                    live(doc, el);
                    id
                });
                history.record(vec![BoardOp::Delete(id)]);
                self.selection = BTreeSet::from([id]);
                self.tool = Tool::Select;
            }
        }
    }

    /// `Esc` a meio de um gesto: devolve o documento ao início dele. `false` = não havia gesto.
    pub fn cancel_gesture(&mut self, doc: &mut BoardDoc) -> bool {
        if self.ink_cancel(doc) {
            return true;
        }
        let Some(g) = self.gesture.take() else {
            return false;
        };
        match g {
            Gesture::Move {
                originals, copies, ..
            } => {
                if copies.is_empty() {
                    for op in undo_ops(doc, &originals) {
                        let _ = op.apply(doc);
                    }
                } else {
                    for id in copies {
                        let _ = BoardOp::Delete(id).apply(doc);
                    }
                }
            }
            Gesture::Resize { originals, .. }
            | Gesture::Rotate { originals, .. }
            | Gesture::Grid { originals, .. } => {
                for op in undo_ops(doc, &originals) {
                    let _ = op.apply(doc);
                }
            }
            Gesture::EndDrag { original, .. } | Gesture::Bend { original, .. } => {
                let _ = BoardOp::Put(original).apply(doc);
            }
            Gesture::Create { id: Some(id), .. } | Gesture::Connect { id: Some(id), .. } => {
                let _ = BoardOp::Delete(id).apply(doc);
            }
            Gesture::Marquee { base, .. } => self.selection = base,
            _ => {}
        }
        true
    }

    /// Cópias vivas de `els` (ids novos, à frente de tudo, na mesma ordem relativa; as setas presas
    /// às cópias); devolve os ids.
    fn duplicate_live(&mut self, doc: &mut BoardDoc, els: &[Element]) -> Vec<ElementId> {
        let mut copies = els.to_vec();
        self.detach_outside(doc, &mut copies);
        let mut map = std::collections::BTreeMap::new();
        for el in &mut copies {
            let id = doc.mint_id();
            map.insert(el.id, id);
            el.id = id;
            el.version = 0;
        }
        crate::wire::remap(&mut copies, &map);
        let ids: Vec<ElementId> = copies
            .into_iter()
            .map(|mut el| {
                el.z = doc.z_on_top();
                let id = el.id;
                live(doc, el);
                id
            })
            .collect();
        self.selection = ids.iter().copied().collect();
        ids
    }
}

/// O lote que devolve o documento ao início do gesto («pôr os originais») — só dos que mudaram:
/// um arrasto que voltou ao sítio não é passo. NÃO o aplica: é o passo que o `History` guarda.
fn undo_ops(doc: &BoardDoc, originals: &[Element]) -> Vec<BoardOp> {
    originals
        .iter()
        .filter(|o| doc.get(o.id).is_some_and(|now| !same_pose(now, o)))
        .map(|o| BoardOp::Put(o.clone()))
        .collect()
}

pub(crate) fn is_note(el: &Element) -> bool {
    el.shape().is_some_and(|s| s.kind.is_note())
}

fn same_pose(a: &Element, b: &Element) -> bool {
    a.x == b.x && a.y == b.y && a.w == b.w && a.h == b.h && a.angle == b.angle && a.kind == b.kind
}
