//! O ponteiro: carregar, arrastar, largar — seleccionar, mover (e duplicar com `Alt`),
//! redimensionar, rodar, criar formas e seleccionar por arrasto.

use std::collections::BTreeSet;

use ph2d_board_model::{Anchor, BoardDoc, BoardOp, Element, ElementId, End, History, ShapeType};
use ph2d_text::TextSystem;

use crate::gesture_end::{is_note, undo_ops};
use crate::guides::Snapper;
use crate::resize::{axis_angle, drag_box, resize_frame, resized};
use crate::select_region::{LASSO_COVER, Region};
use crate::snap::Guide;
use crate::{Dir, Editor, Frame, Handle, Mods, Pointer, Tool, live};

/// O laço guarda um ponto novo a cada tantos px de ECRÃ (o Excalidraw simplifica a `5/zoom`).
pub(crate) const LASSO_STEP_PX: f64 = 2.0;

/// O que o carregar pede a quem chama.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Down {
    /// O editor tomou o gesto.
    Taken,
    /// É a mão: quem chama arrasta a VISTA.
    Pan,
}

/// Lado mínimo de uma forma, em unidades do mundo.
pub(crate) const MIN_SIDE: f64 = 1.0;

pub(crate) enum Gesture {
    Marquee {
        start: [f64; 2],
        cur: [f64; 2],
        base: BTreeSet<ElementId>,
        active: bool,
    },
    /// W23 B12 — o laço: o rasto desenhado (fecha-se sozinho), a selecção de antes (Shift soma).
    Lasso {
        trail: Vec<[f64; 2]>,
        base: BTreeSet<ElementId>,
        active: bool,
    },
    Move {
        start: [f64; 2],
        originals: Vec<Element>,
        targets: Snapper,
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
        targets: Snapper,
        guides: Vec<Guide>,
    },
    Rotate {
        frame: Frame,
        start: [f64; 2],
        originals: Vec<Element>,
        targets: Snapper,
        guides: Vec<Guide>,
    },
    Create {
        kind: ShapeType,
        /// A ferramenta Frame (o `kind` não conta).
        frame: bool,
        start: [f64; 2],
        id: Option<ElementId>,
        targets: Snapper,
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
    /// W23 B5(2) — arrastar o texto `index` de uma seta ao longo da linha (`caption.rs`).
    Caption {
        id: ElementId,
        index: usize,
        grab: [f64; 2],
        start: [f64; 2],
        moved: bool,
        original: Element,
    },
    /// W23 B5(1) — arrastar o segmento do meio do trecho `leg` de uma seta cotovelo (`elbow_drag.rs`).
    Slide {
        id: ElementId,
        leg: usize,
        mid: ph2d_board_route::ElbowMid,
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
        /// O mundo à vista (os alvos das guias recolhem-se quando a nota sai).
        view: [f64; 4],
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

    pub(crate) fn lasso(&self) -> Option<Vec<[f64; 2]>> {
        match self {
            Gesture::Lasso {
                trail,
                active: true,
                ..
            } => Some(trail.clone()),
            _ => None,
        }
    }

    pub(crate) fn guides(&self) -> Vec<Guide> {
        match self {
            Gesture::Move { guides, .. }
            | Gesture::Resize { guides, .. }
            | Gesture::Rotate { guides, .. }
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

impl Editor {
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
        if self.crop_down(doc, p) {
            return Down::Taken;
        }
        if self.text_pointer_down(doc, history, ts, p) {
            self.gesture = Some(Gesture::Text);
            return Down::Taken;
        }
        if self.ink_down(doc, p) {
            return Down::Taken;
        }
        if self.facilitate_down(doc, history, p) {
            return Down::Taken;
        }
        if self.linking {
            let target = self.hit(doc, p);
            self.link_click(doc, history, target);
            return Down::Taken;
        }
        if self.mind_pointer_down(doc, history, ts, p) {
            return Down::Taken;
        }
        match self.tool {
            Tool::Hand => return Down::Pan,
            Tool::Shape(_) | Tool::Frame => {
                let (kind, frame) = match self.tool {
                    Tool::Shape(k) => (k, false),
                    _ => (ShapeType::Rectangle, true),
                };
                self.gesture = Some(Gesture::Create {
                    kind,
                    frame,
                    start: p.world,
                    id: None,
                    targets: self.snap_create(doc, view, p.world),
                    guides: Vec::new(),
                });
                return Down::Taken;
            }
            Tool::Connector => {
                self.gesture = Some(self.begin_connect(doc, p, None, None));
                return Down::Taken;
            }
            Tool::Lasso => {
                self.begin_lasso(p);
                return Down::Taken;
            }
            // W23 B12 — `Ctrl+Alt`+arrastar na Selecção é o laço (o Excalidraw), em cima de tudo.
            Tool::Select if p.mods.ctrl && p.mods.alt => {
                self.begin_lasso(p);
                return Down::Taken;
            }
            Tool::Select | Tool::Pen(_) | Tool::Eraser { .. } | Tool::Laser | Tool::MindMap => {}
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
                crate::wire::WireHandle::Segment(leg) => self.begin_slide(doc, id, leg, p),
                _ => self.begin_bend(doc, id, h, p),
            };
            return Down::Taken;
        }
        if self.caption_down(doc, ts, p) {
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
            // Rodar um frame roda o que vive dentro dele, à volta do mesmo centro; redimensioná-lo
            // não escala os filhos (o oráculo A8).
            let originals: Vec<Element> = if h == Handle::Rotate {
                self.moving(doc)
            } else {
                self.movable(doc)
            }
            .into_iter()
            .filter(|el| el.connector().is_none())
            .collect();
            if originals.is_empty() {
                self.notice = Some(crate::Notice::Locked);
                return Down::Taken;
            }
            self.gesture = Some(match h {
                Handle::Rotate => Gesture::Rotate {
                    frame,
                    start: p.world,
                    targets: self.snap_rotate(doc, view, &originals),
                    originals,
                    guides: Vec::new(),
                },
                Handle::Resize(dir) => Gesture::Resize {
                    dir,
                    frame,
                    originals,
                    targets: self.snap_resize(doc, view, &frame),
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
                    view,
                });
            }
            Some(id) => {
                // Fora do grupo em que se entrou, sai-se dele (oráculo B).
                self.entered = self
                    .entered
                    .filter(|g| doc.get(id).is_some_and(|el| el.groups().contains(g)));
                let pick = self.pick(doc, id);
                let was = self.selection.contains(&id);
                let mut collapse_to = None;
                if p.mods.shift {
                    if was {
                        self.selection.retain(|i| !pick.contains(i));
                        return Down::Taken;
                    }
                    self.selection.extend(pick);
                } else if !was {
                    self.selection = pick;
                } else if self.selection.len() > pick.len() {
                    collapse_to = Some(id);
                }
                let originals = self.moving(doc);
                self.gesture = Some(Gesture::Move {
                    start: p.world,
                    targets: self.snap_move(doc, view, p.world, &originals),
                    originals,
                    active: false,
                    copies: Vec::new(),
                    collapse_to,
                    guides: Vec::new(),
                });
            }
            None => {
                if !p.mods.shift {
                    self.selection.clear();
                    self.leave_groups();
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

    fn begin_lasso(&mut self, p: Pointer) {
        if !p.mods.shift {
            self.selection.clear();
            self.leave_groups();
        }
        self.gesture = Some(Gesture::Lasso {
            trail: vec![p.world],
            base: self.selection.clone(),
            active: false,
        });
    }

    /// O ponteiro andou (com o botão em baixo).
    pub fn pointer_move(&mut self, doc: &mut BoardDoc, ts: &mut TextSystem, p: Pointer) {
        if self.crop_move(doc, p) {
            return;
        }
        if self.ink_move(doc, p) {
            return;
        }
        let Some(mut g) = self.gesture.take() else {
            return;
        };
        let drag = self.metrics.drag * p.px;
        let snap = (!p.mods.ctrl).then_some(self.metrics.snap * p.px);
        match &mut g {
            Gesture::Peel { stack, start, view } => {
                if dist(*start, p.world) <= drag {
                    self.gesture = Some(g);
                    return;
                }
                // Tira a nota de cima da pilha e passa a arrastá-la (largar = UM passo: apagá-la).
                let note = self.peel(doc, stack);
                let id = note.id;
                live(doc, note.clone());
                self.selection = BTreeSet::from([id]);
                let targets = self.snap_move(doc, *view, p.world, std::slice::from_ref(&note));
                self.gesture = Some(Gesture::Move {
                    start: *start,
                    originals: vec![note],
                    targets,
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
            Gesture::Slide { .. } => self.slide_move(doc, &mut g, p),
            Gesture::Caption { .. } => self.caption_move(doc, &mut g, p),
            Gesture::Marquee {
                start,
                cur,
                base,
                active,
            } => {
                *cur = p.world;
                *active |= dist(*start, *cur) > drag;
                if *active {
                    let picked = self.inside(doc, Region::Rect(rect(*start, *cur)));
                    self.selection = base.iter().copied().chain(picked).collect();
                }
            }
            Gesture::Lasso {
                trail,
                base,
                active,
            } => {
                // Um ponto novo a cada LASSO_STEP_PX de ecrã: abaixo disso não se vê.
                let last = *trail.last().expect("nasce com o 1.º ponto");
                if dist(last, p.world) >= LASSO_STEP_PX * p.px {
                    trail.push(p.world);
                }
                *active |= dist(trail[0], p.world) > drag;
                if *active {
                    let picked = self.inside(doc, Region::Lasso(trail, LASSO_COVER));
                    self.selection = base.iter().copied().chain(picked).collect();
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
                    if originals.is_empty() {
                        self.notice = Some(crate::Notice::Locked);
                    }
                    if p.mods.alt {
                        *copies = self.duplicate_live(doc, originals);
                        let copy_state: Vec<Element> = copies
                            .iter()
                            .filter_map(|id| doc.get(*id).cloned())
                            .collect();
                        *originals = copy_state;
                        // Os originais ficam: passam a ser alvos (alinhar a cópia com eles).
                        targets.reskip(doc, copies.iter().copied().collect());
                    }
                }
                let mut d = [p.world[0] - start[0], p.world[1] - start[1]];
                let mut free = [true; 2];
                if p.mods.shift {
                    let k = usize::from(d[0].abs() > d[1].abs());
                    d[k] = 0.0;
                    free[k] = false;
                }
                guides.clear();
                targets.follow(doc, p.world);
                let laid = self.mind_drag_move(doc, originals, copies, p.world);
                if let Some(tol) = snap.filter(|_| !laid) {
                    let (s, g2) = targets.snap_moving(d, tol, free);
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
                // As notas (com a letra, o Miro), as imagens (W14) e os vídeos (W18) escalam inteiros.
                let uniform = (originals.len() > 1
                    && originals.iter().any(|o| !axis_angle(o.angle)))
                    || originals
                        .iter()
                        .any(|o| is_note(o) || o.image().is_some() || o.embed().is_some());
                let mut world = p.world;
                guides.clear();
                // Em proporção a pega não fica onde o ponteiro a pôs: a guia mentiria.
                if let Some(tol) = snap.filter(|_| !p.mods.shift && !uniform) {
                    let (s, g2) = targets.snap_handle(world, tol, dir.sign(), frame.angle);
                    world = [world[0] + s[0], world[1] + s[1]];
                    *guides = g2;
                }
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
                targets,
                guides,
            } => {
                let delta;
                (delta, *guides) = targets.turn(frame, *start, p, snap);
                for o in originals.iter() {
                    live(doc, crate::resize::turned(o, frame.center, delta));
                }
            }
            Gesture::Create {
                kind,
                frame,
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
                // `Shift` faz um quadrado e a nota tem a proporção dela: o ponteiro só manda na
                // largura (na nota) ou em nada (no quadrado).
                let note = kind.note_aspect().is_some() && !*frame;
                if let Some(tol) = snap.filter(|_| !p.mods.shift) {
                    let (s, g2) = targets.snap_point(world, tol, [true, !note]);
                    world = [world[0] + s[0], world[1] + s[1]];
                    *guides = g2;
                }
                let mut bx = drag_box(*start, world, p.mods);
                if let Some(a) = kind.note_aspect().filter(|_| !*frame) {
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
                        let el = self.born(doc, *kind, *frame, *start, bx);
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
        if self.crop_up(doc, history) {
            return;
        }
        if self.ink_up(doc, history, p) {
            return;
        }
        let Some(g) = self.gesture.take() else {
            return;
        };
        match g {
            Gesture::Text | Gesture::Marquee { .. } | Gesture::Lasso { .. } => {}
            Gesture::Peel { stack, .. } => self.selection = BTreeSet::from([stack.id]),
            Gesture::Grid { originals, .. } => history.record(undo_ops(doc, &originals), doc),
            g @ Gesture::Connect { .. } => self.connect_up(doc, history, g),
            g @ Gesture::Bend { .. } => self.bend_up(doc, history, g, p),
            g @ Gesture::Slide { .. } => self.slide_up(doc, history, g),
            g @ Gesture::Caption { .. } => self.caption_up(doc, history, g),
            Gesture::EndDrag { original, .. } => {
                history.record(undo_ops(doc, std::slice::from_ref(&original)), doc);
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
                        self.selection = self.pick(doc, id);
                    }
                    return;
                }
                if let Some(l) = self.mind_lead_of(doc, &originals, &copies) {
                    return self.mind_drop(doc, history, l, &originals, &copies, p);
                }
                // A casa decide-a o PONTEIRO ao largar (oráculo A4/A5).
                let moved: BTreeSet<ElementId> = if copies.is_empty() {
                    originals.iter().map(|o| o.id).collect()
                } else {
                    copies.iter().copied().collect()
                };
                let roots = crate::organize::roots_of(doc, &moved);
                for el in crate::organize::rehome_on_drop(doc, &roots, p.world) {
                    live(doc, el);
                }
                crate::mindmap::settle_moved(doc, moved.iter().copied());
                if copies.is_empty() {
                    history.record(undo_ops(doc, &originals), doc);
                } else {
                    history.record(copies.into_iter().rev().map(BoardOp::Delete).collect(), doc);
                }
            }
            Gesture::Resize { originals, .. } => {
                // Um frame redimensionado adopta o que lhe fica inteiro dentro e solta o filho que
                // ficou inteiro fora (oráculo A8) — no MESMO passo.
                let mut undo = undo_ops(doc, &originals);
                for f in originals.iter().filter(|o| o.as_frame().is_some()) {
                    crate::organize::settle_live(doc, f.id, &mut undo);
                }
                crate::mindmap::settle_moved(doc, originals.iter().map(|o| o.id));
                history.record(undo, doc);
            }
            Gesture::Rotate { originals, .. } => {
                crate::mindmap::settle_moved(doc, originals.iter().map(|o| o.id));
                history.record(undo_ops(doc, &originals), doc);
            }
            Gesture::Create {
                kind,
                frame,
                start,
                id,
                ..
            } => {
                let id = id.unwrap_or_else(|| {
                    let [w, h] = if frame {
                        crate::FRAME_CLICK_SIZE
                    } else {
                        self.metrics.click_size
                    };
                    let bx = if kind.is_note() && !frame {
                        self.note_box(kind, start)
                    } else {
                        [start[0] - w / 2.0, start[1] - h / 2.0, w, h]
                    };
                    let el = self.born(doc, kind, frame, start, bx);
                    let id = el.id;
                    live(doc, el);
                    id
                });
                let mut undo = vec![BoardOp::Delete(id)];
                if frame {
                    // O frame desenhado adopta o que lhe ficou inteiro dentro (oráculo A1).
                    crate::organize::settle_live(doc, id, &mut undo);
                }
                history.record(undo, doc);
                self.selection = BTreeSet::from([id]);
                self.tool = Tool::Select;
            }
        }
    }
}
