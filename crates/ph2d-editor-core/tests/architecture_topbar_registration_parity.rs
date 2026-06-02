//! Architecture gate — topbar pill **registration parity**.
//!
//! Catches the 2026-06-02 "killer" regression class: a topbar pill that is
//! PAINTED (fixture `topbar_clusters()`) and HIT-INDEXED (`cluster_painter.rs`
//! `hit_index.register`) but NOT given an `InteractiveState` in
//! `topbar::populate()`. Such a pill has `is_focusable() == false`, so a
//! pointer-Down never makes it active, the pointer-Up never emits `Click`, and
//! its chrome toggle never fires `ActivateTool` — **the tool is dead on click**,
//! yet unit + CI stay green because nothing else asserts registration parity.
//! (The four vector pills PENCIL/SHAPE/SELECT/DIRECT shipped CI-green-but-dead
//! for multiple sessions; fixed in commit `0661862`. This gate is the
//! institutional memory of that bug — see `feedback_tool_unit_green_integration_dead`
//! / `feedback_panel_populate_register`.)
//!
//! Invariant: every `ids::TOPBAR_*` reached by the topbar PAINT path (the
//! fixture cluster list + the cluster painter's play/right sub-button arrays)
//! MUST also be registered in `populate()`. By convention a clickable pill is
//! registered as `InteractiveState::Button`; group backdrops as `Plain`. This
//! gate enforces *presence* of a registration — the exact thing the killer
//! lacked — by scanning the registration region of `populate()` (everything
//! before the tooltip loop, which would otherwise mask a missing registration).

use std::collections::BTreeSet;
use std::path::Path;

fn read(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Body of `fn <sig>…{ … }` — the slice between the first `{` after `sig` and
/// its matching close brace. Panics if absent/unbalanced (a gate, not prod).
fn fn_body<'a>(src: &'a str, sig: &str) -> &'a str {
    let start = src
        .find(sig)
        .unwrap_or_else(|| panic!("signature `{sig}` not found"));
    let after = &src[start..];
    let brace = after.find('{').expect("fn has no body brace");
    let bytes = after.as_bytes();
    let mut depth = 0usize;
    let mut i = brace;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &after[brace + 1..i];
                }
            }
            _ => {}
        }
        i += 1;
    }
    panic!("unbalanced braces in `{sig}`")
}

/// Every distinct `TOPBAR_<NAME>` identifier referenced via `ids::TOPBAR_…`.
fn topbar_ids(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let needle = "ids::TOPBAR_";
    let prefix = "ids::"; // keep the `TOPBAR_…` part
    let mut rest = src;
    while let Some(pos) = rest.find(needle) {
        let tail = &rest[pos + prefix.len()..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(tail.len());
        out.insert(tail[..end].to_string());
        rest = &tail[end..];
    }
    out
}

#[test]
fn topbar_painted_pills_are_all_registered() {
    let fixture = read("src/screens/hero/fixture.rs");
    let painter = read("src/screens/hero/topbar/cluster_painter.rs");
    let topbar = read("src/screens/hero/topbar/mod.rs");

    // PAINT path: cluster primaries (fixture `topbar_clusters()`) ∪ the
    // play/right sub-button arrays in the cluster painter. Every id here is
    // hit-indexed, so every id here must be focusable.
    let mut painted = topbar_ids(fn_body(&fixture, "fn topbar_clusters("));
    painted.extend(topbar_ids(&painter));

    // REGISTRATION path: the `store.register(id, InteractiveState::…)` loops
    // ONLY. Truncate `populate()` at the tooltip loop (`for (id, text) in …`):
    // a pill listed solely for a tooltip is still un-focusable, so counting
    // tooltip ids as "registered" would mask the very bug this gate exists for.
    let populate_body = fn_body(&topbar, "pub fn populate(");
    let registration_region = populate_body
        .split("for (id, text)")
        .next()
        .expect("populate() body is non-empty");
    let registered = topbar_ids(registration_region);

    let missing: Vec<&str> = painted
        .difference(&registered)
        .map(String::as_str)
        .collect();

    assert!(
        missing.is_empty(),
        "topbar pills are PAINTED + HIT-INDEXED but have NO `InteractiveState` in \
         `topbar::populate()` → `is_focusable() == false` → dead on click (see file \
         header): {missing:?}.\nAdd each `ids::{{…}}` to the `for id in [ … ] {{ \
         store.register(id, InteractiveState::Button {{ … }}) }}` loop in \
         `src/screens/hero/topbar/mod.rs::populate()`."
    );
}
