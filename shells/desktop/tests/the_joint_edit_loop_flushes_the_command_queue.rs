//! **Arch-gate — the §12 joint-edit loop FLUSHES the editor command queue
//! (W-JointParams, 2026-07-25).**
//!
//! ## What this protects
//!
//! `apply_joint_edit` only QUEUES a `SetComponent`; the component changes when
//! `apply_editor_commands` drains the queue. Every OTHER Inspector edit type
//! flushes right after its own apply, inside `inspector_commits::dispatch` (§11
//! physics, ordering, blend, name…). The §12 joint block was deliberately kept
//! OUT of that dispatch (`render_loop/mod.rs`, the comment near the `joint_edits`
//! drain) — and shipped WITHOUT the flush. So a joint parameter edit sat in the
//! queue until some unrelated edit happened to drain it, which the artist
//! experiences as *"changing the parameters does nothing… sometimes it works"*.
//!
//! ## Why a source gate and not a unit test
//!
//! The flush lives in the render loop's per-frame body, which no unit test
//! reaches (it needs a window + the whole frame). The behavioral half —
//! `apply_joint_edit` only queues, `apply_editor_commands` lands it — is pinned
//! by `render_loop::inspector_joint_tests::a_joint_param_edit_lands_only_when_
//! the_queue_is_flushed`. This gate pins that the render loop actually MAKES that
//! call, in the joint loop, before it moves on. Mutation: delete the
//! `apply_editor_commands` call from the joint loop and this goes red.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` vanished from the render loop — if it was renamed, update this gate \
             (and confirm the §12 joint edit still flushes the command queue)"
        )
    })
}

#[test]
fn the_joint_edit_loop_flushes_the_command_queue() {
    let apply = at("inspector_joint::apply_joint_edit(");
    // The flush the fix added. `apply_editor_commands` appears nowhere else in
    // mod.rs (every other flush lives in inspector_commits.rs), so its mere
    // presence here IS the joint-loop flush.
    let flush = SRC.find("apply_editor_commands(").unwrap_or_else(|| {
        panic!(
            "the §12 joint-edit loop no longer flushes the editor command queue — \
             `apply_joint_edit` only QUEUES a SetComponent, so without the flush a \
             joint parameter edit sits in the queue until some other edit drains \
             it. That is the 'sometimes it works' bug (W-JointParams). Add \
             `apply_editor_commands(sim.world_mut(), editor_queue, component_registry)` \
             after `apply_joint_edit`, exactly as inspector_commits::dispatch does \
             for every other Inspector edit type."
        )
    });
    // The create block that follows the joint loop — the flush must be BEFORE it,
    // i.e. inside the loop, not stranded somewhere later.
    let create = at("inspector_joint::create_joint(");
    assert!(
        apply < flush && flush < create,
        "the flush must sit between `apply_joint_edit` and `create_joint` (i.e. \
         inside the joint-edit loop): apply@{apply} flush@{flush} create@{create}"
    );
}
