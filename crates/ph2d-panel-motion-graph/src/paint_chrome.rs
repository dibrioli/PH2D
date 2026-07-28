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
}

pub(crate) fn draw_split_chrome(
    ctx: &mut PaintCtx,
    rect: Rect,
    center: Rect,
    theme: Theme,
    hits: &mut Vec<(NodeId, GraphHitKind, Rect)>,
    state: ChromeState,
) {
    let ChromeState {
        knife_armed,
        probe_armed,
        group_verb,
    } = state;
    let vertical = rect.x > center.x + 0.5;
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
    let chips = [
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
    ];
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
        hits.push((chrome_hit_id(id), GraphHitKind::Chrome { id }, chip));
    }
}
