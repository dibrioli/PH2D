//! **A click on a button is the button's — it never reaches the artwork behind it.**
//!
//! Enio, 2026-07-16: *"os painéis de botões lateral esquerdo e superiores permitem que o clique sobre
//! alguns botões pinte a pintura atrás de si. Ou seja, eles não inibem a propagação do clique."*
//!
//! ## The bug
//!
//! Asking *"is this point the editor's UI?"* has **two halves**:
//!
//! * `store.panel_at(x, y)` — panel **bodies** (Inspector, Hierarchy, the docked Layers/Vector panels).
//!   It knows about rects that panels publish.
//! * `hit_index.hit(x, y)` — every interactive widget the chrome **painted, wherever it painted it**.
//!
//! The left rail and the top bars are **not panels**: they publish no panel rect, they are rows of
//! hit-indexed buttons drawn straight onto the screen. So a guard that asks `panel_at` alone answers
//! *"not the UI"* over every one of those buttons, and the Down falls through to the canvas — the brush
//! paints behind the button the artist just pressed.
//!
//! Three canvas consumers asked exactly half: `painter_canvas_down`, `try_eyedropper_sample`,
//! `try_protect_paint`. The sprite-pick and the Flip draw ask both halves and were fine — so the bug was
//! never "nobody thought of it", it was **the same question answered in two different places, two
//! different ways** ([[feedback_two_doors_to_the_same_question_diverge]]). And it had already been paid
//! for ONCE, per-button: `arm_fill_drag_if_on_button` hard-codes `hit(..) == Some(PAINTER_RAIL_FILL)` to
//! stop the C&F button leaking (Enio, 2026-07-03). One id, patched by hand; every other button still
//! leaked.
//!
//! ## What is gated here
//!
//! 1. the **pure decision** (`forwarding::chrome_claims`) — including the gizmo exception, which is the
//!    part a careless fix gets wrong in the expensive direction;
//! 2. an **arch-gate**: no canvas consumer may ask `panel_at` by hand again. The half-question is the
//!    bug, so the half-question is what the gate forbids.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// **Every ARM that can consume a press before the artwork sees it** — `(file, fn)`.
///
/// Per-ARM, not per-file, and that is not pedantry: `protect_brush.rs` holds three of these, and a
/// file-level check stayed GREEN while `protect_dab`'s guard was deleted outright — the file still called
/// the door from a *different* arm. A layered defence needs a gate per layer
/// ([[feedback_layered_defenses_need_per_layer_gates]]); the mutation said so.
///
/// `protect_dab` is on this list because the gate FOUND it: its two siblings were patched for the panel
/// leak in 2026-05-26 and the core dab was missed entirely — an armed protection brush painted through the
/// UI and swallowed the click, and nothing said a word.
const DOOR: &str = "shells/desktop/src/chrome_hit.rs";

const CANVAS_CONSUMERS: &[(&str, &str)] = &[
    (
        "shells/desktop/src/input_dispatch/painter_canvas_input.rs",
        "painter_canvas_down",
    ),
    (
        "shells/desktop/src/input_dispatch/eyedropper.rs",
        "try_eyedropper_sample",
    ),
    (
        "shells/desktop/src/input_dispatch/protect_brush.rs",
        "protect_dab",
    ),
    (
        "shells/desktop/src/input_dispatch/protect_brush.rs",
        "try_add_area_click",
    ),
];

/// The source of one `fn`, from its signature to the start of the next one at the same level.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let needle = format!("fn {name}(");
    let start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("`fn {name}` vanished — this gate is stale, not passing"));
    let rest = &src[start..];
    // The next function at the same indent ends this one. `\n    fn ` also catches `pub(crate) fn`
    // because the search starts after the visibility keyword.
    let end = rest[1..]
        .find("\n    fn ")
        .or_else(|| rest[1..].find("\n    pub(crate) fn "))
        .or_else(|| rest[1..].find("\n    pub fn "))
        .map_or(rest.len(), |e| e + 1);
    &rest[..end]
}

/// **No canvas consumer asks the chrome question by hand.**
///
/// `panel_at` is half the question, and the half that misses every button in the left rail and the top
/// bars. All three of these files asked exactly that, and all three leaked. They now go through
/// `forwarding::chrome_claims`, which asks both halves and knows the gizmo is not chrome.
///
/// The gate is on the HALF, not on the whole: a file that calls `chrome_claims` is fine, a file that
/// reaches for `panel_at` is re-deriving a question that already has one door — and the next one to do it
/// will get the same half-answer and ship the same bug.
///
/// **Mutation that must bleed:** put `hero.store.panel_at(px, py).is_some()` back into any of the three.
#[test]
fn no_canvas_consumer_asks_the_chrome_question_by_hand() {
    let mut offenders = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    for (rel, _) in CANVAS_CONSUMERS {
        if seen.contains(rel) {
            continue;
        }
        seen.push(rel);
        let src = read(rel);
        for (n, line) in src.lines().enumerate() {
            // Skip prose: the files explain the bug at length, `panel_at` included.
            let code = line.trim_start();
            if code.starts_with("//") || code.starts_with("///") {
                continue;
            }
            if code.contains("panel_at(") {
                offenders.push(format!("{rel}:{} — {}", n + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "these canvas consumers ask the chrome question by hand:\n  {}\n\n`panel_at` is HALF the \
         question. The left rail's buttons and the top bars publish no panel rect, so it answers \"not \
         the UI\" over every one of them and the press falls through to the artwork — the brush paints \
         behind the button the artist just clicked (Enio, 2026-07-16). Ask \
         `forwarding::chrome_claims(panel_at(..), hit_index.hit(..))`: it asks both halves, and it knows \
         the gizmo is not chrome.",
        offenders.join("\n  ")
    );
}

/// **Every consuming ARM actually asks the door.** The sibling of the gate above, and the reason it is
/// not vacuous: deleting a guard entirely satisfies "nobody calls `panel_at`" while leaking every click.
/// An absence gate needs its presence sibling ([[feedback_absence_gate_needs_a_presence_sibling]]).
///
/// It reads the ARM, not the file — the mutation that deleted `protect_dab`'s guard sailed through a
/// file-level version of this, because `protect_brush.rs` still called the door from `try_add_area_click`
/// next door. Three arms in one file is a layered defence, and each layer needs its own gate.
///
/// **Mutation that must bleed:** delete the `pointer_over_chrome` call from any single arm.
#[test]
fn every_canvas_consuming_arm_asks_the_door() {
    for (rel, func) in CANVAS_CONSUMERS {
        let src = read(rel);
        let body = fn_body(&src, func);
        assert!(
            body.contains("pointer_over_chrome(") || body.contains("chrome_claims("),
            "`{func}` ({rel}) guards the artwork against nothing: it asks neither `pointer_over_chrome` \
             nor `chrome_claims`, so a press anywhere — including on a left-rail button or a top-bar chip \
             — starts a canvas gesture behind the chrome. Every arm that can consume a Primary Down \
             before the canvas sees it must ask the door FIRST."
        );
    }
}

/// **The one-off that proves the mechanism is still there, and still one-off.**
///
/// `arm_fill_drag_if_on_button` hard-codes ONE id (`PAINTER_RAIL_FILL`) to stop ONE button leaking into
/// `painter_canvas_down` — the same bug, patched per-button on 2026-07-03. It is left alone deliberately:
/// it does more than guard (it ARMS the ColorDrop drag), so it is not redundant with the door. This gate
/// exists so the next reader knows it is a *gesture*, not a second occlusion check to copy.
///
/// If this ever becomes a plain guard, delete it — the door already covers it.
#[test]
fn the_fill_buttons_one_off_is_a_gesture_not_a_second_occlusion_check() {
    let src = read("shells/desktop/src/input_dispatch/fill_drag.rs");
    assert!(
        src.contains("PAINTER_RAIL_FILL"),
        "the C&F one-off vanished — if it became a plain occlusion check, the door covers it and this \
         gate should go; if the button lost its ColorDrop gesture, this gate is stale"
    );
    assert!(
        src.contains("FILL_DRAG.with"),
        "`arm_fill_drag_if_on_button` no longer ARMS anything, so it is now a per-button occlusion check \
         — exactly the thing `chrome_claims` exists to make unnecessary. Delete it and let the door do it."
    );
}

/// **The live door asks the gizmo BOTH ways** — the canonical table AND the map the gizmo filled in while
/// painting.
///
/// The gizmo registers its hit rects under two id schemes: the **canonical** ids (what the PRIMARY
/// selection paints, and what `is_gizmo_id` knows) and **keyed** ids — `canonical ^ hash(entity bits)` —
/// for every EXTRA selection and for the global gizmo (`register_keyed_handle`). Keyed ids are unforgeable
/// by construction; no static table can classify them. The only thing that knows them is
/// `hero.gizmo.gizmo_hit_map`, which is exactly what the pick cascade consults for the same reason.
///
/// Ask only the table and every extra/global handle is called "chrome": the Painter refuses the press, it
/// falls through to the pick, and **the brush silently becomes a move gesture with the tool still saying
/// Paint** — Enio's regression of 2026-07-16, caused by this very door.
///
/// It is an arch-gate because the unit tests cannot reach it: `chrome_claims` takes the classifier as an
/// argument (so it is testable at all), and the LIVE classifier is assembled inside `pointer_over_chrome`,
/// which needs a painted hero — GPU and a window. Deleting the map half there survived every unit test,
/// and a surviving mutant is a missing gate.
///
/// **Mutation that must bleed:** drop `|| hero.gizmo.gizmo_hit_map.contains_key(&id)` from
/// `pointer_over_chrome`'s classifier.
#[test]
fn the_live_door_asks_the_gizmo_both_ways() {
    let src = read(DOOR);
    let start = src
        .find("pub fn pointer_over_chrome")
        .expect("`pointer_over_chrome` vanished — this gate is stale, not passing");
    let body = &src[start..];
    let end = body.find("\n}").expect("unterminated fn");
    let body = &body[..end];
    // Whitespace-stripped, and it MUST be: the first draft of this gate looked for `gizmo_hit_map`
    // anywhere in the body and passed on the DIAGNOSTIC `eprintln!` that also names it — green because of
    // a debug print, while the classifier itself had been gutted. The gate has to read the ANSWER, not the
    // vocabulary.
    let dense: String = body.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        dense.contains("is_gizmo_id(id)"),
        "the live door no longer asks the canonical gizmo table, so the PRIMARY selection's handles \
         (bbox interior included) are called chrome and the brush cannot paint its own sprite"
    );
    assert!(
        dense.contains("is_gizmo_id(id)||hero.gizmo.gizmo_hit_map.contains_key(&id)"),
        "the live door asks only the canonical gizmo table. Every EXTRA selection and the global gizmo \
         register KEYED ids (`canonical ^ hash(bits)`, `register_keyed_handle`) that no static table can \
         recognise — so they get called chrome, the Painter refuses the press, and the brush turns into a \
         move gesture with the tool still saying Paint. That is exactly the regression of 2026-07-16. The \
         map is the only thing that knows those ids; the pick cascade consults it for the same reason."
    );
}
