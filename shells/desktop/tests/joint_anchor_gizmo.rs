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
        .find("let anchor_hit = hit_id.and_then(")
        .expect("the joint-anchor Down branch is gone");
    let rest = &src[start..];
    let end = rest
        .find("if began_pivot || began_joint_anchor")
        .expect("the branch's closing guard is gone");
    rest[..end].to_string()
}

/// The argument list of the `open_drag(...)` call inside that block — where the
/// question *"which joint is being authored?"* is actually answered.
///
/// ⚠️ Asserting about the whole block cannot answer it: the block legitimately
/// mentions `hero.gizmo.selection` (it WRITES it, to select the grabbed joint),
/// so a "does not mention the selection" test over the block has to be phrased
/// as a spelling — and a mutation that spells it differently walks straight
/// through. Measured: `Entity::from_bits(hero.gizmo.selection.unwrap_or(0))`
/// survived exactly that. The arguments are the property.
fn open_drag_arguments() -> String {
    let block = joint_anchor_down_block();
    let start = block
        .find("open_drag(")
        .expect("the Down branch no longer opens the anchor drag");
    let rest = &block[start..];
    let end = rest.find(");").expect("unterminated open_drag call");
    rest[..end].to_string()
}

/// **A Down on a dot opens the anchor drag for THAT joint and THAT end.**
///
/// Four properties, each a way the wiring would silently break:
///  - the joint and the side come from `resolve_anchor_hit` over the painter's
///    `point_hit_map` — every joint publishes handles (W-J2b), so the ids are
///    keyed by entity bits and nothing else can say which one was clicked;
///  - it does NOT resolve the joint from `hero.gizmo.selection`, which is the
///    pre-W-J2b shape and would author the *selected* joint's anchor from a
///    click on a different joint's dot;
///  - the gesture is `joint_anchor_drag::open_drag`, not a `GizmoDragKind` — a
///    Translate writes a `Transform`, and body B's anchor is not one;
///  - it does NOT `pick_sprites_at_world` — the generic Translate resolves its
///    entity by picking a sprite under the cursor, and a joint has none, so a
///    branch that fell through to that path would drag nothing.
#[test]
fn the_joint_anchor_down_opens_the_anchor_drag_for_its_side() {
    let block = joint_anchor_down_block();
    assert!(
        block.contains("resolve_anchor_hit") && block.contains("point_hit_map"),
        "the Down branch does not resolve the hit through the painter's map — with \
         handles on every joint the ids are keyed by entity bits, so a constant-id \
         match would recognise nothing and every dot would be painted but dead"
    );
    let args = open_drag_arguments();
    assert!(
        args.contains("\n                            joint,"),
        "the drag is not opened on the joint the hit resolved to: {args}"
    );
    assert!(
        !args.contains("selection"),
        "the SELECTION is being passed as the joint to author — a click on one \
         joint's dot would move a different joint's anchor: {args}"
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

/// **Grabbing an anchor selects its joint.**
///
/// The other half of *"without having to select it in the Hierarchy"* (Enio,
/// 2026-07-25). The dot is now how a joint is reached at all — it has no sprite,
/// so the canvas pick can never select it — and a press that grabs an anchor
/// while the Inspector goes on describing whatever was selected before leaves
/// the artist authoring one thing and reading about another.
///
/// Mutation-tested: dropping the two lines leaves §12 showing the previous
/// selection through the whole drag, and this goes red.
#[test]
fn grabbing_an_anchor_selects_its_joint() {
    let block = joint_anchor_down_block();
    assert!(
        block.contains("hero.gizmo.selection = Some(joint.to_bits())"),
        "grabbing an anchor no longer selects its joint — §12 would keep talking \
         about whatever was selected before the press"
    );
    assert!(
        block.contains("hero.gizmo.extra_selection.clear()"),
        "the previous multi-selection survives the grab, so the Inspector shows a \
         group while a single joint's anchor is being authored"
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

/// **The paint pass draws the published point view, LAST among the gizmos.**
///
/// Two properties. Drawing it at all: without the call the shell can publish
/// handles every frame that are never drawn and never registered — the dots are
/// invisible and unclickable, with every behavioural gate still green (they call
/// `paint_point_gizmo` directly).
///
/// And drawing it **after every box gizmo**, which is the z-order Enio asked for
/// (2026-07-25: *"devem ter o Z index mais alto que os outros objetos"*).
/// `HitIndex::hit` walks backwards, so the last registration wins the pixel: an
/// anchor that shares a spot with a sprite's corner handle is grabbed as the
/// anchor. A joint has no sprite to pick and no box of its own, so losing that
/// pixel is not a cosmetic detail — it is the handle becoming unreachable.
///
/// Mutation-tested: moving the call back above the extras/global block goes red.
#[test]
fn the_paint_pass_draws_the_point_gizmo_last() {
    let paint = fs::read_to_string("../../crates/ph2d-editor-core/src/screens/hero/paint.rs")
        .expect("the hero paint pass");
    let point = paint.find("paint_point_gizmo(").expect(
        "the hero paint pass no longer draws the point gizmo — a published \
         joint-anchor handle would never reach the screen or the hit index",
    );
    assert!(
        paint[..point].contains("point_view"),
        "the point gizmo is drawn from something other than `hero.gizmo.point_view`"
    );
    for (what, needle) in [
        ("the primary sprite gizmo", "paint_sprite_gizmo(scene"),
        ("the extra / global gizmos", "GizmoTarget::Global"),
    ] {
        let other = paint.find(needle).expect("the box gizmo pass");
        assert!(
            other < point,
            "the anchor handles are registered BEFORE {what}, so a box handle \
             painted afterwards wins the pixel and the joint's dot cannot be \
             grabbed where the two overlap"
        );
    }
}

/// **The published anchors come from the bridge, for the whole scene, at rest.**
///
/// Three properties of the one publish argument:
///  - it is `point_gizmo::joint_anchor_handles`, which resolves every anchor
///    through `PhysicsBridge::joint_anchor_world` — the SAME door
///    `sync_joint_pivots` uses for the A pivot. A second derivation here is how
///    two dots come to describe different frames (W-AnchorFollow paid 1.771 m
///    for that lesson);
///  - it is handed the BRIDGE and the sim, not a selection — the handles are for
///    every joint (W-J2b), and a selection argument is the shape that made them
///    reachable only through the Hierarchy;
///  - it is gated on the clock, or the dots would take drags against a swinging
///    body and author a pose nobody chose.
#[test]
fn the_published_anchors_come_from_the_bridge_door_for_every_joint() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let start = src
        .find("self.joint_body_pick,")
        .expect("the publish call's joint arguments are gone");
    let block = &src[start..(start + 1600).min(src.len())];
    let call = block
        .find("joint_anchor_handles(")
        .map(|i| &block[i..block[i..].find(')').map_or(block.len(), |e| i + e)])
        .expect("the publish site no longer builds the anchor handles");
    assert!(
        call.contains("physics"),
        "the handles are built without the bridge, so they cannot be coming from \
         its anchor door: {call}"
    );
    assert!(
        !call.contains("selection"),
        "the publish site still narrows the handles by selection — a joint has no \
         sprite, so that makes them reachable only by finding it in the Hierarchy \
         first: {call}"
    );
    assert!(
        call.contains("is_playing"),
        "the handles are no longer gated on the clock — they would take drags \
         against a swinging body and author a pose nobody chose: {call}"
    );
}
