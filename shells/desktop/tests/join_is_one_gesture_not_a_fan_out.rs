//! **Join is one gesture over a PAIR, and must not be fanned out (W3).**
//!
//! Every other §11 physics edit is per-entity, so `render_loop` fans it out
//! over the selection — "make all of these static" is a gesture an artist
//! performs. `Join` arrives through that same action and means the opposite:
//! it is one click about two objects. Fanned out it would run once per
//! selected body and create **two joints between the same pair**, on the very
//! click that is supposed to create one.
//!
//! No unit test can see this. The rule lives inside `render_loop`'s action
//! drain — a function far too large and too entangled with the frame to drive
//! from a test — so the gate reads the source, exactly as
//! `the_z_projection_reads_the_tree_after_the_sync` does for frame ordering.
//!
//! It is deliberately a gate about the SHAPE of the code and not about a
//! literal: what it pins is that the interception exists *before* the fan-out,
//! which is the only place the distinction can be made.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// The window of source that handles `InspectorPhysicsEdit`.
fn physics_edit_arm() -> &'static str {
    let start = SRC
        .find("EditorAction::InspectorPhysicsEdit { entity_bits, edit } => {")
        .expect(
            "the §11 physics edit arm has been renamed — this gate points at \
             nothing and has to be re-aimed",
        );
    let rest = &SRC[start..];
    // The arm ends at the next top-level `EditorAction::` arm.
    let end = rest[1..]
        .find("                    EditorAction::")
        .map(|i| i + 1)
        .unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn join_is_intercepted_before_the_per_entity_fan_out() {
    let arm = physics_edit_arm();

    let join_at = arm.find("PhysicsFieldEdit::Join").expect(
        "the physics edit arm does not mention Join at all — it is being \
         treated as an ordinary per-entity edit, so a click on \"Join Selected \
         Bodies\" creates one joint PER selected body",
    );
    let fan_out_at = arm.find("for &t in &inspector_selection").expect(
        "the physics edit arm no longer fans out over the selection — if that \
         is deliberate this gate should be deleted along with it",
    );
    assert!(
        join_at < fan_out_at,
        "Join is handled AFTER the fan-out over the selection, so it runs \
         once per selected body. Two bodies selected means two joints between \
         the same pair, and the artist clicked once"
    );
}

#[test]
fn the_join_request_carries_exactly_two_bodies() {
    let arm = physics_edit_arm();
    assert!(
        arm.contains("if let [a, b] = inspector_selection[..]"),
        "the Join interception no longer destructures the selection as exactly \
         two entities. A `.first()`/`.get(1)` pair would silently accept three \
         selected bodies and join an arbitrary two of them"
    );
}
