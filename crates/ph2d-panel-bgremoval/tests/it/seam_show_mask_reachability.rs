//! **The Show-mask row: one law, two lenses, measured in ONE expression.**
//!
//! ## The defect
//!
//! The gate that PAINTED the row was an `OR`
//! (`protect_brush_armed || has_protect_mask`); the gate that CONSUMED
//! the toggle — the shell's canvas overlay — is an `AND`
//! (`show_mask && has_protect_mask`). Arming the protection brush was
//! therefore enough to reveal the button, and the button did nothing
//! until the first dab landed. ⚠️ **The dead window was exactly the
//! first state the artist reaches**: pick the tool, arm the brush, press
//! the freshly-revealed "Show mask", watch nothing happen.
//!
//! ## What this gate measures
//!
//! Not focusability (`architecture_panel_wiring_parity` already does
//! that, and it was green with this dead). Not that the click reaches
//! the tool (the sibling `seam.rs` already does that, and it was green
//! too). It measures **the value arriving at the consumer**: for every
//! reachable combination of the three facts, painting the row and the
//! overlay actually rendering are compared *in the same expression*, and
//! any divergence is a failure.
//!
//! The law lives once, in `ph2d_tool_bgremoval::params::mask_overlay_renders`;
//! the row's visibility is DERIVED from it
//! (`mask_overlay_toggle_is_reachable`), so a change to the law moves
//! the row in the same edit.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_bgremoval::state::BgRemovalPanelState;
use ph2d_panel_bgremoval::{BgRemovalPanel, ids, set_current_bgremoval_snapshot};
use ph2d_tool_bgremoval::params::{
    BgRemovalUiSnapshot, mask_overlay_renders, mask_overlay_toggle_is_reachable,
};
use ph2d_ui_testkit::MockPanelHost;

/// A viewport tall enough that the protection block is not clipped away
/// for a reason other than the predicate under test.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2400.0,
};

/// Paint the panel with this snapshot and answer: was the Show-mask
/// button registered in the hit index — i.e. can the artist reach it?
fn row_is_reachable(snapshot: BgRemovalUiSnapshot) -> bool {
    let mut host = MockPanelHost::with_panel::<BgRemovalPanel>();
    let mut state = BgRemovalPanelState;
    set_current_bgremoval_snapshot(Some(snapshot));
    let hits = host.paint::<BgRemovalPanel>(&mut state, VIEWPORT);
    let found = hits.iter().any(|(id, _)| *id == ids::BGR_SHOW_MASK);
    set_current_bgremoval_snapshot(None);
    found
}

fn snapshot(armed: bool, has_mask: bool, show: bool) -> BgRemovalUiSnapshot {
    BgRemovalUiSnapshot {
        protect_brush_armed: armed,
        has_protect_mask: has_mask,
        show_mask: show,
        ..BgRemovalUiSnapshot::default()
    }
}

/// ⭐ **THE gate.** For every combination of the three facts, the row is
/// painted **iff** flipping the toggle would change what the consumer
/// draws. Both lenses are evaluated side by side in one assertion, so
/// they cannot silently drift apart again.
///
/// The failure message names the state, because the state IS the bug
/// report: `armed=true has_mask=false` is the one the artist hits first.
#[test]
fn the_show_mask_row_is_painted_exactly_when_the_toggle_changes_the_overlay() {
    for armed in [false, true] {
        for has_mask in [false, true] {
            for show in [false, true] {
                let painted = row_is_reachable(snapshot(armed, has_mask, show));
                // The consumer's own law, asked twice: does the toggle
                // matter here at all?
                let toggle_matters =
                    mask_overlay_renders(true, has_mask) != mask_overlay_renders(false, has_mask);
                assert_eq!(
                    painted, toggle_matters,
                    "armed={armed} has_mask={has_mask} show={show}: the panel paints the \
                     Show-mask row = {painted}, but flipping it changes the overlay = \
                     {toggle_matters}. The painting lens and the consuming lens have drifted."
                );
            }
        }
    }
}

/// The derived predicate is not allowed to be a second opinion: it must
/// answer exactly what the law answers, over the whole domain.
#[test]
fn the_reachability_predicate_is_derived_from_the_consumers_law() {
    for has_mask in [false, true] {
        assert_eq!(
            mask_overlay_toggle_is_reachable(has_mask),
            mask_overlay_renders(true, has_mask) != mask_overlay_renders(false, has_mask),
            "has_mask={has_mask}: the row predicate stopped being a derivation of the law"
        );
    }
}

/// The specific state the report named, pinned on its own so a
/// regression reads as one line instead of a loop index: **arming the
/// brush must not reveal a button that does nothing.**
#[test]
fn arming_the_brush_alone_does_not_reveal_a_dead_show_mask_button() {
    assert!(
        !row_is_reachable(snapshot(true, false, true)),
        "brush armed, no dab yet: the Show-mask button is painted and the overlay \
         cannot draw — this is the first state the artist reaches"
    );
    assert!(
        row_is_reachable(snapshot(true, true, true)),
        "one dab painted: the toggle now changes the canvas and must be reachable"
    );
    assert!(
        row_is_reachable(snapshot(false, true, false)),
        "a mask exists and the brush is disarmed: the toggle still governs the overlay"
    );
}
