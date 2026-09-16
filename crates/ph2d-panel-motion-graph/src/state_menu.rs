//! ⭐⭐ **O POPUP DO PAINEL DO GRAFO** — o que ele é uma lista DE, e os verbos que uma linha dele
//! dispara.
//!
//! ⚠️ **Irmão de [`super`] por RESPONSABILIDADE:** lá vive o estado da interacção (a vista, a
//! selecção, o arrasto em curso); aqui a lista flutuante — as suas quatro espécies e as duas
//! tabelas de acções que ela desenha. As duas mudam por razões diferentes.

use crate::snapshot::PortChoice;
use ph2d_editor_core::interaction::GraphKey;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// **A row of the node context menu** (right-click a node or a group card, doc 62). The menu is
/// an alternate TRIGGER, never a second implementation: each row routes back through the SAME door
/// its gesture uses — a keyboard verb via [`Self::graph_key`] + `apply_key`, or, for [`Self::Enter`]
/// (whose `graph_key` is `None`), the double-click's `subgraph_gesture::enter`. So the menu cannot
/// drift from the shortcut, and the labels name it, so the menu also TEACHES it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeAction {
    Enter,
    Cut,
    Copy,
    Duplicate,
    Delete,
    Bypass,
    Rename,
    Ungroup,
}

impl NodeAction {
    /// Every action, in display order — Enter first (walk in), the shared edits, Ungroup last
    /// (dissolve). The two `group`-only rows top and tail the list so a plain node's menu (which
    /// shows neither) is the exact list, in the exact order, it always was.
    const ALL: [NodeAction; 8] = [
        NodeAction::Enter,
        NodeAction::Cut,
        NodeAction::Copy,
        NodeAction::Duplicate,
        NodeAction::Delete,
        NodeAction::Bypass,
        NodeAction::Rename,
        NodeAction::Ungroup,
    ];
    /// **Whether this action needs a SINGLE subject.** Rename asks for one name; with many
    /// nodes selected there is no single name to ask about, so its verb is inert and the row
    /// must not appear (Enio, smoke: *"para múltiplos nós opções como rename não podem
    /// aparecer"*). The rest act on any non-empty selection.
    fn requires_single(self) -> bool {
        matches!(self, NodeAction::Rename)
    }
    /// **Whether this action only means something on a GROUP card.** Enter walks INTO a subgraph
    /// and Ungroup dissolves one — neither is meaningful for a plain node, so the rows appear
    /// only when the single subject is a collapsed card. (A card selection is exactly one card,
    /// so `group` implies `!multi`.)
    fn requires_group(self) -> bool {
        matches!(self, NodeAction::Enter | NodeAction::Ungroup)
    }
    /// **The rows to show for this selection** — the ONE list `menu_rows` draws AND `resolve_menu`
    /// dispatches, so row `i` means the same thing on screen and under the cursor. `group`-only
    /// rows (Enter/Ungroup) appear only over a card; single-subject rows (Rename) drop out when
    /// the selection is `multi` (> 1).
    pub(crate) fn visible(multi: bool, group: bool) -> Vec<NodeAction> {
        NodeAction::ALL
            .iter()
            .copied()
            .filter(|a| {
                if a.requires_group() && !group {
                    return false;
                }
                !(multi && a.requires_single())
            })
            .collect()
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            NodeAction::Enter => tr("panel.motion_graph.menu.enter_double_click"),
            NodeAction::Cut => tr("panel.motion_graph.menu.cut_ctrl_x"),
            NodeAction::Copy => tr("panel.motion_graph.menu.copy_ctrl_c"),
            NodeAction::Duplicate => tr("panel.motion_graph.menu.duplicate_ctrl_d"),
            NodeAction::Delete => tr("panel.motion_graph.menu.delete_del"),
            NodeAction::Bypass => tr("panel.motion_graph.menu.toggle_mute_h"),
            NodeAction::Rename => tr("panel.motion_graph.menu.rename_f2"),
            NodeAction::Ungroup => tr("panel.motion_graph.menu.ungroup_ctrl_alt_g"),
        }
    }
    /// The keyboard verb this row runs, or `None` for [`NodeAction::Enter`] — a NAVIGATION
    /// (double-click), not an `apply_key` verb, which the resolve routes through its own door.
    pub(crate) fn graph_key(self) -> Option<GraphKey> {
        Some(match self {
            NodeAction::Cut => GraphKey::Cut,
            NodeAction::Copy => GraphKey::Copy,
            NodeAction::Duplicate => GraphKey::Duplicate,
            NodeAction::Delete => GraphKey::Delete,
            NodeAction::Bypass => GraphKey::Bypass,
            NodeAction::Rename => GraphKey::Rename,
            NodeAction::Ungroup => GraphKey::Ungroup,
            NodeAction::Enter => return None,
        })
    }
}

/// **The backdrop menu's action rows, AFTER the tint swatches** (doc 62) — the two things a
/// backdrop has besides its colour. Each routes through the SAME `apply_key` its shortcut does
/// (`rename::arm` for F2, the backdrop arm of Delete for Del), acting on the backdrop the
/// right-press left SELECTED — so the menu can never drift from the key. This is the ONE list
/// `menu_rows` appends and `resolve_menu` dispatches (past the tints), so a row means the same
/// thing on screen and under the cursor.
pub(crate) const BACKDROP_ACTIONS: [(TextKey, GraphKey); 2] = [
    (
        TextKey::new("panel.motion_graph.menu.rename_f2"),
        GraphKey::Rename,
    ),
    (
        TextKey::new("panel.motion_graph.menu.delete_del"),
        GraphKey::Delete,
    ),
];

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Menu {
    /// How far the list is scrolled, in px. The library has 86 node types and a popup is not
    /// allowed to run off the screen (Enio), so the panel is capped and the list scrolls inside it.
    pub scroll: f32,
    /// Top-left of the popup panel, screen space (the R-click point, clamped at
    /// paint so the list stays on-canvas).
    pub screen: (f32, f32),
    /// Graph-space point the chosen node lands at (the R-click point mapped
    /// through the view — stable under a later pan/zoom while the menu is open).
    pub spawn: (f32, f32),
    pub body: MenuBody,
}

/// What the popup is a list OF.
///
/// An enum rather than two optional fields: the two bodies are alternatives, and a
/// struct that can hold both can hold neither meaningfully — the popup would have to
/// guess which question it was asking.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MenuBody {
    // The node library moved to the shell's full-screen palette (the `OpenLibrary` intent). Plain
    // `A` / R-click, a wire-splice, and a loose-end smart-connect all open THAT now; this enum keeps
    // only the small, read-by-eye local popups.
    /// **The eight tints a backdrop can take** (doc 62) — R-click on a backdrop's header.
    ///
    /// A backdrop's colour is the only thing about it the artist could not change: the tint
    /// cycled by id at birth and stayed there for good. The intent to set it EXISTED and was
    /// handled by the shell; nothing emitted it — the same dead half-gesture the rename found
    /// (doc 61 §2), one shelf down.
    BackdropTints {
        /// The backdrop being re-tinted.
        backdrop: u32,
        /// Its tint right now, so the palette can show you which one you are on.
        current: u8,
    },
    /// **The ports hidden inside a collapsed card** (doc 57 §5) — a wire was dropped on
    /// a card's BODY, and the card exposes no socket for it.
    ///
    /// A card's sockets ARE the wires that already cross its boundary (derived, never
    /// declared), so a *new* wire into a group has nowhere to land: the socket it wants
    /// does not exist yet, and cannot, until the wire exists. Something has to name the
    /// port inside — and only the artist knows which of the group's free ports they
    /// meant. So the drop asks. On the pick, the wire connects to the REAL port inside,
    /// and the card grows the socket **by derivation**, from the edge that now crosses it.
    ///
    /// `rows` is captured at DROP time, not re-derived per frame: the list must not shift
    /// under a cursor that is already travelling toward a row.
    CardPorts {
        rows: Vec<PortChoice>,
        /// The end already in hand — the outside node's port the wire was drawn from
        /// (`forward`) or hunting for (`!forward`).
        other: (u32, u16),
        /// The wire was drawn FORWARD, out of an output: it needs an INPUT inside, and
        /// the connection reads `other -> row`. Backwards, it reads `row -> other`.
        forward: bool,
        /// The input this wire's end was pulled OFF, when the gesture was a wire being
        /// MOVED rather than drawn (doc 45). The pick then moves it — one undo step,
        /// unplug and re-plug — instead of making a second copy of the same wire.
        ///
        /// Until the artist picks, the wire is still drawn where it was: it has not moved,
        /// and a wire drawn nowhere while a menu is open would be a wire the editor
        /// dropped on the floor.
        detach: Option<(u32, u16)>,
    },
    /// **The actions on a node** (right-click a NODE, doc 62) — the missing case of the
    /// context-dependent right-press (backdrop → tints, wire → splice, node → actions). The
    /// target is whatever `open_on_right_press` left SELECTED (the right-clicked node, or the
    /// whole selection if it was part of one), and each pick runs the matching keyboard verb
    /// via `apply_key`, which reads that selection. `multi` (> 1 selected, captured at open)
    /// drops the single-subject rows (Rename); `group` (the single subject is a collapsed card)
    /// adds the group-only rows (Enter / Ungroup) — [`NodeAction::visible`].
    NodeActions { multi: bool, group: bool },
    /// ⭐⭐⭐ **AS OPÇÕES DE UM SELECTOR DO CARTÃO** (report do Enio, 2026-09-07: *«se clicar no
    /// centro (nome) abre-se um dropdown»*, com a foto do selector do Blender).
    ///
    /// ⚠️ **Reusa o popup que já existe** em vez de nascer um segundo: a lista, o recorte ao
    /// canvas, a rolagem, a barra e o hit-test são os mesmos das outras três — um popup novo
    /// seria a segunda resposta a *«como se mostra uma lista neste painel?»*.
    ///
    /// `labels` é capturado ao ABRIR, não re-derivado por quadro: a lista de um canal inclui o
    /// que a corrente de cima cozinhou, e ela não pode mudar debaixo de um cursor que já viaja
    /// para uma linha (a mesma razão que o [`Self::CardPorts`] já documenta).
    ParamOptions {
        node: u32,
        param: &'static str,
        /// O rótulo do param — o cabeçalho diz QUE pergunta a lista está a fazer.
        title: &'static str,
        labels: Vec<String>,
        /// Em qual opção o nó está. ⚠️ `>= labels.len()` = **nenhuma**, e é uma resposta certa
        /// (uma coluna escrita à mão, uma forma ainda não desenhada).
        current: u16,
        /// **Como a escolha se escreve** — a MESMA porta que classifica o clique na row
        /// ([`crate::ClickDoes`]), nunca uma segunda lista de espécies aqui: o valor de um enum
        /// **é** o índice (escrita directa), e uma fonte ou um canal precisam da shell, que
        /// possui a lista e sabe o que cada índice significa.
        kind: crate::ClickDoes,
    },
}
