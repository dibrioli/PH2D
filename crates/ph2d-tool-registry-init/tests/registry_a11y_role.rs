//! HR-12 — Accessibility role lint.
//!
//! Every registered tool manifest declares an `a11y_role: Role`. The
//! type system already enforces presence (the field is non-optional),
//! but this test adds a smoke check that the chosen role is one of
//! the canonical interactive roles a chrome button / panel would use,
//! catching footguns like `Role::Unknown` slipping in from copy-paste.
//!
//! PR 3 — Foundation. List of allowed roles widens as new tool kinds
//! arrive; today it covers Button + ToggleButton + MenuItem which
//! covers TopBar / LeftRail / context menu cases.

use ph2d_a11y::Role;
use ph2d_tool_registry::Registry;
use ph2d_tool_registry_init::register_all;

const ALLOWED_TOOL_ROLES: &[Role] = &[
    Role::Button,
    Role::Switch, // toggles — matches widget/toggle.rs canonical role
    Role::MenuItem,
    Role::CheckBox,
    // Add new roles here as the chrome surface grows; CI fails the
    // first tool registering a role outside this list, forcing
    // explicit review.
];

#[test]
fn every_manifest_uses_an_allowed_a11y_role() {
    let mut reg = Registry::default();
    register_all(&mut reg);
    reg.build().expect("registry should build");

    let mut violations = Vec::new();
    for m in reg.manifests() {
        if !ALLOWED_TOOL_ROLES.contains(&m.a11y_role) {
            violations.push(format!(
                "tool {:?}: a11y_role {:?} is not in the allowed list. \
                 If this role is intentional, add it to ALLOWED_TOOL_ROLES \
                 in tests/registry_a11y_role.rs with a comment justifying it.",
                m.id, m.a11y_role
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "HR-12 a11y role violations:\n{}",
        violations.join("\n")
    );
}
