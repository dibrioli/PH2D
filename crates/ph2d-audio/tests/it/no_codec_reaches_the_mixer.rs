//! **A6 of ADR-0118.** Streaming brought decoding *near* the mixer. It must not bring it *into* it.
//!
//! `ph2d-audio` runs on the real-time audio thread. A codec there would mean allocation,
//! non-deterministic timing, and a supply chain (symphonia, libopus, vorbis) inside the one crate
//! that must stay small enough to reason about — which is exactly why `ph2d-audio-decode` was a
//! separate crate from the beginning.
//!
//! ADR-0118 keeps that line: `ph2d-audio` owns the **rings and the reading**; whoever fills the
//! chunks (`ph2d-audio-stream`, on a worker thread) owns the codecs. This gate is the line, made
//! executable — because "we'll be careful" is not a boundary, and a dependency added in a hurry is
//! how a real-time crate quietly stops being one.

use std::path::Path;

/// Crates that must never appear in `ph2d-audio`'s dependency list. Not exhaustive of every codec
/// in the world, but exhaustive of every one this repo actually uses — the realistic way this gets
/// violated is someone reaching for a decoder that is already in the tree.
const FORBIDDEN: &[&str] = &[
    "symphonia",
    "ph2d-audio-decode",
    "ph2d-audio-opus",
    "ph2d-audio-encode",
    "ph2d-audio-spectral",
    "ph2d-audio-edit",
    "unsafe-libopus",
    "vorbis_rs",
    "ogg",
    "hound",
    "claxon",
    "minimp3",
];

#[test]
fn the_rt_mixer_has_no_codec_dependency() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read ph2d-audio/Cargo.toml");

    // Only the dependency sections — a codec named in a doc comment is a sentence, not an edge.
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
            !deps.contains(bad),
            "ADR-0118 A6: `{bad}` is now a dependency of ph2d-audio. The RT mixer must not decode \
             — it pops already-decoded chunks from a ring, and the producer thread (in \
             ph2d-audio-stream) owns the codecs. If streaming needs something from a decoder, it \
             needs it on the OTHER side of the ring.\n\n[dependencies]\n{deps}"
        );
    }

    // The gate must be looking at something. An empty `deps` would pass every assertion above while
    // proving nothing — the failure mode of every parser-based gate.
    assert!(
        deps.contains("crossbeam-queue"),
        "the manifest parse found no dependencies at all — this gate is not reading what it thinks"
    );
}
