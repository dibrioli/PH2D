//! **The joint-anchor point gizmo reaches the canvas and the bus** — arch-gates
//! over the window-gated seams a unit test cannot drive.
//!
//! The gizmo itself is behavioural-gated where it can be: `ph2d_editor`'s
//! `gizmo::point` proves both dots are drawn AND register hits at their anchors,
//! the shell's `render_loop::point_gizmo` proves the publish rule (a joint at
//! rest, and nothing else, gets handles), and `ph2d-physics-ecs`'s
//! `joint_anchor_authoring` proves what a write to either end does. What those
//! cannot reach is the GLUE — the paint pass that draws the published view, and
//! the pointer-Down that turns a hit into a drag — because both need a live
//! App/HeroScreen. This is the "painted + registered" and "the click reaches the
//! bus" halves of the UI checklist, as arch-gates over the source.

use std::fs;

/// The `began_joint_anchor` block of the pointer-Down handler: from the side
/// resolution to the guard that consumes the flag. This is where a Down on either
/// anchor dot becomes a drag.
fn joint_anchor_down_block() -> String {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    let start = src
        .find("let anchor_side = match hit_id {")
        .expect("the joint-anchor Down branch is gone");
    let rest = &src[start..];
    let end = rest
        .find("if began_pivot || began_joint_anchor")
        .expect("the branch's closing guard is gone");
    rest[..end].to_string()
}

/// **A Down on EITHER dot opens the anchor drag for THAT end.**
///
/// Four properties, each a way the wiring would silently break:
///  - both hit ids are recognised — one handle each for body A and body B;
///  - each maps to its own [`JointSide`], so the two dots do not author the same
///    local (the whole point of the second handle);
///  - the gesture is `joint_anchor_drag::open_drag`, not a `GizmoDragKind` — a
///    Translate writes a `Transform`, and body B's anchor is not one;
///  - it does NOT `pick_sprites_at_world` — the generic Translate resolves its
///    entity by picking a sprite under the cursor, and a joint has none, so a
///    branch that fell through to that path would drag nothing.
#[test]
fn the_joint_anchor_down_opens_the_anchor_drag_for_its_side() {
    let block = joint_anchor_down_block();
    for id in ["GIZMO_JOINT_ANCHOR", "GIZMO_JOINT_ANCHOR_B"] {
        assert!(
            block.contains(id),
            "the Down branch is not keyed on `{id}` — that dot would be painted \
             but a click on it would fall through to the generic path"
        );
    }
    assert!(
        block.contains("JointSide::A") && block.contains("JointSide::B"),
        "the Down branch does not distinguish the two ends — one handle would \
         author the other's anchor"
    );
    assert!(
        block.contains("joint_anchor_drag::open_drag"),
        "the Down branch does not open the anchor drag. A `GizmoDragKind` writes \
         a `Transform`, which body B's anchor is not — the B handle could never \
         author anything"
    );
    assert!(
        !block.contains("pick_sprites_at_world"),
        "the Down branch resolves the entity by picking a sprite — a joint has \
         none, so this would drag nothing"
    );
}

/// **The open anchor drag is advanced on every Move.** Opening a drag nothing
/// follows leaves a handle that grabs and then never tracks the cursor — the
/// state exists, the gesture does not.
#[test]
fn the_move_dispatch_advances_the_anchor_drag() {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    assert!(
        src.contains("self.advance_joint_anchor_drag();"),
        "no Move site advances the joint-anchor drag"
    );
    assert!(
        src.contains("self.joint_anchor_drag = None;"),
        "no Up site closes the joint-anchor drag — it would keep following the \
         cursor after the button is released"
    );
}

/// **A generic Translate no longer re-seeds a joint's anchors.**
///
/// The tail that used to live in `advance_gizmo_drag` cleared
/// `PhysicsJoint::anchored` on every Translate Move of a joint entity. That
/// sentinel is joint-WIDE: it re-derives both locals from the seed policy, so
/// with a B handle in the world it would throw away the anchor the artist had
/// just placed on the other body — silently, with every other gate green.
///
/// This gate pins the removal. Mutation-tested: putting `j.anchored = false` back
/// into `gizmo_drag.rs` goes red here.
#[test]
fn the_generic_translate_does_not_reseed_a_joints_anchors() {
    let src = fs::read_to_string("src/input_dispatch/gizmo_drag.rs").expect("gizmo_drag.rs");
    // Strip comments: the removal is a fact about CODE, and a doc-comment that
    // explains why the tail is gone must not read as the tail being back.
    let code: String = src
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains("anchored = false"),
        "a Translate drag clears `PhysicsJoint::anchored` again — dragging the A \
         dot would reset the B anchor the artist authored with the second handle"
    );
}

/// **The Inspector's Position commit re-seats the A anchor through the door.**
///
/// A joint's Position IS its A anchor, so committing one has to reach the anchor
/// state — and through `set_joint_anchor_world`, not the joint-wide sentinel, for
/// the same reason as the gate above. Typing an X for the pivot must not reset
/// body B's end.
#[test]
fn the_position_commit_reseats_the_anchor_through_the_door() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let start = src
        .find("let joint_pivot_commit =")
        .expect("the Position commit no longer captures the joint pivot");
    let block = &src[start..(start + 3000).min(src.len())];
    assert!(
        block.contains("set_joint_anchor_world"),
        "the committed pivot never reaches the bridge's anchor door"
    );
    assert!(
        block.contains("JointSide::A"),
        "the Position field is the A anchor; the commit must say so"
    );
    let commits =
        fs::read_to_string("src/render_loop/inspector_commits.rs").expect("inspector_commits.rs");
    let commit_code: String = commits
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !commit_code.contains("anchored = false"),
        "the old joint-wide re-seed is back in the Transform commit drain"
    );
}

/// **The paint pass draws the published point view.** The editor-core gizmo pass
/// must call `paint_point_gizmo` on `hero.gizmo.point_view`, or the shell can
/// publish handles every frame that are never drawn and never registered — the
/// dots are invisible and unclickable, with every behavioural gate still green
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

/// **The B anchor the handles are drawn at comes from the bridge's door.**
///
/// `build_point_view` is handed the B anchor rather than deriving one, and the
/// caller must ask `joint_anchor_world` — the SAME function `sync_joint_pivots`
/// uses for the A pivot. A second derivation here is how the two dots would come
/// to describe different frames (the failure W-AnchorFollow paid 1.771 m for).
#[test]
fn the_published_b_anchor_comes_from_the_bridge_door() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let start = src
        .find("self.joint_body_pick,")
        .expect("the publish call's joint arguments are gone");
    let block = &src[start..(start + 1600).min(src.len())];
    assert!(
        block.contains("joint_anchor_world") && block.contains("JointSide::B"),
        "the publish site no longer asks the bridge for the B anchor"
    );
    assert!(
        block.contains("is_playing"),
        "the handles are no longer gated on the clock — they would take drags \
         against a swinging body and author a pose nobody chose"
    );
}
