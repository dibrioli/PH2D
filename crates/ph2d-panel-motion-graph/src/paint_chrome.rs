//! Split chrome (Motion Nodes M1.E9) — the draggable viewport⟂graph divider and
//! the SplitH / SplitV / Fit toolbar, split out of `paint` (panel LOC cap).
//!
//! Both route through the same `GraphSurface` gesture channel the rest of the
//! panel uses: the divider as [`GraphHitKind::SplitDivider`], each chip as
//! [`GraphHitKind::Chrome`] carrying its ordinal. `interact` interprets them
//! (divider drag → `SetSplit`, chips → `SetSplitVertical` / re-fit).

use crate::interact::GroupVerb;
use crate::paint::fnv_id;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::GraphHitKind;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, resolve, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};

// Divider line + toolbar chip metrics (graph-canvas chrome, not design tokens).
const DIVIDER_LINE_W: f32 = 2.0; // LITERAL-PX-OK: divider line thickness
const DIVIDER_HIT_HALF: f32 = 3.0; // LITERAL-PX-OK: divider grab-band half-thickness
const TOOLBAR_INSET: f32 = 8.0; // LITERAL-PX-OK: toolbar inset from the graph corner
const CHIP_SIZE: f32 = 24.0; // LITERAL-PX-OK: toolbar chip square
const CHIP_GAP: f32 = 4.0; // LITERAL-PX-OK: toolbar chip gap
const CHIP_RADIUS: f32 = 6.0; // LITERAL-PX-OK: toolbar chip corner radius
const CHIP_ICON_PAD: f32 = 5.0; // LITERAL-PX-OK: chip icon inset
const CHIP_ICON_STROKE: f32 = 1.5; // LITERAL-PX-OK: chip icon stroke width

/// Chrome control ordinals carried in [`GraphHitKind::Chrome`].
pub(crate) const CHROME_SPLIT_H: u16 = 0;
pub(crate) const CHROME_SPLIT_V: u16 = 1;
pub(crate) const CHROME_FIT: u16 = 2;
/// Add a group backdrop — framing the selection when there is one (F2).
pub(crate) const CHROME_BACKDROP: u16 = 3;
/// Arm the knife (F2) — the chip reads ACTIVE while armed, which is what makes the
/// mode visible: a `K` that silently changed what the next drag does is a mystery
/// (Enio, smoke 2026-07-12: "não entendi K o que faz").
pub(crate) const CHROME_KNIFE: u16 = 4;
/// Arm the probe (F2) — same: armed = active ring, and the next click picks a node.
pub(crate) const CHROME_PROBE: u16 = 5;
/// Collapse the selection into a subgraph (doc 57) — the chip form of Ctrl+G. It
/// reads ACTIVE while there is something to collapse, so the artist can see that the
/// gesture has a subject; with an empty selection it is inert (and says so).
pub(crate) const CHROME_GROUP: u16 = 6;
/// **Auto-arrange** — lay the whole document out in a left-to-right layered flow (a
/// chain becomes one horizontal line). A momentary action, like Fit: it never reads
/// active, it just tidies on press.
pub(crate) const CHROME_ARRANGE: u16 = 7;
/// **Node help on/off** (ADR-0155) — the chip that turns the whole setup diagnoser
/// off (auto-heal + ⚠ inert badges + advisories). It reads ACTIVE while help is on,
/// so the artist can see the system IS running; the shell owns the flag, so this chip
/// emits `SetNodeHelp(!node_help())` rather than flipping a local bool.
pub(crate) const CHROME_NODE_HELP: u16 = 8;
/// **Breadcrumb crumbs start here** — crumb `i` is `CHROME_CRUMB_BASE + i`. They ride
/// the same `Chrome` hit kind as the toolbar chips (an ordinal the panel alone
/// interprets), so navigation costs the foundational interaction vocabulary NOTHING:
/// no new `GraphHitKind`, no new dispatch arm. The base is far above the chip
/// ordinals so the two can never collide as the toolbar grows.
pub(crate) const CHROME_CRUMB_BASE: u16 = 100;

fn split_divider_hit_id() -> NodeId {
    fnv_id("motion_graph/split_divider")
}
fn chrome_hit_id(id: u16) -> NodeId {
    fnv_id(&format!("motion_graph/chrome/{id}"))
}

/// O mesmo id, para o gate de costura que mede a PINTURA (`tests/seam_chip_tooltips.rs`).
///
/// ⚠️ Existe porque o gate tem de perguntar ao store pelo id **que o hover consulta** — uma
/// segunda derivação do id no teste provaria que o teste concorda consigo mesmo, e não que a
/// dica é alcançável.
#[must_use]
pub fn chrome_hit_id_for_tests(id: u16) -> NodeId {
    chrome_hit_id(id)
}

/// Draw the split divider (at the scene boundary) + the SplitH / SplitV / Fit
/// toolbar, pushing their hit rects. `center` is the scene half of the center
/// band; the graph (`rect`) sits below it (horizontal split) or to its right
/// (vertical split), which also picks the active-orientation highlight.
/// What the toolbar has to know about the editor's state to draw itself honestly:
/// which modes are ARMED (their chip wears the Accent ring — a mode with no visible
/// sign is a mystery) and whether Group has a subject at all.
#[derive(Copy, Clone, Debug)]
pub(crate) struct ChromeState {
    pub knife_armed: bool,
    pub probe_armed: bool,
    /// What the Group chip would do if pressed right now — it draws the icon of the
    /// verb it will actually perform (doc 57).
    pub group_verb: crate::interact::GroupVerb,
    /// Whether the node-help system is on (ADR-0155) — the Help chip wears the Accent
    /// ring while it is, the same active-ring language as the armed modes.
    pub node_help: bool,
}

/// The toolbar's chip descriptors — `(ordinal, icon, active)` — as a PURE list, so the
/// paint loop pushes a hit for each and a gate can assert the set is complete (every chip
/// offered, wearing its live state) without building a `PaintCtx`. `vertical` picks the
/// active orientation chip; `state` carries the armed modes and the node-help flag.
pub(crate) fn chip_specs(state: ChromeState, vertical: bool) -> [(u16, IconId, bool); 9] {
    let ChromeState {
        knife_armed,
        probe_armed,
        group_verb,
        node_help,
    } = state;
    [
        (CHROME_SPLIT_H, IconId::SplitHorizontal, !vertical),
        (CHROME_SPLIT_V, IconId::SplitVertical, vertical),
        (CHROME_FIT, IconId::FitView, false),
        (CHROME_BACKDROP, IconId::Backdrop, false),
        (CHROME_KNIFE, IconId::Knife, knife_armed),
        (CHROME_PROBE, IconId::Probe, probe_armed),
        // Group / Ungroup (doc 57) — ONE chip, and it wears the icon of the verb it
        // will actually perform: a card selected and it becomes Ungroup. A chip that
        // looks the same whether or not it can act (and whichever way it will act) is
        // a chip that lies once per press.
        (
            CHROME_GROUP,
            match group_verb {
                GroupVerb::Ungroup => IconId::Ungroup,
                _ => IconId::Group,
            },
            group_verb != GroupVerb::Inert,
        ),
        // Auto-arrange (the layered flow). Never active — it is a momentary tidy, not
        // a mode; the Hierarchy glyph reads as "lay this out as a tree".
        (CHROME_ARRANGE, IconId::Hierarchy, false),
        // Node help (ADR-0155): the whole setup diagnoser on/off. Active ring while ON,
        // so a graph that has stopped offering to fix things says so on its face.
        (CHROME_NODE_HELP, IconId::Help, node_help),
    ]
}

/// **A DICA de cada chip** — o que ele faz, e a tecla quando existe uma.
///
/// ⚠️ **Uma TABELA pura, ao lado da `chip_specs`, e não um `match` no meio do painter.** As duas
/// respondem sobre o MESMO conjunto (*que chips existem* · *o que cada um diz*), e um gate exige
/// que elas concordem — um chip novo sem dica é vermelho, em vez de um botão mudo que só um
/// smoke apanha. Foi um report do Enio (2026-08-28: *"coloque dicas no hover dos botões
/// posicionados no canto inferior esquerdo do canvas"*), e é a mesma lei do rótulo de porta:
/// **um ícone sem nome só é legível para quem já sabe o que ele faz.**
///
/// ⚠️ **A tecla é NOMEADA só onde ela existe.** `Fit`, `Knife`, `Probe` e `Group` têm atalho no
/// `keymap.rs`; os outros cinco não têm nenhum, e inventar um sufixo ali ensinaria uma tecla
/// que não faz nada. ⛔ O `F2` que os doc-comments acima citam é *"fase 2"*, não a tecla F2 —
/// que neste painel é **Rename**.
pub(crate) fn chip_tooltip(id: u16) -> &'static str {
    match id {
        CHROME_SPLIT_H => "Split horizontally",
        CHROME_SPLIT_V => "Split vertically",
        CHROME_FIT => "Fit view \u{00b7} F",
        CHROME_BACKDROP => "Add group backdrop",
        CHROME_KNIFE => "Knife \u{00b7} K \u{00b7} drag across wires to cut",
        CHROME_PROBE => "Probe \u{00b7} P \u{00b7} click a node to read its stream",
        CHROME_GROUP => "Group / Ungroup \u{00b7} Ctrl+G",
        CHROME_ARRANGE => "Auto-arrange the graph",
        CHROME_NODE_HELP => "Node help on/off \u{00b7} badges + auto-fix",
        // ⚠️ Inalcançável: o gate `every_chip_carries_a_tooltip` varre a `chip_specs` inteira.
        // Um braço vazio aqui seria um chip mudo a passar despercebido.
        _ => "",
    }
}

pub(crate) fn draw_split_chrome(
    ctx: &mut PaintCtx,
    rect: Rect,
    // A banda que o divisor parte — aqui só para saber a ORIENTAÇÃO (`HeroLayout::split_band`).
    split_band: Rect,
    theme: Theme,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    state: ChromeState,
) {
    let vertical = rect.x > split_band.x + 0.5;
    // Divider line + a forgiving grab band straddling the boundary edge.
    let (line, band) = if vertical {
        (
            [(rect.x, rect.y), (rect.x, rect.y + rect.h)],
            Rect::new(
                rect.x - DIVIDER_HIT_HALF,
                rect.y,
                2.0 * DIVIDER_HIT_HALF,
                rect.h,
            ),
        )
    } else {
        (
            [(rect.x, rect.y), (rect.x + rect.w, rect.y)],
            Rect::new(
                rect.x,
                rect.y - DIVIDER_HIT_HALF,
                rect.w,
                2.0 * DIVIDER_HIT_HALF,
            ),
        )
    };
    stroke_polyline(
        ctx.scene,
        &line,
        DIVIDER_LINE_W,
        resolve(ColorToken::Border, theme),
    );
    hits.push((split_divider_hit_id(), GraphHitKind::SplitDivider, band));

    // Toolbar chips at the graph's BOTTOM-left. They used to sit at the top-left,
    // where they collided with the editor's own left-hand button rail (undo/redo)
    // — two toolbars stacked in the same corner (Enio, smoke 2026-07-12). The
    // bottom edge is the graph's only permanently free corner: the top carries the
    // split divider band, and the right is where a panned graph tends to spill.
    // The orientation chip matching the current split reads as active (Accent ring).
    let chips = chip_specs(state, vertical);
    let row_y = rect.y + rect.h - TOOLBAR_INSET - CHIP_SIZE;
    for (i, (id, icon, active)) in chips.into_iter().enumerate() {
        let cx = rect.x + TOOLBAR_INSET + i as f32 * (CHIP_SIZE + CHIP_GAP);
        let chip = Rect::new(cx, row_y, CHIP_SIZE, CHIP_SIZE);
        fill_rounded_rect(
            ctx.scene,
            chip,
            CHIP_RADIUS,
            resolve(ColorToken::Bg2, theme),
        );
        let border = if active {
            ColorToken::Accent
        } else {
            ColorToken::Border
        };
        stroke_rounded_rect(ctx.scene, chip, CHIP_RADIUS, 1.0, resolve(border, theme));
        let icon_rect = Rect::new(
            chip.x + CHIP_ICON_PAD,
            chip.y + CHIP_ICON_PAD,
            CHIP_SIZE - 2.0 * CHIP_ICON_PAD,
            CHIP_SIZE - 2.0 * CHIP_ICON_PAD,
        );
        paint_icon(
            ctx.scene,
            icon,
            icon_rect,
            resolve(ColorToken::Text1, theme),
            CHIP_ICON_STROKE,
        );
        let hit = chrome_hit_id(id);
        // A DICA, no mesmo sítio em que o chip nasce: quem acrescentar um chip aqui vê a
        // linha ao lado, e o gate cobra a que faltar.
        ctx.host.store_mut().set_tooltip(hit, chip_tooltip(id));
        hits.push((hit, GraphHitKind::Chrome { id }, chip));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **TODO CHIP DA BARRA TEM DICA** — o censo, varrendo a `chip_specs` inteira.
    ///
    /// ⚠️ É a metade que impede o botão mudo: um chip novo entra na `chip_specs` (senão nem é
    /// pintado) e este gate cobra a linha correspondente na `chip_tooltip`. Sem ele, o braço
    /// `_ => ""` da tabela absorveria o chip novo em silêncio — que é exactamente o modo de
    /// falha que uma tabela com braço de omissão tem.
    ///
    /// ⚠️ **E o CONTROLE: o censo tem de ter visto os nove.** Um laço sobre uma lista vazia
    /// passa — *um zero de «não medido» e um de «tudo certo» são o mesmo byte*.
    #[test]
    fn every_chip_carries_a_tooltip() {
        let chips = chip_specs(state(false), false);
        assert_eq!(chips.len(), 9, "a barra tem nove chips");
        for (id, icon, _) in chips {
            let tip = chip_tooltip(id);
            assert!(
                !tip.is_empty(),
                "o chip {id} ({icon:?}) e' pintado e nao diz o que faz — um icone sem nome so' \
                 e' legivel para quem ja' sabe o que ele faz"
            );
        }
    }

    /// **A DICA NOMEIA A TECLA SÓ ONDE ELA EXISTE.**
    ///
    /// ⚠️ Os quatro atalhos vêm do `keymap.rs` da shell (`F`, `K`, `P`, `Ctrl+G`). Os outros
    /// cinco chips não têm tecla nenhuma, e um sufixo inventado ali ensinaria uma tecla que não
    /// faz nada — o mesmo defeito que um rótulo que promete o que o modelo não entrega.
    #[test]
    fn a_tooltip_names_a_key_only_where_one_exists() {
        for (id, key) in [
            (CHROME_FIT, "F"),
            (CHROME_KNIFE, "K"),
            (CHROME_PROBE, "P"),
            (CHROME_GROUP, "Ctrl+G"),
        ] {
            assert!(
                chip_tooltip(id).contains(key),
                "o chip {id} tem atalho `{key}` e a dica tem de o dizer: {:?}",
                chip_tooltip(id)
            );
        }
        // O CONTROLE: os que NÃO têm tecla não fingem ter — nenhum separador de atalho.
        for id in [
            CHROME_SPLIT_H,
            CHROME_SPLIT_V,
            CHROME_BACKDROP,
            CHROME_ARRANGE,
        ] {
            assert!(
                !chip_tooltip(id).contains('\u{00b7}'),
                "o chip {id} nao tem atalho — a dica nao pode anunciar um: {:?}",
                chip_tooltip(id)
            );
        }
    }

    fn state(node_help: bool) -> ChromeState {
        ChromeState {
            knife_armed: false,
            probe_armed: false,
            group_verb: GroupVerb::Inert,
            node_help,
        }
    }

    /// **The Node Help chip is offered and wears the live state** (ADR-0155). It is in the
    /// toolbar's chip list, so the paint loop pushes its hit — it is reachable by a pointer,
    /// not dead under the mouse — and its active ring follows `node_help`. FALSIFIED by
    /// dropping the chip from [`chip_specs`] (no hit → dead under the mouse) or by a fixed
    /// `active` (the ring would lie about whether the system is running).
    #[test]
    fn the_node_help_chip_is_offered_and_wears_the_live_state() {
        let on = chip_specs(state(true), false);
        assert!(
            on.iter().any(|&(id, icon, active)| id == CHROME_NODE_HELP
                && icon == IconId::Help
                && active),
            "help chip present + active when the system is on"
        );
        let off = chip_specs(state(false), false);
        assert!(
            off.iter()
                .any(|&(id, _, active)| id == CHROME_NODE_HELP && !active),
            "help chip present + inactive when off"
        );
    }
}
