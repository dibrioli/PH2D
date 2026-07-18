//! The breadcrumb: **where in the nesting the animator is**, and the way back out
//! ([ADR-0133] §5).
//!
//! # Why a trail and not a tab
//!
//! The research split the two lineages cleanly: **edit-in-place with a breadcrumb** is the 2D
//! animation lineage (Flash/Animate's edit bar, Harmony's `Top` button), and **a new tab** is
//! the compositing lineage (After Effects). No formal consensus says the tab is worse — but
//! the *symptom* is documented over and over, always the same one: you lose the parent's
//! context, and AE users rebuild it by hand (locking a viewer, `View > New Viewer`) to get
//! back what edit-in-place gives for free. Adobe answered with navigation aids, never with
//! in-place editing.
//!
//! ⚠️ And the panel's existing `Tab::{Keys, Arrange}` is **not** the same axis: the tab says
//! *which half you are looking at*, the trail says *where you are*. They compose — the trail
//! puts you in a container, and the tabs then show that container's keys or its arrangement.
//!
//! # Zero-width at the root
//!
//! At the scene root there is nothing to go back to, so the trail paints nothing and measures
//! zero. A document that never touches containers pays nothing and sees nothing.
//!
//! [ADR-0133]: ../../../docs/architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use crate::ids;

/// One segment's width. Fixed rather than measured: the trail sits in a flow layout that
/// wraps, and a segment that grew with a container's name would make the whole bar reflow
/// every rename.
const SEG_W: f32 = 74.0; // LITERAL-PX-OK: breadcrumb segment width

/// How many segments the trail paints — the root plus the containers entered.
///
/// Capped by the id array ([`ids::TIMELINE_CRUMB`]), which is the honest resource: the chrome
/// cannot mint a hit id at runtime, so a deeper trail would paint a segment nothing could
/// click. ⚠️ This is a cap on the **trail**, not on nesting depth — the ADR measured that and
/// found no resource to justify limiting it.
fn segments(snap: &TimelineViewSnapshot) -> usize {
    if snap.crumbs.is_empty() {
        return 0;
    }
    (snap.crumbs.len() + 1).min(ids::TIMELINE_CRUMB.len())
}

/// How wide the trail paints — the single source the transport flow measures against.
pub(crate) fn width(snap: &TimelineViewSnapshot) -> f32 {
    let n = segments(snap);
    if n == 0 {
        return 0.0;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "n is at most the id array's length"
    )]
    let n = n as f32;
    n * SEG_W + (n - 1.0) * Spacing::Sm.px() * 0.5
}

/// Paint the trail: `[ Scene ][ C1 ][ C2 ]`, each segment a button that pops out to it.
///
/// The LAST segment is where you already are, so clicking it is a no-op — it is painted as
/// part of the trail rather than special-cased, because a trail whose current position is
/// missing reads as "you are nowhere".
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, snap: &TimelineViewSnapshot) {
    let n = segments(snap);
    if n == 0 {
        return;
    }
    let gap = Spacing::Sm.px() * 0.5;
    let mut x = x;
    for depth in 0..n {
        let id = ids::TIMELINE_CRUMB[depth];
        let label: String = if depth == 0 {
            ph2d_i18n::tr("panel.timeline.crumb_root").into()
        } else {
            // `depth - 1`: segment 0 is the root, so container `i` lives at segment `i + 1`.
            snap.crumbs
                .get(depth - 1)
                .map_or_else(String::new, |c| c.1.clone())
        };
        let rect = Rect::new(x, y, SEG_W, ROW_H_PX);
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        paint_button(
            &Button::new(id, label).state(st),
            rect,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(id, rect);
        x += SEG_W + gap;
    }
}
