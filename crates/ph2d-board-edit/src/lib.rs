//! **Os gestos do Quadro** (MiroClone, W1) — o [`Editor`] de UM quadro: a ferramenta, a selecção, o
//! gesto em curso, o texto em edição e a área de transferência.
//!
//! Entra o ponteiro já no MUNDO ([`Pointer`]: ponto, tamanho de um px de ecrã em unidades do mundo,
//! modificadores) e as teclas já traduzidas ([`Command`], [`TextKey`]); sai o documento mudado — por
//! [`BoardOp`], com UM passo de [`History`] por gesto — e o [`Overlay`] que se desenha por cima.
//!
//! ⚠️ **Um arrasto mexe no documento AO VIVO** (o desenho é o do documento, sem segunda fonte de
//! verdade) e guarda o passo de desfazer só ao largar, com o estado do início do gesto.

mod cluster;
mod command;
mod facilitate;
mod frame_box;
mod gesture;
mod gesture_end;
mod guides;
mod ink;
mod links_tags;
mod mindmap;
mod mindmap_drop;
mod notes;
mod notes_layout;
mod organize;
mod overlay;
mod present;
mod resize;
mod snap;
mod templates;
mod text;
mod timer;
mod vote;
mod wire;

use std::collections::BTreeSet;
use std::time::Instant;

use ph2d_board_model::{
    BoardDoc, BoardOp, Element, ElementId, Head, History, Letter, Route, Shape, ShapeType, Style,
};
use ph2d_board_route::RouteCache;

pub use cluster::{
    CLUSTER_BETWEEN, CLUSTER_GAP, CLUSTER_PAD, CLUSTER_TITLE_WORDS, ClusterBy, ClusterNames,
};
pub use facilitate::{VOTE_MINUS_STEP, VoteMark, chip_at};
pub use frame_box::Frame;
pub use gesture::Down;
pub use ink::{LASER_LIFE, PEN_WIDTHS, PenBox, PenPreset, point_segment, world_points};
pub use mindmap::{digits, node_size};
pub use notes::{
    NOTE_DEFAULT_SIZE, NOTE_GAP_K, NOTE_SCALES, NoteStyle, grid_layout, note_scale, note_size,
    parse_cells, reading_order,
};
pub use organize::{Align, Axis, Notice, Order};
pub use overlay::{Overlay, Target, TextOverlay, Wire, grid_handle_offset};
pub use ph2d_board_layout::Move;
pub use present::{
    SLIDE_MARGIN, TRAVEL_POWER, TRAVEL_SECS, camera_at, first_slide, slide_camera, slides, travel,
};
pub use snap::Guide;
pub use templates::Template;
pub use text::{face, hand_lettered};
pub use timer::{
    TIMER_BLINK, TIMER_COARSE_STEP, TIMER_DEFAULT_MIN, TIMER_FINE_UNTIL, TIMER_MAX, Timer,
    TimerRun, step_minutes,
};
pub use vote::{
    VOTES_DEFAULT, VOTES_MAX, VoteSession, VoteSetup, results as vote_results, votable, vote_range,
};

/// Tamanho de uma forma criada com um clique (sem arrastar), em unidades do MUNDO — o formato
/// paisagem de uma caixa de fluxograma.
pub const CLICK_SIZE: [f64; 2] = [160.0, 100.0];
/// Desvio de cada colagem/duplicação, em unidades do MUNDO (o do Figma e do Excalidraw).
pub const PASTE_OFFSET: f64 = 10.0;

/// Os tamanhos de letra que a barra oferece (mundo): P · M · G · GG — os do Excalidraw.
pub const FONT_SIZES: [f64; 4] = [16.0, 20.0, 28.0, 36.0];
/// As espessuras de contorno oferecidas (mundo): as do Excalidraw (fina 1 · grossa 2 · muito grossa 4)
/// e, à frente, uma AINDA MAIS FINA, 0,5 — ordem do dono (07/10: *«a opção mais fina deve ser ainda
/// mais fina»*), a de nascença ([`ph2d_board_model::DEFAULT_STROKE_WIDTH`]).
pub const STROKE_WIDTHS: [f64; 4] = [0.5, 1.0, 2.0, 4.0];
/// O empurrão das setas (mundo): normal e com `Shift` — o do Figma.
pub const NUDGE: [f64; 2] = [1.0, 10.0];
/// O vão entre uma forma e a seguinte que um ponto azul (ou `Ctrl+seta`) cria já ligada, em
/// unidades do MUNDO: metade da largura de uma caixa de nascença ([`CLICK_SIZE`]) — o passo de um
/// fluxograma, com espaço para a seta e para a ponta dela.
pub const NEXT_GAP: f64 = CLICK_SIZE[0] / 2.0;
/// Um frame posto com um clique (sem arrastar), em unidades do MUNDO: 16:9, cinco caixas de nascença
/// de largo. ⚠️ O tamanho de nascença do Miro está por medir (o help só diz que há predefinições).
pub const FRAME_CLICK_SIZE: [f64; 2] = [CLICK_SIZE[0] * 5.0, CLICK_SIZE[0] * 5.0 * 9.0 / 16.0];

/// A ferramenta activa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    /// Arrastar move a vista (quem move a câmara é o editor; aqui só se diz que é a mão).
    Hand,
    Shape(ShapeType),
    /// A seta: arrastar de uma forma (ou do vazio) até outra liga-as.
    Connector,
    /// A caneta ou o marcador (W4): arrastar desenha um traço.
    Pen(ph2d_board_model::Pen),
    /// A borracha: apaga os traços da caneta que tocar; `precise` = só o pedaço por onde passa.
    Eraser {
        precise: bool,
    },
    /// O ponteiro laser: um rasto que se apaga sozinho, fora do documento.
    Laser,
    /// O frame (W5, tecla `F` do Miro): arrastar desenha um; um clique põe um do tamanho de nascença.
    Frame,
    /// O mapa mental (W6): um clique põe a raiz de um mapa novo, já a escrever.
    MindMap,
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
    /// A faixa junto ao contorno de uma forma onde uma ponta de seta se prende a um PONTO FIXO (no
    /// miolo, ao centro) — e até onde, por fora, ela ainda se prende.
    pub bind: f64,
    /// A distância dos pontos azuis de criação rápida ao lado da forma.
    pub dot: f64,
    /// O raio da borracha.
    pub eraser: f64,
    /// A altura da faixa do TÍTULO de um frame, por cima dele (agarra-se por ali e pela borda — o
    /// Miro: *«Click and drag the frame's border or title»*).
    pub frame_title: f64,
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
    /// Começa a escrever na forma seleccionada (se é uma só) — ou no rótulo da seta.
    EditText,
    /// Liga ou desliga uma marca do texto (`Ctrl+B`/`I`/`U`, `Ctrl+Shift+X`): no texto seleccionado
    /// a escrever, ou no texto inteiro das formas seleccionadas.
    Mark(ph2d_board_model::Mark),
    Tool(Tool),
    /// `Ctrl+seta`: cria a forma seguinte, já ligada, do lado `(dx, dy)` (unitário) da única forma
    /// seleccionada — e selecciona-a, para a próxima seguir dela.
    Grow([f64; 2]),
    /// `Ctrl+G` agrupa · `Ctrl+Shift+G` desagrupa (W5).
    Group,
    Ungroup,
    /// `Ctrl+Shift+L`: trava ou destrava a selecção (W5).
    Lock,
    /// `PgUp`/`PgDn` (com `Shift`, um passo): a ordem de desenho (W5).
    Order(Order),
    /// `Ctrl+Alt+K`: liga a selecção a outro elemento (o próximo clique), ou tira-lhe o link.
    Link,
    Align(Align),
    Distribute(Axis),
    /// Mapa mental (W6): `Tab` = o filho · `Enter` a escrever = o irmão logo depois · as setas andam
    /// pela árvore (`(dx, dy)` unitário) · `Ctrl+/` recolhe ou expande · o auto layout do nó · a
    /// direcção do mapa.
    MindChild,
    MindSibling,
    MindNav([f64; 2]),
    MindCollapse,
    MindAuto,
    MindVertical,
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
    /// O estilo da PRÓXIMA forma (e seta) criada — o último escolhido na barra de estilo.
    pub style: Style,
    /// A rota e as pontas da PRÓXIMA seta.
    pub route: Route,
    pub heads: [Head; 2],
    metrics: Metrics,
    routes: RouteCache,
    /// A forma sob o ponteiro (os pontos azuis dela aparecem).
    hover: Option<ElementId>,
    selection: BTreeSet<ElementId>,
    gesture: Option<gesture::Gesture>,
    editing: Option<text::Editing>,
    clipboard: Vec<Element>,
    pastes: u32,
    /// Como nasce a próxima nota.
    pub notes: NoteStyle,
    /// O rascunho do modo em massa, se está aberto.
    bulk: Option<ElementId>,
    /// Notas nascidas sem o moldador à mão (o fim do modo em massa ao trocar de aba): a altura delas
    /// ajusta-se ao texto no próximo [`Editor::overlay`].
    unfitted: Vec<ElementId>,
    /// O estilo do texto em edição mudou sem o moldador (um clique na barra): a forma ajusta-se ao
    /// texto no próximo [`Editor::overlay`].
    resync: bool,
    /// O texto que a última cópia de elementos pôs na área de transferência do sistema — um
    /// `Ctrl+V` com OUTRO texto lá é uma colagem de fora (uma planilha).
    copied_text: Option<String>,
    /// As predefinições das canetas.
    pub pen: PenBox,
    ink: Option<ink::InkGesture>,
    laser: Vec<([f64; 2], Instant)>,
    /// O último ponto do ponteiro no quadro (o anel da borracha).
    cursor: Option<[f64; 2]>,
    /// O grupo em que se entrou com duplo-clique (W5): um clique selecciona o nível abaixo dele.
    entered: Option<ph2d_board_model::GroupId>,
    /// O modo de ligar: o próximo clique escolhe o destino do link (W5).
    linking: bool,
    /// A última recusa, para a vista a mostrar (e a esvaziar com [`Editor::take_notice`]).
    notice: Option<Notice>,
    /// O nome de nascença do frame `n` (1, 2, …) — a vista dá-o já traduzido.
    pub frame_name: fn(usize) -> String,
    /// O nó onde o arrasto de um nó de mapa o largaria como filho (o realce, W6).
    mind_drop: Option<ElementId>,
    /// A sessão de voto aberta (W7).
    vote: Option<VoteSession>,
    /// W12 (ADR-0182) — as notas de outros com o texto escondido de mim: não se escreve nem se vota
    /// no que não se lê. A vista pô-las aqui a cada imagem.
    unread: BTreeSet<ElementId>,
}

impl Editor {
    #[must_use]
    pub fn new(style: Style, metrics: Metrics) -> Self {
        Self {
            tool: Tool::Select,
            style,
            route: Route::default(),
            heads: ph2d_board_model::Connector::DEFAULT_HEADS,
            metrics,
            routes: RouteCache::default(),
            hover: None,
            selection: BTreeSet::new(),
            gesture: None,
            editing: None,
            clipboard: Vec::new(),
            pastes: 0,
            notes: NoteStyle::default(),
            bulk: None,
            unfitted: Vec::new(),
            resync: false,
            copied_text: None,
            pen: PenBox::default(),
            ink: None,
            laser: Vec::new(),
            cursor: None,
            entered: None,
            linking: false,
            notice: None,
            frame_name: |n| n.to_string(),
            mind_drop: None,
            vote: None,
            unread: BTreeSet::new(),
        }
    }

    /// W12 — as notas com o texto escondido de mim (a sessão ao vivo di-lo a cada imagem).
    pub fn set_unread(&mut self, ids: BTreeSet<ElementId>) {
        self.unread = ids;
    }

    /// W12 — as notas com o texto escondido de mim: quem pinta um texto delas (a busca, as linhas de
    /// um resultado) pinta o marcador.
    #[must_use]
    pub fn unread(&self) -> &BTreeSet<ElementId> {
        &self.unread
    }

    #[must_use]
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    /// As rotas das setas de `doc`, em dia (o desenho lê-as daqui: uma cache, uma verdade).
    pub fn routes(&mut self, doc: &BoardDoc) -> &RouteCache {
        self.routes.sync(doc);
        &self.routes
    }

    /// A última recusa (para a vista a mostrar como aviso), esvaziando-a.
    pub fn take_notice(&mut self) -> Option<Notice> {
        self.notice.take()
    }

    /// Os seleccionados, por id.
    #[must_use]
    pub fn selection(&self) -> &BTreeSet<ElementId> {
        &self.selection
    }

    /// Um gesto do ponteiro está em curso (o editor quer os Move e o Up).
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.gesture.is_some() || self.ink.is_some()
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

    /// Os seleccionados que se podem EDITAR (os destravados), em ordem de z — o que muda com um
    /// estilo, uma marca, a ordem ou uma etiqueta (o Miro: travado = *«not moved, edited, or
    /// deleted»*).
    pub(crate) fn editable<'a>(&self, doc: &'a BoardDoc) -> Vec<&'a Element> {
        let mut v = self.selected(doc);
        v.retain(|el| !el.locked());
        v
    }

    /// W12 — os editáveis cujo TEXTO se pode mudar (marcas, cor das letras): nunca o de uma nota que
    /// não leio — o aviso diz porquê (a cor e o resto da nota continuam de todos).
    pub(crate) fn text_editable<'a>(&mut self, doc: &'a BoardDoc) -> Vec<&'a Element> {
        let mut v = self.editable(doc);
        let n = v.len();
        v.retain(|el| !self.unread.contains(&el.id));
        if v.len() < n {
            self.notice = Some(Notice::Private);
        }
        v
    }

    /// A moldura das pegas: a caixa rodada de uma forma (ou traço) só, ou a caixa alinhada de
    /// várias. As setas não entram (têm as pegas das pontas).
    #[must_use]
    pub fn frame(&self, doc: &BoardDoc) -> Option<Frame> {
        let mut sel = self.selected(doc);
        sel.retain(|el| el.connector().is_none());
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
            .editable(doc)
            .into_iter()
            .map(|el| {
                let mut el = el.clone();
                change(el.style_mut());
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
    }

    /// ⭐ **O botão «Rascunho ↔ Final»** (W4): com selecção, troca as formas e setas dela; sem
    /// selecção, o quadro INTEIRO — e devolve o modo novo do quadro (`Some`), que é também o das
    /// formas que nascerem. Fica tudo à mão se havia alguma final, e final se já estava tudo à mão.
    /// As notas entram só na letra; os traços da caneta não entram (já são mão). UM passo. A letra
    /// fixa de um `.excalidraw` (`Letter::Hand`/`Sans`) volta ao modo (`Auto`) em todo o alcançado. Um
    /// texto aberto fecha-se primeiro, no seu próprio passo.
    pub fn toggle_sketch(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        board: bool,
    ) -> Option<bool> {
        // O texto em edição fecha-se antes (é o passo dele): a letra nova não se mede no que está aberto.
        self.commit_text(doc, history);
        let whole = self.selection.is_empty();
        let targets: Vec<&Element> = if whole {
            doc.live_in_z_order()
        } else {
            self.editable(doc)
        }
        .into_iter()
        .filter(|el| sketchable(el))
        .collect();
        let on = if targets.is_empty() {
            !board
        } else {
            !targets.iter().all(|el| el.style().sketch)
        };
        // A letra fixa que um `.excalidraw` trouxe volta ao modo com o botão (decisão do dono, W10).
        let ops = targets
            .into_iter()
            .filter(|el| el.style().sketch != on || el.style().look.letter != Letter::Auto)
            .map(|el| {
                let mut el = el.clone();
                let st = el.style_mut();
                st.sketch = on;
                st.look.letter = Letter::Auto;
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        whole.then(|| {
            self.style.sketch = on;
            on
        })
    }

    /// Troca a rota das setas seleccionadas (UM passo) e guarda-a para as próximas.
    pub fn set_route(&mut self, doc: &mut BoardDoc, history: &mut History, r: Route) {
        self.route = r;
        self.change_connectors(doc, history, |c| c.route = r);
    }

    /// Troca a ponta `which` (`0` início, `1` fim) das setas seleccionadas e guarda-a.
    pub fn set_head(&mut self, doc: &mut BoardDoc, history: &mut History, which: usize, h: Head) {
        self.heads[which] = h;
        self.change_connectors(doc, history, |c| c.heads[which] = h);
    }

    fn change_connectors(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        change: impl Fn(&mut ph2d_board_model::Connector),
    ) {
        let ops = self
            .editable(doc)
            .into_iter()
            .filter_map(|el| {
                let mut el = el.clone();
                change(el.connector_mut()?);
                Some(BoardOp::Put(el))
            })
            .collect();
        history.apply(doc, ops);
    }

    /// Troca o contorno das formas seleccionadas (UM passo).
    pub fn set_shape_type(&mut self, doc: &mut BoardDoc, history: &mut History, t: ShapeType) {
        let ops = self
            .editable(doc)
            .into_iter()
            .filter_map(|el| {
                let mut el = el.clone();
                el.shape_mut()?.kind = t;
                Some(BoardOp::Put(el))
            })
            .collect();
        history.apply(doc, ops);
    }

    /// Uma forma nova (com o estilo actual) na caixa dada, à frente de tudo — ou uma nota (com o
    /// estilo das notas).
    fn new_shape(&self, doc: &mut BoardDoc, kind: ShapeType, bx: [f64; 4]) -> Element {
        if kind.is_note() {
            return self.new_note(doc, kind, bx);
        }
        let shape = Shape {
            kind,
            style: self.style.clone(),
            text: Default::default(),
        };
        Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx)
    }
}

/// O botão «Rascunho ↔ Final» mexe neste elemento? As formas e as setas — as notas também, só na
/// letra (decisão do dono, 10/10: o papel fica liso, o desenho não as passa ao traço tremido).
#[must_use]
pub fn sketchable(el: &Element) -> bool {
    el.connector().is_some() || el.shape().is_some()
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
