//! **The joint-anchor point gizmo reaches the canvas and the bus** — arch-gates
//! over the two window-gated seams a unit test cannot drive.
//!
//! The gizmo itself is behavioural-gated where it can be: `ph2d_editor`'s
//! `gizmo::point` proves the dot is drawn AND registers a hit at the anchor, and
//! the shell's `render_loop::point_gizmo` proves the publish rule (a joint, and
//! nothing else, gets a handle). What those cannot reach is the GLUE — the paint
//! pass that draws the published view, and the pointer-Down that turns a hit into
//! a drag — because both need a live App/HeroScreen. This is the "painted +
//! registered" and "the click reaches the bus" halves of the UI checklist, as
//! arch-gates over the source, the same tool this line uses for its pointer path.

use std::fs;

/// The `began_joint_anchor` block of the pointer-Down handler: from the flag to
/// the guard that consumes it. This is where a Down on the anchor dot becomes a
/// drag, and its correctness is one property — it must open a Translate of the
/// SELECTION, because a joint has no sprite for the canvas-pick Translate to
/// resolve.
fn joint_anchor_down_block() -> String {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    let start = src
        .find("let mut began_joint_anchor = false;")
        .expect("the joint-anchor Down branch is gone");
    let rest = &src[start..];
    let end = rest
        .find("if began_pivot || began_joint_anchor")
        .expect("the branch's closing guard is gone");
    rest[..end].to_string()
}

/// **A Down on the anchor dot opens a Translate drag of the SELECTED joint.**
///
/// Four properties, each a way the wiring would silently break:
///  - keyed on `GIZMO_JOINT_ANCHOR` — it recognises the point handle;
///  - opens `GizmoDragKind::Translate` — a move, not a scale/rotate;
///  - the dragged entity is `hero.gizmo.selection` — the joint itself;
///  - it does NOT `pick_sprites_at_world` — the generic Translate resolves the
///    entity by picking a sprite under the cursor, and a joint has none, so a
///    branch that fell through to that path would drag nothing.
#[test]
fn the_joint_anchor_down_opens_a_translate_of_the_selection() {
    let block = joint_anchor_down_block();
    assert!(
        block.contains("GIZMO_JOINT_ANCHOR"),
        "the Down branch is not keyed on the joint-anchor hit id — the dot would \
         be painted but a click on it would fall through to the generic path"
    );
    assert!(
        block.contains("GizmoDragKind::Translate"),
        "the Down branch does not open a Translate drag"
    );
    assert!(
        block.contains("hero.gizmo.selection"),
        "the Down branch does not drag the SELECTION — a joint has no sprite to \
         pick, so the entity must come from the selection, not a canvas pick"
    );
    assert!(
        !block.contains("pick_sprites_at_world"),
        "the Down branch resolves the entity by picking a sprite — a joint has \
         none, so this would drag nothing"
    );
}

/// **The paint pass draws the published point view.** The editor-core gizmo pass
/// must call `paint_point_gizmo` on `hero.gizmo.point_view`, or the shell can
/// publish a handle every frame that is never drawn and never registered — the
/// dot is invisible and unclickable, with every behavioural gate still green
/// (they call `paint_point_gizmo` directly).
#[test]
fn the_paint_pass_draws_the_point_gizmo() {
    let paint = fs::read_to_string("../../crates/ph2d-editor-core/src/screens/hero/paint.rs")
        .expect("the hero paint pass");
    assert!(
        paint.contains("point_view") && paint.contains("paint_point_gizmo"),
        "the hero paint pass no longer draws the point gizmo from `point_view` — \
         a published joint-anchor handle would never reach the screen or the hit \
         index"
    );
}
