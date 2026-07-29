//! **The LIVE expression preview** (the smoke of 2026-07-29): while an editor is open
//! on a binding, the formula it is projecting drives the REAL property — on its own
//! wall-clock, with the transport stopped.
//!
//! ⚠️ These gates drive `apply_from_doc`, the product's own apply, and read the world
//! back. A gate that only checked the channel would prove that a value was stored, not
//! that anything moved — and "nothing moved" is exactly the report this feature answers.

use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::expr_live::{LiveExpr, is_previewing, live_expr, set_live_expr};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

/// A world with one named sprite, and a doc with that sprite's X bound.
fn scene() -> (World, TimelineDoc, u64, u64) {
    let mut world = World::new();
    let e = world
        .spawn((Transform::default(), Name::new("Ball")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    let target = doc.bind(e, PropKind::TranslationX).get();
    (world, doc, e, target)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

/// **The preview DRIVES the real property**, with no authored expression anywhere.
///
/// ⚠️ Red-first: before the live channel the pass took its early-out — a document with
/// no `binding.expr` had nothing to do — so the card could publish all it liked and the
/// object never moved. That early-out is the first thing this gate would catch coming
/// back.
#[test]
fn an_open_editor_drives_the_real_property() {
    let (mut world, mut doc, e, target) = scene();
    set_live_expr(None);
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(x_of(&world, e), 0.0, "nothing drives it yet");

    set_live_expr(Some(LiveExpr {
        target,
        formula: "value + 3".into(),
        time: 0.0,
    }));
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(
        x_of(&world, e),
        3.0,
        "an open editor drives the property it is open on"
    );
    set_live_expr(None);
}

/// **It animates with the TRANSPORT STOPPED** — the whole request.
///
/// The document clock never moves here (`apply_from_doc` is called at `0.0` every
/// time); only the preview's own clock advances. A preview that rode the playhead
/// would sit perfectly still, which is what the artist would be looking at while
/// tuning a wobble.
#[test]
fn the_preview_animates_while_the_clip_is_paused() {
    let (mut world, mut doc, e, target) = scene();
    let mut seen = Vec::new();
    for step in 0..4 {
        set_live_expr(Some(LiveExpr {
            target,
            formula: "value + time*10".into(),
            time: f64::from(step) * 0.25,
        }));
        // ⚠️ The DOCUMENT clock is frozen at zero for every one of these.
        apply_from_doc(&mut world, &mut doc, 0.0);
        seen.push(x_of(&world, e));
    }
    set_live_expr(None);
    assert_eq!(
        seen,
        vec![0.0, 2.5, 5.0, 7.5],
        "the preview runs on its OWN clock while the transport is stopped"
    );
}

/// **Closing the editor gives the property back**, on the very next apply.
///
/// ⚠️ Not by anyone restoring a saved pose: the keyed pass rewrites the property from
/// the curves (or the rest) every frame, so "stop previewing" is the whole of the
/// undo. A preview that had to remember and replay the old value would be a second
/// answer to what this property is.
#[test]
fn closing_the_editor_gives_the_property_back() {
    let (mut world, mut doc, e, target) = scene();
    set_live_expr(Some(LiveExpr {
        target,
        formula: "value + 42".into(),
        time: 0.0,
    }));
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(x_of(&world, e), 42.0);

    set_live_expr(None);
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(
        x_of(&world, e),
        0.0,
        "the frame after the card closes, the property is the document's again"
    );
}

/// **The preview REPLACES the channel's authored expression, it does not stack on it.**
///
/// ⚠️ The card seeds itself from whatever formula the track already carries, as a
/// `Custom Formula` row — `TrackView::expr` reads the per-clip map, and this test
/// authors into exactly that map. So the sheet ALREADY contains it: leave the authored
/// one composing and the preview applies it to a `value` that already includes it. The
/// artist would be tuning `f(f(x))`, and here `+10` authored plus `+10` previewed would
/// read **20** for a formula that says 10.
#[test]
fn the_preview_replaces_the_authored_expression_it_does_not_stack_on_it() {
    let (mut world, mut doc, e, target) = scene();
    let tgt = ph2d_anim::AnimTarget::new(target);
    doc.set_clip_expr(doc.active_index(), tgt, Some("value + 10".to_string()));
    set_live_expr(None);
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(x_of(&world, e), 10.0, "the authored expression alone");

    set_live_expr(Some(LiveExpr {
        target,
        formula: "value + 10".into(),
        time: 0.0,
    }));
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(
        x_of(&world, e),
        10.0,
        "the preview REPLACES it — 20 would mean the artist is tuning against a double"
    );

    // …and closing the card hands the authored one straight back.
    set_live_expr(None);
    apply_from_doc(&mut world, &mut doc, 0.0);
    assert_eq!(x_of(&world, e), 10.0, "the authored formula resumes");
}

/// **A preview on one binding leaves every other binding alone.**
#[test]
fn a_preview_drives_only_the_binding_it_names() {
    let (mut world, mut doc, e, target) = scene();
    let other = doc.bind(e, PropKind::TranslationY).get();
    assert_ne!(target, other);
    set_live_expr(Some(LiveExpr {
        target: other,
        formula: "value + 5".into(),
        time: 0.0,
    }));
    apply_from_doc(&mut world, &mut doc, 0.0);
    set_live_expr(None);
    let t = *world.get::<Transform>(Entity::from_bits(e)).unwrap();
    assert_eq!(t.translation.y, 5.0, "the named binding runs");
    assert_eq!(t.translation.x, 0.0, "and only that one");
}

/// **The channel is display state and says so.** The shell's undo asks this question
/// and skips its diff while the answer is yes — a world being driven by a preview is
/// not a world anybody authored.
#[test]
fn the_channel_reports_whether_anything_is_previewing() {
    set_live_expr(None);
    assert!(!is_previewing());
    assert!(live_expr().is_none());
    set_live_expr(Some(LiveExpr {
        target: 1,
        formula: "value".into(),
        time: 0.0,
    }));
    assert!(is_previewing());
    set_live_expr(None);
    assert!(!is_previewing());
}
