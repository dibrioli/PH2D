//! **The chip printed the REQUEST, and the panel promised a readout it
//! never painted.**
//!
//! ## The two defects
//!
//! 1. `paint.rs` built the scale chip from `slider_to_scale(track)` —
//!    the raw slider reading. With EPX (which snaps to a whole factor,
//!    and before this pass clamped to `{2, 3, 4}`) the chip said `16.00×`
//!    while Apply baked `4×`. ⚠️ *A control that prints the request
//!    instead of the delivery is the most expensive shape of this bug:
//!    the artist confirms it with their eyes and is wrong.*
//! 2. The module doc has advertised an `"Output: W × H px"` readout
//!    since the panel existed. Nothing painted it.
//!
//! ## What this gate measures
//!
//! The panel's text output is not reachable from a headless paint (the
//! testkit returns hit rects, not glyphs), so this gate reads the paint
//! source and pins the DOOR: every number the artist reads must come
//! from `params::effective_factor` / `params::effective_output_size`,
//! both of which are `project_scale` — the same projection
//! `run_full_resolution` bakes with.
//!
//! The value-level half of the proof lives next door, in
//! `ph2d_tool_upscale::tool`'s `the_readout_and_the_bake_agree_at_every_stop`,
//! which walks all sixteen stops and compares the printed size against
//! the bytes Apply actually produces.

use std::fs;
use std::path::PathBuf;

fn paint_source() -> String {
    let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/paint.rs");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// Strip `//` comment tails so the prose in this file's own rationale
/// cannot satisfy — or trip — the checks below.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐ **The chip must be built from the projection, not the raw track.**
#[test]
fn the_scale_chip_is_built_from_the_effective_factor() {
    let code = code_only(&paint_source());
    assert!(
        code.contains("effective_factor(snapshot.algorithm, track)"),
        "the scale chip no longer prints `effective_factor(..)` — if it prints \
         `slider_to_scale(track)` it is showing the artist's REQUEST while Apply \
         bakes the projection"
    );
    assert!(
        !code.contains("slider_to_scale("),
        "`paint.rs` calls `slider_to_scale(..)` again: that is the raw slider \
         reading, and it disagrees with the bake wherever the algorithm snaps"
    );
}

/// ⭐ **The promised size readout exists, and is derived from the same
/// door** — not recomputed from the track, which would be a second
/// answer to the same question.
#[test]
fn the_output_size_readout_is_painted_through_the_same_door() {
    let code = code_only(&paint_source());
    assert!(
        code.contains("effective_output_size("),
        "the `Output: W x H px` readout the module doc promises is not painted; \
         it was missing entirely before this pass"
    );
    assert!(
        code.contains("snapshot.source_w") && code.contains("snapshot.source_h"),
        "the readout must use the LIVE source size from the snapshot — a readout \
         fed by a constant lies as soon as the artist selects another sprite"
    );
    assert!(
        code.contains("paint_text("),
        "the readout is computed and never drawn"
    );
}

/// The algorithm segment must not still advertise a name the kernel
/// does not implement. This is the label the artist reads.
#[test]
fn the_third_algorithm_is_not_labelled_xbr() {
    let code = code_only(&paint_source());
    assert!(
        !code.contains("\"xBR\""),
        "the third segment is labelled `xBR` again: the kernel is the EPX / \
         Scale2x family, not Hyllian xBR, and shipping an approximation under \
         that name is the defect this pass removed"
    );
    assert!(code.contains("\"EPX\""), "the third segment lost its label");
}
