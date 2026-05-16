//! HR-15 — i18n label key lint.
//!
//! Every registered tool manifest declares a `label_key: &'static str`
//! that must:
//!  1. Use the canonical `tool.<slug>.label` shape so the Fluent
//!     bundle can be auto-generated from manifests.
//!  2. Match the manifest's `id` field (so renames stay coherent).
//!
//! PR 3 — Foundation. When Fluent bundles land (currently
//! `ph2d-i18n` is a stub per SKILL §7), this test extends to validate
//! presence in `crates/ph2d-editor/locales/pt-BR.ftl` and `en-US.ftl`.
//! Until then, format validation alone is the gate.

use ph2d_tool_registry::Registry;
use ph2d_tool_registry_init::register_all;

#[test]
fn every_manifest_label_key_follows_canonical_shape() {
    let mut reg = Registry::default();
    register_all(&mut reg);
    reg.build().expect("registry should build");

    let mut violations = Vec::new();
    for m in reg.manifests() {
        let key = m.label_key;

        // Rule 1 — shape: starts with "tool.", ends with ".label".
        if !key.starts_with("tool.") {
            violations.push(format!(
                "tool {:?}: label_key {:?} must start with 'tool.'",
                m.id, key
            ));
            continue;
        }
        if !key.ends_with(".label") {
            violations.push(format!(
                "tool {:?}: label_key {:?} must end with '.label'",
                m.id, key
            ));
            continue;
        }

        // Rule 2 — slug coherence: middle component matches id.
        // "tool.make_square.label" → slug "make_square" must equal m.id.
        let stripped = key
            .strip_prefix("tool.")
            .and_then(|s| s.strip_suffix(".label"));
        match stripped {
            Some(slug) if slug == m.id => {}
            Some(slug) => {
                violations.push(format!(
                    "tool {:?}: label_key slug {:?} does not match manifest id",
                    m.id, slug
                ));
            }
            None => violations.push(format!("tool {:?}: label_key {:?} malformed", m.id, key)),
        }
    }

    assert!(
        violations.is_empty(),
        "HR-15 label_key violations:\n{}",
        violations.join("\n")
    );
}

#[test]
fn registered_tools_baseline_matches_migration_progress() {
    // Tracks the migration progress through the convention-by-discovery
    // plan. PR 3 baseline was 0 manifests (empty registry); PR 4
    // piloto wires `make_square`. As more tools migrate (PR 6
    // grid-snap, PR 7 bgremoval), bump the lower bound here.
    let mut reg = Registry::default();
    register_all(&mut reg);
    reg.build().expect("registry should build");
    let n = reg.manifests().len();
    assert!(
        n >= 1,
        "expected at least 1 tool registered after PR 4 piloto, got {n}"
    );
    assert!(
        reg.by_id("make_square").is_some(),
        "make_square must be registered after PR 4"
    );
}
