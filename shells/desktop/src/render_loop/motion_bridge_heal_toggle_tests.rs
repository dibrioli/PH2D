//! **The Node Help toggle gates** (ADR-0155, Enio 2026-08-04) — split from
//! `motion_bridge_heal_tests.rs` (shell LOC cap). The toggle is the artist's on/off for the
//! whole system — auto-heal + ⚠ badges + advisories. One flag
//! (`MotionState::node_help_enabled`), three entry points, one gate each — a layered defence
//! needs a per-layer gate ([[feedback_layered_defenses_need_per_layer_gates]]).
//!
//! A CHILD of the tests module, so `use super::*` reaches its fixtures (`wind_setup`,
//! `wire`, `integrate_ids`); the heal entry points live one level further up in `heal`.

use super::super::{heal_one, heal_setup, inert_reaching_output};
use super::*;
use ph2d_editor::ToastQueue;

/// **Node help OFF paints no ⚠ badges.** With the system off, the badge set is empty even
/// for a completed inert setup that badges when it is on — the artist's freedom, and the
/// release valve for a stray `falloff` advisory. FALSIFIED by dropping the `node_help_enabled`
/// guard in `inert_reaching_output` (the force is flagged regardless of the toggle).
#[test]
fn node_help_off_paints_no_badges() {
    let (mut m, _grid, force, out) = wind_setup();
    wire(&mut m, force, 0, out, 0); // completed inert setup
    assert!(
        inert_reaching_output(&m).contains(&force.0),
        "control: on by default, the inert force badges"
    );
    m.node_help_enabled = false;
    assert!(
        inert_reaching_output(&m).is_empty(),
        "off: no badges, whatever the graph says"
    );
}

/// **Node help OFF never auto-heals.** A constructive gesture heals an inert setup when the
/// system is on; with the toggle off, `heal_setup` returns 0 and inserts nothing. FALSIFIED
/// by dropping the guard in `heal_setup` (it would heal regardless of the toggle).
#[test]
fn node_help_off_never_auto_heals() {
    let (mut m, _grid, force, out) = wind_setup();
    wire(&mut m, force, 0, out, 0);
    m.node_help_enabled = false;
    assert_eq!(
        heal_setup(&mut m, &mut ToastQueue::default()),
        0,
        "off: nothing healed"
    );
    assert!(integrate_ids(&m).is_empty(), "off: no integrator inserted");
    // control: turning it on and firing again does heal.
    m.node_help_enabled = true;
    assert_eq!(
        heal_setup(&mut m, &mut ToastQueue::default()),
        1,
        "on: the same setup heals"
    );
}

/// **Node help OFF makes a badge click a clean no-op.** A stale click (a badge painted the
/// frame the toggle went off) must not heal. FALSIFIED by dropping the guard in `heal_one`.
#[test]
fn node_help_off_makes_a_badge_click_a_no_op() {
    let (mut m, _grid, force, out) = wind_setup();
    wire(&mut m, force, 0, out, 0);
    m.node_help_enabled = false;
    heal_one(&mut m, &mut ToastQueue::default(), force);
    assert!(
        integrate_ids(&m).is_empty(),
        "off: a stale badge click heals nothing"
    );
}
