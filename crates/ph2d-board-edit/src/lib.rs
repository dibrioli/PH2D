//! **Os gestos do Quadro** (MiroClone, W1) — o [`Editor`] de UM quadro: a ferramenta, a selecção, o
//! gesto em curso, o texto em edição e a área de transferência.
//!
//! Entra o ponteiro já no MUNDO ([`Pointer`]: ponto, tamanho de um px de ecrã em unidades do mundo,
//! modificadores) e as teclas já traduzidas ([`Command`], [`TextKey`]); sai o documento mudado — por
//! [`BoardOp`], com UM passo de [`History`] por gesto — e o [`Overlay`] que se desenha por cima.
//!
//! ⚠️ **Um arrasto mexe no documento AO VIVO** (o desenho é o do documento, sem segunda fonte de
//! verdade) e guarda o passo de desfazer só ao largar, com o estado do início do gesto.

mod command;
mod gesture;
mod snap;
mod text;

use std::collections::BTreeSet;

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, History, Shape, ShapeType, Style};
use ph2d_text::TextSystem;

pub use gesture::Down;
pub use ph2d_board_layout::Move;
pub use snap::Guide;

/// Tamanho de uma forma criada com um clique (sem arrastar), em unidades do MUNDO — o formato
/// paisagem de uma caixa de fluxograma.
pub const CLICK_SIZE: [f64; 2] = [160.0, 100.0];
/// Desvio de cada colagem/duplicação, em unidades do MUNDO (o do Figma e do Excalidraw).
pub const PASTE_OFFSET: f64 = 10.0;

/// A ferramenta activa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    /// Arrastar move a vista (quem move a câmara é o editor; aqui só se diz que é a mão).
    Hand,
    Shape(ShapeType),
}

/// Modificadores no instante do evento.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    /// `Ctrl` (ou `Cmd`): suspende as guias de alinhamento.
    pub ctrl: bool,
    /// `Alt`: arrastar duplica; redimensionar e criar partem do centro.
    pub alt: bool,
}

/// O ponteiro no mundo do quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pointer {
    pub world: [f64; 2],
    /// Quanto mede UM px de ecrã em unidades do mundo (`1 / zoom`) — as tolerâncias são de ecrã.
    pub px: f64,
    pub mods: Mods,
}

/// Medidas de ECRÃ (px) que o editor recebe da interface — vêm dos tokens, não daqui.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Lado da pega de redimensionar.
    pub handle: f64,
    /// Distância da pega de rodar acima do topo da moldura.
    pub rotate_offset: f64,
    /// Até onde uma borda «cola» numa guia de alinhamento.
    pub snap: f64,
    /// Quanto o dedo anda até um toque virar arrasto.
    pub drag: f64,
    /// Tamanho de uma forma criada com um clique (sem arrastar), em unidades do MUNDO.
    pub click_size: [f64; 2],
    /// Desvio de cada colagem/duplicação, em unidades do MUNDO.
    pub paste_offset: f64,
}

/// Um lado ou canto da moldura de selecção — `(sx, sy)` em `{-1, 0, 1}²`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Dir {
    pub const ALL: [Dir; 8] = [
        Dir::NW,
        Dir::N,
        Dir::NE,
        Dir::E,
        Dir::SE,
        Dir::S,
        Dir::SW,
        Dir::W,
    ];

    #[must_use]
    pub fn sign(self) -> (f64, f64) {
        match self {
            Dir::N => (0.0, -1.0),
            Dir::NE => (1.0, -1.0),
            Dir::E => (1.0, 0.0),
            Dir::SE => (1.0, 1.0),
            Dir::S => (0.0, 1.0),
            Dir::SW => (-1.0, 1.0),
            Dir::W => (-1.0, 0.0),
            Dir::NW => (-1.0, -1.0),
        }
    }
}

/// Uma pega da moldura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Resize(Dir),
    Rotate,
}

/// Uma caixa rodada: o centro, o tamanho e o ângulo (mundo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub center: [f64; 2],
    pub w: f64,
    pub h: f64,
    pub angle: f64,
}

impl Frame {
    #[must_use]
    pub fn of(el: &Element) -> Self {
        Self {
            center: el.center(),
            w: el.w,
            h: el.h,
            angle: el.angle,
        }
    }

    /// Ponto local (relativo ao centro, antes de rodar) → mundo.
    #[must_use]
    pub fn point(&self, local: [f64; 2]) -> [f64; 2] {
        let (s, c) = self.angle.sin_cos();
        [
            self.center[0] + local[0] * c - local[1] * s,
            self.center[1] + local[0] * s + local[1] * c,
        ]
    }

    /// Mundo → ponto local (relativo ao centro, antes de rodar).
    #[must_use]
    pub fn local(&self, world: [f64; 2]) -> [f64; 2] {
        let (s, c) = (-self.angle).sin_cos();
        let (dx, dy) = (world[0] - self.center[0], world[1] - self.center[1]);
        [dx * c - dy * s, dx * s + dy * c]
    }

    /// Onde está a pega `h` (mundo), com `px` = um px de ecrã e as medidas da interface.
    #[must_use]
    pub fn handle(&self, h: Handle, px: f64, m: &Metrics) -> [f64; 2] {
        match h {
            Handle::Resize(d) => {
                let (sx, sy) = d.sign();
                self.point([sx * self.w / 2.0, sy * self.h / 2.0])
            }
            Handle::Rotate => self.point([0.0, -self.h / 2.0 - m.rotate_offset * px]),
        }
    }
}

/// O que se desenha por cima do quadro, no MUNDO (quem desenha converte para o ecrã).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Overlay {
    /// O contorno fino de cada elemento seleccionado.
    pub boxes: Vec<Frame>,
    /// A moldura com as pegas (`None` sem selecção, ou a escrever).
    pub frame: Option<Frame>,
    /// O rectângulo da selecção por arrasto `[x0, y0, x1, y1]`.
    pub marquee: Option<[f64; 4]>,
    pub guides: Vec<Guide>,
    /// O texto em edição: o elemento, os rectângulos da selecção e o do cursor, no espaço do
    /// texto (origem no início do bloco, [`ph2d_board_geom::text_origin`]).
    pub text: Option<TextOverlay>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextOverlay {
    pub element: ElementId,
    /// Onde começa o bloco de texto, em coordenadas locais da caixa do elemento.
    pub origin: [f64; 2],
    pub selection: Vec<[f64; 4]>,
    pub caret: Option<[f64; 4]>,
}

/// Uma ordem do teclado (ou de um menu) — fora da edição de texto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Delete,
    Duplicate,
    SelectAll,
    /// Sai do gesto em curso (devolvendo o que ele mexeu), depois da selecção, depois da ferramenta.
    Escape,
    /// Empurra a selecção por `(dx, dy)` em unidades do MUNDO.
    Nudge([f64; 2]),
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    /// Começa a escrever na forma seleccionada (se é uma só).
    EditText,
    Tool(Tool),
}

/// Uma tecla dentro do texto em edição.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKey {
    Backspace {
        word: bool,
    },
    Delete {
        word: bool,
    },
    Move {
        to: Move,
        extend: bool,
    },
    SelectAll,
    Newline,
    /// `Esc` (ou `Ctrl+Enter`): termina a edição, guardando o texto.
    Commit,
}

/// O editor de um quadro.
pub struct Editor {
    pub tool: Tool,
    /// O estilo da PRÓXIMA forma criada (o último escolhido na barra de estilo).
    pub style: Style,
    metrics: Metrics,
    selection: BTreeSet<ElementId>,
    gesture: Option<gesture::Gesture>,
    editing: Option<text::Editing>,
    clipboard: Vec<Element>,
    pastes: u32,
}

impl Editor {
    #[must_use]
    pub fn new(style: Style, metrics: Metrics) -> Self {
        Self {
            tool: Tool::Select,
            style,
            metrics,
            selection: BTreeSet::new(),
            gesture: None,
            editing: None,
            clipboard: Vec::new(),
            pastes: 0,
        }
    }

    #[must_use]
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// Os seleccionados, por id.
    #[must_use]
    pub fn selection(&self) -> &BTreeSet<ElementId> {
        &self.selection
    }

    /// Um gesto do ponteiro está em curso (o editor quer os Move e o Up).
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.gesture.is_some()
    }

    /// A escrever dentro de uma forma (as teclas são do texto).
    #[must_use]
    pub fn is_editing_text(&self) -> bool {
        self.editing.is_some()
    }

    /// Selecciona exactamente `ids` (os que não existem são ignorados).
    pub fn select(&mut self, doc: &BoardDoc, ids: impl IntoIterator<Item = ElementId>) {
        self.selection = ids
            .into_iter()
            .filter(|id| doc.get(*id).is_some())
            .collect();
    }

    /// Esquece a selecção de elementos que já não existem (depois de um desfazer, por exemplo).
    pub fn prune(&mut self, doc: &BoardDoc) {
        self.selection.retain(|id| doc.get(*id).is_some());
        if self
            .editing
            .as_ref()
            .is_some_and(|e| doc.get(e.id).is_none())
        {
            self.editing = None;
        }
    }

    /// Os seleccionados vivos, em ordem de z.
    fn selected<'a>(&self, doc: &'a BoardDoc) -> Vec<&'a Element> {
        let mut v: Vec<&Element> = self
            .selection
            .iter()
            .filter_map(|id| doc.get(*id))
            .collect();
        v.sort_by(|a, b| a.z.cmp(&b.z));
        v
    }

    /// A moldura das pegas: a caixa rodada de um elemento só, ou a caixa alinhada de vários.
    #[must_use]
    pub fn frame(&self, doc: &BoardDoc) -> Option<Frame> {
        let sel = self.selected(doc);
        match sel.as_slice() {
            [] => None,
            [one] => Some(Frame::of(one)),
            many => {
                let [x0, y0, x1, y1] = union_aabb(many.iter().copied());
                Some(Frame {
                    center: [(x0 + x1) / 2.0, (y0 + y1) / 2.0],
                    w: x1 - x0,
                    h: y1 - y0,
                    angle: 0.0,
                })
            }
        }
    }

    /// Aplica um estilo à selecção (UM passo de desfazer) e guarda-o para as próximas formas.
    pub fn set_style(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        change: impl Fn(&mut Style),
    ) {
        change(&mut self.style);
        let ops = self
            .selected(doc)
            .into_iter()
            .filter_map(|el| {
                let mut el = el.clone();
                change(&mut el.shape_mut()?.style);
                Some(BoardOp::Put(el))
            })
            .collect();
        history.apply(doc, ops);
    }

    /// Troca o contorno das formas seleccionadas (UM passo).
    pub fn set_shape_type(&mut self, doc: &mut BoardDoc, history: &mut History, t: ShapeType) {
        let ops = self
            .selected(doc)
            .into_iter()
            .filter_map(|el| {
                let mut el = el.clone();
                el.shape_mut()?.kind = t;
                Some(BoardOp::Put(el))
            })
            .collect();
        history.apply(doc, ops);
    }

    /// O que desenhar por cima do quadro agora.
    pub fn overlay(&mut self, doc: &BoardDoc, ts: &mut TextSystem) -> Overlay {
        let text = self.editing.as_mut().and_then(|e| e.overlay(doc, ts));
        let editing = text.is_some();
        let sel = self.selected(doc);
        Overlay {
            boxes: if sel.len() > 1 {
                sel.iter().map(|el| Frame::of(el)).collect()
            } else {
                Vec::new()
            },
            frame: if editing { None } else { self.frame(doc) },
            marquee: self.gesture.as_ref().and_then(gesture::Gesture::marquee),
            guides: self
                .gesture
                .as_ref()
                .map(gesture::Gesture::guides)
                .unwrap_or_default(),
            text,
        }
    }

    /// Uma forma nova (com o estilo actual) na caixa dada, à frente de tudo.
    fn new_shape(&self, doc: &mut BoardDoc, kind: ShapeType, bx: [f64; 4]) -> Element {
        let shape = Shape {
            kind,
            style: self.style.clone(),
            text: String::new(),
        };
        Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx)
    }
}

/// A caixa alinhada que contém todos `els` (rodados): `[x0, y0, x1, y1]`.
fn union_aabb<'a>(els: impl Iterator<Item = &'a Element>) -> [f64; 4] {
    els.fold(
        [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
        |acc, el| {
            let b = el.aabb();
            [
                acc[0].min(b[0]),
                acc[1].min(b[1]),
                acc[2].max(b[2]),
                acc[3].max(b[3]),
            ]
        },
    )
}

/// Aplica uma operação ao vivo (durante um gesto: o passo de desfazer guarda-se no fim).
fn live(doc: &mut BoardDoc, el: Element) {
    let _ = BoardOp::Put(el).apply(doc);
}

#[cfg(test)]
mod tests;
