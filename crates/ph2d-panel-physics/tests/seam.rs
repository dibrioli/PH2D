//! **Behavioral seam for the Physics world panel.**
//!
//! Compile-green is worth zero here: every failure this panel can have is a
//! control that paints and does nothing. So the sweep does not pick a
//! representative row — it walks [`rows`], the same table `paint`, `populate`
//! and `event` walk, and asks every one of them
//! ([[feedback_the_fullest_card_premise_rots]]).
//!
//! Two halves, because they fail independently:
//!
//! * **Registered → dispatches**: drive `ValueChanged` and assert the intent.
//! * **Painted → clickable**: run the real `paint`, then click the real painted
//!   rect through the real dispatcher. A widget is not done when it paints; it
//!   is done when a test clicks it
//!   ([[feedback_widget_is_done_when_a_test_clicks_it]]).

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_physics::state::{PhysicsPanelState, PhysicsSnapshot};
use ph2d_panel_physics::{
    PhysicsIntent, PhysicsPanel, drain_intents, ids, rows, set_current_physics,
};
use ph2d_physics_ecs::PhysicsSettings;
use ph2d_ui_testkit::MockPanelHost;

/// A dock-sized viewport. Tall, because the panel has five sections and a
/// paint that ran out of room would register nothing and quietly pass.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1200.0,
};

/// Put a known world in front of the panel and clear anything queued.
fn arrange(settings: PhysicsSettings) -> (MockPanelHost, PhysicsPanelState) {
    set_current_physics(Some(PhysicsSnapshot {
        settings,
        pixels_per_meter: 100.0,
        show_colliders: true,
        body_count: 3,
    }));
    let _ = drain_intents();
    (
        MockPanelHost::with_panel::<PhysicsPanel>(),
        PhysicsPanelState,
    )
}

/// **Every row dispatches, and carries the value the track means.**
///
/// The oracle is not "an intent came out": it is "THIS field moved to the value
/// this track maps to, and no other field moved". A row wired to the wrong
/// setter would still emit an intent.
#[test]
fn every_row_reaches_the_world_settings() {
    for row in rows::rows() {
        let base = PhysicsSettings::default();
        let (mut host, mut state) = arrange(base);

        // A drag writes the slider's stored value, then the dispatcher emits
        // ValueChanged. Simulate both, at a track the default is not already on.
        let track = 0.75_f32;
        host.set_slider_value(row.slider, track);
        let outcome = host
            .apply_panel_event::<PhysicsPanel>(&mut state, WidgetEvent::ValueChanged(row.slider));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "row `{}` ignored a real slider edit — its arm in event.rs is missing",
            row.label
        );

        let intents = drain_intents();
        let PhysicsIntent::SetSettings(got) = intents.first().unwrap_or_else(|| {
            panic!(
                "row `{}` consumed the edit but queued NO intent — the seam ends \
                 inside the panel and the world never hears about it",
                row.label
            )
        }) else {
            panic!("row `{}` queued the wrong kind of intent", row.label);
        };

        // The field this row owns moved to what the track means.
        let mut want = base;
        (row.set)(&mut want, row.value_of(track));
        assert_eq!(
            (row.get)(got),
            (row.get)(&want),
            "row `{}` moved its field to {} but the track {track} means {}",
            row.label,
            (row.get)(got),
            (row.get)(&want)
        );
        // And nothing else did. This is what catches a row wired to a
        // neighbour's setter — a copy-paste that an "an intent came out"
        // oracle cannot see.
        assert_eq!(
            *got, want,
            "row `{}` changed a setting it does not own",
            row.label
        );
    }
}

/// **Each row owns exactly one setting, and its getter and setter agree about
/// which one.**
///
/// ⚠️ This exists because the obvious version of it — inside the sweep above,
/// comparing the emitted settings against `(row.set)(&mut want, …)` — is
/// **circular**: it computes the expectation with the very function the bug
/// would live in, so wiring `gravity_y`'s row to `gravity_x`'s setter moved
/// both sides equally and the gate stayed green under exactly that mutation.
///
/// These two properties need no arithmetic from the table:
///
/// 1. **round-trip** — write `v` through `set`, read it back through `get`. A
///    setter pointing at a different field than its getter fails here.
/// 2. **disjointness** — writing one row's field must not disturb any other
///    row's. Two rows sharing a field fails here, whichever way they were
///    copy-pasted.
#[test]
fn each_row_owns_exactly_one_setting() {
    for row in rows::rows() {
        // 1. get ∘ set == identity, on a value the default is not already on.
        let probe = row.value_of(0.375);
        let mut s = PhysicsSettings::default();
        (row.set)(&mut s, probe);
        let read_back = (row.get)(&s);
        // Integer-valued rows round on the way in, so compare at the row's own
        // resolution rather than demanding bit equality of a rounded number.
        let tolerance = if row.decimals == 0 { 0.5 } else { 1e-3 };
        assert!(
            (read_back - probe).abs() <= tolerance,
            "row `{}`: wrote {probe} through its setter and read {read_back} \
             back through its getter — the two name different fields",
            row.label
        );

        // 2. and it moved nobody else's field.
        for other in rows::rows() {
            if std::ptr::eq(other, row) {
                continue;
            }
            let before = (other.get)(&PhysicsSettings::default());
            let after = (other.get)(&s);
            assert_eq!(
                before, after,
                "row `{}` changed row `{}`'s setting ({before} -> {after}) — \
                 two rows are wired to one field",
                row.label, other.label
            );
        }
    }
}

/// **A chip edit does not double-notify.** The chip is linked to its slider, so
/// the store already mirrored the value and the slider fired its own
/// `ValueChanged`. Forwarding the chip's too would apply one edit twice.
#[test]
fn a_chip_edit_is_swallowed_because_its_slider_already_spoke() {
    for row in rows::rows() {
        let (mut host, mut state) = arrange(PhysicsSettings::default());
        let outcome =
            host.apply_panel_event::<PhysicsPanel>(&mut state, WidgetEvent::ValueChanged(row.chip));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "chip of `{}` was not consumed — it would fall through to the host",
            row.label
        );
        assert!(
            drain_intents().is_empty(),
            "chip of `{}` queued its own intent on top of its slider's — one \
             edit applied twice",
            row.label
        );
    }
}

/// **Every chip carries its registered range**, which is what makes it
/// draggable rather than a two-position switch.
///
/// Without a range the chip derives its drag step from the buffer text and
/// scrubs ~50 units per PIXEL: one pixel of travel slams into the ceiling, so
/// the box becomes a min↔max toggle. The Flip panel's `populate` documents this
/// as the bug that "no test caught — typing the value always worked, only the
/// DRAG was broken". It is not untestable, though: `populate` writes the range
/// into the store, so the store can be asked.
#[test]
fn every_chip_is_draggable_because_its_range_is_registered() {
    let (host, _state) = arrange(PhysicsSettings::default());
    for row in rows::rows() {
        let (min, max, step) = host.store().number_range(row.chip).unwrap_or_else(|| {
            panic!(
                "chip of `{}` has no registered range — its drag would scrub ~50 \
                 units per pixel and behave as a min/max switch",
                row.label
            )
        });
        assert_eq!(
            min,
            f64::from(row.min),
            "chip of `{}`: wrong min",
            row.label
        );
        assert_eq!(
            max,
            f64::from(row.max),
            "chip of `{}`: wrong max",
            row.label
        );
        assert_eq!(step, row.step, "chip of `{}`: wrong step", row.label);
    }
}

/// **The collider toggle asks the SHELL, it does not decide.**
///
/// The flag belongs to `App.show_colliders`, which the `B` key also owns. If
/// this panel kept its own the two would disagree, visibly.
#[test]
fn the_collider_toggle_asks_the_shell_to_flip_its_flag() {
    let (mut host, mut state) = arrange(PhysicsSettings::default());
    let outcome = host.apply_panel_event::<PhysicsPanel>(
        &mut state,
        WidgetEvent::Click(ids::PHYSICS_SHOW_COLLIDERS),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        drain_intents(),
        vec![PhysicsIntent::ToggleColliders],
        "Show Colliders did not ask the shell to flip its flag"
    );
}

/// **Reset restores the engine defaults**, wholesale.
#[test]
fn reset_restores_the_engine_defaults() {
    let edited = PhysicsSettings {
        gravity_y: 0.0,
        substeps: 9,
        ..PhysicsSettings::default()
    };
    let (mut host, mut state) = arrange(edited);
    host.apply_panel_event::<PhysicsPanel>(
        &mut state,
        WidgetEvent::Click(ids::PHYSICS_RESET_DEFAULTS),
    );
    assert_eq!(
        drain_intents(),
        vec![PhysicsIntent::SetSettings(PhysicsSettings::default())],
        "Reset to Defaults did not publish the defaults"
    );
}

/// **Folding a section is panel-local** — it must not reach the world.
///
/// A fold that emitted a settings intent would clear the scrub cache and wake
/// every body in the scene because the artist collapsed a header.
#[test]
fn folding_a_section_never_touches_the_world() {
    for section in rows::SECTIONS {
        let (mut host, mut state) = arrange(PhysicsSettings::default());
        let outcome =
            host.apply_panel_event::<PhysicsPanel>(&mut state, WidgetEvent::Click(section.id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "section `{}` header is not clickable, but its chevron is painted",
            section.title
        );
        assert!(
            host.store().is_collapsed(section.id),
            "section `{}` did not fold",
            section.title
        );
        assert!(
            drain_intents().is_empty(),
            "folding section `{}` published a world settings change",
            section.title
        );
    }
}

/// **Painted ⟹ clickable.** The real `paint`, then the real dispatcher against
/// the rect the paint registered.
///
/// This is the half a hand-pushed `WidgetEvent` skips: a control can paint,
/// hit-register and forward — every other gate green — and still be stone dead
/// under the mouse because `populate` never registered it.
#[test]
fn every_painted_control_is_clickable_where_it_is_drawn() {
    let (mut host, mut state) = arrange(PhysicsSettings::default());
    let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);

    // Every slider, every chip, and every command must be somewhere on screen.
    let mut want: Vec<(&str, ph2d_a11y::NodeId)> = Vec::new();
    for row in rows::rows() {
        want.push((row.label, row.slider));
        want.push((row.label, row.chip));
    }
    for section in rows::SECTIONS {
        want.push((section.title, section.id));
    }
    want.push(("show_colliders", ids::PHYSICS_SHOW_COLLIDERS));
    want.push(("reset", ids::PHYSICS_RESET_DEFAULTS));

    for (name, id) in &want {
        assert!(
            painted.iter().any(|(pid, _)| pid == id),
            "`{name}` ({id:?}) is in the table but the paint never registered it"
        );
    }

    // And the two commands actually answer a pointer at their own centre.
    for (name, id) in [
        ("Show Colliders", ids::PHYSICS_SHOW_COLLIDERS),
        ("Reset to Defaults", ids::PHYSICS_RESET_DEFAULTS),
    ] {
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .expect("registered above");
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(id),
            "`{name}` is painted but something else owns the pixels at its centre"
        );
        let events = host.click_at(cx, cy);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "clicking `{name}` at its painted centre produced no Click — it is \
             registered in the hit index but not focusable in the store"
        );
    }
}

/// **Every one of the 36 matrix cells toggles its OWN pair, both halves.**
///
/// This gate carries more weight than its siblings: the cells are registered in
/// a LOOP, and `architecture_panel_wiring_parity` cannot see loop
/// registrations. Nothing else in the repo would notice these going dead.
///
/// Three claims per cell, because they fail separately: the click is consumed,
/// it flips the pair it names (not a neighbour — the flat-index ↔ `(i,j)`
/// conversion is exactly the kind of arithmetic that is off by one), and it
/// leaves every other pair untouched.
#[test]
fn every_matrix_cell_toggles_its_own_pair() {
    for i in 0..ph2d_physics_ecs::MAX_LAYERS {
        for j in 0..=i {
            let (mut host, mut state) = arrange(PhysicsSettings::default());
            let id = ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(i, j)];
            let outcome =
                host.apply_panel_event::<PhysicsPanel>(&mut state, WidgetEvent::Click(id));
            assert_eq!(
                outcome,
                EventOutcome::Consumed,
                "matrix cell ({i},{j}) is painted but its arm in event.rs is missing"
            );

            let intents = drain_intents();
            let Some(PhysicsIntent::SetSettings(got)) = intents.first() else {
                panic!("matrix cell ({i},{j}) queued no settings intent: {intents:?}");
            };
            let m = ph2d_physics_ecs::LayerMatrix::from_rows(got.layer_matrix);

            assert!(
                !m.collides(i, j),
                "clicking ({i},{j}) on a permissive matrix must SEPARATE that pair"
            );
            assert!(
                !m.collides(j, i),
                "({i},{j}) was cleared but its mirror ({j},{i}) was left set — \
                 rapier ANDs both directions, so a half-written pair is a rule \
                 nobody authored"
            );
            // ...and nobody else moved.
            for a in 0..ph2d_physics_ecs::MAX_LAYERS {
                for b in 0..ph2d_physics_ecs::MAX_LAYERS {
                    let touched = (a, b) == (i, j) || (a, b) == (j, i);
                    if !touched {
                        assert!(
                            m.collides(a, b),
                            "clicking ({i},{j}) also changed ({a},{b}) — the flat \
                             cell index maps to the wrong pair"
                        );
                    }
                }
            }
        }
    }
}

/// **Every matrix cell answers a REAL pointer at its painted centre.**
///
/// ⚠️ This drives `click_at` — the real dispatcher — and not a synthetic
/// `WidgetEvent`, because the two prove different things and the first version
/// of this gate only proved the weaker one. A synthetic event reaches
/// `apply_event` directly; a real click first has to find the cell in the hit
/// index **and** find it FOCUSABLE in the store. Dropping the cells from
/// `populate` left the synthetic gate green — the widget would have been
/// painted, hit-registered, arm-wired, and stone dead under the mouse.
#[test]
fn every_matrix_cell_answers_a_real_click() {
    let (mut host, mut state) = arrange(PhysicsSettings::default());
    let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
    for i in 0..ph2d_physics_ecs::MAX_LAYERS {
        for j in 0..=i {
            let id = ids::PHYSICS_LAYER_CELL[ids::physics_layer_cell_index(i, j)];
            let rect = painted
                .iter()
                .rev()
                .find(|(pid, _)| *pid == id)
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("matrix cell ({i},{j}) was never painted"));
            let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
            assert_eq!(
                host.hit_at(cx, cy),
                Some(id),
                "matrix cell ({i},{j}) is painted but something else owns its centre"
            );
            let events = host.click_at(cx, cy);
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
                "a real click on matrix cell ({i},{j}) produced no Click — it is \
                 in the hit index but not focusable in the store, so `populate` \
                 never registered it"
            );
        }
    }
}

/// **The panel publishes its own rect**, which is what the z-order walk and the
/// wheel/click hit-barrier route on. Without it the panel paints but pointer
/// events fall through to the canvas underneath.
#[test]
fn the_panel_publishes_its_rect() {
    let (mut host, mut state) = arrange(PhysicsSettings::default());
    host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
    let rect = host
        .store()
        .panel_rect(ids::PHYSICS_PANEL)
        .expect("the physics panel never published its rect — dispatch cannot route to it");
    assert!(
        rect.w > 0.0 && rect.h > 0.0,
        "the physics panel published an empty rect ({rect:?})"
    );
}
