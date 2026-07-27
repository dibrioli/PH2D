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
    PhysicsIntent, PhysicsPanel, drain_intents, ids, interact, rows, set_current_physics,
};
use ph2d_physics_ecs::{HoldMode, InteractionSettings, InteractionTool, PhysicsSettings};
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
    arrange_with(settings, InteractionSettings::default())
}

/// [`arrange`] with the interaction tool in a chosen state — what the W-Hand
/// sweep needs, because those rows only PAINT for the tool that owns them.
fn arrange_with(
    settings: PhysicsSettings,
    interaction: InteractionSettings,
) -> (MockPanelHost, PhysicsPanelState) {
    set_current_physics(Some(PhysicsSnapshot {
        settings,
        pixels_per_meter: 100.0,
        show_colliders: true,
        body_count: 3,
        interaction,
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

// ── The Interaction section (W-Hand) ────────────────────────────────────────
// Its own sweep, and NOT a widening of the ones above, for a structural reason:
// these rows only PAINT for the tool that owns them, so a sweep that arranges a
// single state would find most of them missing and would have to be told to
// forgive that — a gate that forgives absence proves nothing about presence.

/// A settings state in which `row` is live. Derived from the row's OWN predicate
/// (`shown`) rather than from a table of row→tool written here: a second copy of
/// that mapping is exactly what drifts when a row moves tool.
fn state_showing(row: &interact::IRow) -> InteractionSettings {
    for tool in InteractionTool::ALL {
        for hold in HoldMode::ALL {
            let it = InteractionSettings {
                tool,
                hold,
                ..InteractionSettings::default()
            };
            if (row.shown)(&it) {
                return it;
            }
        }
    }
    panic!(
        "no tool/hold combination makes `{}` visible: it is a row nobody can \
         ever reach",
        row.label
    );
}

/// **Every Interaction row dispatches, and moves ITS field of the tool state.**
///
/// Mirror of `every_row_reaches_the_world_settings`, other destination: the
/// intent is `SetInteraction`, never `SetSettings` — the two have different
/// lifetimes and only one of them belongs in the project file.
#[test]
fn every_interaction_row_reaches_the_tool_state() {
    for row in interact::IROWS {
        let base = state_showing(row);
        let (mut host, mut state) = arrange_with(PhysicsSettings::default(), base);
        // Drive the slider to a track that is not its current value, so a row
        // wired to the wrong setter cannot pass by coincidence.
        let track = 0.75_f32;
        host.set_slider_value(row.slider, track);
        let out = host
            .apply_panel_event::<PhysicsPanel>(&mut state, WidgetEvent::ValueChanged(row.slider));
        assert_eq!(
            out,
            EventOutcome::Consumed,
            "`{}` did not consume its own ValueChanged",
            row.label
        );
        let intents = drain_intents();
        let got = intents
            .iter()
            .find_map(|i| match i {
                PhysicsIntent::SetInteraction(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("`{}` emitted no SetInteraction: {intents:?}", row.label));
        let want = row.value_of(track);
        let landed = (row.get)(&got);
        assert!(
            (landed - want).abs() < 1e-3,
            "`{}` landed {landed} for track {track} (wanted {want})",
            row.label
        );
        // And no OTHER field moved: a row wired to a neighbour would still emit.
        let mut expected = base;
        (row.set)(&mut expected, want);
        assert_eq!(
            got, expected,
            "`{}` changed a field that is not its own",
            row.label
        );
    }
}

/// **Every Interaction row is PAINTED and clickable for the tool that owns it —
/// and absent for the ones that do not.**
///
/// The absence half is the load-bearing one: the whole point of `shown` is that
/// the spring's gains do not appear while the artist is holding the Blast. A gate
/// that only checked presence would stay green over a section that paints all
/// seven knobs at once, four of which the solver ignores.
#[test]
fn every_interaction_row_is_painted_only_for_its_own_tool() {
    for row in interact::IROWS {
        let live = state_showing(row);
        let (mut host, mut state) = arrange_with(PhysicsSettings::default(), live);
        let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == row.slider)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| {
                panic!(
                    "`{}` is in the table and live for this tool, but the paint \
                     never registered it",
                    row.label
                )
            });
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        assert_eq!(
            host.hit_at(cx, cy),
            Some(row.slider),
            "`{}` is painted but something else owns the pixels at its centre",
            row.label
        );

        // Now a state where it is NOT live. Every row of this table has one
        // (each belongs to one tool), and `state_showing`'s panic already proved
        // the row is reachable at all.
        let dead = InteractionTool::ALL
            .iter()
            .flat_map(|&tool| {
                HoldMode::ALL.iter().map(move |&hold| InteractionSettings {
                    tool,
                    hold,
                    ..InteractionSettings::default()
                })
            })
            .find(|it| !(row.shown)(it))
            .expect("every row of this table belongs to one tool");
        let (mut host2, mut state2) = arrange_with(PhysicsSettings::default(), dead);
        let painted2 = host2.paint::<PhysicsPanel>(&mut state2, VIEWPORT);
        assert!(
            !painted2.iter().any(|(pid, _)| *pid == row.slider),
            "`{}` is painted for a tool that does not use it — a control the \
             solver ignores",
            row.label
        );
    }
}

/// **The two radios select what they name, and their chips answer a real click.**
///
/// The options are registered in a LOOP by the segmented helper, so
/// `architecture_panel_wiring_parity` cannot see them: like the 36 matrix cells,
/// this gate is the only thing covering those widgets. It uses the REAL pointer
/// (`click_at`), because a synthetic `WidgetEvent` skips the focusability check in
/// the store — which is exactly how a painted, hit-indexed, dead chip passes.
#[test]
fn the_interaction_radios_select_what_they_name() {
    for (i, &tool) in InteractionTool::ALL.iter().enumerate() {
        // Arrange on a DIFFERENT tool, so "already selected" cannot pass for
        // "selected by the click".
        let other = InteractionTool::ALL[(i + 1) % InteractionTool::ALL.len()];
        let (mut host, mut state) = arrange_with(
            PhysicsSettings::default(),
            InteractionSettings {
                tool: other,
                ..InteractionSettings::default()
            },
        );
        let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
        let id = ids::PHYSICS_INTERACT_TOOL_OPT[i];
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("tool chip {i} was never painted"));
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let _ = drain_intents();
        for ev in host.click_at(cx, cy) {
            let _ = host.apply_panel_event::<PhysicsPanel>(&mut state, ev);
        }
        let got = drain_intents()
            .iter()
            .find_map(|i| match i {
                PhysicsIntent::SetInteraction(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("clicking tool chip {i} emitted nothing"));
        assert_eq!(got.tool, tool, "tool chip {i} selected the wrong tool");
    }

    for (i, &hold) in HoldMode::ALL.iter().enumerate() {
        let other = HoldMode::ALL[(i + 1) % HoldMode::ALL.len()];
        let (mut host, mut state) = arrange_with(
            PhysicsSettings::default(),
            InteractionSettings {
                tool: InteractionTool::Hand,
                hold: other,
                ..InteractionSettings::default()
            },
        );
        let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
        let id = ids::PHYSICS_HOLD_MODE_OPT[i];
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("hold chip {i} was never painted"));
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let _ = drain_intents();
        for ev in host.click_at(cx, cy) {
            let _ = host.apply_panel_event::<PhysicsPanel>(&mut state, ev);
        }
        let got = drain_intents()
            .iter()
            .find_map(|i| match i {
                PhysicsIntent::SetInteraction(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("clicking hold chip {i} emitted nothing"));
        assert_eq!(got.hold, hold, "hold chip {i} selected the wrong mode");
    }
}

/// **O rádio Hold é oferecido só para a MÃO**, e o de ângulo da ponta só para a
/// POSE. O Blast e o Pull não seguram nada; a Pose segura um corpo mas não com
/// uma mola — ela RESOLVE por ele.
///
/// ⚠️ **A afirmação era `needs_a_body()` e virou `== Hand`, e é correção de
/// GATE.** Enquanto a mão era a única ferramenta que pega um corpo, os dois
/// coincidiam; a Pose separou as perguntas, e um gate escrito sobre o proxy
/// falha sobre produto CORRETO. A propriedade agora é afirmada direto, e o
/// segundo rádio entra no mesmo laço para as duas presenças serem uma varredura
/// só — enumerar *qual rádio pertence a quem* em dois testes é como o terceiro
/// nasce sem nenhum.
#[test]
fn each_tool_gets_exactly_its_own_radio() {
    for &tool in InteractionTool::ALL.iter() {
        let (mut host, mut state) = arrange_with(
            PhysicsSettings::default(),
            InteractionSettings {
                tool,
                ..InteractionSettings::default()
            },
        );
        let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
        let drawn = |id| painted.iter().any(|(pid, _)| *pid == id);
        assert_eq!(
            drawn(ids::PHYSICS_HOLD_MODE_OPT[0]),
            tool == InteractionTool::Hand,
            "the Hold radio is the HAND's, and it disagreed for {tool:?}"
        );
        assert_eq!(
            drawn(ids::PHYSICS_IK_ANGLE_OPT[0]),
            tool == InteractionTool::Pose,
            "the Tip Angle radio is the POSE's, and it disagreed for {tool:?}"
        );
    }
}

/// **Os dois chips do ângulo da ponta despacham o que nomeiam.** Clique real,
/// não `WidgetEvent` sintético: o sintético pula a checagem de FOCABILIDADE no
/// store, e é assim que um chip nasce pintado, hit-registrado e morto sob o
/// mouse (a lição das 36 células).
#[test]
fn the_tip_angle_chips_select_what_they_name() {
    for (i, &want) in [false, true].iter().enumerate() {
        let (mut host, mut state) = arrange_with(
            PhysicsSettings::default(),
            InteractionSettings {
                tool: InteractionTool::Pose,
                // Começa no OUTRO valor, senão clicar no chip já selecionado é
                // indistinguível de um clique que não faz nada.
                ik_match_angle: !want,
                ..InteractionSettings::default()
            },
        );
        let painted = host.paint::<PhysicsPanel>(&mut state, VIEWPORT);
        let id = ids::PHYSICS_IK_ANGLE_OPT[i];
        let rect = painted
            .iter()
            .rev()
            .find(|(pid, _)| *pid == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("tip-angle chip {i} was never painted"));
        let (cx, cy) = (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
        let _ = drain_intents();
        for ev in host.click_at(cx, cy) {
            let _ = host.apply_panel_event::<PhysicsPanel>(&mut state, ev);
        }
        let got = drain_intents()
            .iter()
            .find_map(|it| match it {
                PhysicsIntent::SetInteraction(s) => Some(*s),
                _ => None,
            })
            .unwrap_or_else(|| panic!("clicking tip-angle chip {i} emitted nothing"));
        assert_eq!(
            got.ik_match_angle, want,
            "tip-angle chip {i} set the wrong value"
        );
    }
}
