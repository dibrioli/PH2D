//! **The gates for the catalog** (plano 10 §8).
//!
//! The crate itself emits strings and never parses — `ph2d-expr-parse` and
//! `ph2d-expr` are dev-dependencies, used ONLY here. That is what keeps the
//! catalog a leaf the timeline UI and the Motion node can both consume, and it is
//! the machete-safe shape `ph2d-gpu-cook` uses with the node crates it only gates.

use ph2d_expr::eval;
use ph2d_expr_parse::parse;
use ph2d_expr_recipes::{
    CATALOG, ClockUse, Family, KnobKind, Neutrality, REFUSALS, RecipeStack, Row, RowKind,
    SearchHit, search,
};

mod shared;
use shared::{B, Knobs, row_with};

// ---------------------------------------------------------------- G1

/// **G1 — every recipe emits a formula the ONE parser accepts**, at its defaults
/// AND at both ends of every knob.
///
/// ⚠️ The extremes are not decoration. A zero-width remap range and a zero
/// quantize step both parse; what they produce is `inf`, so the guards
/// (`EmitCtx::nz` / `span_hi`) are what this half actually checks — together with
/// the finite-value assertion below.
#[test]
fn every_recipe_emits_a_formula_the_one_parser_accepts() {
    let mut bad = Vec::new();
    for r in CATALOG {
        for (tag, how) in [
            ("default", Knobs::Default),
            ("low", Knobs::End(false)),
            ("high", Knobs::End(true)),
            ("zero", Knobs::Zero),
        ] {
            let src = RecipeStack {
                rows: vec![row_with(r.id, how)],
            }
            .to_formula();
            match parse(&src) {
                Err(e) => bad.push(format!("  {} [{tag}]: {src}\n    -> {e}", r.id)),
                Ok(e) => {
                    let v = eval(&e, &B(3.5));
                    if !v.is_finite() {
                        bad.push(format!("  {} [{tag}]: {src}\n    -> not finite: {v}", r.id));
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} recipes emit text the parser refuses (or a non-finite value):\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// An UNFILLED link still parses — the formula bar renders on every keystroke,
/// long before the pick-whip has been used.
#[test]
fn an_unfilled_link_still_emits_a_parseable_formula() {
    let linked: Vec<_> = CATALOG
        .iter()
        .filter(|r| r.knobs.iter().any(|k| k.kind == KnobKind::Link))
        .collect();
    assert!(!linked.is_empty(), "the catalog has link recipes to check");
    for r in linked {
        let src = RecipeStack::of(&[r.id]).to_formula();
        assert!(
            parse(&src).is_ok(),
            "{} with an empty link emitted unparseable text: {src}",
            r.id
        );
    }
}

// ---------------------------------------------------------------- G2

/// **G2a — an additive recipe at its neutral knobs is the identity, to the bit.**
///
/// The invariant this repo's audio rack states as *"every effect is a
/// byte-identical no-op at its neutral point"*. Iterating the WHOLE catalog is
/// load-bearing: a neutral checked only on `Shake` passes with `Drift` broken.
#[test]
fn an_additive_recipe_at_its_neutral_knobs_is_the_identity() {
    let mut checked = 0;
    for r in CATALOG {
        let Some(row) = Row::neutral(r.id) else {
            continue;
        };
        let src = RecipeStack { rows: vec![row] }.to_formula();
        let e = parse(&src).unwrap_or_else(|err| panic!("{}: {err}", r.id));
        for v in [-3.5_f32, 0.0, 12.25, 1000.0] {
            let got = eval(&e, &B(v));
            assert_eq!(
                got, v,
                "{} at its neutral must return `value` exactly; {src} gave {got} for {v}",
                r.id
            );
        }
        checked += 1;
    }
    assert!(
        checked >= 10,
        "expected the additive recipes, got {checked}"
    );
}

/// **G2b — the living families all DECLARE a neutral.**
///
/// ⚠️ Without this half, G2a is satisfied by a catalog that declares everything
/// `NoNeutral`: it would simply skip every recipe and pass over nothing. A recipe
/// in Life / Wave / Physics ADDS to the value, so it has an "off", and shipping
/// one without an off is shipping a knob that cannot be undone.
#[test]
fn the_living_families_all_declare_a_neutral() {
    for r in CATALOG {
        let living = matches!(r.family, Family::Life | Family::Wave | Family::PhysicsLite);
        // Wave holds the pure GENERATORS (Blink, Ping-Pong, Orbit…), which replace
        // the value outright; they are named here rather than excused by family.
        let generator = matches!(
            r.id,
            "ping-pong" | "blink" | "pulse" | "orbit-x" | "orbit-y"
        );
        if living && !generator {
            assert!(
                matches!(r.neutral, Neutrality::Additive(_)),
                "{} adds to the value, so it must declare a neutral",
                r.id
            );
        }
    }
}

// ---------------------------------------------------------------- G3

/// **G3 — bypassing a row is byte-identical to removing it.**
///
/// A bypass that merely zeroed a knob would leave the row's arithmetic in the
/// formula: the artist toggles the eye, the text changes, and the picture does
/// not.
#[test]
fn bypassing_a_row_is_byte_identical_to_removing_it() {
    for r in CATALOG {
        let mut with = RecipeStack::of(&["shake", r.id, "limit"]);
        with.rows[1].bypass = true;
        let without = RecipeStack::of(&["shake", "limit"]);
        assert_eq!(
            with.to_formula(),
            without.to_formula(),
            "bypassing {} is not the same as removing it",
            r.id
        );
    }
}

// ---------------------------------------------------------------- G6

/// **G6 — composing two rows equals composing their two functions.**
///
/// The oracle is not another emit: a ONE-row stack has no precedence ambiguity
/// (its `inner` is the atom `value`), so `stack([A]).eval(v)` is unarguable. The
/// gate then asserts `stack([A, B]).eval(v) == B(A(v))` over the whole catalog —
/// 26×26 pairs — which is exactly what catches a missing `paren`.
///
/// ⚠️ Behaviour, never text: comparing strings would fail on whitespace and pass
/// on wrong precedence, which is the failure mode backwards.
#[test]
fn composing_two_rows_equals_composing_their_two_functions() {
    let values: Vec<_> = CATALOG
        .iter()
        .filter(|r| r.kind == RowKind::Value)
        .collect();
    assert!(values.len() > 20, "the value catalog is the fixture here");

    let mut bad = Vec::new();
    for a in &values {
        for b in &values {
            // ⚠️ PERTURBED, not default. `Multiply / Add` is the identity at its
            // defaults (`value*1 + 0`), so the first version of this gate composed
            // two identities and passed with the parenthesisation deleted — the
            // fixture did not contain the phenomenon it existed to catch.
            let solo = |id| {
                let src = RecipeStack {
                    rows: vec![row_with(id, Knobs::Perturbed)],
                }
                .to_formula();
                let e = parse(&src).unwrap_or_else(|err| panic!("{id}: {err} in {src}"));
                move |v: f32| eval(&e, &B(v))
            };
            let (fa, fb) = (solo(a.id), solo(b.id));
            let composed_src = RecipeStack {
                rows: vec![
                    row_with(a.id, Knobs::Perturbed),
                    row_with(b.id, Knobs::Perturbed),
                ],
            }
            .to_formula();
            let Ok(ce) = parse(&composed_src) else {
                bad.push(format!(
                    "  {} then {}: unparseable {composed_src}",
                    a.id, b.id
                ));
                continue;
            };
            for v in [-3.5_f32, 0.0, 7.25] {
                let want = fb(fa(v));
                let got = eval(&ce, &B(v));
                if want.is_finite() != got.is_finite() || (want.is_finite() && want != got) {
                    bad.push(format!(
                        "  {} then {} at value={v}: {got} != {want}\n    {composed_src}",
                        a.id, b.id
                    ));
                    break;
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} ordered pairs do not compose:\n{}",
        bad.len(),
        bad.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
    );
}

// ---------------------------------------------------------------- G11

/// **G11 — a Time row rewrites the clock of the rows BELOW it, and nothing else.**
///
/// Three assertions in one, because they are three different ways to get this
/// wrong: a clock-reading row below it CHANGES, a row above it does NOT, and a
/// [`ClockUse::Own`] row is untouched wherever it sits (`wiggle` builds its own
/// `time` inside the parser — see `catalog/life.rs`).
#[test]
fn a_time_row_rewrites_the_time_of_the_rows_below_it_and_nothing_else() {
    // Below: Sway reads the clock, so Stepped Time above it must reach it.
    let plain = RecipeStack::of(&["sway"]).to_formula();
    let stepped = RecipeStack::of(&["stepped-time", "sway"]).to_formula();
    assert_ne!(
        plain, stepped,
        "a Time row above Sway must change its clock"
    );
    assert!(
        stepped.contains("floor(time*6)/6"),
        "Sway should read the stepped clock: {stepped}"
    );

    // Above: the same row placed BELOW the Time row is unaffected.
    let after = RecipeStack::of(&["sway", "stepped-time"]).to_formula();
    assert_eq!(
        after, plain,
        "a Time row cannot reach back and change the rows above it"
    );

    // Own-clock: Shake is `wiggle`, whose time lives inside the parser.
    let shake = RecipeStack::of(&["shake"]).to_formula();
    let shake_stepped = RecipeStack::of(&["stepped-time", "shake"]).to_formula();
    assert_eq!(
        shake, shake_stepped,
        "a Time row cannot reach `wiggle` — and the catalog says so via ClockUse::Own"
    );
    let r = ph2d_expr_recipes::by_id("shake").unwrap();
    assert_eq!(
        r.clock,
        ClockUse::Own,
        "…so Shake must DECLARE that, or the UI would promise otherwise"
    );
}

// ---------------------------------------------------------------- G12

/// **G12 — a pair recipe names its other half, and the half names it back.**
///
/// Half a circle is not a feature: the gallery offers Orbit as one card and
/// inserts two rows, and it can only do that if the link is symmetric.
#[test]
fn a_pair_recipe_names_its_other_half_and_is_named_back() {
    let mut pairs = 0;
    for r in CATALOG {
        let Some(other) = r.pair else { continue };
        let o = ph2d_expr_recipes::by_id(other)
            .unwrap_or_else(|| panic!("{} points at unknown recipe {other}", r.id));
        assert_eq!(
            o.pair,
            Some(r.id),
            "{} and {other} must name each other",
            r.id
        );
        assert_ne!(o.id, r.id, "a recipe cannot be its own pair");
        pairs += 1;
    }
    assert!(pairs >= 2, "expected at least the Orbit pair, saw {pairs}");
}

// ---------------------------------------------------------------- G7 / G8 / G9

/// **G7 — every recipe is findable by the name the artist already knows.**
///
/// With 55 cards the search IS the interface, and the artist types what they
/// learned in another product. This gate is the reason `aliases` exists.
#[test]
fn every_recipe_is_findable_by_its_industry_name() {
    // (typed, expected recipe id) — the vocabulary of After Effects, Motion,
    // Cavalry and Blender, aimed at OUR cards.
    const PROBES: &[(&str, &str)] = &[
        ("wiggle", "shake"),
        ("camera shake", "shake"),
        ("oscillate", "sway"),
        ("sine", "sway"),
        ("posterizeTime", "stepped-time"),
        ("stop motion", "stepped-time"),
        ("clamp", "limit"),
        ("linear", "remap"),
        ("range mapper", "remap"),
        ("pick whip", "follow"),
        ("strobe", "blink"),
        // ⚠️ The word survives its recipe: `Ramp Loop` was retired as a MEASURED duplicate
        // and `Pulse` inherited its aliases, so the term an artist knows still lands.
        ("sawtooth", "pulse"),
        ("cosine", "sway"),
        ("midpoint", "blend-two"),
        ("negate", "multiply-add"),
        ("mirror", "opposite"),
        ("boomerang", "ping-pong-time"),
        ("follow through", "wave-along-chain"),
        ("falloff", "fade-by-distance"),
        ("gravity", "free-fall"),
        ("formula", "custom"),
        ("snap", "quantize"),
    ];
    for (typed, want) in PROBES {
        let hits = search(typed);
        let found = hits.iter().any(|h| match h {
            SearchHit::Recipe(r) => r.id == *want,
            SearchHit::Refusal(_) => false,
        });
        assert!(
            found,
            "typing {typed:?} must find {want:?}; got {:?}",
            hits.iter()
                .map(|h| match h {
                    SearchHit::Recipe(r) => r.id,
                    SearchHit::Refusal(r) => r.key,
                })
                .take(5)
                .collect::<Vec<_>>()
        );
    }
}

/// **G8 — the search answers for what the catalog REFUSES, with a destination.**
///
/// Refusing silently teaches the artist the tool cannot do it. Typing `loop` has
/// to come back with where the loop lives.
#[test]
fn the_search_answers_for_what_the_catalog_refuses() {
    const PROBES: &[(&str, &str)] = &[
        ("loop", "loop"),
        ("loopOut", "loop"),
        ("ease in", "ease"),
        ("time remap", "time-remap"),
        ("motion path", "path"),
        ("stagger", "stagger"),
        ("collide", "physics"),
        ("vortex", "forces"),
        ("shape blend", "morph"),
        ("exposure", "hold-frame"),
    ];
    for (typed, want) in PROBES {
        let hits = search(typed);
        let hit = hits.iter().find_map(|h| match h {
            SearchHit::Refusal(r) if r.key == *want => Some(*r),
            _ => None,
        });
        let r = hit.unwrap_or_else(|| panic!("typing {typed:?} must surface the {want:?} refusal"));
        assert!(
            !r.to.label().is_empty(),
            "a refusal without a destination is a dead end, which is what it exists to avoid"
        );
    }
}

/// **G9 — the catalog does NOT build what the product already answers.**
///
/// Presence AND absence: each refused idea is named in the refusal table, and no
/// recipe answers to it. Offering "Loop" here would be the second door to
/// `Track.extrap`.
#[test]
fn the_catalog_refuses_what_the_product_already_answers() {
    assert_eq!(REFUSALS.len(), 10, "the refusal list is part of the design");
    for r in REFUSALS {
        // Absence: the top hit for a refusal's own key is never a recipe that
        // claims to do it.
        for alias in r.aliases {
            for h in search(alias) {
                if let SearchHit::Recipe(rec) = h {
                    assert_ne!(
                        rec.label.to_ascii_lowercase(),
                        alias.to_ascii_lowercase(),
                        "{} is refused, but recipe {} answers to it by NAME",
                        r.key,
                        rec.id
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------- hygiene

/// Ids are the identity a row stores and the reopen path compares — duplicates
/// would make `by_id` resolve to whichever came first.
#[test]
fn every_recipe_id_and_knob_key_is_unique() {
    let mut ids: Vec<_> = CATALOG.iter().map(|r| r.id).collect();
    let n = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), n, "duplicate recipe id");

    for r in CATALOG {
        let mut keys: Vec<_> = r.knobs.iter().map(|k| k.key).collect();
        let kn = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), kn, "{} has a duplicate knob key", r.id);
    }

    let mut rk: Vec<_> = REFUSALS.iter().map(|r| r.key).collect();
    let rn = rk.len();
    rk.sort_unstable();
    rk.dedup();
    assert_eq!(rk.len(), rn, "duplicate refusal key");
}

/// A neutral that names a knob the recipe does not have would panic at the point
/// of use; catching it here keeps the failure in the gate rather than in the UI.
#[test]
fn every_declared_neutral_names_a_knob_the_recipe_has() {
    for r in CATALOG {
        if let Neutrality::Additive(overrides) = r.neutral {
            for (key, _) in overrides {
                assert!(
                    r.knobs.iter().any(|k| k.key == *key),
                    "{} declares a neutral for missing knob {key:?}",
                    r.id
                );
            }
        }
    }
}

/// The catalog's SIZE is a fact of the plan, and the gallery is grouped by family
/// — a family that shipped empty would be a drawer that opens onto nothing.
#[test]
fn the_catalog_covers_every_family() {
    // ⚠️ The number is COUNTED, not chosen. It was 55; five entries were retired after a
    // report (*"muitas expressões não passam de mais do mesmo"*) once each had been MEASURED
    // identical to a survivor at some knob setting — `Sway (Cosine)` = Sway with Phase a
    // quarter period, `Ramp Loop` = Pulse at Decay 1 with On/Off swapped, `Mirror` =
    // Opposite at Pivot 0, `Midpoint` = Blend Two at 0.5, `Negate` = Multiply/Add at -1.
    // Each survivor INHERITED the retired one's search words, so no term the artist knows
    // went dead with it.
    assert_eq!(
        CATALOG.len(),
        50,
        "plano 10 §4, menos as cinco duplicatas MEDIDAS"
    );
    for f in Family::ALL {
        assert!(
            CATALOG.iter().any(|r| r.family == f),
            "family {:?} has no recipes",
            f.label()
        );
    }
}

/// An empty sheet drives the property with exactly what it already had — which is
/// what "no rows yet" means, and is what keeps the preview alive while the artist
/// browses.
#[test]
fn an_empty_stack_is_the_identity() {
    let src = RecipeStack::new().to_formula();
    assert_eq!(src, "value");
    let e = parse(&src).unwrap();
    for v in [-1.0_f32, 0.0, 42.0] {
        assert_eq!(eval(&e, &B(v)), v);
    }
}
