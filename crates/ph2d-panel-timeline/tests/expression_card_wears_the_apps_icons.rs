//! **The Expression card wears the app's ICONS, not letters.**
//!
//! ⚠️ Red-first against a report. The first cut of this card drew its bypass toggle as
//! the letters `"O"` / `"o"` and its two remove/close buttons as `"X"`, and Enio asked
//! the question that has no good answer: *"neste app usamos o olhinho para esconder
//! algo. Por que usou um O?"* There was no reason — they were placeholders that shipped.
//!
//! ⚠️ **The tell was in the file the whole time:** the comment on the line above the eye
//! already called it *"bypass eye"*. Prose describing an icon, sitting over code drawing
//! a glyph, is the same shape as every stale-comment bug this repo has paid for — except
//! here the prose was the CORRECT half.
//!
//! Two things are asserted, and they are different questions:
//!
//! * **no letter is used as an icon** — the family, so the fourth one cannot be added
//!   quietly (there were three; I found the third only by sweeping for the first two);
//! * **the eye's polarity matches the app** — open eye when the row is LIVE, closed when
//!   it is bypassed, which is what hierarchy, painter layers, mask rows and the vector
//!   effect stack all do. An artist learns one eye, not one per panel.
//!
//! ⚠️ Why a source scan and not a painted-scene check: the scene is vector geometry, so
//! "which glyph" is not a question the paint output answers cheaply — and the repo has
//! been burned the other way too (the `Layer`/`Layers` pair was rejected in a smoke
//! because two DIFFERENT identifiers drew the SAME figure). What a scan can prove is the
//! thing that actually regressed: a letter where an `IconId` belongs. The CLICK reaching
//! the sheet is proven next door, by `a_row_can_be_bypassed_and_removed`, driving real
//! pointer events — the two halves together are the gate.

const COLUMNS: &str = include_str!("../src/expr_modal_columns.rs");
const PAINT: &str = include_str!("../src/expr_modal_paint.rs");

/// Every `expr_button` label in the card is either an i18n key lookup or a real word —
/// never a single character standing in for a picture.
#[test]
fn no_letter_is_used_as_an_icon() {
    for (file, src) in [
        ("expr_modal_columns.rs", COLUMNS),
        ("expr_modal_paint.rs", PAINT),
    ] {
        for (n, line) in src.lines().enumerate() {
            let Some(rest) = line.split_once("expr_button(").map(|(_, r)| r) else {
                continue;
            };
            // A short quoted literal handed to a text button is a glyph wearing a
            // string's clothes. `ph2d_i18n::tr(..)` and real words are fine.
            let short_literal = rest
                .split('"')
                .skip(1)
                .step_by(2)
                .any(|lit| lit.chars().count() <= 2 && !lit.is_empty());
            assert!(
                !short_literal,
                "{file}:{} draws a letter where an IconId belongs — \
                 use `expr_icon_button` with the app's glyph:\n{line}",
                n + 1
            );
        }
    }
}

/// **The eye is open when the row is live and closed when it is bypassed.**
///
/// ⚠️ The polarity is the half a mutation would get wrong silently: swapping the two
/// still paints an eye, still registers the hit, still mutes the row when clicked — the
/// behavioural gate next door stays green, and the icon reads inside-out forever.
#[test]
fn the_eye_is_open_when_the_row_is_live() {
    let at = COLUMNS
        .find("let eye_glyph = if row.bypass {")
        .expect("the bypass glyph is chosen from `row.bypass`");
    let arm = &COLUMNS[at..at + 200];
    let closed = arm
        .find("IconId::EyeClosed")
        .expect("a bypassed row shows a CLOSED eye");
    let open = arm.find("IconId::Eye\n").or_else(|| {
        arm.find("IconId::Eye ")
            .or_else(|| arm.find("IconId::Eye,"))
    });
    let open = open.expect("a live row shows an OPEN eye");
    assert!(
        closed < open,
        "polarity is inside-out: `row.bypass` must take the CLOSED eye, and it is the \
         first arm of this `if`:\n{arm}"
    );
}

/// Positive control: the scanner is looking at the real files and can find the door it
/// claims to police. Without this, deleting `expr_icon_button` — or renaming the two
/// source files — would leave both gates above vacuously green.
#[test]
fn the_scanner_reads_the_real_card() {
    assert!(
        COLUMNS.contains("pub(crate) fn expr_icon_button("),
        "the icon door must exist in the file this gate scans"
    );
    assert!(
        PAINT.contains("expr_icon_button(ctx, theme, ids::EXPR_MODAL_CLOSE"),
        "the card's own close button must go through it too"
    );
    assert!(
        COLUMNS.contains("expr_button("),
        "…and the TEXT button still exists, for the footer's real words"
    );
}
