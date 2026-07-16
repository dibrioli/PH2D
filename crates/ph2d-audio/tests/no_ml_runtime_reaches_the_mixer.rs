//! **A4 of ADR-0123.** W7 brought a neural network into the audio module. It must not bring it
//! into the RT mixer.
//!
//! `ph2d-audio` runs on the real-time audio thread. A `tract` inference session there would mean
//! allocation, unbounded and unpredictable timing, a 130-crate supply chain and a 7.6 MB model
//! inside the one crate that must stay small enough to reason about — which is exactly why the ML
//! denoise (`ph2d-audio-ml`) is a separate crate, offline, on the control thread, from the start.
//!
//! This is the sibling of `no_codec_reaches_the_mixer` (ADR-0118): the same line, drawn again for
//! the ML boundary, made executable — because "the denoise is offline, we'll keep it that way" is
//! not a boundary, and a dependency added in a hurry is how a real-time crate quietly stops being
//! one.
//!
//! **Its presence sibling** ([[feedback_absence_gate_needs_a_presence_sibling]]) lives in
//! `ph2d-audio-ml/tests/parity_with_reference_cli.rs`: "the ML never reaches the mixer" is green in
//! total silence, so it is paired with "the ML actually denoises" (+12 dB over the W5). One without
//! the other proves nothing.

use std::path::Path;

/// Crates that must never appear in `ph2d-audio`'s dependency list — the ML inference stack and
/// the crate that owns it. Not the whole ML world, but every piece this repo actually pulls, which
/// is the realistic way this gets violated: someone reaches into the tree that is already there.
const FORBIDDEN: &[&str] = &[
    "ph2d-audio-ml",
    "deep_filter",
    "tract",
    "tract-core",
    "tract-onnx",
    "tract-pulse",
    "tract-hir",
    "tract-nnef",
    "tract-linalg",
    "ndarray",
];

#[test]
fn the_rt_mixer_has_no_ml_runtime_dependency() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-audio/Cargo.toml");

    // Only the dependency sections — a crate named in a doc comment is a sentence, not an edge.
    let deps: String = text
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("[dependencies]"))
        .take_while(|l| {
            let t = l.trim();
            !t.starts_with('[') || t == "[dependencies]"
        })
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    for bad in FORBIDDEN {
        assert!(
            !dep_names(&deps).any(|name| name == *bad),
            "ADR-0123 A4: `{bad}` is now a dependency of ph2d-audio. The RT mixer must not run a \
             neural network — the ML denoise is offline, on the control thread, in `ph2d-audio-ml`. \
             If the mixer needs a denoised buffer, it needs it produced on the OTHER side of the \
             control/RT boundary.\n\n[dependencies]\n{deps}"
        );
    }

    // The gate must be looking at something. An empty `deps` would pass every assertion above while
    // proving nothing — the failure mode of every parser-based gate.
    assert!(
        deps.contains("crossbeam-queue"),
        "the manifest parse found no dependencies at all — this gate is not reading what it thinks"
    );
}

/// The dependency name on each `deps` line — the token before `=`/whitespace. So `ndarray` matches
/// a real `ndarray` dep but not `some-ndarray-helper`, and a comment already stripped upstream never
/// reaches here.
fn dep_names(deps: &str) -> impl Iterator<Item = &str> {
    deps.lines().filter_map(|l| {
        let t = l.trim();
        if t.is_empty() || t.starts_with('[') {
            return None;
        }
        Some(t.split(['=', ' ']).next().unwrap_or("").trim())
    })
}
