//! The two SOLO views of [`apply_from_doc`](crate::apply::apply_from_doc): the
//! active clip alone ([`apply_active_clip`], the panel's **Keys** tab) and a
//! container's interior alone ([`apply_container`], the Containers edit view).
//!
//! Both are siblings of the scene apply — same evaluator, one clip / one container
//! resolved in isolation — split out of `apply.rs` for the LOC cap. Each ends in the
//! ADR-0144 expression pass, on ITS cut clock, honouring `skip` and the `composed`
//! coverage the keyed pass built (so an expression rides its strip, never the world).

use std::collections::BTreeMap;

use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_ecs::{Entity, World};

use crate::apply::{refresh_liveness_and_rest, remapped_time, write_prop};
use crate::doc::TimelineDoc;
use crate::prop::PropKind;
use crate::stack_eval;

/// **Apply container `container`'s INTERIOR alone, at ITS local clock `t`** — the
/// Containers editing view's solo, the sibling of [`apply_active_clip`] one level up
/// (Enio, 2026-07-22: *"o playback deve ser relativo ao container aberto em edição"*).
///
/// Same evaluator as the scene, ROOTED (`rebuild_rooted`): the interior you watch
/// while editing and the interior an instance plays in the scene are ONE answer.
/// Sparsity holds — a channel no lane of the container keys is left untouched, so
/// objects outside the container keep the scene pose they had.
pub fn apply_container(
    world: &mut World,
    doc: &mut TimelineDoc,
    container: usize,
    t: f64,
    skip: impl Fn(u64) -> bool,
) {
    refresh_liveness_and_rest(world, doc);
    let mut scratch = doc.take_scratch();
    scratch.rebuild_rooted(doc, Some(container), t);
    // The coverage mask + pre-expression value the expression pass reads (ADR-0144);
    // built only with a formula present so the no-expression path stays zero-alloc.
    let mut composed: BTreeMap<(u64, PropKind), f32> = BTreeMap::new();
    let has_expr = doc.bindings().iter().any(|b| b.expr.is_some());
    for b in doc.bindings() {
        if b.missing || skip(b.entity) || b.prop == PropKind::TimeRemap {
            continue;
        }
        let sampled = stack_eval::sample_stack(
            doc,
            &scratch,
            stack_eval::Query {
                entity: b.entity,
                target: b.target,
                prop: b.prop,
                rest: b.rest.unwrap_or(0.0),
            },
        )
        .map(AnimValue::Float);
        if let (Some(v), Some(e)) = (sampled, Entity::try_from_bits(b.entity)) {
            if has_expr && let AnimValue::Float(f) = &v {
                composed.insert((b.entity, b.prop), *f);
            }
            // ⚠️ Perguntado só para Position, e o curto-circuito é o que importa: o
            // `auto_orient` varre os bindings, então fazê-lo por binding tornaria o
            // apply quadrático — a doença que o `clock.rs` já curou uma vez neste
            // mesmo laço.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(world, e, b, v, orient);
        }
    }
    doc.put_scratch(scratch);

    // ADR-0144 — on the CONTAINER's CUT local clock (`container_cut`), honouring
    // `skip` and the `composed` coverage: past the container's authored end (or where
    // the container does not key a scene object) an expression freezes with the keys.
    let expr_t = doc.container_cut(container, t);
    crate::expr_pass::run(world, doc, expr_t, &skip, &composed);
}

/// **Apply the ACTIVE CLIP ALONE at clip time `clip_t`, ignoring the stack** —
/// the AE precomp / "solo the clip" model the panel's **Keys** tab drives.
///
/// This is what makes editing a clip's keys honest: you see and pose exactly the
/// curves you are editing, so a lane above cannot hide the motion (the ADR-0115
/// R9 "Overridden" case never arises here — there IS no stack in this view).
///
/// The clip's own clock is per-entity ([`remapped_time`], the active clip's Time
/// Remap), so a Time-Remapped object stays remapped even when soloed. Structurally
/// it is the empty-stack branch of [`apply_from_doc_except`](crate::apply_from_doc_except),
/// reached WITHOUT a stack present — the scratch is not needed because there is only
/// one clip and no blend to resolve.
///
/// `skip` is honoured identically (the gizmo-dragged entity, the displaced pin).
pub fn apply_active_clip(
    world: &mut World,
    doc: &mut TimelineDoc,
    clip_t: f64,
    skip: impl Fn(u64) -> bool,
) {
    refresh_liveness_and_rest(world, doc);
    // The clip's authored duration cuts its own clock (Enio, 2026-07-23) — the
    // same cut the stacked path applies in `stack_frames`, so the Keys solo and
    // an instance of this clip freeze at the same instant.
    let clip_t = doc.clip_cut(doc.active_index(), clip_t);
    let mut composed: BTreeMap<(u64, PropKind), f32> = BTreeMap::new();
    let has_expr = doc.bindings().iter().any(|b| b.expr.is_some());

    for b in doc.bindings() {
        if b.missing || skip(b.entity) || b.prop == PropKind::TimeRemap {
            continue;
        }
        // The entity's own clip clock — the same door the stacked/solo apply reads
        // and the same door K seeds through, so a soloed pose and a keyed one land
        // at the identical instant.
        let t_entity = remapped_time(doc, b.entity, clip_t);
        // Skip empty tracks so a just-created binding never forces a default pose.
        let sampled = doc.active_clip().track(b.target).and_then(|tr| {
            if tr.is_empty() {
                None
            } else {
                Some(tr.sample(t_entity))
            }
        });
        if let (Some(v), Some(e)) = (sampled, Entity::try_from_bits(b.entity)) {
            if has_expr && let AnimValue::Float(f) = &v {
                composed.insert((b.entity, b.prop), *f);
            }
            // ⚠️ auto_orient: por-binding tornaria o apply quadrático (o mesmo laço
            // que o `clock.rs` curou) — daí o curto-circuito, só para Position.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(world, e, b, v, orient);
        }
    }

    // ADR-0144 — on the clip's own CUT clock (`clip_t`, already clamped), honouring
    // `skip` and the `composed` coverage.
    crate::expr_pass::run(world, doc, clip_t, &skip, &composed);
}
