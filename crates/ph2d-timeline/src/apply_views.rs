//! The two SOLO views of [`apply_from_doc`](crate::apply::apply_from_doc): the
//! active clip alone ([`apply_active_clip`], the panel's **Keys** tab) and a
//! container's interior alone ([`apply_container`], the Containers edit view).
//!
//! Both are siblings of the scene apply — same evaluator, one clip / one container
//! resolved in isolation — split out of `apply.rs` for the LOC cap. Each ends in the
//! ADR-0144 expression pass, on ITS cut clock, honouring `skip` and the `composed`
//! coverage the keyed pass built (so an expression rides its strip, never the world).

use std::collections::BTreeMap;

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, World};

use crate::apply::{refresh_liveness_and_rest, remapped_time};
use crate::apply_prop::write_prop;
use crate::doc::TimelineDoc;
use crate::frame_solve::{self, LinkFrame};
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
    // ADR-0146 W3 — the container view composes in TOPOLOGICAL order too, so a per-clip
    // prop-link (`value + Sprite.x`) inside the container reads the source's already-faded
    // value in the SAME frame, exactly as the scene apply does (`apply_from_doc_except`).
    // A formula-free container builds NEITHER the names map nor the order — `order` is
    // `None`, the loop walks the natural binding order, and nothing allocates (HR-3).
    let scheduled = frame_solve::any_formula(doc);
    let mut links = LinkFrame::default();
    let order = scheduled.then(|| {
        links.names = frame_solve::build_names(world, doc);
        frame_solve::seed_links(world, doc, &mut links.links);
        // …and the sources the timeline does NOT animate, straight from the world. This is
        // what makes `Follow` follow an object the artist merely placed (the report).
        frame_solve::seed_unbound_links(world, doc, &links.names, &mut links.links);
        frame_solve::topo_order(doc, &links.names)
    });
    let n = doc.bindings().len();
    for idx in 0..n {
        let bi = order.as_ref().map_or(idx, |o| o[idx]);
        let b = &doc.bindings()[bi];
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
            &links,
        );
        if let (Some(f), Some(e)) = (sampled, Entity::try_from_bits(b.entity)) {
            if scheduled {
                links.links.insert((b.entity, b.prop), f64::from(f));
                composed.insert((b.entity, b.prop), f);
            }
            // ⚠️ Perguntado só para Position, e o curto-circuito é o que importa: o
            // `auto_orient` varre os bindings, então fazê-lo por binding tornaria o
            // apply quadrático — a doença que o `clock.rs` já curou uma vez neste
            // mesmo laço.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(
                world,
                e,
                b,
                doc.path_for(b.target),
                AnimValue::Float(f),
                orient,
            );
        }
    }

    // ADR-0146 W2 — only GLOBAL drivers run in the post-pass now, on the CONTAINER's CUT
    // local clock; the container's per-clip expressions were resolved as lane sources in the
    // blend above (`sample_stack` -> `eval_frame`).
    let expr_t = doc.container_cut(container, t);
    crate::expr_pass::run(world, doc, expr_t, &skip, &composed, &mut links);
    // Persist the composed map so the container-view autokey reads what the apply wrote (C2, W6).
    scratch.composed_links = links;
    doc.put_scratch(scratch);
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
    // ADR-0146 W3 — the Keys solo composes in TOPOLOGICAL order too, so a per-clip
    // prop-link (`value + Sprite.x`) reads the source's already-faded value in the SAME
    // frame while editing the active clip, exactly as the scene apply does. A formula-free
    // clip builds NEITHER the names map nor the order — `order` is `None`, the loop walks
    // the natural binding order, and nothing allocates (HR-3).
    let scheduled = frame_solve::any_formula(doc);
    let mut links = LinkFrame::default();
    let order = scheduled.then(|| {
        links.names = frame_solve::build_names(world, doc);
        frame_solve::seed_links(world, doc, &mut links.links);
        // …and the sources the timeline does NOT animate, straight from the world. This is
        // what makes `Follow` follow an object the artist merely placed (the report).
        frame_solve::seed_unbound_links(world, doc, &links.names, &mut links.links);
        frame_solve::topo_order(doc, &links.names)
    });

    let n = doc.bindings().len();
    for idx in 0..n {
        let bi = order.as_ref().map_or(idx, |o| o[idx]);
        let b = &doc.bindings()[bi];
        if b.missing || skip(b.entity) || b.prop == PropKind::TimeRemap {
            continue;
        }
        // The entity's own clip clock — the same door the stacked/solo apply reads
        // and the same door K seeds through, so a soloed pose and a keyed one land
        // at the identical instant.
        let t_entity = remapped_time(doc, b.entity, clip_t);
        // The SECOND sample site (ADR-0146 W2, C1): a keyed sample OR a per-clip
        // expression over it. An empty track with no expression is None, so a just-created
        // binding never forces a default pose.
        let sampled = stack_eval::solo_source_value(
            doc,
            doc.active_index(),
            b.target,
            t_entity,
            b.rest.unwrap_or(0.0),
            &links,
        );
        if let (Some(f), Some(e)) = (sampled, Entity::try_from_bits(b.entity)) {
            if scheduled {
                links.links.insert((b.entity, b.prop), f64::from(f));
                composed.insert((b.entity, b.prop), f);
            }
            // ⚠️ auto_orient: por-binding tornaria o apply quadrático (o mesmo laço
            // que o `clock.rs` curou) — daí o curto-circuito, só para Position.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(
                world,
                e,
                b,
                doc.path_for(b.target),
                AnimValue::Float(f),
                orient,
            );
        }
    }

    // ADR-0146 W2 — on the clip's own CUT clock (`clip_t`, already clamped). Only GLOBAL
    // drivers run in the post-pass now; the active clip's per-clip expressions were resolved
    // as lane sources at the sample site above (`solo_source_value`).
    crate::expr_pass::run(world, doc, clip_t, &skip, &composed, &mut links);
    // Persist the composed map for the Keys-view autokey (`autokey_props_solo` -> `curve_value`,
    // C2, W6). This view keeps no local scratch, so stash straight onto the doc's.
    doc.stash_composed_links(links);
}
