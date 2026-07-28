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
//!
//! ⚠️ **The "inside the loop" half is asserted by BRACE MATCHING, not by a
//! landmark that follows.** The first version bounded the flush with the block
//! that came next (`create_joint(`) — and W-J4 legitimately replaced that call
//! with `join_chain(`, so a gate about the FLUSH went red over a rename it had no
//! opinion about. A byte landmark is a proxy that expires; the loop's own extent
//! is the property.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// The byte range of the `for … in &joint_edits { … }` body, by brace matching.
fn joint_loop_body() -> (usize, usize) {
    let head = SRC
        .find("for &(bits, edit) in &joint_edits {")
        .unwrap_or_else(|| {
            panic!(
                "the §12 joint-edit loop vanished from the render loop — if it was \
                 restructured, update this gate (and confirm the edit still flushes \
                 the editor command queue per edit)"
            )
        });
    let open = SRC[head..].find('{').expect("the loop opens a block") + head;
    let mut depth = 0usize;
    for (i, c) in SRC[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return (open, open + i);
                }
            }
            _ => {}
        }
    }
    panic!("the joint-edit loop body is never closed");
}

#[test]
fn the_joint_edit_loop_flushes_the_command_queue() {
    let (open, close) = joint_loop_body();
    let body = &SRC[open..close];
    // `apply_editor_commands` appears nowhere else in mod.rs (every other flush
    // lives in inspector_commits.rs), so its presence INSIDE this body IS the
    // joint-loop flush.
    let apply = body
        .find("inspector_joint::apply_joint_edit(")
        .unwrap_or_else(|| {
            panic!(
                "`apply_joint_edit` is no longer called inside the joint-edit loop — \
                 if the §12 edits moved, update this gate and confirm they still flush."
            )
        });
    let flush = body.find("apply_editor_commands(").unwrap_or_else(|| {
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
    // PER edit: the flush follows the apply within the SAME loop body, which is
    // the whole point — `apply_joint_edit` read-modify-writes the whole component,
    // so a second edit that read a not-yet-applied first one would drop it.
    assert!(
        apply < flush,
        "the flush must follow `apply_joint_edit` inside the loop body: \
         apply@{apply} flush@{flush} (body {open}..{close})"
    );
}

/// **Os DOIS verbos ESTRUTURAIS da §12 são despachados no MESMO laço** — o que
/// apaga a corda e o que acrescenta uma roldana.
///
/// Nenhum dos dois é uma edição de componente: `Remove` despawna a corda,
/// `AddWheel` spawna uma roldana. Os dois seguem por `else if` ANTES do
/// `apply_joint_edit`, e é por isso que este gate existe — um verbo estrutural que
/// caísse no braço de edição chamaria `joint_with_edit`, que devolve `None` para
/// ele, e o clique não faria nada em silêncio. Foi assim que a shell tratou o
/// `Remove` desde o W3, e é a forma que o `AddWheel` copia.
///
/// ⚠️ **Arch-gate porque este laço vive no corpo por-frame da render loop**, que
/// nenhum unit test alcança (precisa de janela). A metade COMPORTAMENTAL de cada
/// um é gateada onde ela mora: `adding_a_wheel_puts_it_on_the_rope` para o spawn,
/// e o despawn é o caminho que todo objeto toma.
///
/// Mutação: tire o braço do `AddWheel` e o clique cai no `apply_joint_edit`, que
/// não tem o que fazer com ele — este gate fica vermelho.
#[test]
fn the_structural_joint_verbs_are_dispatched_before_the_field_edits() {
    let (open, close) = joint_loop_body();
    let body = &SRC[open..close];
    let apply = body
        .find("inspector_joint::apply_joint_edit(")
        .expect("o braço de edição de campo");
    for (verb, call) in [
        ("Remove", "despawn("),
        ("AddWheel", "inspector_joint::add_pulley_wheel("),
    ] {
        let arm = body
            .find(&format!("JointFieldEdit::{verb}"))
            .unwrap_or_else(|| {
                panic!(
                    "`{verb}` não é mais despachado no laço da §12 — ele é ESTRUTURAL \
                 (spawn/despawn de objeto), não uma edição de campo, e cair no \
                 `apply_joint_edit` o faria virar um clique sem efeito."
                )
            });
        let done = body.find(call).unwrap_or_else(|| {
            panic!("`{verb}` é reconhecido mas nada chama `{call}` — o braço ficou vazio")
        });
        assert!(
            arm < apply && done < apply,
            "`{verb}` tem de ser resolvido ANTES do braço de edição de campo: \
             arm@{arm} call@{done} apply@{apply}"
        );
    }
}
