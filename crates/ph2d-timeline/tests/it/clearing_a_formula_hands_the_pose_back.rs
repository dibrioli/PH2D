//! **Deleting a formula gives the property back** (FASE 0.4 do plano 12) — the half of
//! *"mesmo deletando as expressões, elas ficam atuando"* (Enio) that the audit of
//! 2026-07-29 actually MEASURED (§4 D-I).
//!
//! | | prop SEM keys | prop COM keys |
//! |---|---|---|
//! | before the formula | 0.0000 | 7.0000 |
//! | with `value + 250` | 250.0000 | 257.0000 |
//! | **after DELETE + apply** | **250.0000** ✗ | 7.0000 ✓ |
//! | one frame later | **250.0000** ✗ | — |
//!
//! **Mechanism:** the blend is deliberately SPARSE. `clip_anim_source` returns `None` for
//! a clip that neither keys nor drives the channel, `solo_source_value` propagates it, and
//! the apply's write is inside `if let (Some(f), Some(e))` — so **nobody writes**, and the
//! property keeps standing exactly where the formula left it. Sparsity is right (it is
//! what keeps a just-bound property from being forced to a default), and the price was
//! paid by the commonest case there is: a binding with no keys.
//!
//! ⚠️ **Why every existing gate was green over this:** they are all KEYED. On a keyed
//! property the curve rewrites the pose every frame, so *cleared* and *not cleared* are
//! indistinguishable — the fixture could not contain the phenomenon. The `expr_pass` did
//! own a hand-back (`take_restore`) and its doc-comment even explains the bare-binding
//! problem, but it was wired to ONE event: the end of a live PREVIEW.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

fn one(name: &str) -> (World, Entity) {
    let mut w = World::new();
    let e = w
        .spawn((Transform::from_translation(Vec2::ZERO), Name::new(name)))
        .id();
    (w, e)
}

fn x_at(w: &mut World, e: Entity, doc: &mut TimelineDoc, t: f64) -> f32 {
    apply_from_doc(w, doc, t);
    w.get::<Transform>(e).unwrap().translation.x
}

/// Stamp a PER-CLIP formula — the channel `Apply` writes (`set_clip_expr`), which is the
/// one the artist's gesture uses.
fn set_clip_formula(doc: &mut TimelineDoc, e: Entity, prop: PropKind, src: Option<&str>) {
    let tgt = doc.bind(e.to_bits(), prop);
    let clip = doc.active_index();
    doc.set_clip_expr(clip, tgt, src.map(str::to_string));
}

/// **A prop with NO KEYS gets its pose back when the formula is deleted.**
///
/// Born RED at 250.0000 — the number in the module table.
#[test]
fn clearing_a_formula_hands_the_pose_back_even_on_a_bare_binding() {
    let (mut w, e) = one("Ball");
    let mut doc = TimelineDoc::new();

    // The authored pose, and the frame that captures it as `rest`.
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        0.0,
        "starts where it stands"
    );

    set_clip_formula(&mut doc, e, PropKind::TranslationX, Some("value + 250"));
    let driven = x_at(&mut w, e, &mut doc, 0.0);
    assert_eq!(driven, 250.0, "PREMISE: the formula drives it");

    // The artist clears the row and presses Apply.
    set_clip_formula(&mut doc, e, PropKind::TranslationX, None);
    let after = x_at(&mut w, e, &mut doc, 0.0);
    assert_eq!(
        after, 0.0,
        "deleting the formula must give the property back; it stayed at {after}"
    );

    // ...and it STAYS given back. A hand-back that happens once and is then re-driven by
    // a stale ledger entry would read as a one-frame flicker.
    let later = x_at(&mut w, e, &mut doc, 0.05);
    assert_eq!(later, 0.0, "and it stays handed back on later frames");
}

/// **A KEYED prop is untouched by this** — the control, and the reason the defect hid.
///
/// The curve owns the pose, so the hand-back must NOT fire: firing here would undo the
/// artist's own animation on the frame they cleared a formula off it.
#[test]
fn a_keyed_property_is_rewritten_by_its_curve_and_never_by_the_hand_back() {
    let (mut w, e) = one("Ball");
    let mut doc = TimelineDoc::new();
    let s = RationalTime::from_seconds;
    for (t, v) in [(0.0, 7.0), (1.0, 7.0)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            s(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 7.0, "the curve holds it");

    set_clip_formula(&mut doc, e, PropKind::TranslationX, Some("value + 250"));
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        257.0,
        "PREMISE: over the key"
    );

    set_clip_formula(&mut doc, e, PropKind::TranslationX, None);
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        7.0,
        "the key answers for the channel — the hand-back must not fight it"
    );
}

/// **Hand-posing a bare bound property still works.**
///
/// ⚠️ This is the gate that rules out the tempting STATELESS cure — *"a binding with no
/// source writes its `rest`"*. That rule closes the reported defect and breaks something
/// the app depends on: a bound-but-unkeyed property is exactly what the artist poses by
/// hand before pressing K, and snapping it back to a `rest` captured minutes ago would
/// make the second keyframe impossible to author (the displaced-pose pin exists for this).
/// The hand-back is owed only where a formula WAS driving, never merely where nothing is.
#[test]
fn a_bare_bound_property_can_still_be_posed_by_hand() {
    let (mut w, e) = one("Ball");
    let mut doc = TimelineDoc::new();
    let _ = doc.bind(e.to_bits(), PropKind::TranslationX);
    apply_from_doc(&mut w, &mut doc, 0.0); // captures rest = 0

    // The artist drags the object. Nothing has ever driven this channel.
    w.get_mut::<Transform>(e).unwrap().translation.x = 42.0;
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        42.0,
        "an unkeyed, unformula'd property is the ARTIST's — the apply must leave it alone"
    );
}

/// **The pose that is handed back never depends on WHEN it was noted.**
///
/// ⚠️ This gate replaces a guard I built and then deleted, and the deletion is the finding.
/// The ledger is written by the blend, and the blend is also sampled by `pose_at` (onion
/// ghosts, motion paths) and by `autokey` — at *arbitrary* times. So I threaded a typed
/// `Ledger::{Note, Quiet}` through six call sites so only the real frame could write a
/// note. **Then the mutation survived the whole suite** (474 green with `pose_at` poisoning
/// the ledger on purpose), and the reason is a proof, not luck:
///
/// > The hand-back only ever fires where `composed` has NO entry — i.e. where the blend
/// > produced nothing. The only branch that produces nothing is the one with no
/// > `value_track`, and there the pre-expression value **is `rest`** — a constant. A
/// > constant noted at 1.5 s and a constant noted at 0 s are the same number.
///
/// So the guard could not fail, and a guard that cannot fail is read as protecting
/// something and makes the next person's reasoning wrong. It is gone; this is the invariant
/// it was standing on, asserted where it can be seen. ⚠️ **If the sparsity of
/// `clip_anim_source` is ever widened** (FASE B of the plan contemplates exactly that), the
/// premise dies and the note's instant starts to matter — which is why the number below is
/// measured against a query at a DIFFERENT time.
#[test]
fn the_handed_back_pose_is_the_rest_however_the_ledger_was_filled() {
    let (mut w, e) = one("Ball");
    let mut doc = TimelineDoc::new();
    w.get_mut::<Transform>(e).unwrap().translation.x = 5.0;
    apply_from_doc(&mut w, &mut doc, 0.0);
    set_clip_formula(&mut doc, e, PropKind::TranslationX, Some("value + 250"));
    assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 255.0, "PREMISE: rest is 5");

    // Query the pose somewhere ELSE in time — the door that writes the ledger from a
    // moment the artist is not looking at.
    let _ = ph2d_timeline::pose_at(&w, &doc, e.to_bits(), 1.5);

    set_clip_formula(&mut doc, e, PropKind::TranslationX, None);
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        5.0,
        "the hand-back is the binding's REST, whatever instant last filled the ledger"
    );
}

/// **The GLOBAL channel too** (ADR-0144's `binding.expr`), not just the per-clip one.
///
/// Two writers, one property: the audit found the reader takes `per-clip ?? global` while
/// only the per-clip has an authoring gesture (§4 D-H). Whichever the rewrite keeps, the
/// hand-back has to answer for both, or clearing through the surviving one leaves the
/// other's residue on screen.
#[test]
fn the_global_channel_hands_the_pose_back_as_well() {
    let (mut w, e) = one("Ball");
    let mut doc = TimelineDoc::new();
    let tgt = doc.bind(e.to_bits(), PropKind::TranslationX);
    apply_from_doc(&mut w, &mut doc, 0.0);

    let set = |doc: &mut TimelineDoc, src: Option<&str>| {
        doc.bindings_mut()
            .iter_mut()
            .find(|b| b.target == tgt)
            .expect("just bound")
            .expr = src.map(str::to_string);
    };

    set(&mut doc, Some("value + 250"));
    assert_eq!(x_at(&mut w, e, &mut doc, 0.0), 250.0, "PREMISE: driven");
    set(&mut doc, None);
    assert_eq!(
        x_at(&mut w, e, &mut doc, 0.0),
        0.0,
        "clearing the global formula must give the property back too"
    );
}
