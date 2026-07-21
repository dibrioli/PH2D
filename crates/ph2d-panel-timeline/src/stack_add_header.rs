//! **The Arrange tab's ADD header** — "+ Lane" and "+ Container", split out of
//! `stack_lane_paint.rs` when that file crossed the panel LOC cap (609/600) growing the
//! label-proportional split. A unit in its own right: the header is the one part of the
//! lane area that exists even over an EMPTY stack (it is how the first lane is made),
//! which is also why the Arrange column floor follows the TAB (`geom::min_label_w`).

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonState, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Spacing, Theme};

use crate::ids;

/// The Arrange tab's two ADDs, sharing the label column's header strip: **+ Lane** and
/// **+ Container**.
///
/// They sit together because they are the two ways to grow the stack you are looking at — one
/// adds a row, the other adds a *piece* — and both land in whichever stack is open, so
/// containers nest by pressing the second one twice.
pub(crate) fn paint_add_lane(ctx: &mut PaintCtx, theme: Theme, header: Rect) {
    let gap = Spacing::Sm.px() * 0.5;
    let labels = [
        ph2d_i18n::tr("panel.timeline.add_lane"),
        ph2d_i18n::tr("panel.timeline.add_container"),
    ];
    let widths = add_widths(header.w, gap, [&labels[0], &labels[1]]);
    let mut x = header.x;
    for ((id, label), w) in [ids::TIMELINE_ADD_LANE, ids::TIMELINE_ADD_CONTAINER]
        .into_iter()
        .zip(labels.iter())
        .zip(widths)
    {
        let rect = Rect::new(x, header.y, w, header.h);
        let st = ctx
            .host
            .store()
            .button_state(id)
            .unwrap_or(ButtonState::Normal);
        paint_button(
            &Button::new(id, label.to_string()).state(st),
            rect,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(id, rect);
        x += w + gap;
    }
}

/// **How the ADD header splits between its two buttons** — by each label's LENGTH, never
/// 50/50.
///
/// An even split gives "+ Lane" and "+ Container" the same box while one label is nearly
/// twice the other: at the column's floor the long one was crushed down to a bare "+"
/// (Enio's screenshot, 2026-07-20). Splitting by character share puts the room where the
/// text is, reads from the SAME strings the buttons paint (a hand-tuned ratio would drift
/// the day a label changes), and is pure so the fit is testable without a text system.
pub(crate) fn add_widths(header_w: f32, gap: f32, labels: [&str; 2]) -> [f32; 2] {
    let total = (header_w - gap).max(0.0);
    let chars = labels.map(|l| l.chars().count().max(1));
    #[expect(clippy::cast_precision_loss, reason = "label lengths are tiny")]
    let share = chars[0] as f32 / (chars[0] + chars[1]) as f32;
    let first = total * share;
    [first, total - first]
}

