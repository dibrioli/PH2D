//! **Two §11 gestures are about the SELECTION, and must not be fanned out.**
//!
//! `Join` (W3) is one click about a PAIR. `Bake` (W4) is one click about the
//! whole selection, satisfied by ONE run of the simulation. Both arrive through
//! the same `InspectorPhysicsEdit` action as the ordinary per-entity edits, and
//! both mean the opposite of what fanning out would do.
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

/// **Bake is intercepted before the fan-out too (W4).**
///
/// The failure is quieter than Join's and more expensive. A fanned-out bake
/// produces the *right numbers* — the simulation is deterministic, so every run
/// agrees — while re-simulating the entire scene once per selected body, and
/// filing a separate undo step for each. Nothing looks wrong: the curves are
/// correct, the scene is correct, and undoing "the bake" simply takes as many
/// Ctrl+Z presses as there were objects, which reads as the undo being flaky
/// rather than the bake being wrong.
#[test]
fn bake_is_intercepted_before_the_per_entity_fan_out() {
    let arm = physics_edit_arm();

    let bake_at = arm.find("PhysicsFieldEdit::Bake").expect(
        "the physics edit arm does not mention Bake at all — it is being \
         treated as an ordinary per-entity edit, so baking a selection of N \
         bodies runs the whole simulation N times and leaves N undo steps",
    );
    let fan_out_at = arm.find("for &t in &inspector_selection").expect(
        "the physics edit arm no longer fans out over the selection — if that \
         is deliberate this gate should be deleted along with it",
    );
    assert!(
        bake_at < fan_out_at,
        "Bake is handled AFTER the fan-out over the selection, so it runs once \
         per selected body: the same simulation, N times over, and N undo steps \
         for one click"
    );
}

/// **A bake with nothing selected still bakes the entity the section is about.**
///
/// The fan-out's empty case is the single-selection case, and the interception
/// has to reproduce it or the button does nothing at all when exactly one body
/// is selected — which is the commonest way anyone will use it.
#[test]
fn the_bake_request_falls_back_to_the_inspected_entity() {
    let arm = physics_edit_arm();
    assert!(
        arm.contains("vec![entity_bits]"),
        "the Bake interception does not fall back to `entity_bits` when the \
         selection list is empty — the button would be dead in the ordinary \
         one-body case"
    );
}
