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

/// The lane tabs' ADD header: **+ Lane** always, and **+ Container** only where a container
/// is what you would be making.
///
/// # Why "+ Container" is not on Arrange
///
/// It used to be, and it meant *"make a container AND drop an instance of it here"* — one
/// button doing the two different acts of creating an asset and placing it. That is what made
/// a container indistinguishable from a lane on screen (Enio, 2026-07-21). Now creating lives
/// on the **Containers** tab (where containers live) and placing is the lane's `+` with the
/// container picked in the source dropdown — one meaning per control, and nesting is just
/// placing a container inside a container.
pub(crate) fn paint_add_lane(ctx: &mut PaintCtx, theme: Theme, header: Rect, tab: crate::tab::Tab) {
    let gap = Spacing::Sm.px() * 0.5;
    let labels = [
        ph2d_i18n::tr("panel.timeline.add_lane"),
        ph2d_i18n::tr("panel.timeline.add_container"),
    ];
    let widths = header_widths(header.w, gap, tab, [labels[0], labels[1]]);
    let mut x = header.x;
    for ((id, label), w) in [ids::TIMELINE_ADD_LANE, ids::TIMELINE_ADD_CONTAINER]
        .into_iter()
        .zip(labels.iter())
        .zip(widths)
    {
        // Not painted AND not hit-registered: a dimmed control that still dispatches is a
        // click that silently does nothing ([[feedback_disabled_button_still_dispatches]]).
        if w <= 0.0 {
            continue;
        }
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

/// **How the ADD header splits, per tab** — the one door the paint lays out from, and the
/// pure function a gate can ask without a text system.
///
/// A zero width is the refusal: the paint skips those, so "+ Container" on a tab that does
/// not make containers is neither painted NOR hit-registered. A dimmed control that still
/// dispatches is a click that silently does nothing
/// ([[feedback_disabled_button_still_dispatches]]).
pub(crate) fn header_widths(
    header_w: f32,
    gap: f32,
    tab: crate::tab::Tab,
    labels: [&str; 2],
) -> [f32; 2] {
    if tab == crate::tab::Tab::Containers {
        add_widths(header_w, gap, labels)
    } else {
        // One button takes the strip whole.
        [header_w, 0.0]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tab::{ALL, Tab};

    /// **"+ Container" só existe onde um container é o que você faria.**
    ///
    /// Ele criava E colocava — os dois atos numa tecla — e era isso que tornava um container
    /// indistinguível de uma lane na tela (Enio, 2026-07-21). Largura zero é a recusa: o
    /// desenho pula, então o botão não é pintado NEM registrado.
    #[test]
    fn the_container_button_lives_only_on_the_tab_that_makes_containers() {
        for tab in ALL {
            let [lane, cont] = header_widths(200.0, 4.0, tab, ["+ Lane", "+ Container"]);
            assert!(lane > 0.0, "{tab:?}: '+ Lane' existe em toda aba de lanes");
            if tab == Tab::Containers {
                assert!(cont > 0.0, "é a aba onde containers nascem");
            } else {
                assert!(
                    cont <= 0.0,
                    "{tab:?} não faz containers — largura zero é a recusa, veio {cont}"
                );
            }
        }
    }

    /// **A tira inteira é usada nos dois casos** — um botão sozinho toma a coluna, dois a
    /// dividem pelo comprimento do rótulo. Uma sobra faria a coluna parecer quebrada.
    #[test]
    fn the_header_strip_is_fully_spent() {
        let (w, gap) = (200.0, 4.0);
        for tab in ALL {
            let [a, b] = header_widths(w, gap, tab, ["+ Lane", "+ Container"]);
            let used = if b > 0.0 { a + b + gap } else { a };
            assert!((used - w).abs() < 1e-3, "{tab:?}: gastou {used} de {w}");
        }
    }
}
