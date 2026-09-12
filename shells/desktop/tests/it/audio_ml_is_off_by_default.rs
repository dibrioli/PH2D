//! **A1 of ADR-0123: a default build is byte-untouched by W7.**
//!
//! The whole ML denoise — `tract`, the 130-crate inference stack, the 7.6 MB DeepFilterNet model —
//! sits behind the `audio-ml` feature, which is **OFF by default**. This gate proves that
//! *structurally*, from the manifests, so a default `cargo build` never resolves `tract`, never
//! compiles the inference stack, never embeds the model.
//!
//! Why parse manifests instead of running `cargo tree`? Because the manifest is what *decides* the
//! tree: an `optional` dependency that only one non-default feature turns on cannot appear in a
//! default build. That is a sound, fast, hermetic proof — no cargo subprocess, no lockfile, nothing
//! that flakes on CI.
//!
//! # ⚠️⚠️ Re-anchored on the LAW on 2026-09-12 — the old pin read a DEAD line
//!
//! When the audio code left the shell for `ph2d-audio-desktop` (`line/shell-folhas`, 12/09 — today the family `ph2d-app-audio`), the
//! optional `ph2d-audio-ml` dependency moved with it — and the shell kept a copy of the line that
//! nothing compiled against. This gate was its **only reader**: pin 1 asked *«does the SHELL declare
//! `ph2d-audio-ml` as optional?»*. The `cargo machete` of the W2 Phase-D integration removed the dead
//! line and this gate went red — loud, which is the good half — while the law it guards still held.
//! ⇒ *a textual gate can be the only thing keeping a dead dependency alive* (HOWTO §2.18).
//!
//! The pins now follow the law wherever it lives, and pin 1 is **stronger** than before: it scans
//! **every manifest of the workspace**, so an ML dependency made non-optional in ANY crate — which the
//! old shell-only pin could never see — goes red here.
//!
//! ⭐ Verified against the oracle (`cargo tree -p ph2d-host-desktop -e normal,build`) on 12/09: the
//! default graph has **1 052** packages and no `ph2d-audio-ml`/`tract`; with `--features audio-ml` it
//! has **1 147**, `ph2d-audio-ml` and the `tract-*` stack included. And each pin below was proven by
//! mutation against the built test binary.
//!
//! The things that would let the ML stack leak into a default build, each pinned:
//! 1. `ph2d-audio-ml` becomes a non-optional dependency of **any** workspace crate.
//! 2. `audio-ml` gets listed in the shell's `default`.
//! 3. Some *other* shell feature enables `audio-ml`, `dep:ph2d-audio-ml` or `<carrier>/audio-ml`.
//! 4. The shell's plain dependency on a carrier turns the carrier's `audio-ml` on.
//! 5. A carrier enables `dep:ph2d-audio-ml` from its `default` or from any feature but `audio-ml`.
//!
//! And a sentinel: the shell's `audio-ml` really forwards to a carrier whose `audio-ml` really says
//! `dep:ph2d-audio-ml` — the chain this gate guards.

use std::fs;
use std::path::{Path, PathBuf};

const ML: &str = "ph2d-audio-ml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("shells/desktop has two parents")
        .to_path_buf()
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

/// The `[features]` table, as text — from the `[features]` header *line* (not a mention of it in a
/// comment) to the next top-level `[section]`.
fn features_table(manifest: &str) -> Option<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let t = line.trim();
        if !inside {
            inside = t == "[features]";
            continue;
        }
        if t.starts_with('[') && t.ends_with(']') && !t.contains('=') {
            break;
        }
        out.push(line);
    }
    inside.then(|| out.join("\n"))
}

/// The names of the array features, in order.
fn feature_names(manifest: &str) -> Vec<String> {
    features_table(manifest)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.starts_with('#') || !t.contains('[') {
                return None;
            }
            let (name, rest) = t.split_once('=')?;
            let name = name.trim();
            (rest.trim_start().starts_with('[') && !name.contains(' ')).then(|| name.to_string())
        })
        .collect()
}

/// The body of a `name = [ ... ]` feature, flattened to one line — tolerant of the column alignment
/// the shell uses (`audio-ml             = [...]`) and of arrays spanning several lines (`default`).
fn feature_body(manifest: &str, name: &str) -> Option<String> {
    let feats = features_table(manifest)?;
    let lines: Vec<&str> = feats.lines().collect();
    let i = lines.iter().position(|l| {
        l.trim_start()
            .strip_prefix(name)
            .is_some_and(|r| r.trim_start().starts_with('='))
    })?;
    let mut buf = String::new();
    for l in &lines[i..] {
        buf.push_str(l);
        buf.push(' ');
        if l.contains(']') {
            break;
        }
    }
    let open = buf.find('[')?;
    let close = buf[open..].find(']')? + open;
    Some(buf[open + 1..close].to_string())
}

/// The dependency lines `dep = ...` of a manifest (every table: normal, dev, build, target).
fn dep_lines<'a>(manifest: &'a str, dep: &str) -> Vec<&'a str> {
    manifest
        .lines()
        .filter(|l| {
            l.trim_start()
                .strip_prefix(dep)
                .is_some_and(|r| r.trim_start().starts_with('='))
        })
        .collect()
}

fn package_name(manifest: &str) -> String {
    manifest
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("name = \"")
                .and_then(|r| r.strip_suffix('"'))
        })
        .expect("a manifest names its package")
        .to_string()
}

#[test]
fn audio_ml_is_optional_and_off_by_default() {
    let root = root();

    // 1. EVERY workspace crate that declares the ML crate declares it OPTIONAL.
    let mut carriers: Vec<(String, String)> = Vec::new();
    let mut scanned = 0usize;
    for dir in ["crates", "shells", "tools", "tests"] {
        let Ok(rd) = fs::read_dir(root.join(dir)) else {
            continue;
        };
        for e in rd.flatten() {
            let m = e.path().join("Cargo.toml");
            if !m.is_file() {
                continue;
            }
            scanned += 1;
            let text = read(&m);
            let lines = dep_lines(&text, ML);
            if lines.is_empty() {
                continue;
            }
            for l in &lines {
                assert!(
                    l.contains("optional = true"),
                    "`{}` declares `{ML}` WITHOUT `optional = true` — every default build of a \
                     crate that depends on it compiles the ML stack (ADR-0123 A1):\n  {l}",
                    m.display()
                );
            }
            carriers.push((package_name(&text), text));
        }
    }
    assert!(
        scanned >= 300,
        "control: only {scanned} workspace manifests scanned — the scan lost its subject"
    );
    assert!(
        !carriers.is_empty(),
        "control: NO workspace crate declares `{ML}` — the ML crate was renamed or removed, and this \
         gate would pass on nothing"
    );

    let shell = read(&root.join("shells/desktop/Cargo.toml"));

    // 2. `audio-ml` is NOT in the shell's default feature set.
    let default = feature_body(&shell, "default").expect("the shell has a `default` feature");
    assert!(
        !default.contains("audio-ml"),
        "`audio-ml` is listed in the shell's `default` — it must be opt-in (ADR-0123 A1). \
         default = [{default}]"
    );

    // 3. No shell feature OTHER than `audio-ml` enables the stack, by any of the three spellings.
    for feat in feature_names(&shell) {
        if feat == "audio-ml" {
            continue;
        }
        let body = feature_body(&shell, &feat).unwrap_or_default();
        let via_carrier = carriers
            .iter()
            .any(|(c, _)| body.contains(&format!("{c}/audio-ml")));
        assert!(
            !body.contains("\"audio-ml\"") && !body.contains("dep:ph2d-audio-ml") && !via_carrier,
            "shell feature `{feat}` enables the ML stack — only `audio-ml` may (ADR-0123 A1):\n  \
             {feat} = [{body}]"
        );
    }

    // 4. The shell's plain dependency on a carrier does not switch the carrier's `audio-ml` on.
    for (c, _) in &carriers {
        for l in dep_lines(&shell, c) {
            assert!(
                !l.contains("audio-ml"),
                "the shell's dependency on the carrier `{c}` turns `audio-ml` on unconditionally \
                 (ADR-0123 A1):\n  {l}"
            );
        }
    }

    // 5. A carrier enables the ML crate ONLY from its own `audio-ml` feature.
    for (c, text) in &carriers {
        if let Some(d) = feature_body(text, "default") {
            assert!(
                !d.contains("audio-ml") && !d.contains("dep:ph2d-audio-ml"),
                "the carrier `{c}` turns the ML stack on in its own `default` = [{d}]"
            );
        }
        for feat in feature_names(text) {
            if feat == "audio-ml" {
                continue;
            }
            let body = feature_body(text, &feat).unwrap_or_default();
            assert!(
                !body.contains("dep:ph2d-audio-ml"),
                "carrier `{c}`: feature `{feat}` enables `{ML}` — only `audio-ml` may:\n  \
                 {feat} = [{body}]"
            );
        }
    }

    // Sentinel: the chain shell `audio-ml` → carrier `audio-ml` → `dep:ph2d-audio-ml` is real.
    let ml = feature_body(&shell, "audio-ml").expect("the shell has an `audio-ml` feature");
    let forwarded: Vec<&str> = carriers
        .iter()
        .filter(|(c, text)| {
            ml.contains(&format!("{c}/audio-ml"))
                && feature_body(text, "audio-ml").is_some_and(|b| b.contains("dep:ph2d-audio-ml"))
        })
        .map(|(c, _)| c.as_str())
        .collect();
    assert!(
        !forwarded.is_empty(),
        "sentinel: the shell's `audio-ml` = [{ml}] does not forward to a carrier whose `audio-ml` \
         enables `dep:{ML}` (carriers: {:?}) — this gate is not testing the chain it guards",
        carriers.iter().map(|(c, _)| c.as_str()).collect::<Vec<_>>()
    );
}
