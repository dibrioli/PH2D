//! **A1 of ADR-0123: a default build is byte-untouched by W7.**
//!
//! The whole ML denoise — `tract`, the 130-crate inference stack, the 7.6 MB DeepFilterNet model —
//! sits behind the `audio-ml` feature, which is **OFF by default**. This gate proves that
//! *structurally*, from the shell's manifest, so a default `cargo build` never resolves `tract`,
//! never compiles the inference stack, never embeds the model.
//!
//! Why parse the manifest instead of running `cargo tree`? Because the manifest is what *decides*
//! the tree: an `optional` dependency that only one non-default feature turns on cannot appear in a
//! default build. That is a sound, fast, hermetic proof — no cargo subprocess, no lockfile, nothing
//! that flakes on CI. (The `cargo tree -p ph2d-host-desktop` form was also checked by hand during
//! the build-out and showed no `tract`; this is the version that stays green forever.)
//!
//! The three things that would let the ML stack leak into a default build, each pinned:
//! 1. `ph2d-audio-ml` becomes a non-optional dependency.
//! 2. `audio-ml` gets listed in `default`.
//! 3. Some *other* feature (one `default` pulls) enables `audio-ml` or `dep:ph2d-audio-ml`.

use std::path::Path;

fn manifest() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    std::fs::read_to_string(&p).expect("read shells/desktop/Cargo.toml")
}

/// The `[features]` table, as text — from the `[features]` header *line* (not a mention of it in a
/// comment: the shell manifest has both) to the next top-level `[section]`.
fn features_table(manifest: &str) -> String {
    let mut out = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let t = line.trim();
        if !inside {
            if t == "[features]" {
                inside = true;
            }
            continue;
        }
        // A new top-level table ends the features section.
        if t.starts_with('[') && t.ends_with(']') && !t.contains('=') {
            break;
        }
        out.push(line);
    }
    assert!(inside, "shell manifest has no [features] table header");
    out.join("\n")
}

/// The body of a `name = [ ... ]` array feature, flattened to one line — spanning as many source
/// lines as it takes to reach the closing `]`. `default` is multi-line, so a single-line scan would
/// miss most of it.
fn feature_body(manifest: &str, name: &str) -> String {
    let feats = features_table(manifest);
    let feats = feats.as_str();
    let key = format!("{name} =");
    let start = feats
        .find(&key)
        .unwrap_or_else(|| panic!("feature `{name}` not found in [features]"));
    let after = &feats[start + key.len()..];
    let open = after.find('[').expect("feature value is an array");
    let close = after[open..].find(']').expect("array is closed") + open;
    after[open + 1..close].replace('\n', " ")
}

#[test]
fn audio_ml_is_optional_and_off_by_default() {
    let m = manifest();

    // 1. The crate is an OPTIONAL dependency — never pulled unless a feature asks.
    let dep_line = m
        .lines()
        .find(|l| l.trim_start().starts_with("ph2d-audio-ml ="))
        .expect("ph2d-audio-ml must be declared as a dependency of the shell");
    assert!(
        dep_line.contains("optional = true"),
        "ph2d-audio-ml must be `optional = true` — a non-optional ML dependency is compiled by \
         every default build (ADR-0123 A1):\n  {dep_line}"
    );

    // 2. `audio-ml` is NOT in the default feature set.
    let default = feature_body(&m, "default");
    assert!(
        !default.contains("audio-ml"),
        "`audio-ml` is listed in `default` — it must be opt-in (ADR-0123 A1). default = [{default}]"
    );

    // 3. No feature OTHER than `audio-ml` itself enables the crate. If any default-reachable
    //    feature said `dep:ph2d-audio-ml` (or pulled `audio-ml`), the stack would leak in.
    for line in m.lines() {
        let t = line.trim();
        if t.starts_with('#') || !t.contains('=') {
            continue;
        }
        let Some(feat) = t.split('=').next().map(str::trim) else {
            continue;
        };
        // Only feature lines (name = [ ... ]); skip the [dependencies] `ph2d-audio-ml = { ... }`.
        if !t.contains('[') || feat == "audio-ml" || feat.contains(' ') {
            continue;
        }
        assert!(
            !t.contains("dep:ph2d-audio-ml") && !t.contains("\"audio-ml\""),
            "feature `{feat}` enables the ML stack — only `audio-ml` may (ADR-0123 A1):\n  {t}"
        );
    }

    // Sentinel: the gate is actually reading the wiring it claims to guard.
    assert!(
        m.contains("audio-ml") && m.contains("dep:ph2d-audio-ml"),
        "the manifest has no `audio-ml` feature at all — this gate is not testing what it thinks"
    );
}
