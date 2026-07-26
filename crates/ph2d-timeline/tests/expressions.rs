//! **Property expressions** (Wave C / ADR-0144). The formula drives a property in
//! a SEPARATE post-composition pass; these gates pin the engine end to end (parse
//! -> eval -> write into the world), the fade isolation (the pass never calls
//! `stack_eval`), and the cycle/error/determinism guarantees the ADR promised.
//!
//! The oracle throughout is the WORLD after `apply_from_doc`: the composed pose an
//! artist actually sees, not an intermediate the pass computes.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform, World};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc};

/// A world with one named sprite entity at the origin.
fn one(name: &str) -> (World, Entity) {
    let mut w = World::new();
    let e = w
        .spawn((Transform::from_translation(Vec2::ZERO), Name::new(name)))
        .id();
    (w, e)
}

/// Drive `(e, prop)` by the formula `src` — binds the target (creating it if the
/// prop has no keys) and stamps the expression on the binding.
fn drive(doc: &mut TimelineDoc, e: Entity, prop: PropKind, src: &str) {
    let tgt = doc.bind(e.to_bits(), prop);
    doc.bindings_mut()
        .iter_mut()
        .find(|b| b.target == tgt)
        .expect("just bound")
        .expr = Some(src.to_string());
}

/// Key `(e, prop)` linearly `v0 -> v1` over `0..dur` (Linear).
fn ramp(doc: &mut TimelineDoc, e: Entity, prop: PropKind, v0: f32, v1: f32, dur: f64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(e.to_bits(), prop, s(0.0), AnimValue::Float(v0), Interp::Linear);
    doc.insert_key(e.to_bits(), prop, s(dur), AnimValue::Float(v1), Interp::Linear);
}

fn x_at(w: &mut World, e: Entity, doc: &mut TimelineDoc, t: f64) -> f32 {
    apply_from_doc(w, doc, t);
    w.get::<Transform>(e).unwrap().translation.x
}

fn y_at(w: &mut World, e: Entity, doc: &mut TimelineDoc, t: f64) -> f32 {
    apply_from_doc(w, doc, t);
    w.get::<Transform>(e).unwrap().translation.y
}

/// `time*10` is a ramp: x(t) = 10·t, with NO keyframes at all — an expression can
/// drive a property that has none. (Mutation: the pass early-out fires for a doc
/// with expressions -> x stays 0.)
#[test]
fn time_times_ten_is_a_ramp() {
    let (mut w, e) = one("R");
    let mut doc = TimelineDoc::new();
    drive(&mut doc, e, PropKind::TranslationX, "time*10");
    assert!((x_at(&mut w, e, &mut doc, 0.5) - 5.0).abs() < 1e-4);
    assert!((x_at(&mut w, e, &mut doc, 2.0) - 20.0).abs() < 1e-4);
}

/// `value` is the KEYED value (the AE pre-expression value): `value + 100` rides
/// the keyframes. At t=0.5 a 0->10 ramp over 1 s is 5, so the driven x is 105.
#[test]
fn value_rides_the_keyframes() {
    let (mut w, e) = one("V");
    let mut doc = TimelineDoc::new();
    ramp(&mut doc, e, PropKind::TranslationX, 0.0, 10.0, 1.0);
    drive(&mut doc, e, PropKind::TranslationX, "value + 100");
    assert!((x_at(&mut w, e, &mut doc, 0.5) - 105.0).abs() < 1e-3);
    assert!((x_at(&mut w, e, &mut doc, 1.0) - 110.0).abs() < 1e-3);
}

/// `wiggle` is deterministic (same seed reproduces) AND per-binding (two bindings
/// with the SAME formula differ — distinct seeds). It also stays inside `[-amp,
/// amp]`. (Mutation: a constant seed -> the two bindings coincide.)
#[test]
fn wiggle_is_deterministic_by_seed() {
    let (mut w, e) = one("W");
    let mut doc = TimelineDoc::new();
    // Same formula on X and Y — distinct targets -> distinct seeds.
    drive(&mut doc, e, PropKind::TranslationX, "wiggle(2, 20)");
    drive(&mut doc, e, PropKind::TranslationY, "wiggle(2, 20)");

    apply_from_doc(&mut w, &mut doc, 1.3);
    let xf = *w.get::<Transform>(e).unwrap();
    let (wx, wy) = (xf.translation.x, xf.translation.y);
    assert!(wx.abs() <= 20.0 + 1e-4 && wy.abs() <= 20.0 + 1e-4, "within [-amp, amp]");
    assert!((wx - wy).abs() > 1e-3, "two bindings, two seeds -> two values");

    // Reproducible: a fresh identical doc gives the same X at the same time.
    let (mut w2, e2) = one("W");
    let mut doc2 = TimelineDoc::new();
    drive(&mut doc2, e2, PropKind::TranslationX, "wiggle(2, 20)");
    assert_eq!(x_at(&mut w, e, &mut doc, 1.3), x_at(&mut w2, e2, &mut doc2, 1.3));
}

/// A prop-link follows a KEYED source with no lag: `Src.x` on Dst reads Src's
/// composed x this frame. (Mutation: resolving the name to nothing -> Dst reads 0.)
#[test]
fn a_prop_link_follows_a_keyed_source() {
    let mut w = World::new();
    let src = w.spawn((Transform::from_translation(Vec2::ZERO), Name::new("Src"))).id();
    let dst = w.spawn((Transform::from_translation(Vec2::ZERO), Name::new("Dst"))).id();
    let mut doc = TimelineDoc::new();
    ramp(&mut doc, src, PropKind::TranslationX, 0.0, 10.0, 1.0); // Src.x = 10t
    drive(&mut doc, dst, PropKind::TranslationX, "Src.x");

    apply_from_doc(&mut w, &mut doc, 0.7);
    let sx = w.get::<Transform>(src).unwrap().translation.x;
    let dx = w.get::<Transform>(dst).unwrap().translation.x;
    assert!((sx - 7.0).abs() < 1e-3, "src keyed to 7");
    assert!((dx - sx).abs() < 1e-3, "dst mirrors src, no lag");
}

/// A cycle reads the pass-start snapshot and NEVER explodes: `A = B.x + 1`,
/// `B = A.x + 1`, cross-linked. Over many frames every value stays finite (the
/// Jacobi sweep reads last frame's value at the edge). (Mutation: reading the
/// live world instead of the snapshot could diverge / read a half-written frame.)
#[test]
fn a_cycle_reads_the_snapshot_and_does_not_explode() {
    let mut w = World::new();
    let a = w.spawn((Transform::from_translation(Vec2::ZERO), Name::new("A"))).id();
    let b = w.spawn((Transform::from_translation(Vec2::ZERO), Name::new("B"))).id();
    let mut doc = TimelineDoc::new();
    drive(&mut doc, a, PropKind::TranslationX, "B.x + 1");
    drive(&mut doc, b, PropKind::TranslationX, "A.x + 1");

    for _ in 0..200 {
        apply_from_doc(&mut w, &mut doc, 0.0);
        let ax = w.get::<Transform>(a).unwrap().translation.x;
        let bx = w.get::<Transform>(b).unwrap().translation.x;
        assert!(ax.is_finite() && bx.is_finite(), "a cycle must never explode");
    }
}

/// A malformed formula keeps the KEYED value — a parse error is a fallback, never
/// a crash or a zero. (Mutation: writing 0.0 on parse error -> x would be 0.)
#[test]
fn a_parse_error_keeps_the_keyed_value() {
    let (mut w, e) = one("E");
    let mut doc = TimelineDoc::new();
    ramp(&mut doc, e, PropKind::TranslationX, 0.0, 10.0, 1.0);
    drive(&mut doc, e, PropKind::TranslationX, "time * ("); // malformed
    assert!((x_at(&mut w, e, &mut doc, 0.5) - 5.0).abs() < 1e-3, "keeps the keyed 5");
}

/// A binding WITHOUT an expression is untouched even while the pass runs for
/// another binding: Y is keyed-only, X is expression-driven. Y stays keyed-exact.
/// (This is the byte-identity promise, per-binding: the pass writes only what it
/// drives.)
#[test]
fn a_binding_without_expr_is_untouched_while_the_pass_runs() {
    let (mut w, e) = one("U");
    let mut doc = TimelineDoc::new();
    ramp(&mut doc, e, PropKind::TranslationY, 0.0, 10.0, 1.0); // Y keyed, no expr
    drive(&mut doc, e, PropKind::TranslationX, "time*10"); // X drives the pass to run
    assert!((y_at(&mut w, e, &mut doc, 0.5) - 5.0).abs() < 1e-4, "Y stays keyed");
}

/// **Arch-gate: the expression pass never touches the fade evaluator.** The whole
/// isolation of ADR-0144 rests on the pass staying out of `stack_eval`/the blend;
/// a source scan pins it so a future edit that reaches in fails here, loudly.
#[test]
fn the_expression_pass_never_calls_stack_eval() {
    let src = include_str!("../src/expr_pass.rs");
    // Strip the module doc-comment (it NAMES stack_eval to explain the rule) so the
    // scan tests the CODE, not the prose.
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains("stack_eval"),
        "the expression pass must not call stack_eval (fade isolation, ADR-0144)"
    );
}

/// **Arch-gate: ONE parser.** The Motion node must delegate to `ph2d-expr-parse`,
/// not carry its own lexer — two parsers for one IR would drift (ADR-0144 §5).
#[test]
fn the_motion_node_delegates_to_the_one_parser() {
    let src = include_str!("../../ph2d-node-motion-expression/src/parse.rs");
    assert!(
        src.contains("ph2d_expr_parse::parse"),
        "the Motion node must delegate to the shared parser"
    );
    assert!(
        !src.contains("fn lex("),
        "the Motion node must not carry its own lexer (the single-door rule)"
    );
}
