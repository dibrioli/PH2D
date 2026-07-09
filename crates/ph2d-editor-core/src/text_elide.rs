//! Single-line, ellipsized text — the counterpart to [`crate::paint::paint_text`],
//! whose `max_width` is a **wrap** budget.
//!
//! A label one pixel too wide for its column silently becomes two lines and
//! spills into the row below (the timeline track names did exactly that). Rows,
//! list items and any other fixed-height slot need text that truncates instead.
//!
//! Its own module because `paint.rs` sits at its frozen LOC cap (the gate's
//! rule is to drive those DOWN, never up).

use ph2d_text::TextSystem;
use ph2d_vector::{Color, VectorScene};

use crate::paint::paint_text;

/// The ellipsis appended to text that does not fit. Inside Inter's coverage
/// (U+2026 is not one of the arrow / technical blocks the tofu gate rejects).
const ELLIPSIS: &str = "\u{2026}";

/// Paint `text` on **one line**, ellipsized when it does not fit `max_width`.
///
/// [`paint_text`] treats `max_width` as a *wrap* budget, so a label one pixel
/// too wide silently becomes two lines and spills into the row below. Anything
/// that must stay on its own line — list rows, track names — belongs here.
///
/// `max_width` too small for even the ellipsis paints nothing.
#[allow(clippy::too_many_arguments)]
pub fn paint_text_elided(
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    max_width: f32,
    color: Color,
) {
    if max_width <= 0.0 {
        return;
    }
    if text_system.prefix_width(text, font_size) <= max_width {
        // `INFINITY`, not `max_width`: it fits, and passing the budget back would
        // let a sub-pixel measurement disagreement re-introduce the wrap.
        paint_text(
            text_system,
            scene,
            text,
            x,
            y,
            font_size,
            f32::INFINITY,
            color,
        );
        return;
    }
    let Some(elided) = elide(text_system, text, font_size, max_width) else {
        return;
    };
    paint_text(
        text_system,
        scene,
        &elided,
        x,
        y,
        font_size,
        f32::INFINITY,
        color,
    );
}

/// The longest `<prefix>…` of `text` that measures within `max_width`, or `None`
/// when not even the ellipsis fits. Binary search over char boundaries, so a
/// multi-byte name is never cut mid-glyph.
fn elide(
    text_system: &mut TextSystem,
    text: &str,
    font_size: f32,
    max_width: f32,
) -> Option<String> {
    if text_system.prefix_width(ELLIPSIS, font_size) > max_width {
        return None;
    }
    let bounds: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    // Invariant: `lo` chars always fit, `hi` chars never do.
    let (mut lo, mut hi) = (0usize, bounds.len());
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        let candidate = format!("{}{ELLIPSIS}", &text[..bounds[mid]]);
        if text_system.prefix_width(&candidate, font_size) <= max_width {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(format!("{}{ELLIPSIS}", &text[..bounds[lo]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elide_trims_to_the_widest_prefix_that_fits() {
        let mut text = TextSystem::without_system_fonts();
        let long = "Translate Y  #4591";
        let full = text.prefix_width(long, 12.0);
        let budget = full * 0.6;
        let out = elide(&mut text, long, 12.0, budget).expect("something fits");
        assert!(out.ends_with(ELLIPSIS), "{out:?}");
        assert!(long.starts_with(out.trim_end_matches(ELLIPSIS)), "{out:?}");
        assert!(text.prefix_width(&out, 12.0) <= budget, "{out:?} overruns");
        // ...and it is the WIDEST such prefix: one more char overflows.
        let kept = out.trim_end_matches(ELLIPSIS).chars().count();
        let more: String = long.chars().take(kept + 1).collect();
        assert!(text.prefix_width(&format!("{more}{ELLIPSIS}"), 12.0) > budget);
    }

    #[test]
    fn elide_gives_up_when_not_even_the_ellipsis_fits() {
        let mut text = TextSystem::without_system_fonts();
        assert_eq!(elide(&mut text, "Translate Y", 12.0, 0.5), None);
    }

    #[test]
    fn elide_never_cuts_a_multi_byte_char_in_half() {
        let mut text = TextSystem::without_system_fonts();
        let name = "Rotação · ângulo";
        let full = text.prefix_width(name, 12.0);
        for frac in [0.2, 0.4, 0.6, 0.8] {
            if let Some(out) = elide(&mut text, name, 12.0, full * frac) {
                // A mid-char cut would have panicked in `elide` already; assert
                // the surviving prefix is a real prefix of the original.
                assert!(name.starts_with(out.trim_end_matches(ELLIPSIS)));
            }
        }
    }
}
