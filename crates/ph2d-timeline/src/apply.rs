//! [`apply_from_doc`] — the document-driven apply pass: sample every binding's
//! track at the playhead and write the resolved property into its entity.
//!
//! This is the general counterpart to [`crate::apply_sprite_animations`] (the
//! per-component path, kept for programmatic use). It resolves each
//! [`TargetBinding`] through the sprite convention: the five `Transform`
//! properties natively (via `ph2d-ecs`), and `Opacity` into `Sprite.tint[3]`
//! behind the optional `render` feature (so the base crate stays free of the
//! GPU dependency). A binding whose entity is dead is flagged `missing` and
//! skipped — never a silent no-op (P6).

use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_ecs::{Entity, Transform, World};

use crate::doc::TimelineDoc;
use crate::prop::PropKind;
use crate::sprite::SpriteProp;
use crate::{stack_eval, stack_frames};

/// Sample every binding in `doc`'s active clip at time `t` (seconds) and write
/// the resolved value into each bound entity. Updates each binding's `missing`
/// flag by liveness. Call once per frame after advancing the Playhead.
pub fn apply_from_doc(world: &mut World, doc: &mut TimelineDoc, t: f64) {
    apply_from_doc_except(world, doc, t, |_| false);
}

/// Like [`apply_from_doc`], but leaves every entity whose bits `skip` claims
/// untouched — the caller owns those transforms this frame: the one under a
/// live gizmo drag, and any pose the user displaced while paused that is
/// waiting for a manual K (the displaced-pose pin). Their `missing` flags are
/// still refreshed; only the write is skipped. With auto-key armed the drag
/// records keys, so once the gesture ends `apply_from_doc` resumes on the newly
/// recorded pose and the entity holds.
pub fn apply_from_doc_except(
    world: &mut World,
    doc: &mut TimelineDoc,
    t: f64,
    skip: impl Fn(u64) -> bool,
) {
    refresh_liveness_and_rest(world, doc);

    // Pass 2 — the live strips and, inside each, one clock per remapped ENTITY.
    // Never one per binding: that was the quadratic (`clock.rs`).
    let mut scratch = doc.take_scratch();
    scratch.rebuild(doc, t);
    let stacked = !doc.stack().is_empty();

    // Pass 3 — write. Reads the document, writes the world: no `&mut doc` here,
    // so the binding list is a plain slice.
    for b in doc.bindings() {
        if b.missing {
            continue;
        }
        // The user owns this entity's transform this frame (gizmo drag /
        // displaced-pose pin) — don't clobber it from the document.
        if skip(b.entity) {
            continue;
        }
        // Time Remap is the timeline's own meta-property, never a scene write:
        // it was CONSUMED above as the entity's sampling clock.
        if b.prop == PropKind::TimeRemap {
            continue;
        }
        // THE branch (ADR-0115 §6): an empty stack is the single-clip path, on
        // the same code it always ran. A stack blends its lanes instead.
        let sampled = if stacked {
            stack_eval::sample_stack(
                doc,
                &scratch,
                stack_eval::Query {
                    entity: b.entity,
                    target: b.target,
                    prop: b.prop,
                    rest: b.rest.unwrap_or(0.0),
                },
            )
            .map(AnimValue::Float)
        } else {
            // The active clip's authored duration cuts the solo clock too — the
            // scratch's solo entry was built at the cut (`stack_frames`), and the
            // clock must be read at the SAME instant it was built at.
            let t_solo = doc.clip_cut(doc.active_index(), t);
            let t_entity = stack_eval::solo_source_time(&scratch, b.entity, t_solo);
            // Skip empty tracks so a just-created binding never forces the
            // property to a default value.
            doc.active_clip().track(b.target).and_then(|tr| {
                if tr.is_empty() {
                    None
                } else {
                    Some(tr.sample(t_entity))
                }
            })
        };
        // Same rule as pass 1: a detached binding (`entity = 0`) has no object to write to,
        // and `from_bits` would panic rather than tell us so.
        if let (Some(v), Some(e)) = (sampled, Entity::try_from_bits(b.entity)) {
            // ⚠️ Perguntado só para Position, e o curto-circuito é o que importa: o
            // `auto_orient` varre os bindings, então fazê-lo por binding tornaria o
            // apply quadrático — a doença que o `clock.rs` já curou uma vez neste
            // mesmo laço.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(world, e, b, v, orient);
        }
    }

    doc.put_scratch(scratch); // capacity retained — zero-alloc next frame (HR-3)

    // ADR-0144 — the SEPARATE post-composition pass: after every keyed prop is
    // written, expressions read the composed values and overwrite the driven
    // props. Early-out (byte-identical) when no binding carries a formula; never
    // touches `stack_eval`/the blend, so the fade is untouched by construction.
    crate::expr_pass::run(world, doc, t);
}

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
}

/// Pass 1 — liveness (P6), and the one chance to capture `rest`.
///
/// It precedes the clock pass on purpose: a dead entity's Time Remap track must
/// not drive anything, and resolving clocks against last frame's flags would let
/// it. And `rest` is read HERE, before any write this frame — the world still
/// holds the pose the animator left the object in.
///
/// **One door.** Both the stacked apply and the solo apply
/// ([`apply_active_clip`]) refresh liveness the exact same way; a second copy
/// would drift on the next mine (the `try_from_bits` sentinel below was one such
/// mine already).
fn refresh_liveness_and_rest(world: &mut World, doc: &mut TimelineDoc) {
    let n = doc.bindings().len();
    for i in 0..n {
        let (entity_bits, prop) = {
            let b = &doc.bindings()[i];
            (b.entity, b.prop)
        };
        // **`try_from_bits`, never `from_bits`.** A binding can be DETACHED — `entity = 0`,
        // which is the sentinel [`crate::resolve_entities`] writes for "this one has no live
        // object; find it by `wire_id`" (a project just loaded is nothing BUT detached
        // bindings). And `0` is not a null in bevy: the index is a `NonZero<u32>`, so
        // `Entity::from_bits(0)` **panics** — it does not return a dead entity.
        //
        // Pass 1 is the pass that DECIDES `missing`, so it cannot skip a binding to avoid
        // decoding it. Decode fallibly instead: undecodable bits are exactly what `missing`
        // means. (This was a live mine the day the loader started using the sentinel it was
        // already documented to write.)
        let entity = Entity::try_from_bits(entity_bits);
        let alive = entity.is_some_and(|e| world.get_entity(e).is_ok());
        doc.bindings_mut()[i].missing = !alive;
        if let Some(entity) = entity.filter(|_| alive)
            && doc.bindings()[i].rest.is_none()
            && prop != PropKind::TimeRemap
        {
            // Position's `rest` is a DISTANCE along its path — the track's units —
            // so it is read through the trajectory, never off a Transform field.
            let rest = if prop == PropKind::Position {
                crate::apply_path::read_rest(world, entity, &doc.bindings()[i])
            } else {
                read_prop(world, entity, prop)
            };
            doc.bindings_mut()[i].rest = rest;
        }
    }
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
/// it is the empty-stack branch of [`apply_from_doc_except`], reached WITHOUT a
/// stack present — the scratch is not needed because there is only one clip and no
/// blend to resolve.
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
            // ⚠️ Perguntado só para Position, e o curto-circuito é o que importa: o
            // `auto_orient` varre os bindings, então fazê-lo por binding tornaria o
            // apply quadrático — a doença que o `clock.rs` já curou uma vez neste
            // mesmo laço.
            let orient = b.prop == PropKind::Position
                && doc.auto_orient(b.entity) == crate::AutoOrient::Active;
            write_prop(world, e, b, v, orient);
        }
    }

    // ADR-0144 — the expression pass, on the clip's own clock (the same `clip_t`
    // the keyed props were sampled at). Early-out when nothing is driven.
    crate::expr_pass::run(world, doc, clip_t);
}

/// Read one property back out of an entity — the exact inverse of
/// [`write_prop`], and the reason `rest` can be captured without the shell's
/// help. Also what the inverse-blend key authoring will need (ADR-0115 R9).
pub(crate) fn read_prop(world: &World, entity: Entity, prop: PropKind) -> Option<f32> {
    if let Some(sp) = prop.as_sprite_transform() {
        let xf = world.get::<Transform>(entity)?;
        return Some(match sp {
            SpriteProp::TranslationX => xf.translation.x,
            SpriteProp::TranslationY => xf.translation.y,
            SpriteProp::Rotation => xf.rotation,
            SpriteProp::ScaleX => xf.scale.x,
            SpriteProp::ScaleY => xf.scale.y,
        });
    }
    #[cfg(feature = "render")]
    if prop == PropKind::Opacity {
        return world
            .get::<ph2d_render::Sprite>(entity)
            .map(|sprite| sprite.tint[3]);
    }
    let _ = (world, entity);
    None
}

/// The time `entity`'s tracks sample at: its Time Remap track's value at the
/// playhead — the AE model (slope < 1 slow motion, flat freeze, negative
/// reverse) — or the playhead itself when it has none / an empty track (the
/// identity: binding "Time" changes nothing until keys are authored).
///
/// The **single-entity** path, for key authoring outside the frame loop
/// (auto-key's diff, the displaced-pose pin, `K`). The apply resolves the whole
/// document at once through [`crate::clock::ClockIndex`] — every caller here
/// asks about one entity, so a scan is the right shape.
///
/// Seeding and sampling MUST come through this same function: a derived
/// coordinate whose author uses a different transform than its reader is the bug
/// that cost this module three rounds.
///
/// Zero-alloc (HR-3, the paused bridge path is gated).
pub fn remapped_time(doc: &TimelineDoc, entity: u64, t: f64) -> f64 {
    crate::clock::remapped_time_in(doc.active_clip(), doc.bindings(), entity, t)
}

/// **Where a key authored right now lands**, in the active clip's own time — or
/// `None` when there is no single answer and the key must be refused.
///
/// A track is authored in the clip's time, and the playhead runs in the
/// timeline's. Without a stack the only gap between them is the entity's Time
/// Remap, and [`remapped_time`] closes it. With a stack there is a second map on
/// top (the strip's), and it can fail to be a function: if the clip you are
/// editing is playing **twice** at this instant, "here" names two places in it,
/// and choosing one silently would drop the key somewhere the animator never
/// looked. If it is playing **zero** times, "here" names none.
///
/// Seeding and sampling go through the same composition — the strip's map, then
/// the clip's own clock — because a derived coordinate written by one transform
/// and read by another is the bug that has broken this module three times over.
#[must_use]
pub fn key_time(doc: &TimelineDoc, entity: u64, t: f64) -> Option<f64> {
    key_home(doc, entity, t).ok()
}

/// [`key_time`], with the **reason** when there is no answer.
///
/// The refusal is the whole point (ADR-0115 R9), and a refusal nobody can see is
/// indistinguishable from a bug: the animator drags, the object snaps back, and
/// nothing says why. The evaluator is the only place that knows which of the three
/// things went wrong, so it is the place that names it — the shell only speaks it.
///
/// `key_time` is this, thrown away: every caller that does not surface the reason
/// says so by calling that one instead.
pub fn key_home(doc: &TimelineDoc, entity: u64, t: f64) -> Result<f64, crate::KeyRefusal> {
    // **Which stack gates the answer is the SCRATCH's root**, not the scene's: a
    // Containers editing view primes rooted (`prime_rooted`), and there "is there a
    // stack between the playhead and the clip?" is a question about the CONTAINER's
    // lanes. Asking `doc.stack()` here kept authoring inside a container hostage to
    // whatever the SCENE happened to hold (Enio, 2026-07-22).
    if doc.scratch().root().is_none() && doc.stack().is_empty() {
        // The active clip's authored duration cuts this clock BEFORE the remap —
        // the same composition both solo lanes of the apply run
        // ([`apply_active_clip`] and the empty-stack lane of
        // [`apply_from_doc_except`], `clip_cut` then the entity's clock). Beyond
        // the cut the apply freezes every pose at `curve(cut)`, so "where does a
        // key authored right now land" is the BOUNDARY — the frame the animator
        // is looking at. Handing back the raw time here dropped keys at instants
        // the apply never samples: scrubbing past the authored end with AutoKey
        // armed minted a key per frame off the frozen pose (the 2026-07-23
        // superbug; [[feedback_derived_coordinate_seed_must_match_sample]]).
        // The stacked branch below never had the hole — each strip's `t_clip`
        // was built through `cut_source`.
        let t = doc.clip_cut(doc.active_index(), t);
        return Ok(remapped_time(doc, entity, t));
    }
    let scratch = doc.scratch();
    debug_assert_scratch_at(scratch, t);
    let strip = stack_eval::sole_strip_of(scratch, doc.active_index())?;
    // The strip is live (`sole_strip_of` said so), so its clock has an entry —
    // `NotPlaying` here would mean the scratch disagreed with itself.
    stack_eval::strip_source_time(scratch, &strip, entity).ok_or(crate::KeyRefusal::NotPlaying)
}

/// **Where the active clip's own clock stands at timeline time `t`** — or why it
/// has no single answer.
///
/// This is [`key_home`] with the entity half taken off: the OUTER map only
/// (timeline → clip), which is what a *ruler* measures. `key_home` then composes
/// the INNER map (that clip's Time Remap for one entity) on top; a ruler has no
/// entity to ask, and the tracks it rules over may each carry a different one.
///
/// The two share [`stack_eval::sole_strip_of`] deliberately: "is the clip I am
/// editing playing exactly once right now" must have ONE answer, or the ruler
/// would draw a playhead at an instant K refuses to key
/// ([[feedback_two_doors_to_the_same_question_diverge]]).
///
/// **Without a stack the clip IS the timeline**, so the answer is `t` itself —
/// which is why a document that never touches the feature sees a ruler that
/// behaves exactly as it always has.
///
/// Same precondition as [`key_home`]: under a stack the answer comes from the
/// scratch, so the caller must have primed it at `t`
/// ([`TimelineDoc::prime_stack`]).
pub fn clip_playhead(doc: &TimelineDoc, t: f64) -> Result<f64, crate::KeyRefusal> {
    // Root-aware for the same reason as [`key_home`]: rooted, the gate is the
    // container's lanes, not the scene's.
    if doc.scratch().root().is_none() && doc.stack().is_empty() {
        return Ok(t);
    }
    let scratch = doc.scratch();
    debug_assert_scratch_at(scratch, t);
    Ok(stack_eval::sole_strip_of(scratch, doc.active_index())?.t_clip)
}

/// Hold the caller to the promise that the scratch describes `t`.
///
/// Under a stack every answer here comes from the SCRATCH, so these functions are
/// only as honest as that promise. In production the apply has just built it at
/// this same instant, which is exactly what makes the coupling invisible — and
/// exactly the shape of bug that has broken this module three times over
/// ([[feedback_derived_coordinate_seed_must_match_sample]]). Check it rather than
/// trust it: a reader pointed at another instant gets a confidently wrong answer.
fn debug_assert_scratch_at(scratch: &stack_frames::StackScratch, t: f64) {
    debug_assert!(
        scratch.built_at().to_bits() == t.to_bits(),
        "the stack scratch describes t={}, not t={t}: prime_stack first",
        scratch.built_at()
    );
}

/// Write one resolved property value into an entity, via the sprite resolver.
///
/// Takes the whole BINDING, not just the kind: [`PropKind::Position`]'s value is a
/// distance, and turning it back into a place needs that binding's trajectory.
pub(crate) fn write_prop(
    world: &mut World,
    entity: Entity,
    b: &crate::TargetBinding,
    v: AnimValue,
    orient: bool,
) {
    let prop = b.prop;
    // Position (uma distância → ponto, ADR-0141) e os cinco de sprite-transform são POSE:
    // vão pela porta única `pose::set_transform_field`, a MESMA que o `pose_at` do onion
    // usa, para que não haja uma 2ª derivação da pose a divergir (ADR-0142).
    if prop == PropKind::Position || prop.as_sprite_transform().is_some() {
        if let Some(mut xf) = world.get_mut::<Transform>(entity) {
            crate::pose::set_transform_field(&mut xf, b, v, orient);
        }
        return;
    }
    // The Morph `t` (the Vector line's animatable channel). `VecMorph` lives in `ph2d-ecs`, which
    // this crate already depends on for `Transform`/`World` — no feature gate, no new dep. The
    // clamp is the motor's (`Plan::at`), so an ease with overshoot (back/elastic) touches the end
    // and stops instead of breaking the shape.
    if prop == PropKind::Morph
        && let AnimValue::Float(f) = v
        && let Some(mut m) = world.get_mut::<ph2d_ecs::VecMorph>(entity)
    {
        m.t = f;
        return;
    }
    // Non-Transform properties. `Opacity` needs the render crate (Sprite lives
    // there); gated so the base timeline runtime stays GPU-free.
    #[cfg(feature = "render")]
    if prop == PropKind::Opacity
        && let AnimValue::Float(f) = v
        && let Some(mut sprite) = world.get_mut::<ph2d_render::Sprite>(entity)
    {
        sprite.tint[3] = f.clamp(0.0, 1.0);
    }
    #[cfg(not(feature = "render"))]
    let _ = (world, entity, prop);
}

#[cfg(test)]
mod time_remap_tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_core::Vec2;

    /// A world with one sprite entity whose TranslationX is keyed 0 → 10 over
    /// `0..4 s` (linear: `x(t_source) = 2.5·t_source`).
    fn rig() -> (World, Entity, TimelineDoc) {
        let mut w = World::new();
        let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        for (t, v) in [(0.0, 0.0), (4.0, 10.0)] {
            doc.insert_key(
                e.to_bits(),
                PropKind::TranslationX,
                s(t),
                AnimValue::Float(v),
                Interp::Linear,
            );
        }
        (w, e, doc)
    }

    fn x_at(w: &mut World, e: Entity, doc: &mut TimelineDoc, t: f64) -> f32 {
        apply_from_doc(w, doc, t);
        w.get::<Transform>(e).unwrap().translation.x
    }

    fn remap_key(doc: &mut TimelineDoc, e: Entity, t: f64, source: f32, interp: Interp) {
        doc.insert_key(
            e.to_bits(),
            PropKind::TimeRemap,
            RationalTime::from_seconds(t),
            AnimValue::Float(source),
            interp,
        );
    }

    #[test]
    fn a_remap_curve_retimes_every_other_track_of_its_entity() {
        // Remap (0 → 0, 2 → 4): the playhead covers the 4 s animation in 2 s —
        // double speed. At playhead 1 the source time is 2 → x = 5.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 2.0, 4.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 5.0, "2x speed");
        assert_eq!(
            x_at(&mut w, e, &mut doc, 2.0),
            10.0,
            "done in half the time"
        );
    }

    #[test]
    fn a_flat_remap_freezes_and_a_falling_one_reverses() {
        let (mut w, e, mut doc) = rig();
        // Freeze: a single key holds source time at 2 s → x pinned at 5.
        remap_key(&mut doc, e, 0.0, 2.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 5.0);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 5.0, "frozen while t moves");
        // Reverse: remap (0 → 4, 4 → 0) plays the animation backwards.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 4.0, Interp::Linear);
        remap_key(&mut doc, e, 4.0, 0.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 10.0, "starts at the end");
        assert_eq!(x_at(&mut w, e, &mut doc, 4.0), 0.0, "ends at the start");
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 7.5, "backwards through 3 s");
    }

    #[test]
    fn a_single_seeded_time_key_keeps_the_identity_clock() {
        // THE "Time bugado" case: K seeds one identity key (a non-Hold interp).
        // The track's flat-hold would freeze the clock at that one value for
        // every playhead time — the sprite snapped back and no further pose
        // could be authored. Outside the keyed range the clock must extrapolate
        // at slope 1: one seeded key behaves exactly like zero keys.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 1.0, 1.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 1.0), 2.5, "on the key: identity");
        assert_eq!(
            x_at(&mut w, e, &mut doc, 3.0),
            7.5,
            "after the key the clock keeps advancing (x(3) = 7.5, not frozen at 2.5)"
        );
        assert_eq!(
            x_at(&mut w, e, &mut doc, 0.0),
            0.0,
            "before the key too (x(0) = 0, not held at the key's value)"
        );
    }

    #[test]
    fn extrapolation_continues_a_ramp_at_normal_speed_but_a_hold_last_key_freezes() {
        // 2x ramp (0 → 0, 1 → 2): past the last key the clock resumes at
        // slope 1 from where the ramp left it — source 2 + (t − 1).
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 1.0, 2.0, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 10.0, "source 4 at t = 3");
        // The same ramp with a HOLD last key freezes from there on (the
        // deliberate AE freeze-frame survives the extrapolation rule).
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        remap_key(&mut doc, e, 1.0, 2.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 3.0), 5.0, "held at source 2");
        // Extrapolating backwards below zero clamps like every playhead time.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 2.0, 0.5, Interp::Linear);
        assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 0.0, "clamped at source 0");
    }

    #[test]
    fn an_empty_or_missing_remap_track_is_the_identity() {
        // Bind "Time" with no keys: nothing changes until the user authors.
        let (mut w, e, mut doc) = rig();
        doc.bind(e.to_bits(), PropKind::TimeRemap);
        assert_eq!(x_at(&mut w, e, &mut doc, 2.0), 5.0, "identity: x(2) = 5");
        // And another entity's remap never leaks onto this one.
        let stranger = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        remap_key(&mut doc, stranger, 0.0, 99.0, Interp::Hold);
        assert_eq!(x_at(&mut w, e, &mut doc, 2.0), 5.0, "per-entity clock");
    }

    #[test]
    fn the_remap_track_itself_never_writes_a_scene_property() {
        // A Time key whose VALUE is huge must not bleed into any Transform
        // field — the remap binding is consumed as a clock, never written.
        let (mut w, e, mut doc) = rig();
        remap_key(&mut doc, e, 0.0, 0.0, Interp::Linear);
        let before = *w.get::<Transform>(e).unwrap();
        apply_from_doc(&mut w, &mut doc, 0.0);
        let after = *w.get::<Transform>(e).unwrap();
        assert_eq!(before.translation.y, after.translation.y);
        assert_eq!(before.rotation, after.rotation);
        assert_eq!(before.scale, after.scale);
        assert_eq!(
            after.translation.x, 0.0,
            "x follows its own track at source 0"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_core::Vec2;

    /// The live-dragged entity keeps its manipulated transform; every other
    /// bound entity is still driven from the document.
    #[test]
    fn live_entity_is_not_clobbered_by_the_document() {
        let mut w = World::new();
        let dragged = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let other = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        // Both entities have a TranslationX key pinning them to 5.0 at t=0.
        for e in [dragged, other] {
            doc.insert_key(
                e.to_bits(),
                PropKind::TranslationX,
                s(0.0),
                AnimValue::Float(5.0),
                Interp::Hold,
            );
        }
        // Simulate a gizmo drag having just written the dragged entity to 42.
        w.get_mut::<Transform>(dragged).unwrap().translation.x = 42.0;

        let dragged_bits = dragged.to_bits();
        apply_from_doc_except(&mut w, &mut doc, 0.0, |bits| bits == dragged_bits);

        assert_eq!(
            w.get::<Transform>(dragged).unwrap().translation.x,
            42.0,
            "the live-dragged entity must keep its manipulated pose (document must not fight it)"
        );
        assert_eq!(
            w.get::<Transform>(other).unwrap().translation.x,
            5.0,
            "every other bound entity is still driven from the document"
        );

        // With nothing skipped, the document reclaims the dragged one too.
        apply_from_doc_except(&mut w, &mut doc, 0.0, |_| false);
        assert_eq!(w.get::<Transform>(dragged).unwrap().translation.x, 5.0);
    }
}

#[cfg(all(test, feature = "render"))]
mod render_tests {
    use super::*;
    use crate::TimelineDoc;
    use ph2d_anim::{Interp, RationalTime};
    use ph2d_render::Sprite;

    #[test]
    fn opacity_drives_sprite_tint_alpha() {
        let mut w = World::new();
        let e = w
            .spawn(Sprite::atlas(0, [10.0, 10.0], [1.0, 1.0, 1.0, 1.0]))
            .id();
        let mut doc = TimelineDoc::new();
        let s = RationalTime::from_seconds;
        doc.insert_key(
            e.to_bits(),
            PropKind::Opacity,
            s(0.0),
            AnimValue::Float(1.0),
            Interp::Linear,
        );
        doc.insert_key(
            e.to_bits(),
            PropKind::Opacity,
            s(2.0),
            AnimValue::Float(0.0),
            Interp::Hold,
        );

        apply_from_doc(&mut w, &mut doc, 1.0); // midpoint → alpha 0.5
        let a = w.get::<Sprite>(e).unwrap().tint[3];
        assert!(
            (a - 0.5).abs() < 1e-5,
            "opacity animated tint alpha to 0.5, got {a}"
        );
    }
}
