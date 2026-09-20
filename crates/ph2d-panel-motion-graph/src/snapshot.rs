//! Graph view snapshot (Motion Nodes M1.E10) — the state-publish channel from
//! the shell bridge to this panel.
//!
//! The bridge builds a [`GraphViewSnapshot`] each frame from the shell-owned
//! `MotionDoc` + `NodeRegistry` (via [`snapshot_from`]) and hands it over the
//! [`set_current_motion_graph`] thread-local; `paint` reads it back with
//! [`current_snapshot`]. Neither side downcasts the other (mold:
//! `ph2d_panel_vector::set_current_vector_style`). The panel returns edits the
//! other way as `GraphIntent`s (M1.E10 phase 2; not yet wired).

use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint};
use ph2d_nodegraph::port::{Clock, Dim, Domain};
use std::cell::RefCell;

#[path = "snapshot_intent.rs"]
mod intent;
pub use intent::library_pick;
pub use intent::{GraphIntent, RenameTarget};

#[path = "snapshot_drop.rs"]
mod drop_targets;

#[path = "snapshot_menu.rs"]
mod menu;
pub use drop_targets::{
    ChoiceTarget, HiddenPorts, PortChoice, card_hidden_ports, set_card_hidden_ports,
};
pub(crate) use menu::menu_rows;
// The compatible-type filter for smart-connect: a loose end dropped in empty space opens the
// palette showing only the node types that output can feed (`interact_drop`).
pub(crate) use menu::{menu_catalog, menu_catalog_back};

/// **O ECO de uma largada** — irmão cortado no tecto de LOC do painel (600) e por
/// RESPONSABILIDADE: este ficheiro é o RETRATO que a shell publica todo quadro, aquele é um
/// canal lateral com vida própria (nasce de uma acção e morre sozinho).
#[path = "snapshot_piscada.rs"]
mod piscada;
pub use piscada::{Piscada, current_graph_flash, set_graph_flash};

/// One socket on a node card. **Colour ← [`Domain`], shape ← [`Dim`]** (plan §2.4) —
/// two orthogonal readouts: the hue says which data family, the glyph ([`socket_glyph`])
/// says a single value (○) vs a multi-component column (◇). `clock` completes the
/// [`ph2d_nodegraph::port::PortType`] axes so the editor's live wire-compatibility
/// preview matches `connects_directly` (domain + dim + clock) without touching the
/// graph; the membrane check stays server-side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortView {
    pub name: &'static str,
    pub domain: Domain,
    pub dim: Dim,
    pub clock: Clock,
}

/// The glyph a socket wears — its DIMENSIONALITY at a glance, orthogonal to the domain
/// colour. A single scalar (a "value" a parameter can read) is a circle; a
/// multi-component column (a position/colour/geometry stream) is a diamond. It is the
/// visible half of what `connects_directly` enforces on the `dim` axis: two sockets of
/// different SHAPE cannot join by a plain edge, so the artist sees the refusal before
/// dragging.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum SocketGlyph {
    /// A single scalar — the "value" the artist wires into a parameter (○).
    Value,
    /// A multi-component column — a "column" of per-element data (◇).
    Column,
}

/// The glyph for a port of dimensionality `dim`: `Scalar` is a value, everything else is
/// a column. A pure function of the REAL [`Dim`], enumerated (never a `_ =>`) so a new
/// axis forces a decision here instead of silently drawing the wrong shape — the same
/// discipline that keeps the polygon-sides gate honest.
#[must_use]
pub(crate) fn socket_glyph(dim: Dim) -> SocketGlyph {
    match dim {
        Dim::Scalar => SocketGlyph::Value,
        Dim::Vec2 | Dim::Vec3 | Dim::Vec4 | Dim::Mat2 | Dim::Mat3 | Dim::Mat4 => {
            SocketGlyph::Column
        }
    }
}

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod snapshot_tests;

/// **What a card in the view actually IS** (doc 57). The panel paints and hits all
/// three the same way — a card is a card — but they answer to different verbs, and
/// the difference is not cosmetic: a double-click ENTERS a subgraph, and a ghost
/// refuses to be dragged or deleted because it does not live at this level.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeViewKind {
    /// An ordinary node of the graph, at the level being viewed.
    Node,
    /// A **collapsed subgraph**: one card standing for every node inside it. Its
    /// `id` is a subgraph id tagged with [`SUBGRAPH_VIEW_TAG`] (the two id spaces
    /// are independent, so an untagged id would be ambiguous), and its ports are
    /// the edges that CROSS its boundary — derived, never declared (doc 57 §3).
    Subgraph,
    /// A **ghost**: a node from OUTSIDE this level that is wired across the
    /// boundary, drawn read-only where the wire enters. Houdini's *indirect input*
    /// ("a node-like item that appears inside subnets and corresponds to the node
    /// wired into the subnet") — with the real node's name on it, because we can
    /// afford to say which node it is.
    ///
    /// It is what keeps the inside of a group from LYING: without it, a member fed
    /// from outside would draw an empty input socket, which is the one thing a
    /// socket must never do.
    Ghost,
}

/// A view id with this bit set names a SUBGRAPH, not a node (doc 57 §4). Node ids
/// are minted from 0 upward by `Graph::next_id` and subgraph ids by their own
/// counter, so the two spaces overlap and a bare `u32` would be ambiguous the
/// moment a document has both a node 3 and a subgraph 3 — which is the common case,
/// not the corner one. The shell mints the tag and the shell decodes it; the panel
/// only ever asks *is this a card?*.
pub const SUBGRAPH_VIEW_TAG: u32 = 0x8000_0000; // LITERAL-COLOR-OK: an id tag bit, not a colour

/// Is this view id a collapsed subgraph card?
pub fn is_subgraph_view(id: u32) -> bool {
    id & SUBGRAPH_VIEW_TAG != 0
}

/// One step of the breadcrumb: the level it walks to (`None` = the root canvas)
/// and what to write on it. Built by the shell from the parent chain, root first —
/// Blender's *"breadcrumbs in the top left corner of the node editor"*, Houdini's
/// clickable path gadget.
#[derive(Clone, Debug, PartialEq)]
pub struct Crumb {
    pub level: Option<u32>,
    pub title: String,
}

/// One node card in the view.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeView {
    /// `NodeId.0` — the opaque handle carried through the `GraphSurface` channel.
    /// For a [`NodeViewKind::Subgraph`] card this is the subgraph id | [`SUBGRAPH_VIEW_TAG`].
    pub id: u32,
    /// Node, collapsed subgraph, or boundary ghost (see [`NodeViewKind`]).
    pub kind: NodeViewKind,
    /// English display label (registry UI metadata, else the type name).
    pub display_name: String,
    /// Header tint category + body silhouette (registry UI metadata).
    pub category: NodeUiCategory,
    pub silhouette: NodeSilhouette,
    /// Top-left in graph space.
    pub x: f32,
    pub y: f32,
    pub inputs: Vec<PortView>,
    pub outputs: Vec<PortView>,
    /// ⭐⭐ **A porta por onde um fio DEVE aterrar neste nó** — side-metadata do registo
    /// ([`ph2d_node_registry::NodeRegistry::primary_input`]), `0` para quem não declara.
    ///
    /// ⛔⛔ **Ela viaja no retrato porque o painel decide um aterramento e NÃO tem o registo**
    /// (o `node_body_target`, o fio largado sobre o CORPO de um cartão). Sem ela, o painel
    /// respondia *«a primeira entrada LIVRE e compatível»* — que num `motion.duplicator` é sempre
    /// a `shape`, porque as duas entradas têm o MESMO tipo e nem o tipo nem o `validate` as
    /// distinguem. *Um facto sobre o TIPO não se re-deriva do desenho: viaja com ele.*
    pub primary_input: u16,
    /// **The inline readout** (F2): what this node produced on THIS frame's cook — the
    /// number the artist would otherwise have to aim the probe at, on every card at once.
    ///
    /// **`None` means the node was never cooked**, which is not an error but the single
    /// most useful thing a node graph can tell you: *nothing downstream consumes this
    /// card*. A freshly dropped node, a chain the artist forgot to wire into the Output, a
    /// branch orphaned by the knife — all of them are blank, and the blankness is the
    /// diagnosis. (The shell fills this from the cook's MEMO, so it costs a lookup, never a
    /// second evaluation — see `Cook::peek`.)
    pub readout: Option<String>,
    /// **How many instances this node emitted** this frame (`None` = never cooked). The wires
    /// leaving it are drawn thicker the heavier the stream (F3): a scatter that fans 12 points
    /// into 5 000 says so in the WIDTH of its wire, with no number to read.
    pub count: Option<u32>,
    /// **Its output CHANGED since last frame** — data is flowing down its wires right now, and
    /// they march (F3). TouchDesigner's reading: *"when you see the wires between nodes
    /// animating, it means the upstream node is cooking"*. A constant branch is wired, alive,
    /// and simply still — and it draws still.
    pub hot: bool,
    /// This node is a **sink** (`motion.output`). The graph is PULLED from the sinks, so this
    /// is where "does anything consume me?" gets answered from (F3, [`crate::flow::live_set`]).
    pub is_sink: bool,
    /// **The postage stamp** (F3): a bounded SUBSAMPLE of the positions this node emitted, in
    /// world units — drawn as a little scatter on the card. Nuke's reading: the thumbnails
    /// *"show what each node passes onto the next node in the tree"*, which is the question a
    /// wire cannot answer no matter how well it is drawn (*where does the spiral become a
    /// grid?*).
    ///
    /// `None` for a node with no positions (a VALUE node's stamp is its number) and for a node
    /// the cook never pulled. The shell caps the point count, so the cost is bounded by the
    /// number of CARDS, never by the size of the stream — which is why this can be on by
    /// default, where Nuke has to tell you to switch its thumbnails off on a heavy script.
    pub preview: Option<Vec<[f32; 2]>>,
    /// **Switched OFF** (bypass/mute — H). The node's op did not run this frame; its primary
    /// input passed straight through ([`ph2d_nodegraph::Graph::node_bypassed`]). The card draws
    /// dimmed with a strike, so *"this node is inert on purpose"* reads at a glance instead of
    /// looking like a chain the artist forgot to wire.
    pub bypassed: bool,
    /// **Semantically INERT — and wrong, not just idle** (ADR-0155). This node produces a
    /// transient column (a force's `accel`) that reaches the Output but no node on the way
    /// consumes it, so the scene stays still with no error. The card grows a ⚠ badge whose
    /// click asks the shell to fix it (an integrator it forgot) or to explain (a choice only
    /// the artist can make). The shell fills this from `ph2d_motion_diagnose`, exactly as it
    /// fills [`Self::is_sink`]; the panel only paints and forwards the click — it has the
    /// snapshot, not the graph. This is a different reading from [`Self::bypassed`] (off on
    /// purpose) and from a blank [`Self::readout`] (reaches no sink at all): here the wiring
    /// LOOKS complete and simply does nothing.
    pub inert: bool,
    /// **The baked-tile THUMBNAIL** (doc 86 A5): a mini-render of the OBJECT this node is a
    /// source of, shown in the moldura in place of the point scatter. `Some` only when the
    /// node's stream carries a **uniform, non-zero `texture_id`** (a `source.object` / a
    /// duplicator of one shape) — the shell fills it from the object bake (§A2/A3) by that id.
    /// A positional-only node (no object) keeps `None` and draws its scatter: the two answer
    /// different questions — *what* an instance draws vs *where* the copies go.
    pub thumbnail: Option<PreviewThumb>,
    /// ⭐⭐⭐ **OS PARAMS DESENHADOS NO CARTÃO** (ciclo 1 — [doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md);
    /// decisão do Enio, 2026-09-05: *«como no Blender, os parâmetros dos nós devem ser
    /// desenhados nos nós e vamos retirar o painel lateral»*).
    ///
    /// ⚠️ **É [`CardParam`] e NÃO o `ParamRow` da ponte, e a razão é MEDIDA**
    /// (doc 103 §7): uma row do painel carrega um `String` por rótulo e outro por valor, e
    /// 20 cartões × 5 rows seriam **200 alocações por quadro**. Aqui tudo o que vem do
    /// registry é `&'static` e o valor é um `f32` — **zero alocação por row** —, e quem
    /// formata o número é o PINTOR, só para os cartões que de facto desenha.
    ///
    /// ⚠️ **Quem decide QUAIS params entram é a shell**, pela porta única
    /// `motion_bridge_params::params_visible::shown_params` — a mesma que o painel usa. Uma
    /// segunda conjunção de gates aqui seria exactamente como um param passa a aparecer num
    /// sítio do app e não noutro.
    pub params: Vec<CardParam>,
    /// ⭐⭐ **AS SECÇÕES DA FAIXA** — os grupos de params que o registry declara
    /// (`register_param_groups`), com o estado de dobra que a shell resolve.
    ///
    /// ⚠️ **[`Self::params`] já vem FILTRADA**: uma secção fechada não deixa as suas rows na
    /// lista. É o que mantém [`crate::geom::card_h`] uma função pura do snapshot — a dobra é
    /// estado de EDITOR e vive na shell, ao lado de tudo o resto que a vista resolve.
    pub sections: Vec<CardSection>,
}

#[path = "snapshot_card.rs"]
mod card;
pub use card::{CardParam, CardSection, RowText};

#[path = "snapshot_thumb.rs"]
mod thumb;
pub use thumb::PreviewThumb;

/// One wire in the view. Its color is the source port's [`Domain`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEdgeView {
    pub from_node: u32,
    pub from_port: u16,
    pub to_node: u32,
    pub to_port: u16,
    pub delayed: bool,
    pub out_domain: Domain,
}

/// One backdrop (group region) in the view — the document's `Backdrop`, resolved
/// into the panel's own vocabulary (the panel never sees `ph2d-motion-doc`). Pure
/// decoration: it is drawn behind the wires and cards and never cooks.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphBackdropView {
    /// Stable document id (drives selection + every backdrop intent).
    pub id: u32,
    /// Top-left in graph space.
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Tint index into the `graph-backdrop-1..8` tokens (clamped at paint).
    pub color: u8,
    pub title: String,
}

/// The probe's live reading of one node's output (F2). Published by the shell each
/// cook: `samples` is a ring of the most recent values, oldest first, so the panel
/// can draw a sparkline without keeping any history of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct ProbeView {
    /// The probed node.
    pub node: u32,
    /// What the number MEANS — a value stream reads out its scalar, an instance
    /// stream reads out how many instances it carries. English (app UI).
    pub label: String,
    /// The latest reading (also the last element of `samples`), or `None` when
    /// this frame has **no reading to give** — today: the cook ran GPU-resident,
    /// which does not feed the CPU memo the probe reads.
    ///
    /// It is an `Option` and not a sentinel number because the one thing a
    /// readout may never do is present a wrong number as a right one (the
    /// ADR-0125 rule). The card draws `—`; the artist sees "no reading here",
    /// which is the truth, instead of a stale or invented value.
    pub value: Option<f32>,
    /// The recent readings, oldest first (up to `PROBE_SAMPLES`).
    pub samples: Vec<f32>,
}

/// How many ticks of history the probe's sparkline shows (~1s at 60 Hz).
pub const PROBE_SAMPLES: usize = 60;

/// The whole graph, resolved to primitives the panel can paint without touching
/// the registry or the graph directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphViewSnapshot {
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdgeView>,
    /// The probe's live reading, when one is armed on a node (F2).
    pub probe: Option<ProbeView>,
    /// Group regions, painted behind everything. Filled by the shell bridge from
    /// `MotionDoc::backdrops` (not by [`snapshot_from`], which only sees the
    /// graph — the backdrops live on the document, not in the cook).
    pub backdrops: Vec<GraphBackdropView>,
    /// **The level this view is showing** (doc 57): `None` = the root canvas, else
    /// the subgraph that was entered. The panel does not own it — the SHELL does
    /// (like the probe), so that an undo which deletes the subgraph you are
    /// standing in can put you back on solid ground.
    pub level: Option<u32>,
    /// The path from the root to [`Self::level`], root first — one clickable crumb
    /// per step. Always at least one entry (the root).
    pub breadcrumb: Vec<Crumb>,
    /// The playhead, in seconds — the ONE clock the marching dashes read (F3). The panel has
    /// no clock of its own and must not grow one: a flow animation driven by a paint counter
    /// would keep marching on a paused graph, which is precisely the lie the dashes exist to
    /// not tell.
    pub now: f32,
}

/// One addable node type in the add-node menu (M1.E7). Copy — the canonical
/// `type_name` (fed straight back as [`GraphIntent::AddNode`]) and the English
/// `display` label are both `&'static` (the registry's manifest / UI metadata);
/// `category` tints the row's dot so the palette teaches the library map.
#[derive(Copy, Clone, Debug)]
pub struct NodeChoice {
    pub type_name: &'static str,
    pub display: &'static str,
    pub category: NodeUiCategory,
    /// The node type's INPUT ports (straight off its `NodeManifest`, hence
    /// `&'static`). Carried so the panel can answer "what could this wire feed?"
    /// on its own — the smart-connect popup filters the catalog by it, and the
    /// panel must not have to ask the shell a question the manifest already
    /// answers.
    pub inputs: &'static [ph2d_nodegraph::node::PortSpec],
    /// ⭐⭐ **As portas de SAÍDA do tipo** — o espelho dos [`Self::inputs`], e ele existe pela
    /// mesma razão: a paleta aberta por um fio puxado de uma ENTRADA (ordem do dono, 2026-09-19)
    /// tem de mostrar *«quem pode ALIMENTAR isto»*, e essa pergunta lê-se nas saídas. *O painel
    /// não pode ter de perguntar à shell uma coisa que o manifesto já responde.*
    pub outputs: &'static [ph2d_nodegraph::node::PortSpec],
}

/// Two choices are the same choice when they name the same node TYPE — the
/// canonical name is the identity (`PortSpec` itself carries no `PartialEq`, and
/// comparing port lists would say nothing the type name does not).
impl PartialEq for NodeChoice {
    fn eq(&self, other: &Self) -> bool {
        self.type_name == other.type_name
    }
}
impl Eq for NodeChoice {}

thread_local! {
    static CURRENT: RefCell<Option<GraphViewSnapshot>> = const { RefCell::new(None) };
    static INTENTS: RefCell<Vec<GraphIntent>> = const { RefCell::new(Vec::new()) };
    static CATALOG: RefCell<Vec<NodeChoice>> = const { RefCell::new(Vec::new()) };
    static SELECTION: RefCell<Vec<u32>> = const { RefCell::new(Vec::new()) };
    /// **O param que a mão está a ARRASTAR agora** — `(nó, nome)`.
    static PARAM_SCRUB: RefCell<Option<(u32, &'static str)>> = const { RefCell::new(None) };
    static BACKDROP_SELECTION: RefCell<Option<u32>> = const { RefCell::new(None) };
    static SELECTION_REQUEST: RefCell<Option<Vec<u32>>> = const { RefCell::new(None) };
    /// **The node-help system on/off** (ADR-0155): a shell→panel scalar the toolbar
    /// chip reads to draw its state and to flip. It lives HERE, not on
    /// [`GraphViewSnapshot`], because it is editor UX state (like the selection), not
    /// resolved graph geometry — and the shell owns it (the diagnoser rides it). ON
    /// until the shell says otherwise, so a paint before the first frame shows help on.
    static NODE_HELP: RefCell<bool> = const { RefCell::new(true) };
    /// ⭐⭐ **O TEXTO INTEIRO de cada param de texto** — `(nó, param, valor)`, publicado pela
    /// shell ao lado do snapshot.
    ///
    /// ⛔⛔ **Existe porque o [`CardParam`] guarda o texto TRUNCADO** ([`RowText`], o que cabe na
    /// row) e semear uma caixa de edição com ele **destruiria o resto da string** no `Enter` —
    /// que é, uma letra acima, a armadilha que o `param_edit` já nomeia para o número
    /// (*«a row mostra `0.50` e comitar essa leitura destruiria um `0.503`»*).
    ///
    /// ⚠️ **E é um canal lateral, não um campo:** o [`GraphNodeView`] é construído em **31
    /// sítios** desta árvore, e apender-lhe um campo é a forma que a memória do repo nomeia
    /// (*«tipo construído em N sítios prefere componente opcional a campo apendado»*). Aqui o
    /// molde já existe — é o mesmo dos outros seis publicadores acima.
    static CARD_TEXTS: RefCell<Vec<(u32, &'static str, String)>> =
        const { RefCell::new(Vec::new()) };

    /// ⭐⭐⭐ **AS OPÇÕES DE CADA SELECTOR** — o que a LISTA de um enum, de uma fonte publicada
    /// ou de um canal oferece, e em qual delas o nó está (report do Enio, 2026-09-07: *«se
    /// clicar no centro (nome) abre-se um dropdown»*).
    ///
    /// ⚠️ **Um canal lateral, e não um campo do [`CardParam`], pela mesma razão do
    /// [`CARD_TEXTS`]** — mas aqui há uma segunda: uma lista de nomes por row é exactamente a
    /// alocação por quadro que a medição do doc 103 recusou. Daí a [`CardChoices`] ter duas
    /// caras: um enum do registry viaja como `&'static` (**zero** alocação, e são 138 das 143
    /// rows de selector do catálogo), e só as listas VIVAS — o que o artista desenhou, o que a
    /// corrente de cima cozinhou — pagam `String`s, que são meia dúzia.
    static CARD_CHOICES: RefCell<Vec<(u32, &'static str, CardChoices, u16)>> =
        const { RefCell::new(Vec::new()) };
}

/// **O que a lista de um selector oferece** — em duas caras, porque as duas fontes têm custos
/// diferentes e a diferença é medida (ver [`CARD_CHOICES`]).
#[derive(Clone, Debug, PartialEq)]
pub enum CardChoices {
    /// Rótulos que vivem no binário — os `labels` de um `ParamWidget::Enum`.
    Static(&'static [&'static str]),
    /// Rótulos VIVOS — as formas que o artista desenhou, as colunas que a corrente cozinhou.
    Live(Vec<String>),
}

impl CardChoices {
    /// Os rótulos, uma cópia só — chamada **ao ABRIR** a lista, nunca por quadro.
    fn labels(&self) -> Vec<String> {
        match self {
            // ⚠️ **A tradução das opções acontece AQUI e em mais sítio nenhum desta lista** — e é de
            // graça, porque este método corre ao ABRIR e não por quadro (ver o doc acima).
            Self::Static(l) => l.iter().map(|s| ph2d_i18n::tr(s).to_string()).collect(),
            Self::Live(l) => l.clone(),
        }
    }
}

/// Publica as opções de cada selector de cada cartão (shell → painel). Ver [`CARD_CHOICES`].
pub fn set_card_choices(choices: Vec<(u32, &'static str, CardChoices, u16)>) {
    CARD_CHOICES.with(|c| *c.borrow_mut() = choices);
}

/// As opções de `(nó, param)` e o índice do que está escolhido — `None` quando a shell não as
/// publicou (o param não é um selector, ou o cartão não está na vista).
///
/// ⚠️ **O índice pode ser `>= labels.len()`, e isso é a resposta certa:** significa *nenhuma
/// das opções* — uma coluna escrita à mão, uma forma que ainda não foi desenhada. A lista abre
/// sem nada marcado, que é honesto; inventar uma marca diria que o nó está numa opção em que
/// ele não está.
pub(crate) fn card_choices_of(node: u32, param: &str) -> Option<(Vec<String>, u16)> {
    CARD_CHOICES.with(|c| {
        c.borrow()
            .iter()
            .find(|(n, p, _, _)| *n == node && *p == param)
            .map(|(_, _, ch, cur)| (ch.labels(), *cur))
    })
}

/// Publish whether the node-help system is on (shell bridge → panel, ADR-0155). Set
/// every frame from `MotionState::node_help_enabled` so the toolbar chip draws the
/// live state; the panel reads it with [`node_help`] to draw the chip and to compute
/// the toggle it requests.
/// Publica o texto INTEIRO dos params de texto de cada cartão (shell → painel). Ver
/// [`CARD_TEXTS`]: o cartão desenha o truncado e a caixa de edição abre com este.
pub fn set_card_texts(texts: Vec<(u32, &'static str, String)>) {
    CARD_TEXTS.with(|c| *c.borrow_mut() = texts);
}

/// O texto inteiro de `(nó, param)` — `None` quando a shell não o publicou.
pub(crate) fn card_text_of(node: u32, param: &str) -> Option<String> {
    CARD_TEXTS.with(|c| {
        c.borrow()
            .iter()
            .find(|(n, p, _)| *n == node && *p == param)
            .map(|(_, _, v)| v.clone())
    })
}

pub fn set_node_help(on: bool) {
    NODE_HELP.with(|c| *c.borrow_mut() = on);
}

/// Read whether the node-help system is on (panel). Drives the chip's active ring and
/// the `SetNodeHelp(!node_help())` it emits — non-destructive, like the selection read.
pub fn node_help() -> bool {
    NODE_HELP.with(|c| *c.borrow())
}

/// Publish the current node selection (panel `paint` → shell bridge, M1.P1). The
/// bridge reads it to build the selected node's params snapshot for the params
/// panel. Written every frame from the ephemeral `state.selected`.
pub fn set_graph_selection(selection: Vec<u32>) {
    SELECTION.with(|c| *c.borrow_mut() = selection);
}

/// Read the published node selection (shell bridge). Empty when nothing is
/// selected or the Motion tool is inactive.
pub fn current_graph_selection() -> Vec<u32> {
    SELECTION.with(|c| c.borrow().clone())
}

/// ⭐⭐ **Publica qual param está a ser ARRASTADO** — `(nó, nome)`, ou `None` fora do gesto.
///
/// Ordem do dono (2026-09-19): *«permita visualizar o ponto do pivot ao arrastar os parâmetros de
/// pivot»*. Um gizmo de CANVAS que acende enquanto a mão mexe num knob do CARTÃO precisa de
/// atravessar a fronteira painel→shell, e este é o mesmo canal da selecção — escrito todo quadro,
/// lido por quem quiser, sem ninguém chamar ninguém (ADR-0075).
///
/// ⚠️ **É o NOME e não a `row`**: a linha é uma coordenada do pintor, e resolvê-la do outro lado
/// seria a segunda cópia do `band_at`.
pub fn set_graph_param_scrub(v: Option<(u32, &'static str)>) {
    PARAM_SCRUB.with(|c| *c.borrow_mut() = v);
}

/// Lê o param em arrasto (shell). `None` quando a mão não está num knob.
#[must_use]
pub fn current_graph_param_scrub() -> Option<(u32, &'static str)> {
    PARAM_SCRUB.with(|c| *c.borrow())
}

/// Hand the panel a NEW selection (shell bridge → panel). The one channel that
/// runs against the usual direction, and it has to: only the shell knows the ids
/// it just minted (Ctrl+D's duplicates), and the copies must end up selected — or
/// the drag that naturally follows would move the ORIGINALS. Drained by the panel
/// on its next `process`.
pub fn request_graph_selection(nodes: Vec<u32>) {
    SELECTION_REQUEST.with(|c| *c.borrow_mut() = Some(nodes));
}

/// Take the pending selection request, if any (panel).
pub(crate) fn take_selection_request() -> Option<Vec<u32>> {
    SELECTION_REQUEST.with(|c| c.borrow_mut().take())
}

/// Look at the pending request WITHOUT consuming it — how a seam gate on the shell
/// side checks what an edit handed back. Deliberately non-destructive: a reader that
/// took it would steal the panel's selection and the bug would surface as "sometimes
/// the copies aren't selected", which is exactly the class of bug this channel exists
/// to prevent.
pub fn pending_graph_selection() -> Option<Vec<u32>> {
    SELECTION_REQUEST.with(|c| c.borrow().clone())
}

/// Publish the selected BACKDROP (panel `paint` → shell bridge). Mutually
/// exclusive with the node selection by construction — selecting a backdrop
/// clears the nodes and vice-versa — so the params panel always has exactly one
/// subject to show properties for.
pub fn set_graph_backdrop_selection(selected: Option<u32>) {
    BACKDROP_SELECTION.with(|c| *c.borrow_mut() = selected);
}

/// Read the selected backdrop (shell bridge). `None` when a node (or nothing) is
/// selected, or the Motion tool is inactive.
pub fn current_graph_backdrop_selection() -> Option<u32> {
    BACKDROP_SELECTION.with(|c| *c.borrow())
}

/// Publish the addable-node catalog (shell bridge → panel). Set once on tool
/// activation (the registry is fixed at boot) and cleared to empty on deactivate.
pub fn set_current_node_catalog(catalog: Vec<NodeChoice>) {
    CATALOG.with(|c| *c.borrow_mut() = catalog);
}

/// Read the published catalog (panel, only while the add-node menu is open, so no
/// per-frame clone in the common path).
pub(crate) fn current_catalog() -> Vec<NodeChoice> {
    CATALOG.with(|c| c.borrow().clone())
}

/// Whether two ports could be wired together — the panel-side rule (domain + dim + clock,
/// `connects_directly` minus the membrane; the shell still has the last word).
pub(crate) fn same_type(a: &PortView, b: &PortView) -> bool {
    a.domain == b.domain && a.dim == b.dim && a.clock == b.clock
}

/// Queue an edit for the shell bridge to apply (panel → shell).
///
/// **Public so the shell's seam tests can drive the REAL intent path** — the same
/// queue the panel writes and `drain_intents` reads. A test that called the shell's
/// apply functions directly would prove the apply and skip the wiring, which is the
/// half that breaks (DIRETIVA_IMPLEMENTACAO §2: a missing end of the seam is a
/// dropped click, not a compile error).
pub fn push_intent(intent: GraphIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Drain the queued edits (shell bridge, each frame). Capacity-retaining.
pub fn drain_intents() -> Vec<GraphIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Publish the current graph snapshot (shell bridge → panel). `None` while the
/// Motion tool is inactive.
pub fn set_current_motion_graph(snapshot: Option<GraphViewSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// Read the published snapshot (panel `paint`). Empty when nothing is published.
pub(crate) fn current_snapshot() -> GraphViewSnapshot {
    CURRENT.with(|c| c.borrow().clone().unwrap_or_default())
}

// The view-snapshot BUILDER (data → paint-ready view) lives in a sibling for the panel LOC
// cap; re-exported so `crate::snapshot::snapshot_from` (and the lib.rs re-export) still resolve.
#[path = "snapshot_build.rs"]
mod build;
pub use build::snapshot_from;

#[cfg(test)]
#[path = "snapshot_choices_tests.rs"]
mod testes_das_escolhas;
