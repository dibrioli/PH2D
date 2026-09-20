//! **What the popup lists** (Motion Nodes doc 54 + 59) — the ONE row source, read by the
//! paint, by the hit-test and by the panel's geometry. Sibling of `snapshot` (panel LOC cap).
//!
//! They used to disagree: the paint drew `current_catalog()` (all 86 types) while the click
//! resolved against the *filtered* smart-connect list, so on a wire-drop the artist read one
//! row and pressed another. Two derivations of the same list is the bug; one is the fix — and
//! the search (doc 59) makes it matter more, because now the list is re-ordered too.

use super::{GraphViewSnapshot, NodeChoice, PortView, current_catalog};
use crate::state::{Menu, MenuBody};
use ph2d_i18n::TextKey;
use ph2d_tokens::ColorToken;

/// **One row of the popup** — a label, a tinted dot, and (for the library) the node type
/// a pick would create.
///
/// The popup has ONE row source, and `paint`, the hit-test and the panel's GEOMETRY all
/// read it. They used to disagree: the paint drew `current_catalog()` (all 86 types) while
/// the click resolved against the *filtered* smart-connect list, so on a wire-drop the
/// artist clicked "Attractor" and got whichever type sat at that index in a list they were
/// never shown. Two derivations of the same list is the bug; one is the fix.
pub(crate) struct MenuRow<'a> {
    pub label: &'a str,
    /// The dot beside the label. A row does not care what its colour MEANS — for a node it is
    /// the category, for a tint it is the tint itself — so it carries the token, not the reason.
    pub dot: ColorToken,
    /// Marked as the one you are already on (the backdrop's current tint). `false` everywhere
    /// else: a list of things to ADD has no current entry.
    pub selected: bool,
}

/// **The eight backdrop tints, named for what they are** (doc 62). They are an OKLCH hue wheel
/// (`graph-backdrop-1..8`: hues 20, 60, 110, 150, 200, 250, 300, and a near-neutral at 320), so
/// the names are read off the hues rather than invented — "Colour 5" is not a name anybody can
/// use. English (app UI, HR-15).
pub(crate) const TINT_NAMES: [TextKey; 8] = [
    TextKey::new("panel.motion_graph.menu.red"),
    TextKey::new("panel.motion_graph.menu.amber"),
    TextKey::new("panel.motion_graph.menu.lime"),
    TextKey::new("panel.motion_graph.menu.green"),
    TextKey::new("panel.motion_graph.menu.teal"),
    TextKey::new("panel.motion_graph.menu.blue"),
    TextKey::new("panel.motion_graph.menu.violet"),
    TextKey::new("panel.motion_graph.menu.grey"),
];

/// The rows the popup shows. The node LIBRARY moved to the shell's full-screen palette; the local
/// popups left here are read by eye:
///
/// **CardPorts:** the rows were already chosen and filtered at drop time (they name ports the panel
/// cannot see from here); it just reads them back. **NodeActions / BackdropTints:** one row per verb
/// / tint. None reads the snapshot (only the removed library filter did), so `_snap` is unused now,
/// kept so callers that already hold the snapshot need not change.
pub(crate) fn menu_rows<'a>(_snap: &GraphViewSnapshot, menu: &'a Menu) -> Vec<MenuRow<'a>> {
    match &menu.body {
        MenuBody::CardPorts { rows, .. } => rows
            .iter()
            .map(|p| MenuRow {
                label: &p.label,
                dot: crate::paint::cat_token(p.category),
                selected: false,
            })
            .collect(),
        // The node context menu (doc 62): one row per applicable keyboard verb, in
        // `NodeAction` order. A neutral dot — a verb has no category to tint, and the label
        // carries the shortcut, which is the row's whole job. `multi` drops the single-subject
        // rows (Rename); `group` adds the group-only rows (Enter / Ungroup) over a card.
        MenuBody::NodeActions { multi, group } => crate::state::NodeAction::visible(*multi, *group)
            .into_iter()
            .map(|a| MenuRow {
                label: a.label(),
                dot: ColorToken::Text2,
                selected: false,
            })
            .collect(),
        // The palette IS its own preview: each row's dot is painted in the tint it sets, so you
        // pick the colour by looking at it rather than by reading its name.
        MenuBody::BackdropTints { current, .. } => TINT_NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| MenuRow {
                label: name.tr(),
                dot: crate::backdrop::tint_token(i as u8),
                selected: i as u8 == *current,
            })
            // Then the backdrop's other actions (Rename / Delete) — a neutral dot, since a verb
            // has no colour to preview, and the label carries the shortcut. `resolve_menu`
            // dispatches these same last rows.
            .chain(
                crate::state::BACKDROP_ACTIONS
                    .iter()
                    .map(|(label, _)| MenuRow {
                        label: label.tr(),
                        dot: ColorToken::Text2,
                        selected: false,
                    }),
            )
            .collect(),
        // ⭐ **As opções de um selector do cartão** (report do Enio, 07/09). Ponto neutro: uma
        // opção não tem categoria a tingir. `selected` marca onde o nó ESTÁ — que é o que
        // distingue uma lista de escolha de uma lista de acções.
        MenuBody::ParamOptions {
            labels, current, ..
        } => labels
            .iter()
            .enumerate()
            .map(|(i, l)| MenuRow {
                label: l,
                dot: ColorToken::Text2,
                selected: i == *current as usize,
            })
            .collect(),
    }
}

/// The library entries a loose output can feed — the compatible-type filter the palette shows for
/// smart-connect (`interact_drop`). With no `from` it is the whole catalog.
pub(crate) fn menu_catalog(snap: &GraphViewSnapshot, from: Option<(u32, u16)>) -> Vec<NodeChoice> {
    let all = current_catalog();
    let Some((from_node, from_port)) = from else {
        return all;
    };
    let Some(out) = snap
        .nodes
        .iter()
        .find(|n| n.id == from_node)
        .and_then(|n| n.outputs.get(from_port as usize))
    else {
        return all;
    };
    all.into_iter()
        .filter(|c| c.inputs.iter().any(|i| accepts(&i.ty, out)))
        .collect()
}

/// ⭐⭐⭐ **As entradas da biblioteca que podem ALIMENTAR uma entrada solta** — o espelho de
/// [`menu_catalog`], ordem do dono (2026-09-19): *«puxar um fio de um slot de entrada (à esquerda
/// do nó) ainda não chama o modal de nós compatíveis. Faça isso possível.»*
///
/// ⛔⛔ **A recusa que isto desfaz estava escrita, e a premissa dela era falsa:** o
/// `interact_socket` dizia que uma largada para trás *«teria de adivinhar o que a alimenta, que é
/// um menu da biblioteca INTEIRA»*. Não é — é exactamente a mesma filtragem por tipo, lida do
/// outro lado do fio. *Uma recusa por «seria a biblioteca toda» tem de contar a biblioteca toda
/// primeiro.*
pub(crate) fn menu_catalog_back(
    snap: &GraphViewSnapshot,
    to: Option<(u32, u16)>,
) -> Vec<NodeChoice> {
    let all = current_catalog();
    let Some((to_node, to_port)) = to else {
        return all;
    };
    let Some(inp) = snap
        .nodes
        .iter()
        .find(|n| n.id == to_node)
        .and_then(|n| n.inputs.get(to_port as usize))
    else {
        return all;
    };
    all.into_iter()
        .filter(|c| c.outputs.iter().any(|o| feeds(&o.ty, inp)))
        .collect()
}

/// Whether an OUTPUT port of a candidate could feed this input — a MESMA regra do [`accepts`],
/// lida do outro lado. ⚠️ Escrita como função própria porque os dois argumentos têm tipos
/// diferentes (`PortType` contra `PortView`) e não por a lei ser outra: ela é a mesma tripla.
fn feeds(output: &ph2d_nodegraph::port::PortType, inp: &PortView) -> bool {
    output.domain == inp.domain && output.dim == inp.dim && output.clock == inp.clock
}

/// Whether an input port could take the dragged output (the panel-side rule:
/// domain + dim + clock — `connects_directly` minus the membrane).
fn accepts(input: &ph2d_nodegraph::port::PortType, out: &PortView) -> bool {
    input.domain == out.domain && input.dim == out.dim && input.clock == out.clock
}
